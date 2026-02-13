---
name: para-notes
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

## Creating Project Notes

Use `pman` to create and manage project notes:

```bash
pman new "Project Name" --status active
pman new "Feature Work" --area some-repo
pman archive proj-XX
```

This creates:
- `Projects/proj-<n>-<slug>/README.md`
- Entry in `Projects/_registry.md`

## Note I/O From Any Directory

Prefer `pman` note commands for file reads and edits, instead of raw shell file operations:

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
# PROJ-XX: Name

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

1. Check `Notes/Projects/` for an existing project note
2. If present, update it rather than creating a new one
3. Use `pman new` only when starting genuinely new work

## Workspace And Project Boundaries

- Each project subdirectory is typically its own git repository.
- The workspace root may not be a git repository.
- Read each repository's `README.md` before making changes.
- Check for project-specific `CLAUDE.md` files.
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
pman archive proj-XX
```

Moves the project to `Archives/Projects/` and updates the registry.

## File Formats

- Primary content: Markdown (`.md`)
- Diagrams: Mermaid charts
