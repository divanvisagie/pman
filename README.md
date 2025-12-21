# pman

`pman` is an opinionated framework for a project workflow that maximizes context reuse, note management, and collaboration between humans, AI agents, and Unix tooling. It draws from the Unix philosophy: small, deterministic commands with predictable outputs and a strict hierarchical file structure. The `pman` repository is the source of truth for this workflow, and this README serves as the manual.

The reason it exists is simple: most workflows fail in the boring parts. Naming, structure, and bookkeeping drift over time. `pman` makes those parts deterministic so projects stay searchable, notes stay aligned, and people (or agents) stay in sync.

Whether you use the `pman` tool or not, the workflow still stands. It can be executed manually, by an AI agent, or by the CLI itself. It can even be implemented in the physical world with notebooks, sticky notes, and string if that's what you're into. The value is the workflow, not the tooling.

## What this gives you

Consistency without friction:

- Projects are always created the same way.
- Notes stay aligned with the codebase.
- Archives are predictable and searchable.
- Tooling can rely on a stable filesystem shape.
- Humans and LLM agents share the same context across the full project lifecycle.

The result is a workspace that scales without becoming a mess.

`pman` is designed for mixed teams of humans and LLM agents. By enforcing a single source of truth for project notes and status, every participant has the same context and can operate within the same workflow from discovery through delivery. While it can be used purely for notes, its original design purpose was to make software development workflows deterministic and repeatable.

## How the workflow works

1. **Workspace layout**: `~/src` holds projects by reverse-domain. Notes live in `~/src/Notes` using PARA.
2. **Project creation**: `pman new` creates a project note in `Notes/Projects/` with a chronological `PROJ-<n>` id and slug.
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

## CLI Reference

The CLI reference lives in `docs/cli.md`, including install and command usage.

## Resources

This repo includes a generic `AGENTS.md` template at `resources/AGENTS.md`. It documents baseline project conventions so the workflow can be reproduced without relying on this specific codebase. Adapt it for each workspace and keep it close to the root so humans and AI agents share the same operational context.

Keep the agents file up to date. You do not need to be a genius to maintain it: when you notice repeated undesired behavior, ask the agent to update the file directly (e.g., “please add to the agents file not to do X again”).

The HTML manual homepage is `docs/index.html` and is intended for GitHub Pages or any static hosting. Development notes live in `docs/development.md`.

## Roadmap

- `init` and `verify` commands for `~/src` layout and `AGENTS.md` placement.
- `init` should be a prompt-by-prompt wizard to set up `~/src` and Notes.
- `notes` commands to set or verify the Notes root and manage symlinks.
- `list` and `status` commands for PARA reporting.
