# pman

`pman` is an opinionated workflow for agentic programming.

LLMs are great at generating code and iterating through implementation problems, but they struggle with context. Context is the hardest part of software development. `pman` flips the dynamic: you become the context manager, while your coding agent focuses on code, frameworks, and documentation. Each plays to their strength.

The key insight is that most file changes happen through the agent chat as an intermediary. Instead of editing files or running commands directly, you work through your coding agent. Because the agent is configured with workflow rules via `AGENTS.md` and skills, it can enforce conventions automatically: creating project notes before coding, updating the registry, following commit formats. You describe intent; the agent handles execution within the established structure.

This doesn't mean you can't edit files directly. Sketch out pseudocode in vim, tweak a config by hand, or use whatever tool fits the moment. The workflow is interactive: when you make changes outside the chat, tell your agent to look at what you did. Your agent, your editor, and any other tool are tools in the toolbox, not the entire toolbox.

Unlike throwaway planning, `pman` treats plans as persistent artifacts, like source files, but managed in a separate, centralized Notes directory. By documenting every change, you build a reference set for future work. Changed service A and now service B needs updating? Pull in context from A's project note. The Notes vault becomes your cross-system memory.

**Agent compatibility**: This workflow is designed to work with coding agents that can read workspace guidance files and skills. `pman` maintains canonical workflow rules in `AGENTS.md` and provides bridge symlinks for supported CLIs such as Claude and Codex.

## Glossary

| Term       | Meaning                                                             |
|------------|---------------------------------------------------------------------|
| Project    | Time-bound effort to achieve an outcome (feature, bugfix, refactor) |
| System     | What the project changes—may span one or more repositories          |
| Repository | A git repo containing code                                          |

A **project** changes a **system**. A **system** is made of one or more **repositories**. Projects live in `Notes/Projects/`, repositories live in the workspace.

## What this gives you

Consistency without friction:

- Projects are always created the same way.
- Notes stay aligned with the system.
- Archives are predictable and searchable.
- Tooling can rely on a stable filesystem shape.
- You and your coding agent share the same context across the full project lifecycle.

The result is a workspace that scales without becoming a mess.

## How the workflow works

1. **Create a project note**: `pman new` creates a project note in `Notes/Projects/` with a chronological `PROJ-<n>` id and slug.
2. **Plan collaboratively**: Work with your coding agent to develop the plan in the project note. Discuss goals, constraints, trade-offs, and approach. The plan lives in the note, not in chat history.
3. **Execute**: Once the plan is complete, start writing code. The plan is the spec—follow it.
4. **Record outcomes**: Update the project note with what worked, what changed, and any follow-up tasks.
5. **Archive**: `pman archive` moves the project note to `Notes/Archives/Projects/` and updates the registry.

The registry (`Notes/Projects/_registry.md`) is the authoritative index of active and archived projects.

## Making changes

The core principle: **plan before you code**.

Code changes only begin after the plan is done. This prevents wasted effort and keeps everyone aligned. The project note becomes the single source of truth for *why* a change was made, while the code and git history record *what* changed.

## Workspace model

`pman` expects a Notes directory following PARA (Projects, Areas, Resources, Archives):

```
Notes/
  Projects/
    _registry.md
  Areas/
  Resources/
  Archives/
    Projects/
```

The workspace structure for your repositories is up to you—document it in your workspace `README.md`.

## Install

See [`docs/cli.md`](docs/cli.md) for CLI install and command reference.

## Configuration

`pman` ships a generic agent rules file plus a user-maintained workspace README:

### AGENTS.md

Generic workflow instructions for agentic coding tools. Place at your workspace root.

```sh
cp resources/AGENTS.md ./
```

Contains: workflow rules, commands, boundaries. Written as directives for coding agents, not documentation for humans.

### README.md (user-maintained)

Your workspace-specific configuration. Document your:

- Directory layout and organization
- Custom tools and commands
- Project creation workflow
- System-specific conventions

Agents read both files, combining generic workflow with your specific setup.

### Skills

Skills extend capabilities for specific workflows. This repo includes one merged skill in `resources/skills/`:

| Skill        | Purpose                                                                        |
| ------------ | ------------------------------------------------------------------------------ |
| `para-notes` | PARA note management, note I/O commands, and workspace/project boundary guidance |

Canonical skill install path:

```sh
mkdir -p ./.pman/skills/para-notes
cp resources/skills/para-notes/SKILL.md ./.pman/skills/para-notes/SKILL.md
```

When supported agent CLIs are installed, `pman init`/`pman update` create bridge symlinks:

- `CLAUDE.md` -> `AGENTS.md` (when `claude` is installed)
- `.claude/skills/para-notes` -> `.pman/skills/para-notes` (when `claude` is installed)
- `.codex/skills/para-notes` -> `.pman/skills/para-notes` (when `codex` is installed)

## Upgrading

Because `AGENTS.md` and skills are generic, upgrading is simple:

```sh
pman update --path .
```

Your `README.md` stays untouched—no merge conflicts.

## Contributing

When updating this README, ensure the following files stay in sync:

- `docs/index.html`: The HTML manual mirrors the README content
- `resources/AGENTS.md`: The template should reflect current workflow guidance

## Roadmap

- `init` command: wizard to set up workspace and Notes directory
- `verify` command: check workspace structure and configuration
- `list` and `status` commands for project reporting
