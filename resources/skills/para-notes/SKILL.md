---
name: para-notes
description: Managing notes using the PARA method (Projects, Areas, Resources, Archives). Use when creating project notes, organizing notes, searching notes, working with proj-XXXX prefixed projects, or tracking SDLC progress.
allowed-tools: Bash(pman:*), Bash(fd:*), Bash(rg:*), Read, Write, Edit
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
