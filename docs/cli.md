# pman CLI Reference

This document covers the CLI behavior. The workflow manual lives in the README.

## Core concepts

- A **project** is a time-bound effort (feature, bugfix, refactor)—not a repository. A repo may have many projects; a project may touch multiple repos.
- The **registry** (`Notes/Projects/_registry.md`) is the authoritative index of all active and archived projects.
- Every change belongs to a project. The workflow: create a project note → plan collaboratively with the model → execute code changes once the plan is complete.
- For note file operations, the canonical primitives are `pman read`, `pman write`, and `pman edit`. `cat/head/tail/wc/less` are notes-scoped wrappers for familiar ergonomics.

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

### read

Read a note file relative to the Notes root.

```sh
pman read Projects/proj-22-some-project/README.md
pman read Projects/proj-22-some-project/README.md --numbered
pman read Projects/proj-22-some-project/README.md --lines 10:30 --numbered
```

Options:
- `--notes-dir <path>` overrides the Notes root.
- `--lines <start:end>` selects an inclusive 1-based line range.
- `--numbered` adds line numbers to output.

### write

Replace an entire note file.

```sh
pman write Projects/proj-22-some-project/README.md --content "# PROJ-22: Name"
printf '# PROJ-22: Name\n' | pman write Projects/proj-22-some-project/README.md
pman write Areas/team/notes.md --create-dirs --content "text"
```

Options:
- `--notes-dir <path>` overrides the Notes root.
- `--create-dirs` creates missing parent directories.
- `--content <text>` writes explicit content; if omitted, stdin is used.

### edit

Replace an inclusive line range within a note.

```sh
pman edit Projects/proj-22-some-project/README.md --replace-lines 20:25 --with "new text"
pman edit Projects/proj-22-some-project/README.md --replace-lines 20:25 --with "new text" --expect "old text"
```

Options:
- `--notes-dir <path>` overrides the Notes root.
- `--replace-lines <start:end>` selects the inclusive range to replace.
- `--with <text>` sets replacement text.
- `--expect <text>` guards against stale context by requiring exact current text in the selected range.

### cat/head/tail/wc/less

Notes-scoped wrappers that resolve paths from Notes root and enforce containment:

```sh
pman cat Projects/proj-22-some-project/README.md
pman head Projects/proj-22-some-project/README.md --lines 40
pman tail Projects/proj-22-some-project/README.md --lines 40
pman wc Projects/proj-22-some-project/README.md --lines --words
pman less Projects/proj-22-some-project/README.md
```

Options:
- All support `--notes-dir <path>`.
- `head` and `tail` support `--lines <n>` (default `10`).
- `wc` supports `--lines`, `--words`, `--bytes`, `--chars`.

Behavior:
- `less` automatically degrades to non-interactive `cat` behavior when no TTY is present.

### skill generate

Print a complete `SKILL.md` template to stdout.

```sh
pman skill generate > .claude/skills/para-notes/SKILL.md
pman skill generate para-notes > .claude/skills/para-notes/SKILL.md
```

Options:
- `<profile>` is optional and defaults to `para-notes`.

## Notes

- Slugs are derived from the project name (ASCII alnum, dash-separated).
- Slugs are unique across both `Projects` and `Archives/Projects`.
- Area slugs are optional; when set, they become a prefix in the directory slug.
- Note I/O commands resolve and canonicalize paths from the Notes root, rejecting out-of-root targets.
