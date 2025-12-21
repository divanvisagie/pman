# Repository Guidelines

This file captures the shared workflow conventions for a `~/src` workspace. It is intended to give humans and AI agents the same operating context, regardless of tooling.

## Project Structure & Module Organization
- The workspace root is `~/src` (example layout below).
- Notes live at `~/src/Notes` and always follow PARA (Projects, Areas, Resources, Archives).
- Projects follow a reverse-domain structure (e.g. `com/example/project-name`).
- Every repo in the reverse-domain tree should have a corresponding Area note.
- Project notes live in `Notes/Projects/` and may reference Areas for long-running work.
- Store shared workflow docs in a stable location and reference them from project notes.
- Keep each project independent with its own README, build, and release flow.

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

## Tools
- `pman` manages the workflow and deterministic Notes operations.
- `rg` (ripgrep) is the default for fast text and file search.
- `gh` (GitHub CLI) is the default for interacting with GitHub repositories.
- `csep` provides semantic search over local text (https://github.com/divanvisagie/csep).
- `cgip` is a CLI for interacting with OpenAI-compatible LLM APIs (https://github.com/divanvisagie/chat-gipitty).

## Coding Style & Naming Conventions
- Follow per-project formatter/linter defaults.
- Keep names descriptive; avoid ambiguous abbreviations.

## SDLC (Normal Path)
- Create a project note before starting work.
- Capture goals, constraints, and decisions as you go.
- Keep changes small and test alongside code changes.

## Testing Guidelines
- Add or update tests with each behavior change.
- Record test commands run in the project note.

## Commit & Pull Request Guidelines
- Use concise, imperative commit titles with a brief rationale in the body.
- Mention tests executed when relevant.
