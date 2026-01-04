# pman CLI Reference

This document covers the CLI behavior. The workflow manual lives in the README.

## Core concepts

- A **project** is a time-bound effort (feature, bugfix, refactor)—not a repository. A repo may have many projects; a project may touch multiple repos.
- The **registry** (`Notes/Projects/_registry.md`) is the authoritative index of all active and archived projects.
- Every change belongs to a project. The workflow: create a project note → plan collaboratively with the model → execute code changes once the plan is complete.

## Install

```sh
cargo install --git https://github.com/divanvisagie/pman
```

Then initialize a workspace:

```sh
cd ~/src  # or any directory you want as workspace root
pman init
```

## Commands

### init

Initialize a new pman workspace.

```sh
pman init              # current directory
pman init ~/src        # specific path
```

Creates:
- `Notes/Projects/`, `Notes/Areas/`, `Notes/Resources/`, `Notes/Archives/Projects/`
- `Notes/Projects/_registry.md` with header template
- `CLAUDE.md` (generic workflow rules)
- `.claude/skills/para-notes/SKILL.md`
- `.claude/skills/project-structure/SKILL.md`

Behavior:
- Skips any file or directory that already exists (never overwrites)
- Safe to run multiple times

### update

Update CLAUDE.md and skills to the versions embedded in your pman binary.

```sh
pman update              # current directory
pman update --path ~/src # specific path
```

Updates:
- `CLAUDE.md`
- `.claude/skills/para-notes/SKILL.md`
- `.claude/skills/project-structure/SKILL.md`

Behavior:
- Always overwrites (these files are generic; user config belongs in README.md)
- To get newer versions, update pman itself: `cargo install --git https://github.com/divanvisagie/pman`

### verify

Check workspace setup and report any issues.

```sh
pman verify              # current directory
pman verify --path ~/src # specific path
```

Checks:
- Notes directory structure (Projects, Areas, Resources, Archives/Projects)
- `Notes/Projects/_registry.md`
- `CLAUDE.md`
- `.claude/skills/para-notes/SKILL.md`
- `.claude/skills/project-structure/SKILL.md`

Behavior:
- Reports ✓ for present items, ✗ for missing
- Exits with code 1 if any issues found
- Suggests `pman init` or `pman update` to fix

### new

Create a new project note and registry entry.

```sh
pman new "Project Name" --status active
pman new "Runes Notes" --area religion
```

Creates:
- `Notes/Projects/proj-<n>-<slug>/README.md`
- Appends an entry to the registry (`Notes/Projects/_registry.md`)

Options:
- `--status <status>` sets the registry status (default: `active`).
- `--area <slug>` prefixes the project slug with the area.
- `--notes-dir <path>` overrides the Notes root.

### archive

Archive a project by directory prefix or full name.

```sh
pman archive proj-22
pman archive proj-22-some-project
```

Moves:
- `Notes/Projects/proj-22-*/` → `Notes/Archives/Projects/proj-22-*/`
- Updates the registry (`Notes/Projects/_registry.md`) status to `archived` with the new path.

Options:
- `--notes-dir <path>` overrides the Notes root.

## Notes

- Slugs are derived from the project name (ASCII alnum, dash-separated).
- Slugs are unique across both `Projects` and `Archives/Projects`.
- Area slugs are optional; when set, they become a prefix in the directory slug.
