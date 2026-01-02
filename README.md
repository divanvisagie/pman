# pman

`pman` is an opinionated workflow for agentic programming with Claude Code.

LLMs are great at generating code and iterating through implementation problems, but they struggle with context. Context is the hardest part of software development. `pman` flips the dynamic: you become the context manager, while Claude focuses on code, frameworks, and documentation. Each plays to their strength.

The key insight is that most file changes happen through the agent chat as an intermediary. Instead of editing files or running commands directly, you work through Claude. Because Claude is configured with the workflow rules via `CLAUDE.md` and skills, it enforces conventions automatically: creating project notes before coding, updating the registry, following commit formats. You describe intent; Claude handles execution within the established structure.

This doesn't mean you can't edit files directly. Sketch out pseudocode in vim, tweak a config by hand, or use whatever tool fits the moment. The workflow is interactive: when you make changes outside the chat, tell Claude to look at what you did. Claude, your editor, and any other tool are tools in the toolbox—not the entire toolbox. We don't do Emacs here.

Unlike throwaway planning, `pman` treats plans as persistent artifacts, like source files, but managed in a separate, centralized Notes repository. By documenting every change, you build a reference set for future work. Changed service A and now service B needs updating? Pull in context from A's project note. The Notes vault becomes your cross-project memory.

Initializing Claude at the `~/src` root also lets you reference other repositories directly. Building an API client? Point Claude at the API source in a sibling repo and have it implement the client. The shared workspace means cross-repo work is natural.

Whether you use the `pman` tool or not, the workflow still stands. It can be executed manually, by Claude, or by the CLI itself. The value is the workflow, not the tooling.

**Agent compatibility**: This workflow is developed and tested exclusively with Claude Code. It may work with other AI coding agents, but I only test with Claude and have found it to be the most effective for this style of work. The workflow, prompts, and tooling are optimized for Claude's capabilities.

## What this gives you

Consistency without friction:

- Projects are always created the same way.
- Notes stay aligned with the codebase.
- Archives are predictable and searchable.
- Tooling can rely on a stable filesystem shape.
- You and Claude share the same context across the full project lifecycle.

The result is a workspace that scales without becoming a mess.

`pman` is designed for humans working with Claude Code. By enforcing a single source of truth for project notes and status, both you and Claude share the same context across the full project lifecycle. While it can be used purely for notes, its original design purpose was to make software development workflows deterministic and repeatable.

## How the workflow works

1. **Workspace layout**: `~/src` holds projects by reverse-domain. Notes live in `~/src/Notes` using PARA.
2. **Project creation**: `pman new` creates a project note in `Notes/Projects/` with a chronological `PROJ-<n>` id and slug.
3. **Project tracking**: The registry (`Notes/Projects/_registry.md`) is the authoritative index of active projects.
4. **Archiving**: `pman archive` moves the project note into `Notes/Archives/Projects/` and updates the registry to `archived`.
5. **Deterministic slugs**: Each project gets a unique slug like `proj-22-my-feature`. Slugs make projects easy to reference in conversation and across notes. If you use a ticketing system (Jira, Linear, GitHub Issues), modify your CLAUDE.md to use that format (e.g., `PROJ-123-my-feature`) so project notes align with your existing workflow.

## Projects vs repositories

A **repository** is a codebase. A **project** is a time-bound effort to achieve a specific outcome: adding a feature, fixing a bug, or refactoring a module. Repositories don't map one-to-one to projects; a single repo may have many projects over its lifetime, and a project might touch multiple repos.

When you make a change, that change belongs to a project. The project note captures the planning, decisions, and outcomes. The registry (`Notes/Projects/_registry.md`) tracks all active and archived projects.

## Making changes

The core principle: **plan before you code**.

1. **Create a project note**: Use `pman new` to create a project note. Check the registry for existing projects you might continue.
2. **Plan collaboratively**: Work with the model to develop the plan in the project note. Discuss goals, constraints, trade-offs, and approach. The plan lives in the note, not in chat history.
3. **Execute**: Once the plan document is complete, start writing code. The plan is the spec; follow it.
4. **Record outcomes**: Update the project note with what worked, what changed, and any follow-up tasks.

Code changes only begin after the plan is done. This prevents wasted effort and keeps everyone aligned. The project note becomes the single source of truth for *why* a change was made, while the code and git history record *what* changed.

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

The CLI reference lives in [`docs/cli.md`](docs/cli.md), including install and command usage.

## Resources

This repo includes a `CLAUDE.md` template at `resources/CLAUDE.md`. It configures Claude Code for this workspace: commands, tools, project structure, and workflow. Place it at the root of `~/src` or in individual projects.

Keep `CLAUDE.md` up to date. When you notice repeated undesired behavior, ask Claude to update the file directly (e.g., "please add to CLAUDE.md not to do X again").

### Skills

Claude Code skills extend capabilities for specific workflows. This repo includes two skills in `resources/skills/`:

| Skill               | Purpose                                              |
| ------------------- | ---------------------------------------------------- |
| `para-notes`        | PARA note management, project notes, SDLC tracking   |
| `project-structure` | Reverse-domain `~/src` layout, `gb` project creation |

Install by copying to `~/.claude/skills/`:

```sh
cp -r resources/skills/* ~/.claude/skills/
```

Skills are defined in `SKILL.md` files and provide context and tool permissions for specific tasks.

The HTML manual homepage is `docs/index.html` and is hosted at https://divanv.com/pman/. Development notes live in [`docs/development.md`](docs/development.md).

## Contributing

When updating this README, ensure the following files stay in sync:

- `docs/index.html`: The HTML manual mirrors the README content
- `resources/CLAUDE.md`: The template should reflect current workflow guidance

## Roadmap

- `init` and `verify` commands for `~/src` layout and `CLAUDE.md` placement.
- `init` should be a prompt-by-prompt wizard to set up `~/src` and Notes.
- `notes` commands to set or verify the Notes root and manage symlinks.
- `list` and `status` commands for PARA reporting.
