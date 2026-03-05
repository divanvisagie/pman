# pman CLI Reference

This document covers the CLI behavior. The workflow manual lives in the README.

## Core concepts

- A **project** is a time-bound effort (feature, bugfix, refactor)—not a repository. A repo may have many projects; a project may touch multiple repos.
- The **registry** (`Notes/Projects/_registry.md`) is the authoritative index of all active and archived projects.
- Every change belongs to a project. The workflow: create a project note → plan collaboratively with the model → execute code changes once the plan is complete.
- For note file operations, the canonical primitives are `pman read`, `pman write`, and `pman edit`. `cat/head/tail/wc/less` are notes-scoped wrappers for familiar ergonomics.

## Install

```sh
cargo install --git https://github.com/divanvisagie/pman
```

Then initialize a workspace:

```sh
cd ~/src  # or any directory you want as workspace root
pman init
```

## Notes Root Resolution

For commands that resolve the Notes root (`new`, `archive`, `read`, `write`, `edit`, `cat`, `head`, `tail`, `wc`, `less`), pman uses this precedence:

1. `--notes-dir <path>`
2. `PMAN_NOTES_DIR`
3. Existing automatic discovery (`~/Notes`, then ancestor discovery)

## Project Directory Prefix

For `pman new`, the project directory prefix defaults to `proj`:

- `Notes/Projects/proj-<n>-<slug>/README.md`

Override it by setting `PMAN_PROJECT_PREFIX` to an ASCII alphanumeric value:

```sh
PMAN_PROJECT_PREFIX=ticket pman new "Example Project"
# -> Notes/Projects/ticket-<n>-example-project/README.md
```

If `PMAN_PROJECT_PREFIX` is unset or empty, pman falls back to `proj`.

## Commands

### init

Initialize a new pman workspace.

```sh
pman init              # current directory
pman init ~/src        # specific path
```

Creates:
- `Notes/Projects/`, `Notes/Areas/`, `Notes/Resources/`, `Notes/Archives/Projects/`
- `Notes/Projects/_registry.md` with header template
- `AGENTS.md` (generic workflow rules)
- `.pman/skills/para-notes/SKILL.md` (canonical skill install)
- If `claude` is installed: `CLAUDE.md` -> `AGENTS.md` and `.claude/skills/para-notes` -> `.pman/skills/para-notes`
- If `codex` is installed: `.codex/skills/para-notes` -> `.pman/skills/para-notes`

Behavior:
- Skips any file or directory that already exists (never overwrites)
- Safe to run multiple times

### update

Update AGENTS.md and canonical skills to the versions embedded in your pman binary.

```sh
pman update              # current directory
pman update --path ~/src # specific path
```

Updates:
- `AGENTS.md`
- `.pman/skills/para-notes/SKILL.md`
- Agent bridge symlinks for installed CLIs (`claude`, `codex`)

Behavior:
- Always overwrites (these files are generic; user config belongs in README.md)
- To get newer versions, update pman itself: `cargo install --git https://github.com/divanvisagie/pman`

### verify

Check workspace setup and report any issues.

```sh
pman verify              # current directory
pman verify --path ~/src # specific path
```

Checks:
- Notes directory structure (Projects, Areas, Resources, Archives/Projects)
- `Notes/Projects/_registry.md`
- `AGENTS.md`
- `.pman/skills/para-notes/SKILL.md`
- If `claude` is installed: `CLAUDE.md` -> `AGENTS.md` and `.claude/skills/para-notes` -> `.pman/skills/para-notes`
- If `codex` is installed: `.codex/skills/para-notes` -> `.pman/skills/para-notes`

Behavior:
- Reports ✓ for present items, ✗ for missing
- Exits with code 1 if any issues found
- Suggests `pman init` or `pman update` to fix

### new

Create a new project note and registry entry.

```sh
pman new "Project Name" --status active
pman new "Runes Notes" --area religion
pman new myslug-1192-mythingy
```

Creates:
- `Notes/Projects/<prefix>-<n>-<slug>/README.md` (default prefix: `proj`)
- Appends an entry to the registry (`Notes/Projects/_registry.md`)

Explicit name mode:
- If the name is slug-like (no spaces, contains `-`), pman uses it as the project directory name (for example `myslug-1192-mythingy` or `z2222-lol-cats`).
- In explicit mode, ID is derived from the explicit name (`MYSLUG-1192` for `myslug-1192-mythingy`; otherwise uppercased full name, e.g. `Z2222-LOL-CATS`) and `--area` is not supported.
- If the directory already exists in Projects or Archives, creation fails with an error.

Options:
- `--status <status>` sets the registry status (default: `active`).
- `--area <slug>` prefixes the project slug with the area.
- `--notes-dir <path>` overrides the Notes root.

### archive

Archive a project by directory prefix or full name.

```sh
pman archive proj-22
pman archive proj-22-some-project
```

Moves:
- `Notes/Projects/proj-22-*/` → `Notes/Archives/Projects/proj-22-*/`
- Updates the registry (`Notes/Projects/_registry.md`) status to `archived` with the new path.

Options:
- `--notes-dir <path>` overrides the Notes root.

### list

List projects from the registry.

```sh
pman list                 # active projects
pman list --status all    # all projects
pman list --status archived
```

Options:
- `--status <value>` filters by status. Default is `active`; use `all` to disable filtering.
- `--notes-dir <path>` overrides the Notes root.

### read

Read a note file relative to the Notes root.

```sh
pman read Projects/proj-22-some-project/README.md
pman read Projects/proj-22-some-project/README.md --numbered
pman read Projects/proj-22-some-project/README.md --lines 10:30 --numbered
```

Options:
- `--notes-dir <path>` overrides the Notes root.
- `--lines <start:end>` selects an inclusive 1-based line range.
- `--numbered` adds line numbers to output.

### write

Replace an entire note file.

```sh
pman write Projects/proj-22-some-project/README.md --content "# PROJ-22: Name"
printf '# PROJ-22: Name\n' | pman write Projects/proj-22-some-project/README.md
pman write Areas/team/notes.md --create-dirs --content "text"
```

Options:
- `--notes-dir <path>` overrides the Notes root.
- `--create-dirs` creates missing parent directories.
- `--content <text>` writes explicit content; if omitted, stdin is used.

### edit

Replace an inclusive line range within a note.

```sh
pman edit Projects/proj-22-some-project/README.md --replace-lines 20:25 --with "new text"
pman edit Projects/proj-22-some-project/README.md --replace-lines 20:25 --with "new text" --expect "old text"
```

Options:
- `--notes-dir <path>` overrides the Notes root.
- `--replace-lines <start:end>` selects the inclusive range to replace.
- `--with <text>` sets replacement text.
- `--expect <text>` guards against stale context by requiring exact current text in the selected range.

### cat/head/tail/wc/less

Notes-scoped wrappers that resolve paths from Notes root and enforce containment:

```sh
pman cat Projects/proj-22-some-project/README.md
pman head Projects/proj-22-some-project/README.md --lines 40
pman tail Projects/proj-22-some-project/README.md --lines 40
pman wc Projects/proj-22-some-project/README.md --lines --words
pman less Projects/proj-22-some-project/README.md
```

Options:
- All support `--notes-dir <path>`.
- `head` and `tail` support `--lines <n>` (default `10`).
- `wc` supports `--lines`, `--words`, `--bytes`, `--chars`.

Behavior:
- `less` automatically degrades to non-interactive `cat` behavior when no TTY is present.

### skill generate

Print a complete `SKILL.md` template to stdout.

```sh
pman skill generate > .pman/skills/para-notes/SKILL.md
pman skill generate para-notes > .pman/skills/para-notes/SKILL.md
```

Options:
- `<profile>` is optional and defaults to `para-notes`.

### mcp

Start the pman MCP server for note/project tools.

```sh
# HTTP transport (default)
pman mcp --notes-dir ~/Notes
pman mcp --bind 127.0.0.1 --port 3100 --notes-dir ~/Notes

# stdio transport (for subprocess-based MCP clients)
pman mcp --transport stdio --notes-dir ~/Notes
```

Options:
- `--transport <http|stdio>` selects MCP transport (default: `http`).
- `--notes-dir <path>` overrides the Notes root.
- HTTP-only options: `--bind`, `--port`, `--tls-cert`, `--tls-key`.

Behavior:
- `http`: serves streamable HTTP MCP at `http://<bind>:<port>/mcp` (or HTTPS with TLS flags).
- `stdio`: serves MCP over stdin/stdout for local client process spawning.

### pman-mcp (hybrid MCP shim)

Run a Python MCP server that proxies tool calls to the Rust `pman` CLI.

Install with `pipx` (recommended for the hybrid model):

```sh
pipx install "git+https://github.com/divanvisagie/pman"
```

Run:

```sh
pman-mcp --transport stdio --notes-dir ~/Notes
pman-mcp --transport streamable-http --host 127.0.0.1 --port 8000 --notes-dir ~/Notes
```

Prerequisite:
- Rust `pman` must be installed and available on `PATH` (or provide `--pman-bin`).

Options:
- `--transport <stdio|streamable-http>` selects transport mode.
- `--pman-bin <path-or-name>` overrides the Rust `pman` binary command (default: `pman`).
- `--notes-dir <path>` passes a Notes root override to proxied `pman` commands.
- `--host <host>` and `--port <port>` configure bind address for streamable HTTP transport.
- `--streamable-http-path <path>` sets the streamable HTTP endpoint path (default: `/mcp`).

Behavior:
- Exposes the same core MCP tool names as Rust `pman mcp` (`notes_read`, `notes_write`, `notes_edit`, `project_list`, `project_new`, `project_archive`).
- Uses subprocess execution of `pman` for strict core-command parity in the hybrid rollout.

## Notes

- Slugs are derived from the project name (ASCII alnum, dash-separated).
- Slugs are unique across both `Projects` and `Archives/Projects`.
- Area slugs are optional; when set, they become a prefix in the directory slug.
- Note I/O commands resolve and canonicalize paths from the Notes root, rejecting out-of-root targets.
