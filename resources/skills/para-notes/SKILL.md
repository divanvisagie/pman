---
name: para-notes
description: Managing notes using the PARA method (Projects, Areas, Resources, Archives). Use when creating project notes, organizing notes, searching notes, working with proj-XXXX prefixed projects, or tracking SDLC progress.
allowed-tools: Bash(fd:*), Bash(rg:*), Read, Write, Edit
---

# PARA Notes System

The `~/src/Notes` directory is a second brain organized using the PARA method.

## Structure

| Folder     | Purpose                                                       |
|------------|---------------------------------------------------------------|
| Projects/  | Active endeavors with goals and deadlines; clear end          |
| Areas/     | Ongoing responsibilities without clear end (repos, interests) |
| Resources/ | Reference materials (books, recipes, software notes)          |
| Archives/  | Inactive items from other categories                          |

## Project Notes

For work following a normal SDLC path, create a project note:

```
Projects/proj-XXXX-<slug>/README.md
```

Where `XXXX` is a sequential number and `<slug>` is a short descriptive name.

The registry at `Projects/_registry.md` tracks all active and archived projects.

### SDLC Tracking in Project Notes

Track these phases in the project README:

1. **Discovery**: goals, constraints, and success criteria
2. **Design**: approach, data flow, and user impact
3. **Implementation**: milestones, scope changes, and key decisions
4. **Validation**: tests/builds run and results
5. **Release/maintenance**: rollout steps, follow-ups, and open questions

## Areas for Repositories

Every repo in the reverse-domain tree (`~/src/com/...`) should have a corresponding Area note for long-running work without a clear end.

## Searching Notes

Use `fd` and `rg` for searching:

```bash
# Find notes by filename
fd <pattern> ~/src/Notes

# Search note contents
rg <pattern> ~/src/Notes
```

## Before Starting Work

1. Check if a corresponding project already exists under `Notes/Projects/`
2. If present, update it rather than creating a new one
3. Only create a new project note if the user explicitly asks or there's no existing project

## File Formats

- Primary content: Markdown (`.md`)
- Diagrams: Use Mermaid charts (not ASCII art)

## Archiving

Old projects move to `Archives/Projects/` when completed or abandoned. Update the registry status to `archived`.
