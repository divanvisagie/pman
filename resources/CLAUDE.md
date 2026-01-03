# pman Workflow

Instructions for Claude Code. User-specific paths and tools are in README.md.

## Before Making Changes

1. Check `Notes/Projects/` for an existing project note
2. If none exists, create one with `pman new "<name>"`
3. Develop the plan in the project note with the user
4. Do not write code until the plan is complete and approved

## Creating Project Notes

```bash
pman new "Project Name" --status active
pman archive proj-XX
```

The registry at `Notes/Projects/_registry.md` is the authoritative index.

## Searching

Always use `fd` and `rg`. Never use `find` or `grep`.

```bash
fd <pattern>              # Find files by name
rg <pattern>              # Search file contents
fd <pattern> Notes/       # Search notes
```

## Commands

Use standard tooling. Each project README lists specific commands.

| Task   | Examples                               |
| ------ | -------------------------------------- |
| Build  | `cargo build`, `make`, `npm run build` |
| Test   | `cargo test`, `make test`, `npm test`  |
| GitHub | `gh pr create`, `gh issue list`        |

## Project Structure

- Each project subdirectory is its own git repository
- Read a project's README.md before making changes
- See workspace README.md for directory layout

## Testing

- Add or update tests with each behavior change
- Record test commands in the project note
- Follow TDD: red -> green -> refactor

## Git

- Never commit or push without explicit user permission
- Never stage or unstage without explicit user permission
- Commit messages: title + body explaining the reasoning

## Boundaries

**Always:**
- Use `fd` and `rg` for searching
- Read the project README before changes
- Keep changes small and focused
- Run `date` first for time-based questions

**Ask first:**
- Before running long-running commands

**Never:**
- Commit, push, stage, or unstage without permission
