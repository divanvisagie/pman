---
description: Guidelines for AI coding agents working in this repository structure
---

# Repository Guidelines

## Commands

Prefer standard tooling and keep instructions reproducible.

| Task            | Command Examples                          |
| --------------- | ----------------------------------------- |
| Build           | `cargo build`, `make`, `npm run build`    |
| Test            | `cargo test`, `make test`, `npm test`     |
| Search          | `rg <pattern>`                            |
| GitHub          | `gh pr create`, `gh issue list`           |
| Semantic Search | `csep <query>`                            |
| Notes/Workflow  | `pman`                                    |

Each project README lists its specific build and test commands.

## Tools

| Tool   | Purpose                                       | Reference                                        |
| ------ | --------------------------------------------- | ------------------------------------------------ |
| `pman` | Deterministic Notes operations and workflow   |                                                  |
| `rg`   | Fast text and file search                     |                                                  |
| `gh`   | GitHub CLI for repos, PRs, and issues         |                                                  |
| `csep` | Semantic search over local text               | https://github.com/divanvisagie/csep             |
| `cgip` | CLI for OpenAI-compatible LLM APIs            | https://github.com/divanvisagie/chat-gipitty     |

## Project Structure

Workspace root is `~/src` with a reverse-domain layout.

```
~/src/
├── com/
│   ├── example/        # Personal projects
│   └── orgname/        # Organization projects
└── Notes/              # PARA vault
```

- `~/src/Notes` is the PARA vault (Projects, Areas, Resources, Archives).
- Every repo in the reverse-domain tree should have a corresponding Area note.
- Project notes live in `Notes/Projects/` and may reference Areas for long-running work.
- Each project is independent with its own README, build, and release flow.
- Keep shared workflow docs stable and reference them from project notes.

## Coding Style

- Follow per-project formatter/linter defaults.
- Keep names descriptive; avoid ambiguous abbreviations.
- Avoid comments unless they clarify non-obvious logic; a comment introducing a block likely means the block should be a function.

## Testing

- Add or update tests with each behavior change.
- Record test commands run in the project note.
- Follow TDD when making changes: red -> green -> refactor.

## Workflow (SDLC)

- Create a project note before starting work.
- Capture goals, constraints, and decisions as you go.
- Keep changes small and test alongside code changes.

## Git & Pull Requests

- Use concise, imperative commit titles with a brief rationale in the body.
- Mention tests executed when relevant.

## Boundaries

### Always Do

- Prefer standard tooling (`cargo test`, `make test`, etc.).
- Read the project README before making changes.
- Keep changes small and focused.

### Ask First

- If a command may run for a long time, prompt the user and offer the exact command so they can run it themselves.

### Never Do

- Never commit or push without explicit user permission.
