use anyhow::{Context, Result, bail};
use chrono::Local;
use regex::Regex;
use std::fs;
use std::fs::OpenOptions;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const REGISTRY_HEADER: &str = "# Project Registry\n\nFlat list of project notes. IDs are chronological and unique across all projects.\n\n| ID | Name | Status | Created | Note |\n| --- | --- | --- | --- | --- |\n";

// Embedded resources
const AGENTS_MD: &str = include_str!("../resources/AGENTS.md");
const PROJECT_SKILL: &str = include_str!("../resources/skills/project/SKILL.md");
const NOTES_DIR_ENV_VAR: &str = "PMAN_NOTES_DIR";
const PROJECT_PREFIX_ENV_VAR: &str = "PMAN_PROJECT_PREFIX";
const DEFAULT_PROJECT_PREFIX: &str = "proj";
const FORCE_CLAUDE_PRESENT_ENV_VAR: &str = "PMAN_FORCE_CLAUDE_PRESENT";
const FORCE_CODEX_PRESENT_ENV_VAR: &str = "PMAN_FORCE_CODEX_PRESENT";

#[derive(Debug, Clone, Copy, Default)]
pub struct WcFlags {
    pub lines: bool,
    pub words: bool,
    pub bytes: bool,
    pub chars: bool,
}

#[derive(Debug, Clone, Copy)]
struct LineRange {
    start: usize,
    end: usize,
}

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

    if let Some(path) = std::env::var_os(NOTES_DIR_ENV_VAR) {
        let path = PathBuf::from(path);
        if !path.as_os_str().is_empty() {
            return Ok(path);
        }
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

    bail!("Could not locate Notes root; set {NOTES_DIR_ENV_VAR} or use --notes-dir to specify it")
}

fn project_dir_prefix() -> String {
    let Some(value) = std::env::var_os(PROJECT_PREFIX_ENV_VAR) else {
        return DEFAULT_PROJECT_PREFIX.to_string();
    };
    let value = value.to_string_lossy().trim().to_ascii_lowercase();
    if value.is_empty() || !value.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return DEFAULT_PROJECT_PREFIX.to_string();
    }
    value
}

pub fn read_note(
    notes_dir: Option<PathBuf>,
    path: &Path,
    lines: Option<&str>,
    numbered: bool,
) -> Result<String> {
    let (all_lines, trailing_newline) = read_note_lines(notes_dir, path)?;

    let (range, selected_has_trailing_newline) = match lines {
        Some(spec) => {
            let range = parse_line_range(spec)?;
            ensure_range_in_bounds(range, all_lines.len())?;
            let has_trailing = if all_lines.is_empty() {
                false
            } else {
                range.end < all_lines.len() || (range.end == all_lines.len() && trailing_newline)
            };
            (range, has_trailing)
        }
        None => (
            if all_lines.is_empty() {
                LineRange { start: 1, end: 1 }
            } else {
                LineRange {
                    start: 1,
                    end: all_lines.len(),
                }
            },
            trailing_newline,
        ),
    };

    if all_lines.is_empty() {
        return Ok(String::new());
    }

    if !numbered {
        return Ok(render_line_range(
            &all_lines,
            range,
            selected_has_trailing_newline,
        ));
    }

    let slice = &all_lines[(range.start - 1)..range.end];
    let mut output = slice
        .iter()
        .enumerate()
        .map(|(offset, line)| format!("{:>6}\t{line}", range.start + offset))
        .collect::<Vec<String>>()
        .join("\n");
    if selected_has_trailing_newline {
        output.push('\n');
    }
    Ok(output)
}

pub fn write_note(
    notes_dir: Option<PathBuf>,
    path: &Path,
    content: &str,
    create_dirs: bool,
) -> Result<PathBuf> {
    let root = canonical_notes_root(notes_dir)?;
    let target = resolve_writable_note_file(&root, path, create_dirs)?;
    fs::write(&target, content)
        .with_context(|| format!("Failed to write note {}", target.display()))?;
    Ok(target)
}

pub fn edit_note(
    notes_dir: Option<PathBuf>,
    path: &Path,
    replace_lines: &str,
    with_text: &str,
    expect: Option<&str>,
) -> Result<PathBuf> {
    let root = canonical_notes_root(notes_dir)?;
    let target = resolve_existing_note_file(&root, path)?;
    let content = fs::read_to_string(&target)
        .with_context(|| format!("Failed to read note {}", target.display()))?;
    let range = parse_line_range(replace_lines)?;

    let (mut lines, mut trailing_newline) = split_lines(&content);
    ensure_range_in_bounds(range, lines.len())?;

    let current = if lines.is_empty() {
        String::new()
    } else {
        let has_trailing =
            range.end < lines.len() || (range.end == lines.len() && trailing_newline);
        render_line_range(&lines, range, has_trailing)
    };

    if let Some(expected) = expect {
        if current != expected {
            bail!(
                "Expected text mismatch for lines {}:{}",
                range.start,
                range.end
            );
        }
    }

    let (replacement_lines, replacement_trailing) = split_lines(with_text);
    let original_len = lines.len();
    if original_len == 0 {
        lines = replacement_lines;
        trailing_newline = replacement_trailing;
    } else {
        lines.splice((range.start - 1)..range.end, replacement_lines);
        if range.end == original_len {
            trailing_newline = replacement_trailing;
        }
    }

    let updated = join_lines(&lines, trailing_newline);
    fs::write(&target, updated)
        .with_context(|| format!("Failed to write note {}", target.display()))?;
    Ok(target)
}

pub fn cat_note(notes_dir: Option<PathBuf>, path: &Path) -> Result<String> {
    read_note(notes_dir, path, None, false)
}

pub fn head_note(notes_dir: Option<PathBuf>, path: &Path, count: usize) -> Result<String> {
    let (lines, trailing_newline) = read_note_lines(notes_dir, path)?;
    if lines.is_empty() || count == 0 {
        return Ok(String::new());
    }
    let end = count.min(lines.len());
    let range = LineRange { start: 1, end };
    let has_trailing = end < lines.len() || (end == lines.len() && trailing_newline);
    Ok(render_line_range(&lines, range, has_trailing))
}

pub fn tail_note(notes_dir: Option<PathBuf>, path: &Path, count: usize) -> Result<String> {
    let (lines, trailing_newline) = read_note_lines(notes_dir, path)?;
    if lines.is_empty() || count == 0 {
        return Ok(String::new());
    }
    let start = lines.len().saturating_sub(count) + 1;
    let range = LineRange {
        start,
        end: lines.len(),
    };
    Ok(render_line_range(&lines, range, trailing_newline))
}

pub fn wc_note(notes_dir: Option<PathBuf>, path: &Path, flags: WcFlags) -> Result<String> {
    let root = canonical_notes_root(notes_dir)?;
    let target = resolve_existing_note_file(&root, path)?;
    let content = fs::read_to_string(&target)
        .with_context(|| format!("Failed to read note {}", target.display()))?;

    let line_count = content.bytes().filter(|byte| *byte == b'\n').count();
    let word_count = content.split_whitespace().count();
    let byte_count = content.as_bytes().len();
    let char_count = content.chars().count();

    let mut rows = Vec::new();
    let any_flags = flags.lines || flags.words || flags.bytes || flags.chars;
    if !any_flags || flags.lines {
        rows.push(format!("lines: {line_count}"));
    }
    if !any_flags || flags.words {
        rows.push(format!("words: {word_count}"));
    }
    if !any_flags || flags.bytes {
        rows.push(format!("bytes: {byte_count}"));
    }
    if !any_flags || flags.chars {
        rows.push(format!("chars: {char_count}"));
    }

    Ok(rows.join("\n") + "\n")
}

pub fn less_note(notes_dir: Option<PathBuf>, path: &Path) -> Result<Option<String>> {
    let root = canonical_notes_root(notes_dir)?;
    let target = resolve_existing_note_file(&root, path)?;

    let is_tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    if is_tty && std::env::var_os("PMAN_FORCE_CAT").is_none() {
        let status = Command::new("less")
            .arg(&target)
            .status()
            .with_context(|| format!("Failed to run less for {}", target.display()))?;
        if !status.success() {
            bail!("less exited with non-zero status");
        }
        Ok(None)
    } else {
        let content = fs::read_to_string(&target)
            .with_context(|| format!("Failed to read note {}", target.display()))?;
        Ok(Some(content))
    }
}

pub fn generate_skill(profile: &str) -> Result<String> {
    match profile {
        "project" => Ok(generate_project_skill()),
        _ => bail!("Unknown profile {profile}; supported profiles: project"),
    }
}

fn parse_line_range(spec: &str) -> Result<LineRange> {
    let mut parts = spec.splitn(2, ':');
    let start = parts
        .next()
        .context("Line range must be in start:end format")?
        .parse::<usize>()
        .context("Line range start must be a positive integer")?;
    let end = parts
        .next()
        .context("Line range must be in start:end format")?
        .parse::<usize>()
        .context("Line range end must be a positive integer")?;

    if start == 0 || end == 0 {
        bail!("Line range values are 1-based and must be greater than zero");
    }
    if end < start {
        bail!("Line range end must be greater than or equal to start");
    }

    Ok(LineRange { start, end })
}

fn ensure_range_in_bounds(range: LineRange, line_count: usize) -> Result<()> {
    if line_count == 0 {
        if range.start == 1 && range.end == 1 {
            return Ok(());
        }
        bail!(
            "Line range {}:{} is out of bounds for empty file",
            range.start,
            range.end
        );
    }

    if range.end > line_count {
        bail!(
            "Line range {}:{} is out of bounds for a {}-line file",
            range.start,
            range.end,
            line_count
        );
    }

    Ok(())
}

fn render_line_range(lines: &[String], range: LineRange, trailing_newline: bool) -> String {
    if lines.is_empty() {
        return String::new();
    }

    let mut output = lines[(range.start - 1)..range.end].join("\n");
    if trailing_newline {
        output.push('\n');
    }
    output
}

fn split_lines(content: &str) -> (Vec<String>, bool) {
    if content.is_empty() {
        return (Vec::new(), false);
    }

    let trailing_newline = content.ends_with('\n');
    let mut lines = content
        .split('\n')
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    if trailing_newline {
        lines.pop();
    }
    (lines, trailing_newline)
}

fn join_lines(lines: &[String], trailing_newline: bool) -> String {
    let mut output = lines.join("\n");
    if trailing_newline {
        output.push('\n');
    }
    output
}

fn read_note_lines(notes_dir: Option<PathBuf>, path: &Path) -> Result<(Vec<String>, bool)> {
    let root = canonical_notes_root(notes_dir)?;
    let target = resolve_existing_note_file(&root, path)?;
    let content = fs::read_to_string(&target)
        .with_context(|| format!("Failed to read note {}", target.display()))?;
    Ok(split_lines(&content))
}

fn canonical_notes_root(notes_dir: Option<PathBuf>) -> Result<PathBuf> {
    let root = resolve_notes_dir(notes_dir)?;
    let canonical = fs::canonicalize(&root)
        .with_context(|| format!("Failed to resolve notes root {}", root.display()))?;
    if !canonical.is_dir() {
        bail!("Notes root is not a directory: {}", canonical.display());
    }
    Ok(canonical)
}

fn resolve_existing_note_file(root: &Path, relative: &Path) -> Result<PathBuf> {
    if relative.is_absolute() {
        bail!(
            "Path must be relative to notes root: {}",
            relative.display()
        );
    }

    let joined = root.join(relative);
    let canonical = fs::canonicalize(&joined)
        .with_context(|| format!("Failed to resolve note path {}", joined.display()))?;
    ensure_contained(root, &canonical)?;

    if !canonical.is_file() {
        bail!("Path is not a file: {}", canonical.display());
    }

    Ok(canonical)
}

fn resolve_writable_note_file(root: &Path, relative: &Path, create_dirs: bool) -> Result<PathBuf> {
    if relative.is_absolute() {
        bail!(
            "Path must be relative to notes root: {}",
            relative.display()
        );
    }

    let name = relative
        .file_name()
        .context("Path must include a file name")?;
    let parent_rel = relative.parent().unwrap_or_else(|| Path::new(""));
    let joined_parent = root.join(parent_rel);

    if !joined_parent.exists() {
        if create_dirs {
            fs::create_dir_all(&joined_parent).with_context(|| {
                format!(
                    "Failed to create parent directories {}",
                    joined_parent.display()
                )
            })?;
        } else {
            bail!(
                "Parent directory does not exist: {} (use --create-dirs)",
                joined_parent.display()
            );
        }
    }

    let canonical_parent = fs::canonicalize(&joined_parent)
        .with_context(|| format!("Failed to resolve parent path {}", joined_parent.display()))?;
    ensure_contained(root, &canonical_parent)?;

    let target = canonical_parent.join(name);
    if target.exists() {
        let canonical_target = fs::canonicalize(&target)
            .with_context(|| format!("Failed to resolve note path {}", target.display()))?;
        ensure_contained(root, &canonical_target)?;
        if canonical_target.is_dir() {
            bail!("Path is a directory: {}", canonical_target.display());
        }
        return Ok(canonical_target);
    }

    Ok(target)
}

fn ensure_contained(root: &Path, target: &Path) -> Result<()> {
    if target.starts_with(root) {
        return Ok(());
    }
    bail!("Resolved path escapes notes root: {}", target.display())
}

fn generate_project_skill() -> String {
    PROJECT_SKILL.to_string()
}

pub fn create_project(
    paths: &NotesPaths,
    name: &str,
    status: &str,
    area: Option<&str>,
) -> Result<PathBuf> {
    ensure_registry(paths)?;

    let (project_id, dir_name, project_name, area_section) = if let Some(explicit_dir_name) =
        explicit_project_dir_name(name)?
    {
        if area.is_some() {
            bail!("--area is not supported with explicit project names");
        }
        if dir_name_in_use(paths, &explicit_dir_name) {
            bail!("Project already exists in Projects or Archives: {explicit_dir_name}");
        }
        (
            project_id_from_dir(&explicit_dir_name)?,
            explicit_dir_name.clone(),
            explicit_dir_name,
            String::new(),
        )
    } else {
        let registry_contents = fs::read_to_string(&paths.registry)
            .with_context(|| format!("Failed to read registry at {}", paths.registry.display()))?;
        let next_id = next_project_id(&registry_contents);
        let slug = slugify(name)?;
        let area_slug = area.map(slugify).transpose()?;
        let project_prefix = project_dir_prefix();

        let slug_full = match area_slug.as_deref() {
            Some(area_value) => format!("{area_value}-{slug}"),
            None => slug.clone(),
        };

        if slug_in_use(paths, &slug_full)? {
            bail!("Slug already exists in Projects or Archives: {slug_full}");
        }

        let dir_name = format!("{project_prefix}-{next_id}-{slug_full}");
        let area_section = area_slug
            .as_deref()
            .map(|value| format!("area: {value}\n"))
            .unwrap_or_default();
        (
            format!("PROJ-{next_id}"),
            dir_name,
            name.to_string(),
            area_section,
        )
    };

    let note_dir = paths.projects_dir.join(&dir_name);
    let note_path = note_dir.join("README.md");

    if note_path.exists() {
        bail!("Project note already exists: {}", note_path.display());
    }

    fs::create_dir_all(&note_dir)
        .with_context(|| format!("Failed to create project directory {}", note_dir.display()))?;

    let created = Local::now().format("%Y-%m-%d");
    let content = format!(
        "---\nstatus: {status}\n{area}---\n\n# {id}: {name}\n\n**Created**: {created}\n\n## Summary\n- \n\n## Notes\n- \n\n## Next\n- \n",
        id = project_id,
        name = project_name,
        created = created,
        status = status,
        area = area_section
    );

    fs::write(&note_path, content)
        .with_context(|| format!("Failed to write note {}", note_path.display()))?;

    let registry_line = format!(
        "| {id} | {name} | {status} | {created} | [{dir}/README.md]({dir}/README.md) |",
        id = project_id,
        name = project_name,
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

pub fn list_projects(paths: &NotesPaths, status: Option<&str>) -> Result<String> {
    let contents = fs::read_to_string(&paths.registry)
        .with_context(|| format!("Failed to read registry at {}", paths.registry.display()))?;
    let wanted_status = status.map(|value| value.to_ascii_lowercase());
    let mut rows = Vec::new();

    for line in contents.lines() {
        if !line.starts_with("| ") || line.starts_with("| ---") {
            continue;
        }

        let parts = line
            .trim_matches('|')
            .split('|')
            .map(|part| part.trim().to_string())
            .collect::<Vec<String>>();
        if parts.len() < 5 {
            continue;
        }

        if let Some(wanted) = &wanted_status {
            if parts[2].to_ascii_lowercase() != *wanted {
                continue;
            }
        }

        rows.push(format!(
            "{}\t{}\t{}\t{}",
            parts[0], parts[2], parts[1], parts[4]
        ));
    }

    if rows.is_empty() {
        return Ok("No projects found.\n".to_string());
    }

    Ok(rows.join("\n") + "\n")
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

    let proj_id = project_id_from_registry_note_path(&paths.registry, dir_name)?;
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

fn explicit_project_dir_name(input: &str) -> Result<Option<String>> {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.chars().any(|ch| ch.is_whitespace()) {
        return Ok(None);
    }

    let normalized = slugify(trimmed)?;
    if !normalized.contains('-') {
        return Ok(None);
    }

    Ok(Some(normalized))
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

fn dir_name_in_use(paths: &NotesPaths, dir_name: &str) -> bool {
    paths.projects_dir.join(dir_name).exists()
        || paths.archives_projects_dir.join(dir_name).exists()
}

fn slug_in_use(paths: &NotesPaths, slug: &str) -> Result<bool> {
    if has_slug_in_dir(&paths.projects_dir, slug)? {
        return Ok(true);
    }

    if paths.archives_projects_dir.exists() && has_slug_in_dir(&paths.archives_projects_dir, slug)?
    {
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
    let mut parts = name.splitn(3, '-');
    let prefix = parts.next().unwrap_or_default();
    let id = parts.next().unwrap_or_default();
    let tail = parts.next().unwrap_or_default();

    if prefix.is_empty() || !prefix.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return false;
    }

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
    if !dir_name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    {
        bail!("Invalid project directory name: {dir_name}");
    }

    let mut parts = dir_name.splitn(3, '-');
    let prefix = parts.next().unwrap_or_default();
    let id = parts.next().unwrap_or_default();
    let tail = parts.next().unwrap_or_default();

    if prefix.is_empty() || !prefix.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        bail!("Invalid project directory name: {dir_name}");
    }

    if id.is_empty() {
        bail!("Invalid project directory name: {dir_name}");
    }

    if id.chars().all(|ch| ch.is_ascii_digit()) && !tail.is_empty() {
        return Ok(format!("{}-{}", prefix.to_ascii_uppercase(), id));
    }

    Ok(dir_name.to_ascii_uppercase())
}

fn project_id_from_registry_note_path(registry: &Path, dir_name: &str) -> Result<String> {
    let contents = fs::read_to_string(registry)
        .with_context(|| format!("Failed to read registry {}", registry.display()))?;

    for line in contents.lines() {
        if !line.starts_with("| ") || line.starts_with("| ---") {
            continue;
        }
        let parts = line
            .trim_matches('|')
            .split('|')
            .map(|part| part.trim().to_string())
            .collect::<Vec<String>>();
        if parts.len() < 5 {
            continue;
        }

        if registry_note_cell_matches_dir_name(&parts[4], dir_name) {
            return Ok(parts[0].clone());
        }
    }

    bail!("Registry entry not found for project directory {dir_name}")
}

fn registry_note_cell_matches_dir_name(cell: &str, dir_name: &str) -> bool {
    let Some(link_start) = cell.find("](") else {
        return false;
    };
    let rest = &cell[(link_start + 2)..];
    let Some(link_end) = rest.find(')') else {
        return false;
    };
    let target = &rest[..link_end];
    let trimmed = target.trim_start_matches("./");
    let trimmed = trimmed.strip_suffix("/README.md").unwrap_or(trimmed);
    let trimmed = trimmed.trim_end_matches('/');
    let actual = trimmed.rsplit('/').next().unwrap_or_default();
    actual == dir_name
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

fn parse_bool_env(value: &str) -> Option<bool> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn forced_tool_presence(tool: &str) -> Option<bool> {
    let env_name = match tool {
        "claude" => FORCE_CLAUDE_PRESENT_ENV_VAR,
        "codex" => FORCE_CODEX_PRESENT_ENV_VAR,
        _ => return None,
    };
    std::env::var(env_name)
        .ok()
        .and_then(|value| parse_bool_env(&value))
}

fn is_tool_available(tool: &str) -> bool {
    if let Some(forced) = forced_tool_presence(tool) {
        return forced;
    }

    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };

    #[cfg(windows)]
    let exts: Vec<std::ffi::OsString> = std::env::var_os("PATHEXT")
        .map(|value| {
            value
                .to_string_lossy()
                .split(';')
                .filter(|part| !part.is_empty())
                .map(std::ffi::OsString::from)
                .collect()
        })
        .unwrap_or_else(|| vec![".EXE".into(), ".CMD".into(), ".BAT".into()]);

    for dir in std::env::split_paths(&path_var) {
        #[cfg(unix)]
        {
            if dir.join(tool).is_file() {
                return true;
            }
        }

        #[cfg(windows)]
        {
            if dir.join(tool).is_file() {
                return true;
            }
            for ext in &exts {
                if dir
                    .join(format!("{tool}{}", ext.to_string_lossy()))
                    .is_file()
                {
                    return true;
                }
            }
        }
    }
    false
}

fn canonical_skill_dir(workspace: &Path) -> PathBuf {
    workspace.join(".pman").join("skills").join("project")
}

fn canonical_skill_file(workspace: &Path) -> PathBuf {
    canonical_skill_dir(workspace).join("SKILL.md")
}

fn claude_skill_link(workspace: &Path) -> PathBuf {
    workspace.join(".claude").join("skills").join("project")
}

fn codex_skill_link(workspace: &Path) -> PathBuf {
    workspace.join(".codex").join("skills").join("project")
}

#[cfg(unix)]
fn create_dir_symlink(target: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link).with_context(|| {
        format!(
            "Failed to create symlink {} -> {}",
            link.display(),
            target.display()
        )
    })
}

#[cfg(windows)]
fn create_dir_symlink(target: &Path, link: &Path) -> Result<()> {
    std::os::windows::fs::symlink_dir(target, link).with_context(|| {
        format!(
            "Failed to create symlink {} -> {}",
            link.display(),
            target.display()
        )
    })
}

#[cfg(unix)]
fn create_file_symlink(target: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link).with_context(|| {
        format!(
            "Failed to create symlink {} -> {}",
            link.display(),
            target.display()
        )
    })
}

#[cfg(windows)]
fn create_file_symlink(target: &Path, link: &Path) -> Result<()> {
    std::os::windows::fs::symlink_file(target, link).with_context(|| {
        format!(
            "Failed to create symlink {} -> {}",
            link.display(),
            target.display()
        )
    })
}

fn remove_existing_path(path: &Path) -> Result<()> {
    let metadata =
        fs::symlink_metadata(path).with_context(|| format!("Failed to stat {}", path.display()))?;
    let kind = metadata.file_type();
    if kind.is_symlink() || kind.is_file() {
        fs::remove_file(path).with_context(|| format!("Failed to remove {}", path.display()))?;
    } else if kind.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("Failed to remove {}", path.display()))?;
    } else {
        fs::remove_file(path).with_context(|| format!("Failed to remove {}", path.display()))?;
    }
    Ok(())
}

fn symlink_matches(link: &Path, expected_target: &Path) -> Result<bool> {
    let metadata = match fs::symlink_metadata(link) {
        Ok(value) => value,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err).with_context(|| format!("Failed to stat {}", link.display())),
    };
    if !metadata.file_type().is_symlink() {
        return Ok(false);
    }
    let target = fs::read_link(link)
        .with_context(|| format!("Failed to read symlink {}", link.display()))?;
    let resolved = if target.is_absolute() {
        target
    } else {
        link.parent().unwrap_or_else(|| Path::new("")).join(target)
    };
    Ok(resolved == expected_target)
}

fn ensure_file(path: &Path, content: &str, overwrite: bool) -> Result<bool> {
    if path.exists() && !overwrite {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(true)
}

fn ensure_symlink(
    target: &Path,
    link: &Path,
    replace_existing: bool,
    is_dir: bool,
) -> Result<bool> {
    if symlink_matches(link, target)? {
        return Ok(false);
    }

    if link.exists() || fs::symlink_metadata(link).is_ok() {
        if !replace_existing {
            return Ok(false);
        }
        remove_existing_path(link)?;
    }

    if let Some(parent) = link.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    if is_dir {
        create_dir_symlink(target, link)?;
    } else {
        create_file_symlink(target, link)?;
    }
    Ok(true)
}

/// Initialize a new pman workspace at the given path.
/// Creates Notes directory structure, AGENTS.md, and canonical pman skills.
/// Skips any file or directory that already exists.
pub fn init_workspace(workspace: &Path) -> Result<()> {
    println!("Initializing pman workspace at {}", workspace.display());

    let notes_dirs = [
        workspace.join("Notes").join("Projects"),
        workspace.join("Notes").join("Areas"),
        workspace.join("Notes").join("Resources"),
        workspace.join("Notes").join("Archives").join("Projects"),
    ];

    for dir in &notes_dirs {
        if dir.exists() {
            println!(
                "  skip: {} (exists)",
                dir.strip_prefix(workspace).unwrap_or(dir).display()
            );
        } else {
            fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create {}", dir.display()))?;
            println!(
                "  create: {}",
                dir.strip_prefix(workspace).unwrap_or(dir).display()
            );
        }
    }

    let registry = workspace
        .join("Notes")
        .join("Projects")
        .join("_registry.md");
    if registry.exists() {
        println!(
            "  skip: {} (exists)",
            registry
                .strip_prefix(workspace)
                .unwrap_or(&registry)
                .display()
        );
    } else {
        fs::write(&registry, REGISTRY_HEADER)
            .with_context(|| format!("Failed to create {}", registry.display()))?;
        println!(
            "  create: {}",
            registry
                .strip_prefix(workspace)
                .unwrap_or(&registry)
                .display()
        );
    }

    let agents_md = workspace.join("AGENTS.md");
    if ensure_file(&agents_md, AGENTS_MD, false)? {
        println!(
            "  create: {}",
            agents_md
                .strip_prefix(workspace)
                .unwrap_or(&agents_md)
                .display()
        );
    } else {
        println!(
            "  skip: {} (exists)",
            agents_md
                .strip_prefix(workspace)
                .unwrap_or(&agents_md)
                .display()
        );
    }

    if is_tool_available("claude") {
        let claude_md = workspace.join("CLAUDE.md");
        let changed = ensure_symlink(&agents_md, &claude_md, false, false)?;
        println!(
            "  {}: {}",
            if changed { "create" } else { "skip" },
            claude_md
                .strip_prefix(workspace)
                .unwrap_or(&claude_md)
                .display()
        );
    }

    let canonical_skill_file = canonical_skill_file(workspace);
    if ensure_file(&canonical_skill_file, PROJECT_SKILL, false)? {
        println!(
            "  create: {}",
            canonical_skill_file
                .strip_prefix(workspace)
                .unwrap_or(&canonical_skill_file)
                .display()
        );
    } else {
        println!(
            "  skip: {} (exists)",
            canonical_skill_file
                .strip_prefix(workspace)
                .unwrap_or(&canonical_skill_file)
                .display()
        );
    }

    let canonical_skill_dir = canonical_skill_dir(workspace);
    if is_tool_available("claude") {
        let link = claude_skill_link(workspace);
        let changed = ensure_symlink(&canonical_skill_dir, &link, false, true)?;
        println!(
            "  {}: {}",
            if changed { "create" } else { "skip" },
            link.strip_prefix(workspace).unwrap_or(&link).display()
        );
    }

    if is_tool_available("codex") {
        let link = codex_skill_link(workspace);
        let changed = ensure_symlink(&canonical_skill_dir, &link, false, true)?;
        println!(
            "  {}: {}",
            if changed { "create" } else { "skip" },
            link.strip_prefix(workspace).unwrap_or(&link).display()
        );
    }

    println!("\nWorkspace initialized. Create a README.md with your custom configuration.");
    Ok(())
}

/// Verify workspace setup and report any issues.
/// Returns true if all checks pass, false otherwise.
pub fn verify_workspace(workspace: &Path) -> Result<bool> {
    println!("Verifying pman workspace at {}", workspace.display());

    let mut all_ok = true;
    let notes_dirs = [
        ("Notes/Projects", workspace.join("Notes").join("Projects")),
        ("Notes/Areas", workspace.join("Notes").join("Areas")),
        ("Notes/Resources", workspace.join("Notes").join("Resources")),
        (
            "Notes/Archives/Projects",
            workspace.join("Notes").join("Archives").join("Projects"),
        ),
    ];

    for (name, path) in &notes_dirs {
        if path.exists() {
            println!("  ✓ {}", name);
        } else {
            println!("  ✗ {} (missing)", name);
            all_ok = false;
        }
    }

    let registry = workspace
        .join("Notes")
        .join("Projects")
        .join("_registry.md");
    if registry.exists() {
        println!("  ✓ Notes/Projects/_registry.md");
    } else {
        println!("  ✗ Notes/Projects/_registry.md (missing)");
        all_ok = false;
    }

    let agents_md = workspace.join("AGENTS.md");
    if agents_md.exists() {
        println!("  ✓ AGENTS.md");
    } else {
        println!("  ✗ AGENTS.md (missing)");
        all_ok = false;
    }

    let canonical_skill_file = canonical_skill_file(workspace);
    if canonical_skill_file.exists() {
        println!("  ✓ .pman/skills/project/SKILL.md");
    } else {
        println!("  ✗ .pman/skills/project/SKILL.md (missing)");
        all_ok = false;
    }

    if is_tool_available("claude") {
        let claude_md = workspace.join("CLAUDE.md");
        if symlink_matches(&claude_md, &agents_md)? {
            println!("  ✓ CLAUDE.md -> AGENTS.md");
        } else {
            println!("  ✗ CLAUDE.md -> AGENTS.md (missing or incorrect symlink)");
            all_ok = false;
        }

        let link = claude_skill_link(workspace);
        let target = canonical_skill_dir(workspace);
        if symlink_matches(&link, &target)? {
            println!("  ✓ .claude/skills/project -> .pman/skills/project");
        } else {
            println!(
                "  ✗ .claude/skills/project -> .pman/skills/project (missing or incorrect symlink)"
            );
            all_ok = false;
        }
    }

    if is_tool_available("codex") {
        let link = codex_skill_link(workspace);
        let target = canonical_skill_dir(workspace);
        if symlink_matches(&link, &target)? {
            println!("  ✓ .codex/skills/project -> .pman/skills/project");
        } else {
            println!(
                "  ✗ .codex/skills/project -> .pman/skills/project (missing or incorrect symlink)"
            );
            all_ok = false;
        }
    }

    if all_ok {
        println!("\nWorkspace OK.");
    } else {
        println!(
            "\nIssues found. Run 'pman init' to create missing directories, or 'pman update' to restore AGENTS.md and skills."
        );
    }

    Ok(all_ok)
}

/// Update AGENTS.md and skills to the latest embedded versions.
/// Always overwrites canonical files and refreshes symlink bridges for installed agents.
pub fn update_workspace(workspace: &Path) -> Result<()> {
    println!("Updating pman resources at {}", workspace.display());

    let agents_md = workspace.join("AGENTS.md");
    ensure_file(&agents_md, AGENTS_MD, true)?;
    println!("  update: AGENTS.md");

    let canonical_skill_file = canonical_skill_file(workspace);
    ensure_file(&canonical_skill_file, PROJECT_SKILL, true)?;
    println!("  update: .pman/skills/project/SKILL.md");

    let canonical_skill_dir = canonical_skill_dir(workspace);
    if is_tool_available("claude") {
        let claude_md = workspace.join("CLAUDE.md");
        ensure_symlink(&agents_md, &claude_md, true, false)?;
        println!("  update: CLAUDE.md -> AGENTS.md");

        let link = claude_skill_link(workspace);
        ensure_symlink(&canonical_skill_dir, &link, true, true)?;
        println!("  update: .claude/skills/project -> .pman/skills/project");
    }

    if is_tool_available("codex") {
        let link = codex_skill_link(workspace);
        ensure_symlink(&canonical_skill_dir, &link, true, true)?;
        println!("  update: .codex/skills/project -> .pman/skills/project");
    }

    println!(
        "\nResources updated to pman v{}.",
        env!("CARGO_PKG_VERSION")
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    struct NotesDirEnvGuard {
        previous: Option<OsString>,
    }

    impl NotesDirEnvGuard {
        fn set(value: &Path) -> Self {
            let previous = std::env::var_os(NOTES_DIR_ENV_VAR);
            // SAFETY: Tests mutate process-global environment only while holding
            // `notes_env_lock`, so no concurrent env mutation occurs in this module.
            unsafe { std::env::set_var(NOTES_DIR_ENV_VAR, value) };
            Self { previous }
        }
    }

    struct ProjectPrefixEnvGuard {
        previous: Option<OsString>,
    }

    impl ProjectPrefixEnvGuard {
        fn set(value: &str) -> Self {
            let previous = std::env::var_os("PMAN_PROJECT_PREFIX");
            // SAFETY: Tests mutate process-global environment only while holding
            // `notes_env_lock`, so no concurrent env mutation occurs in this module.
            unsafe { std::env::set_var("PMAN_PROJECT_PREFIX", value) };
            Self { previous }
        }
    }

    struct EnvVarGuard {
        name: &'static str,
        previous: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set(name: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(name);
            // SAFETY: Tests mutate process-global environment only while holding
            // `notes_env_lock`, so no concurrent env mutation occurs in this module.
            unsafe { std::env::set_var(name, value) };
            Self { name, previous }
        }
    }

    impl Drop for NotesDirEnvGuard {
        fn drop(&mut self) {
            // SAFETY: The lock held by each test serializes environment writes.
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var(NOTES_DIR_ENV_VAR, value),
                    None => std::env::remove_var(NOTES_DIR_ENV_VAR),
                }
            }
        }
    }

    impl Drop for ProjectPrefixEnvGuard {
        fn drop(&mut self) {
            // SAFETY: The lock held by each test serializes environment writes.
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var("PMAN_PROJECT_PREFIX", value),
                    None => std::env::remove_var("PMAN_PROJECT_PREFIX"),
                }
            }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            // SAFETY: The lock held by each test serializes environment writes.
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var(self.name, value),
                    None => std::env::remove_var(self.name),
                }
            }
        }
    }

    fn notes_env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn setup_notes_root() -> (tempfile::TempDir, PathBuf) {
        let temp = tempdir().unwrap();
        let root = temp.path().join("Notes");
        fs::create_dir_all(&root).unwrap();
        (temp, root)
    }

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
        let _lock = notes_env_lock();
        let _prefix_guard = ProjectPrefixEnvGuard::set("proj");
        let temp = tempdir().unwrap();
        let root = temp.path();
        let paths = NotesPaths::from_root(root.to_path_buf());
        fs::create_dir_all(&paths.projects_dir).unwrap();
        fs::write(&paths.registry, REGISTRY_HEADER).unwrap();

        let note_path = create_project(&paths, "Runes Notes", "active", Some("religion")).unwrap();

        assert!(
            note_path
                .to_string_lossy()
                .contains("proj-1-religion-runes-notes/README.md")
        );
    }

    #[test]
    fn create_project_uses_configured_prefix_from_env() {
        let _lock = notes_env_lock();
        let _prefix_guard = ProjectPrefixEnvGuard::set("ticket");
        let temp = tempdir().unwrap();
        let root = temp.path();
        let paths = NotesPaths::from_root(root.to_path_buf());
        fs::create_dir_all(&paths.projects_dir).unwrap();
        fs::write(&paths.registry, REGISTRY_HEADER).unwrap();

        let note_path = create_project(&paths, "Runes Notes", "active", None).unwrap();
        assert!(
            note_path
                .to_string_lossy()
                .contains("ticket-1-runes-notes/README.md")
        );
    }

    #[test]
    fn create_project_uses_default_prefix_when_env_empty() {
        let _lock = notes_env_lock();
        let _prefix_guard = ProjectPrefixEnvGuard::set("");
        let temp = tempdir().unwrap();
        let root = temp.path();
        let paths = NotesPaths::from_root(root.to_path_buf());
        fs::create_dir_all(&paths.projects_dir).unwrap();
        fs::write(&paths.registry, REGISTRY_HEADER).unwrap();

        let note_path = create_project(&paths, "Runes Notes", "active", None).unwrap();
        assert!(
            note_path
                .to_string_lossy()
                .contains("proj-1-runes-notes/README.md")
        );
    }

    #[test]
    fn create_project_accepts_explicit_dir_name() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        let paths = NotesPaths::from_root(root.to_path_buf());
        fs::create_dir_all(&paths.projects_dir).unwrap();
        fs::write(&paths.registry, REGISTRY_HEADER).unwrap();

        let note_path = create_project(&paths, "myslug-1192-mythingy", "active", None).unwrap();
        assert!(
            note_path
                .to_string_lossy()
                .contains("myslug-1192-mythingy/README.md")
        );

        let note = fs::read_to_string(&note_path).unwrap();
        assert!(note.starts_with("---\nstatus: active\n---"));
        assert!(note.contains("# MYSLUG-1192: myslug-1192-mythingy"));

        let registry = fs::read_to_string(&paths.registry).unwrap();
        assert!(registry.contains("| MYSLUG-1192 | myslug-1192-mythingy | active |"));
    }

    #[test]
    fn create_project_accepts_non_numeric_explicit_dir_name() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        let paths = NotesPaths::from_root(root.to_path_buf());
        fs::create_dir_all(&paths.projects_dir).unwrap();
        fs::write(&paths.registry, REGISTRY_HEADER).unwrap();

        let note_path = create_project(&paths, "z2222-lol-cats\n", "active", None).unwrap();
        assert!(
            note_path
                .to_string_lossy()
                .contains("z2222-lol-cats/README.md")
        );

        let note = fs::read_to_string(&note_path).unwrap();
        assert!(note.starts_with("---\nstatus: active\n---"));
        assert!(note.contains("# Z2222-LOL-CATS: z2222-lol-cats"));

        let registry = fs::read_to_string(&paths.registry).unwrap();
        assert!(registry.contains("| Z2222-LOL-CATS | z2222-lol-cats | active |"));
    }

    #[test]
    fn create_project_rejects_area_for_explicit_dir_name() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        let paths = NotesPaths::from_root(root.to_path_buf());
        fs::create_dir_all(&paths.projects_dir).unwrap();
        fs::write(&paths.registry, REGISTRY_HEADER).unwrap();

        let err = create_project(&paths, "myslug-1192-mythingy", "active", Some("ops"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("--area is not supported"));
    }

    #[test]
    fn archive_project_updates_registry_for_explicit_dir_name() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        let paths = NotesPaths::from_root(root.to_path_buf());
        fs::create_dir_all(&paths.projects_dir).unwrap();
        fs::create_dir_all(&paths.archives_projects_dir).unwrap();

        let proj_dir = paths.projects_dir.join("myslug-1192-mythingy");
        fs::create_dir_all(&proj_dir).unwrap();
        fs::write(proj_dir.join("README.md"), "test").unwrap();

        let registry = format!(
            "{header}| MYSLUG-1192 | myslug-1192-mythingy | active | 2026-02-14 | [myslug-1192-mythingy/README.md](myslug-1192-mythingy/README.md) |\n",
            header = REGISTRY_HEADER
        );
        fs::write(&paths.registry, registry).unwrap();

        archive_project(&paths, "myslug-1192").unwrap();

        let updated = fs::read_to_string(&paths.registry).unwrap();
        assert!(updated.contains("| MYSLUG-1192 | myslug-1192-mythingy | archived |"));
        assert!(updated.contains("../Archives/Projects/myslug-1192-mythingy/README.md"));
    }

    #[test]
    fn archive_project_updates_registry_for_non_numeric_explicit_dir_name() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        let paths = NotesPaths::from_root(root.to_path_buf());
        fs::create_dir_all(&paths.projects_dir).unwrap();
        fs::create_dir_all(&paths.archives_projects_dir).unwrap();

        let proj_dir = paths.projects_dir.join("z2222-lol-cats");
        fs::create_dir_all(&proj_dir).unwrap();
        fs::write(proj_dir.join("README.md"), "test").unwrap();

        let registry = format!(
            "{header}| Z2222-LOL-CATS | z2222-lol-cats | active | 2026-02-14 | [z2222-lol-cats/README.md](z2222-lol-cats/README.md) |\n",
            header = REGISTRY_HEADER
        );
        fs::write(&paths.registry, registry).unwrap();

        archive_project(&paths, "z2222").unwrap();

        let updated = fs::read_to_string(&paths.registry).unwrap();
        assert!(updated.contains("| Z2222-LOL-CATS | z2222-lol-cats | archived |"));
        assert!(updated.contains("../Archives/Projects/z2222-lol-cats/README.md"));
    }

    #[test]
    fn list_projects_filters_by_status() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        let paths = NotesPaths::from_root(root.to_path_buf());
        fs::create_dir_all(&paths.projects_dir).unwrap();

        let registry = format!(
            "{header}| PROJ-1 | One | active | 2026-02-14 | [proj-1-one/README.md](proj-1-one/README.md) |\n| PROJ-2 | Two | archived | 2026-02-14 | [../Archives/Projects/proj-2-two/README.md](../Archives/Projects/proj-2-two/README.md) |\n",
            header = REGISTRY_HEADER
        );
        fs::write(&paths.registry, registry).unwrap();

        let active = list_projects(&paths, Some("active")).unwrap();
        assert!(active.contains("PROJ-1\tactive\tOne"));
        assert!(!active.contains("PROJ-2"));

        let all = list_projects(&paths, None).unwrap();
        assert!(all.contains("PROJ-1\tactive\tOne"));
        assert!(all.contains("PROJ-2\tarchived\tTwo"));
    }

    #[test]
    fn init_workspace_writes_agents_and_canonical_skill_without_agent_bridges() {
        let _lock = notes_env_lock();
        let _claude = EnvVarGuard::set(FORCE_CLAUDE_PRESENT_ENV_VAR, "0");
        let _codex = EnvVarGuard::set(FORCE_CODEX_PRESENT_ENV_VAR, "0");
        let temp = tempdir().unwrap();
        let workspace = temp.path();

        init_workspace(workspace).unwrap();

        assert!(workspace.join("AGENTS.md").exists());
        assert!(workspace.join(".pman/skills/project/SKILL.md").exists());
        assert!(!workspace.join("CLAUDE.md").exists());
        assert!(!workspace.join(".claude/skills/project").exists());
        assert!(!workspace.join(".codex/skills/project").exists());
    }

    #[cfg(unix)]
    #[test]
    fn init_workspace_creates_agent_symlink_bridges_when_tools_present() {
        let _lock = notes_env_lock();
        let _claude = EnvVarGuard::set(FORCE_CLAUDE_PRESENT_ENV_VAR, "1");
        let _codex = EnvVarGuard::set(FORCE_CODEX_PRESENT_ENV_VAR, "1");
        let temp = tempdir().unwrap();
        let workspace = temp.path();

        init_workspace(workspace).unwrap();

        let agents_md = workspace.join("AGENTS.md");
        let canonical_dir = canonical_skill_dir(workspace);
        assert!(symlink_matches(&workspace.join("CLAUDE.md"), &agents_md).unwrap());
        assert!(
            symlink_matches(&workspace.join(".claude/skills/project"), &canonical_dir).unwrap()
        );
        assert!(symlink_matches(&workspace.join(".codex/skills/project"), &canonical_dir).unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn update_workspace_replaces_existing_agent_bridge_paths() {
        let _lock = notes_env_lock();
        let _claude = EnvVarGuard::set(FORCE_CLAUDE_PRESENT_ENV_VAR, "1");
        let _codex = EnvVarGuard::set(FORCE_CODEX_PRESENT_ENV_VAR, "1");
        let temp = tempdir().unwrap();
        let workspace = temp.path();

        fs::write(workspace.join("CLAUDE.md"), "old").unwrap();
        fs::create_dir_all(workspace.join(".claude/skills/project")).unwrap();
        fs::write(workspace.join(".claude/skills/project/SKILL.md"), "old").unwrap();
        fs::create_dir_all(workspace.join(".codex/skills/project")).unwrap();
        fs::write(workspace.join(".codex/skills/project/SKILL.md"), "old").unwrap();

        update_workspace(workspace).unwrap();

        let agents_md = workspace.join("AGENTS.md");
        let canonical_dir = canonical_skill_dir(workspace);
        assert!(symlink_matches(&workspace.join("CLAUDE.md"), &agents_md).unwrap());
        assert!(
            symlink_matches(&workspace.join(".claude/skills/project"), &canonical_dir).unwrap()
        );
        assert!(symlink_matches(&workspace.join(".codex/skills/project"), &canonical_dir).unwrap());
        assert_eq!(
            fs::read_to_string(workspace.join(".pman/skills/project/SKILL.md")).unwrap(),
            PROJECT_SKILL
        );
    }

    #[cfg(unix)]
    #[test]
    fn verify_workspace_checks_agent_bridges_when_tools_present() {
        let _lock = notes_env_lock();
        let _claude = EnvVarGuard::set(FORCE_CLAUDE_PRESENT_ENV_VAR, "1");
        let _codex = EnvVarGuard::set(FORCE_CODEX_PRESENT_ENV_VAR, "1");
        let temp = tempdir().unwrap();
        let workspace = temp.path();

        init_workspace(workspace).unwrap();
        assert!(verify_workspace(workspace).unwrap());
    }

    #[test]
    fn resolve_notes_dir_uses_env_var_when_flag_not_set() {
        let _lock = notes_env_lock();
        let temp = tempdir().unwrap();
        let env_notes = temp.path().join("EnvNotes");
        fs::create_dir_all(&env_notes).unwrap();
        let _guard = NotesDirEnvGuard::set(&env_notes);

        let resolved = resolve_notes_dir(None).unwrap();
        assert_eq!(resolved, env_notes);
    }

    #[test]
    fn resolve_notes_dir_prefers_explicit_flag_over_env_var() {
        let _lock = notes_env_lock();
        let temp = tempdir().unwrap();
        let env_notes = temp.path().join("EnvNotes");
        let explicit_notes = temp.path().join("ExplicitNotes");
        fs::create_dir_all(&env_notes).unwrap();
        let _guard = NotesDirEnvGuard::set(&env_notes);

        let resolved = resolve_notes_dir(Some(explicit_notes.clone())).unwrap();
        assert_eq!(resolved, explicit_notes);
    }

    #[test]
    fn read_note_supports_numbered_ranges() {
        let (_temp, root) = setup_notes_root();
        let file = root.join("Projects").join("sample.md");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "alpha\nbeta\ngamma\n").unwrap();

        let output = read_note(
            Some(root),
            Path::new("Projects/sample.md"),
            Some("2:3"),
            true,
        )
        .unwrap();
        assert_eq!(output, "     2\tbeta\n     3\tgamma\n");
    }

    #[test]
    fn edit_note_enforces_expected_text_guard() {
        let (_temp, root) = setup_notes_root();
        let file = root.join("Projects").join("sample.md");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "one\ntwo\nthree\n").unwrap();

        let err = edit_note(
            Some(root.clone()),
            Path::new("Projects/sample.md"),
            "2:2",
            "updated",
            Some("wrong\n"),
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("Expected text mismatch"));

        edit_note(
            Some(root.clone()),
            Path::new("Projects/sample.md"),
            "2:2",
            "updated",
            Some("two\n"),
        )
        .unwrap();
        let updated = fs::read_to_string(&file).unwrap();
        assert_eq!(updated, "one\nupdated\nthree\n");
    }

    #[test]
    fn edit_note_replaces_empty_file_range() {
        let (_temp, root) = setup_notes_root();
        let file = root.join("Resources").join("empty.md");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "").unwrap();

        edit_note(
            Some(root.clone()),
            Path::new("Resources/empty.md"),
            "1:1",
            "hello\nworld\n",
            None,
        )
        .unwrap();

        let updated = fs::read_to_string(&file).unwrap();
        assert_eq!(updated, "hello\nworld\n");
    }

    #[test]
    fn write_note_creates_dirs_when_requested() {
        let (_temp, root) = setup_notes_root();
        let target_rel = Path::new("Areas/team/notes.md");

        write_note(Some(root.clone()), target_rel, "body", true).unwrap();
        let updated = fs::read_to_string(root.join(target_rel)).unwrap();
        assert_eq!(updated, "body");
    }

    #[cfg(unix)]
    #[test]
    fn write_note_blocks_symlink_escape() {
        use std::os::unix::fs::symlink;

        let (_temp, root) = setup_notes_root();
        let outside = root.parent().unwrap().join("outside");
        fs::create_dir_all(&outside).unwrap();
        let link = root.join("escape");
        symlink(&outside, &link).unwrap();

        let err = write_note(Some(root), Path::new("escape/evil.md"), "bad", false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("escapes notes root"));
    }

    #[test]
    fn wc_note_reports_selected_counts() {
        let (_temp, root) = setup_notes_root();
        let file = root.join("Projects").join("wc.md");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "a bb\nccc\n").unwrap();

        let output = wc_note(
            Some(root),
            Path::new("Projects/wc.md"),
            WcFlags {
                words: true,
                ..WcFlags::default()
            },
        )
        .unwrap();
        assert_eq!(output, "words: 3\n");
    }

    #[test]
    fn less_note_falls_back_to_cat_without_tty() {
        let (_temp, root) = setup_notes_root();
        let file = root.join("Projects").join("less.md");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "hello\n").unwrap();

        // SAFETY: this test runs single-threaded; no other thread reads PMAN_FORCE_CAT.
        unsafe { std::env::set_var("PMAN_FORCE_CAT", "1") };
        let output = less_note(Some(root), Path::new("Projects/less.md")).unwrap();
        unsafe { std::env::remove_var("PMAN_FORCE_CAT") };
        assert_eq!(output.as_deref(), Some("hello\n"));
    }

    #[test]
    fn generate_skill_supports_para_notes() {
        let output = generate_skill("project").unwrap();
        assert!(output.contains("name: project"));
        assert!(output.contains("pman read"));
        assert!(output.contains("pman edit"));
    }
}
