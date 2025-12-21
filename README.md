# pman

`pman` defines and enforces a workflow framework for a reverse-domain `~/src` workspace paired with a PARA-style Notes vault. It is not just a helper script: it is the contract for how projects are created, tracked, and archived in this system.

## Why this exists

Most workflows fail because the boring parts are inconsistent: naming, structure, and bookkeeping drift over time. `pman` makes those parts deterministic so that:

- Projects are always created the same way.
- Notes stay aligned with the codebase.
- Archives are predictable and searchable.
- Tooling can rely on a stable filesystem shape.
- Humans and LLM agents share the same context across the full project lifecycle.

The result is a workspace that scales without becoming a mess.

`pman` is designed for mixed teams of humans and LLM agents. By enforcing a single source of truth for project notes and status, every participant has the same context and can operate within the same workflow from discovery through delivery. While it can be used purely for notes, its original design purpose was to make software development workflows deterministic and repeatable.

## The workflow

1. **Workspace layout**: `~/src` holds projects by reverse-domain. Notes live in `~/src/Notes` using PARA.
2. **Project creation**: `pman new` creates a project note in `Notes/Projects/` with a chronological `PROJ-XXXX` id and slug.
3. **Project tracking**: The registry (`Notes/Projects/_registry.md`) is the authoritative index of active projects.
4. **Archiving**: `pman archive` moves the project note into `Notes/Archives/Projects/` and updates the registry to `archived`.
5. **Determinism**: Slugs are unique across both active and archived projects, so the history remains unambiguous.

## Workspace model

This tool assumes a filesystem layout like:

```
~/src/
  Notes/
    Projects/
    Areas/
    Resources/
    Archives/Projects/
```

The Notes vault follows PARA (Projects, Areas, Resources, Archives). `pman` manages only the deterministic project lifecycle bits.

## Install

Local install from source:

```sh
cargo install --path /Users/divan/src/com/divanv/pman
```

Or run directly:

```sh
cargo run -- new "Project Name" --status active
```

## Commands

### new

Create a new project note and register it.

```sh
pman new "Project Name" --status active
```

Creates:
- `Notes/Projects/proj-XXXX-<slug>/README.md`
- Appends to `Notes/Projects/_registry.md`

### archive

Archive a project by directory prefix or full name.

```sh
pman archive proj-0022
pman archive proj-0022-some-project
```

Moves:
- `Notes/Projects/proj-0022-*/` -> `Notes/Archives/Projects/proj-0022-*/`
- Updates registry status to `archived` with the new path.

## Options

- `--notes-dir <path>` overrides the Notes root (helpful if run outside `~/src` or if Notes is elsewhere).
- `--status <status>` sets the registry status for `new` (default: `active`).

## Behavior details

- Slugs are derived from the project name (ASCII alnum, dash-separated).
- Slugs are unique across both `Projects` and `Archives/Projects`.
- Registry format matches the existing `_registry.md` table.

## Development

Run tests:

```sh
cargo test
```

Tests cover slug behavior, id allocation, and archive registry updates.

## Roadmap

- `init` and `verify` commands for `~/src` layout and `AGENTS.md` placement.
- `notes` commands to set or verify the Notes root and manage symlinks.
- `list` and `status` commands for PARA reporting.
