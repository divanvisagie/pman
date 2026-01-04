use anyhow::{bail, Context, Result};
use chrono::Local;
use regex::Regex;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

const REGISTRY_HEADER: &str = "# Project Registry\n\nFlat list of project notes. IDs are chronological and unique across all projects.\n\n| ID | Name | Status | Created | Note |\n| --- | --- | --- | --- | --- |\n";

// Embedded resources
const CLAUDE_MD: &str = include_str!("../resources/CLAUDE.md");
const PARA_NOTES_SKILL: &str = include_str!("../resources/skills/para-notes/SKILL.md");
const PROJECT_STRUCTURE_SKILL: &str = include_str!("../resources/skills/project-structure/SKILL.md");

#[derive(Debug, Clone)]
pub struct NotesPaths {
    pub root: PathBuf,
    pub projects_dir: PathBuf,
    pub archives_projects_dir: PathBuf,
    pub registry: PathBuf,
}

impl NotesPaths {
    pub fn from_root(root: PathBuf) -> Self {
        let projects_dir = root.join("Projects");
        let archives_projects_dir = root.join("Archives").join("Projects");
        let registry = projects_dir.join("_registry.md");
        Self {
            root,
            projects_dir,
            archives_projects_dir,
            registry,
        }
    }
}

pub fn resolve_notes_dir(notes_dir: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = notes_dir {
        return Ok(path);
    }

    // Default to ~/Notes
    if let Some(home) = std::env::var_os("HOME") {
        let default_notes = PathBuf::from(home).join("Notes");
        if default_notes.exists() {
            return Ok(default_notes);
        }
    }

    if let Some(path) = find_notes_root_from_path(&std::env::current_exe()?) {
        return Ok(path);
    }

    if let Ok(current_dir) = std::env::current_dir() {
        if let Some(path) = find_notes_root_from_path(&current_dir) {
            return Ok(path);
        }
    }

    bail!("Could not locate Notes root; use --notes-dir to specify it")
}

pub fn create_project(
    paths: &NotesPaths,
    name: &str,
    status: &str,
    area: Option<&str>,
) -> Result<PathBuf> {
    ensure_registry(paths)?;

    let registry_contents = fs::read_to_string(&paths.registry)
        .with_context(|| format!("Failed to read registry at {}", paths.registry.display()))?;
    let next_id = next_project_id(&registry_contents);
    let slug = slugify(name)?;
    let area_slug = area.map(slugify).transpose()?;

    let slug_full = match area_slug.as_deref() {
        Some(area_value) => format!("{area_value}-{slug}"),
        None => slug.clone(),
    };

    if slug_in_use(paths, &slug_full)? {
        bail!("Slug already exists in Projects or Archives: {slug_full}");
    }

    let dir_name = format!("proj-{next_id}-{slug_full}");
    let note_dir = paths.projects_dir.join(&dir_name);
    let note_path = note_dir.join("README.md");

    if note_path.exists() {
        bail!("Project note already exists: {}", note_path.display());
    }

    fs::create_dir_all(&note_dir)
        .with_context(|| format!("Failed to create project directory {}", note_dir.display()))?;

    let created = Local::now().format("%Y-%m-%d");
    let area_section = area_slug
        .as_deref()
        .map(|value| format!("\n## Area\n- {value}\n"))
        .unwrap_or_default();
    let content = format!(
        "# PROJ-{id}: {name}\n\n**Created**: {created}\n\n## Summary\n- \n\n## Status\n- {status}{area}\n## Notes\n- \n\n## Next\n- \n",
        id = next_id,
        name = name,
        created = created,
        status = status,
        area = area_section
    );

    fs::write(&note_path, content)
        .with_context(|| format!("Failed to write note {}", note_path.display()))?;

    let registry_line = format!(
        "| PROJ-{id} | {name} | {status} | {created} | [{dir}/README.md]({dir}/README.md) |",
        id = next_id,
        name = name,
        status = status,
        created = created,
        dir = dir_name
    );

    let mut registry = OpenOptions::new()
        .append(true)
        .open(&paths.registry)
        .with_context(|| format!("Failed to open registry {}", paths.registry.display()))?;
    writeln!(registry, "{registry_line}")?;

    Ok(note_path)
}

pub fn archive_project(paths: &NotesPaths, input: &str) -> Result<PathBuf> {
    let src_dir = find_project_dir(&paths.projects_dir, input)?;
    let dir_name = src_dir
        .file_name()
        .and_then(|name| name.to_str())
        .context("Project directory name is not valid UTF-8")?;

    let dest_dir = paths.archives_projects_dir.join(dir_name);
    if dest_dir.exists() {
        bail!("Archive target already exists: {}", dest_dir.display());
    }

    fs::create_dir_all(&paths.archives_projects_dir).with_context(|| {
        format!(
            "Failed to create archive directory {}",
            paths.archives_projects_dir.display()
        )
    })?;
    fs::rename(&src_dir, &dest_dir)
        .with_context(|| format!("Failed to move project to {}", dest_dir.display()))?;

    let proj_id = project_id_from_dir(dir_name)?;
    let note_path = if dest_dir.join("README.md").exists() {
        format!("../Archives/Projects/{dir}/README.md", dir = dir_name)
    } else {
        format!("../Archives/Projects/{dir}/", dir = dir_name)
    };

    update_registry_entry(&paths.registry, &proj_id, &note_path)?;

    Ok(dest_dir)
}

fn ensure_registry(paths: &NotesPaths) -> Result<()> {
    if paths.registry.exists() {
        return Ok(());
    }

    fs::create_dir_all(&paths.projects_dir).with_context(|| {
        format!(
            "Failed to create projects directory {}",
            paths.projects_dir.display()
        )
    })?;
    fs::write(&paths.registry, REGISTRY_HEADER).with_context(|| {
        format!(
            "Failed to create registry file {}",
            paths.registry.display()
        )
    })?;

    Ok(())
}

fn next_project_id(registry_contents: &str) -> u32 {
    let re = Regex::new(r"PROJ-(\d+)").expect("valid regex");
    let mut max_id = 0u32;
    for cap in re.captures_iter(registry_contents) {
        if let Ok(num) = cap[1].parse::<u32>() {
            if num > max_id {
                max_id = num;
            }
        }
    }
    max_id + 1
}

pub fn slugify(name: &str) -> Result<String> {
    let mut slug = String::new();
    let mut prev_dash = false;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        bail!("Project name produces an empty slug");
    }

    Ok(slug)
}

fn slug_in_use(paths: &NotesPaths, slug: &str) -> Result<bool> {
    if has_slug_in_dir(&paths.projects_dir, slug)? {
        return Ok(true);
    }

    if paths.archives_projects_dir.exists() && has_slug_in_dir(&paths.archives_projects_dir, slug)? {
        return Ok(true);
    }

    Ok(false)
}

fn has_slug_in_dir(dir: &Path, slug: &str) -> Result<bool> {
    if !dir.exists() {
        return Ok(false);
    }

    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = match name.to_str() {
            Some(value) => value,
            None => continue,
        };
        if slug_matches_dir(name, slug) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn slug_matches_dir(name: &str, slug: &str) -> bool {
    let suffix = match name.strip_prefix("proj-") {
        Some(value) => value,
        None => return false,
    };

    let mut parts = suffix.splitn(2, '-');
    let id = match parts.next() {
        Some(value) => value,
        None => return false,
    };
    let tail = match parts.next() {
        Some(value) => value,
        None => return false,
    };

    if id.is_empty() || !id.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }

    tail == slug
}

fn find_project_dir(projects_dir: &Path, input: &str) -> Result<PathBuf> {
    let direct = projects_dir.join(input);
    if direct.exists() {
        return Ok(direct);
    }

    let mut matches = Vec::new();
    for entry in fs::read_dir(projects_dir)
        .with_context(|| format!("Failed to read {}", projects_dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = match name.to_str() {
            Some(value) => value,
            None => continue,
        };
        if name.starts_with(&format!("{input}-")) {
            matches.push(entry.path());
        }
    }

    match matches.len() {
        0 => bail!("No project directory matching {input}"),
        1 => Ok(matches.remove(0)),
        _ => bail!("Multiple matches for {input}"),
    }
}

fn project_id_from_dir(dir_name: &str) -> Result<String> {
    let suffix = dir_name
        .strip_prefix("proj-")
        .ok_or_else(|| anyhow::anyhow!("Invalid project directory name: {dir_name}"))?;
    let id = suffix
        .splitn(2, '-')
        .next()
        .unwrap_or_default();

    if id.is_empty() || !id.chars().all(|ch| ch.is_ascii_digit()) {
        bail!("Invalid project directory name: {dir_name}");
    }

    Ok(format!("PROJ-{}", id))
}

fn update_registry_entry(registry: &Path, proj_id: &str, note_path: &str) -> Result<()> {
    let mut lines = fs::read_to_string(registry)
        .with_context(|| format!("Failed to read registry {}", registry.display()))?
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<String>>();

    let mut updated = false;
    for line in &mut lines {
        if !line.starts_with(&format!("| {proj_id} |")) {
            continue;
        }

        let mut parts = line
            .trim_matches('|')
            .split('|')
            .map(|part| part.trim().to_string())
            .collect::<Vec<String>>();
        if parts.len() < 5 {
            bail!("Registry entry malformed for {proj_id}");
        }

        parts[2] = "archived".to_string();
        parts[4] = format!("[{note}]({note})", note = note_path);
        *line = format!("| {} |", parts.join(" | "));
        updated = true;
        break;
    }

    if !updated {
        bail!("Registry entry not found for {proj_id}");
    }

    let updated_contents = lines.join("\n") + "\n";
    fs::write(registry, updated_contents)
        .with_context(|| format!("Failed to write registry {}", registry.display()))?;

    Ok(())
}

fn find_notes_root_from_path(path: &Path) -> Option<PathBuf> {
    for ancestor in path.ancestors() {
        if ancestor.join("Projects").is_dir() && ancestor.join("Archives").is_dir() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

/// Initialize a new pman workspace at the given path.
/// Creates Notes directory structure, CLAUDE.md, and skills.
/// Skips any file or directory that already exists.
pub fn init_workspace(workspace: &Path) -> Result<()> {
    println!("Initializing pman workspace at {}", workspace.display());

    // Create Notes directory structure
    let notes_dirs = [
        workspace.join("Notes").join("Projects"),
        workspace.join("Notes").join("Areas"),
        workspace.join("Notes").join("Resources"),
        workspace.join("Notes").join("Archives").join("Projects"),
    ];

    for dir in &notes_dirs {
        if dir.exists() {
            println!("  skip: {} (exists)", dir.strip_prefix(workspace).unwrap_or(dir).display());
        } else {
            fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create {}", dir.display()))?;
            println!("  create: {}", dir.strip_prefix(workspace).unwrap_or(dir).display());
        }
    }

    // Create registry file
    let registry = workspace.join("Notes").join("Projects").join("_registry.md");
    if registry.exists() {
        println!("  skip: {} (exists)", registry.strip_prefix(workspace).unwrap_or(&registry).display());
    } else {
        fs::write(&registry, REGISTRY_HEADER)
            .with_context(|| format!("Failed to create {}", registry.display()))?;
        println!("  create: {}", registry.strip_prefix(workspace).unwrap_or(&registry).display());
    }

    // Create CLAUDE.md
    let claude_md = workspace.join("CLAUDE.md");
    if claude_md.exists() {
        println!("  skip: {} (exists)", claude_md.strip_prefix(workspace).unwrap_or(&claude_md).display());
    } else {
        fs::write(&claude_md, CLAUDE_MD)
            .with_context(|| format!("Failed to create {}", claude_md.display()))?;
        println!("  create: {}", claude_md.strip_prefix(workspace).unwrap_or(&claude_md).display());
    }

    // Create skills
    let skills = [
        (workspace.join(".claude").join("skills").join("para-notes").join("SKILL.md"), PARA_NOTES_SKILL),
        (workspace.join(".claude").join("skills").join("project-structure").join("SKILL.md"), PROJECT_STRUCTURE_SKILL),
    ];

    for (path, content) in &skills {
        if path.exists() {
            println!("  skip: {} (exists)", path.strip_prefix(workspace).unwrap_or(path).display());
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create {}", parent.display()))?;
            }
            fs::write(path, content)
                .with_context(|| format!("Failed to create {}", path.display()))?;
            println!("  create: {}", path.strip_prefix(workspace).unwrap_or(path).display());
        }
    }

    println!("\nWorkspace initialized. Create a README.md with your custom configuration.");
    Ok(())
}

/// Verify workspace setup and report any issues.
/// Returns true if all checks pass, false otherwise.
pub fn verify_workspace(workspace: &Path) -> Result<bool> {
    println!("Verifying pman workspace at {}", workspace.display());

    let mut all_ok = true;

    // Check Notes directory structure
    let notes_dirs = [
        ("Notes/Projects", workspace.join("Notes").join("Projects")),
        ("Notes/Areas", workspace.join("Notes").join("Areas")),
        ("Notes/Resources", workspace.join("Notes").join("Resources")),
        ("Notes/Archives/Projects", workspace.join("Notes").join("Archives").join("Projects")),
    ];

    for (name, path) in &notes_dirs {
        if path.exists() {
            println!("  ✓ {}", name);
        } else {
            println!("  ✗ {} (missing)", name);
            all_ok = false;
        }
    }

    // Check registry file
    let registry = workspace.join("Notes").join("Projects").join("_registry.md");
    if registry.exists() {
        println!("  ✓ Notes/Projects/_registry.md");
    } else {
        println!("  ✗ Notes/Projects/_registry.md (missing)");
        all_ok = false;
    }

    // Check CLAUDE.md
    let claude_md = workspace.join("CLAUDE.md");
    if claude_md.exists() {
        println!("  ✓ CLAUDE.md");
    } else {
        println!("  ✗ CLAUDE.md (missing)");
        all_ok = false;
    }

    // Check skills
    let skills = [
        (".claude/skills/para-notes/SKILL.md", workspace.join(".claude").join("skills").join("para-notes").join("SKILL.md")),
        (".claude/skills/project-structure/SKILL.md", workspace.join(".claude").join("skills").join("project-structure").join("SKILL.md")),
    ];

    for (name, path) in &skills {
        if path.exists() {
            println!("  ✓ {}", name);
        } else {
            println!("  ✗ {} (missing)", name);
            all_ok = false;
        }
    }

    // Summary
    if all_ok {
        println!("\nWorkspace OK.");
    } else {
        println!("\nIssues found. Run 'pman init' to create missing directories, or 'pman update' to restore CLAUDE.md and skills.");
    }

    Ok(all_ok)
}

/// Update CLAUDE.md and skills to the latest embedded versions.
/// Always overwrites existing files.
pub fn update_workspace(workspace: &Path) -> Result<()> {
    println!("Updating pman resources at {}", workspace.display());

    // Update CLAUDE.md
    let claude_md = workspace.join("CLAUDE.md");
    fs::write(&claude_md, CLAUDE_MD)
        .with_context(|| format!("Failed to write {}", claude_md.display()))?;
    println!("  update: CLAUDE.md");

    // Update skills
    let skills = [
        (workspace.join(".claude").join("skills").join("para-notes").join("SKILL.md"), PARA_NOTES_SKILL),
        (workspace.join(".claude").join("skills").join("project-structure").join("SKILL.md"), PROJECT_STRUCTURE_SKILL),
    ];

    for (path, content) in &skills {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        fs::write(path, content)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        println!("  update: {}", path.strip_prefix(workspace).unwrap_or(path).display());
    }

    println!("\nResources updated to pman v{}.", env!("CARGO_PKG_VERSION"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn slugify_rejects_empty() {
        let err = slugify("!!!").unwrap_err().to_string();
        assert!(err.contains("empty slug"));
    }

    #[test]
    fn slugify_compacts_dashes() {
        let slug = slugify("Hello, World!!!").unwrap();
        assert_eq!(slug, "hello-world");
    }

    #[test]
    fn next_project_id_increments() {
        let registry = "| PROJ-0002 | Example |\n| PROJ-2 | Example |";
        assert_eq!(next_project_id(registry), 3);
    }

    #[test]
    fn create_project_blocks_archived_slug() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        let paths = NotesPaths::from_root(root.to_path_buf());
        fs::create_dir_all(&paths.projects_dir).unwrap();
        fs::create_dir_all(&paths.archives_projects_dir).unwrap();
        fs::write(&paths.registry, REGISTRY_HEADER).unwrap();

        let archived = paths.archives_projects_dir.join("proj-1-test-slug");
        fs::create_dir_all(archived).unwrap();

        let err = create_project(&paths, "Test Slug", "active", None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("Slug already exists"));
    }

    #[test]
    fn archive_project_updates_registry() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        let paths = NotesPaths::from_root(root.to_path_buf());
        fs::create_dir_all(&paths.projects_dir).unwrap();
        fs::create_dir_all(&paths.archives_projects_dir).unwrap();

        let proj_dir = paths.projects_dir.join("proj-3-sample");
        fs::create_dir_all(&proj_dir).unwrap();
        fs::write(proj_dir.join("README.md"), "test").unwrap();

        let registry = format!(
            "{header}| PROJ-3 | Sample | active | 2025-01-01 | [proj-3-sample/README.md](proj-3-sample/README.md) |\n",
            header = REGISTRY_HEADER
        );
        fs::write(&paths.registry, registry).unwrap();

        archive_project(&paths, "proj-3").unwrap();

        let updated = fs::read_to_string(&paths.registry).unwrap();
        assert!(updated.contains("| PROJ-3 | Sample | archived |"));
        assert!(updated.contains("../Archives/Projects/proj-3-sample/README.md"));
    }

    #[test]
    fn create_project_includes_area_slug() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        let paths = NotesPaths::from_root(root.to_path_buf());
        fs::create_dir_all(&paths.projects_dir).unwrap();
        fs::write(&paths.registry, REGISTRY_HEADER).unwrap();

        let note_path = create_project(&paths, "Runes Notes", "active", Some("religion"))
            .unwrap();

        assert!(note_path
            .to_string_lossy()
            .contains("proj-1-religion-runes-notes/README.md"));
    }
}
