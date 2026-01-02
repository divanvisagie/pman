# Claude Code Guidelines

This file configures Claude Code for this workspace. Place it at the root of `~/src` or in individual projects.

## How We Work Together

Most file changes happen through this chat. Instead of editing files or running commands directly, describe what you want and let Claude handle execution. Claude is configured with the workflow rules below, so it enforces conventions automatically: creating project notes before coding, updating the registry, following commit formats.

You can still edit files directly—sketch pseudocode in vim, tweak a config by hand, or use whatever tool fits the moment. When you do, tell Claude to look at what you changed so it stays in sync. Claude is one tool in the toolbox, not the entire toolbox.

## Skills

Claude Code skills extend capabilities for specific workflows. Install from the pman repo:

```sh
cp -r /path/to/pman/resources/skills/* ~/.claude/skills/
```

| Skill               | Purpose                                              |
| ------------------- | ---------------------------------------------------- |
| `para-notes`        | PARA note management, project notes, SDLC tracking   |
| `project-structure` | Reverse-domain `~/src` layout, `gb` project creation |

Skills are defined in `SKILL.md` files and provide context and tool permissions for specific tasks.

## Commands

Prefer standard tooling and keep instructions reproducible.

| Task            | Command Examples                          |
| --------------- | ----------------------------------------- |
| Build           | `cargo build`, `make`, `npm run build`    |
| Test            | `cargo test`, `make test`, `npm test`     |
| Search          | `fd <pattern>`, `rg <pattern>`            |
| GitHub          | `gh pr create`, `gh issue list`           |
| Semantic Search | `csep <query>`                            |
| Notes/Workflow  | `pman`                                    |

Each project README lists its specific build and test commands.

## Tools

| Tool   | Purpose                                     |
| ------ | ------------------------------------------- |
| `pman` | Deterministic project notes and workflow    |
| `fd`   | Fast file search by name/pattern            |
| `rg`   | Fast text search in file contents           |
| `gh`   | GitHub CLI for repos, PRs, and issues       |
| `csep` | Semantic search over local text             |

## Project Structure

Workspace root is `~/src` with a reverse-domain layout.

```
~/src/
├── com/
│   ├── divanv/            # Local-only personal projects
│   └── github/divanvisagie/  # GitHub-hosted repos
└── Notes/                 # PARA vault
    └── Projects/
        └── _registry.md   # Index of all projects
```

- `~/src/Notes` is the PARA vault (Projects, Areas, Resources, Archives).
- Every repo in the reverse-domain tree should have a corresponding Area note.
- Project notes live in `Notes/Projects/` and may reference Areas for long-running work.
- The registry (`Notes/Projects/_registry.md`) is the authoritative index of active and archived projects.
- Each repo is independent with its own README, build, and release flow.

### Projects vs repositories

A **repository** is a codebase. A **project** is a time-bound effort—adding a feature, fixing a bug, refactoring a module. Repos don't map one-to-one to projects; a single repo may have many projects over its lifetime, and a project might touch multiple repos. When you make a change, that change belongs to a project.

## Workflow (SDLC)

**Plan before you code.**

1. **Create a project note**: Use `pman new` to create a project note. Check the registry for existing projects you might continue.
2. **Plan collaboratively**: Work with the user to develop the plan in the project note. Discuss goals, constraints, trade-offs, and approach. The plan lives in the note, not in chat history.
3. **Execute**: Once the plan document is complete, start writing code. The plan is the spec—follow it.
4. **Record outcomes**: Update the project note with what worked, what changed, and any follow-up tasks.

Code changes only begin after the plan is done. When asked to make a change, first ensure a project note exists, then develop the plan there. Do not write code until the plan is complete and approved.

## Coding Style

- Follow per-project formatter/linter defaults.
- Keep names descriptive; avoid ambiguous abbreviations.
- Avoid comments unless they clarify non-obvious logic.

## Testing

- Add or update tests with each behavior change.
- Record test commands run in the project note.
- Follow TDD when making changes: red → green → refactor.

## Git & Pull Requests

- Use concise, imperative commit titles with a brief rationale in the body.
- Mention tests executed when relevant.

## Boundaries

### Always Do

- Use `fd` and `rg` for searching—never `find` or `grep`.
- Prefer standard tooling (`cargo test`, `make test`, etc.).
- Read the project README before making changes.
- Keep changes small and focused.

### Ask First

- If a command may run for a long time, prompt the user and offer the exact command so they can run it themselves.

### Never Do

- Never commit or push without explicit user permission.

## Keeping This File Up to Date

This file is the contract between you and Claude. When Claude does something you don't want, ask it to update this file directly: "Please add to CLAUDE.md not to do X again." This way, the lesson persists across sessions.
