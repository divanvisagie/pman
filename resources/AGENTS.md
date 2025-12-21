# Repository Guidelines

## Project Structure & Module Organization
- Workspace root is `~/src` with a reverse-domain layout (e.g. `com/example/project-name`).
- `~/src/Notes` is the PARA vault (Projects, Areas, Resources, Archives).
- Every repo in the reverse-domain tree should have a corresponding Area note.
- Project notes live in `Notes/Projects/` and may reference Areas for long-running work.
- Each project is independent with its own README, build, and release flow.
- Keep shared workflow docs stable and reference them from project notes.

Example layout:
```
~/src/
├── com/
│   ├── example/        # Personal projects
│   └── orgname/        # Organization projects
└── Notes/              # PARA vault
```

## Build, Test, and Development Commands
- Each project README lists its build and test commands.
- Prefer standard tooling (`cargo test`, `make test`, etc.) and keep instructions reproducible.
- If a command may run for a long time, prompt the user and offer the exact command so they can run it themselves.

## Tools
- `pman` manages deterministic Notes operations and the workflow conventions.
- `rg` (ripgrep) is the default for fast text and file search.
- `gh` (GitHub CLI) is the default for interacting with GitHub repositories.
- `csep` provides semantic search over local text (https://github.com/divanvisagie/csep).
- `cgip` is a CLI for interacting with OpenAI-compatible LLM APIs (https://github.com/divanvisagie/chat-gipitty).

## Coding Style & Naming Conventions
- Follow per-project formatter/linter defaults.
- Keep names descriptive; avoid ambiguous abbreviations.
- Avoid comments unless they clarify non-obvious logic; a comment introducing a block likely means the block should be a function.

## SDLC (Normal Path)
- Create a project note before starting work.
- Capture goals, constraints, and decisions as you go.
- Keep changes small and test alongside code changes.

## Testing Guidelines
- Add or update tests with each behavior change.
- Record test commands run in the project note.
- Follow TDD when making changes: red -> green -> refactor.

## Commit & Pull Request Guidelines
- Use concise, imperative commit titles with a brief rationale in the body.
- Mention tests executed when relevant.
- Never commit or push without explicit user permission.
