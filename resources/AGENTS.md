# Repository Guidelines (Generic)

## Project Structure & Module Organization
- Use a predictable root layout; keep each project in its own folder with a clear purpose.
- The Notes vault lives at the root of `~/src` as `~/src/Notes` and always follows PARA (Projects, Areas, Resources, Archives).
- Projects under `~/src` follow a reverse-domain structure (e.g. `com/divanv/project-name`).
- Every repo in these subfolders should be represented as a note in `Notes/Areas/`, even if it also has a project note.
- Project notes may reference area notes when a project is part of a long-running responsibility.
- Store shared workflow docs in a stable location and reference them from project notes.
- Keep each project independent with its own README, build, and release flow.

## Build, Test, and Development Commands
- Document build and test commands in each project README.
- Prefer standard tooling (`cargo test`, `make test`, etc.) and keep instructions reproducible.

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
