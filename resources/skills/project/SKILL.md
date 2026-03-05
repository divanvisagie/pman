---
name: project
description: Managing notes with PARA and understanding workspace/project boundaries. Use when creating project notes, navigating repositories, organizing notes, searching notes, or tracking SDLC progress.
allowed-tools: Bash(pman:*), Bash(fd:*), Bash(rg:*), Bash(ls:*), Read, Write, Edit
---

# PARA Notes

The Notes directory is organized using the PARA method.

## Structure

| Folder     | Purpose                                              |
|------------|------------------------------------------------------|
| Projects/  | Active endeavors with goals and deadlines; clear end |
| Areas/     | Ongoing responsibilities without clear end           |
| Resources/ | Reference materials                                  |
| Archives/  | Inactive items from other categories                 |

## Bootstrap

If `pman` is not available in the workspace, install and initialize with:

```bash
cargo install --git https://github.com/divanvisagie/pman
pman init .
```

## MCP First, CLI Fallback

When MCP tools are available, prefer MCP tool calls for note and project operations:
- `project_list`, `project_new`, `project_archive`
- `notes_read`, `notes_write`, `notes_edit`

Use `pman` CLI commands as fallback when MCP is unavailable, not connected, or missing required capability.

## Creating Project Notes

Use MCP `project_*` tools first, or `pman` CLI fallback:

```bash
pman list                        # active projects
pman list --status all           # all statuses
pman new "Project Name" --status active
pman new "Feature Work" --area some-repo
pman new z2222-lol-cats          # explicit project directory name
pman archive proj-XX
```

`pman new` behavior:
- Standard mode (`"Project Name"`): creates `Projects/proj-<n>-<slug>/README.md`.
- Explicit mode (`z2222-lol-cats`): if name is slug-like (contains `-`, no spaces), uses that exact directory name.
- Explicit mode sets ID from explicit name (`MYSLUG-1192` for `myslug-1192-mythingy`; otherwise uppercased full name, e.g. `Z2222-LOL-CATS`).
- Explicit mode does not support `--area`.

All modes append an entry to `Projects/_registry.md`.

## Note I/O From Any Directory

Prefer MCP `notes_*` tools for file reads and edits. If MCP is unavailable, use `pman` note commands instead of raw shell file operations:

```bash
pman read Projects/proj-98-example/README.md --numbered
pman edit Projects/proj-98-example/README.md --replace-lines 10:14 --with "new text" --expect "old text"
pman write Projects/proj-98-example/README.md --content "# PROJ-98: ..."
```

Use wrappers as convenience commands when needed:

```bash
pman cat Projects/proj-98-example/README.md
pman head Projects/proj-98-example/README.md --lines 40
pman tail Projects/proj-98-example/README.md --lines 40
pman wc Projects/proj-98-example/README.md --lines --words
pman less Projects/proj-98-example/README.md
```

`read`/`write`/`edit` are the core primitives. `cat`/`head`/`tail`/`less` are thin wrappers for familiar ergonomics.

## Project Note Template

```markdown
# <ID>: Name

## Summary
-

## Status
- active

## Notes
-

## Next
-
```

## Before Starting Work

1. List active projects with `pman list`
2. Check `Notes/Projects/` for an existing project note
3. If present, update it rather than creating a new one
4. Use `pman new` only when starting genuinely new work

## Workspace And Project Boundaries

- Each project subdirectory is typically its own git repository.
- The workspace root may not be a git repository.
- Read each repository's `README.md` before making changes.
- Check for project-specific `AGENTS.md` files.
- Use the build and test commands specified by each repository's README.

Projects vs repositories:
- A repository is a codebase.
- A project (PARA) is a time-bound effort.
- A project may span multiple repositories.

When making a code change in any repository, keep project-note tracking in `Notes/Projects/`.

Cross-project references can use relative paths, for example:

```text
../sibling-project/src/
../../other-org/shared-lib/
```

## Searching Notes

```bash
fd <pattern> Notes/          # Find by filename
rg <pattern> Notes/          # Search contents
```

## Registry

`Projects/_registry.md` is the authoritative index of all projects.

| ID      | Name    | Status   | Created    | Note                    |
| ------- | ------- | -------- | ---------- | ----------------------- |
| PROJ-1  | Example | active   | 2025-01-01 | [link](proj-1-example/) |

## Archiving

```bash
pman archive <project-prefix-or-dir-name>
```

Moves the project to `Archives/Projects/` and updates the registry.

## File Formats

- Primary content: Markdown (`.md`)
- Diagrams: Mermaid charts
