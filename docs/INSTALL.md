# pman Installation

Instructions for a coding agent to set up the pman workflow in the current working directory.

## Preferred Setup (via CLI)

Install (or update) `pman` from GitHub using cargo:

```bash
cargo install --git https://github.com/divanvisagie/pman
```

Initialize the current directory as a pman workspace:

```bash
pman init .
```

This creates the Notes structure, `AGENTS.md`, and canonical skills in workspace-local paths.

## Update Embedded Resources

```bash
pman update --path .
```

## Verification

After setup, verify:

1. `./Notes/Projects/_registry.md` exists
2. `./AGENTS.md` exists
3. `./.pman/skills/project/SKILL.md` exists
4. `pman verify --path .` reports workspace OK
5. `pman --help` works

## User README

The user should create their own `./README.md` documenting:

- Directory layout and organization
- Custom tools and commands
- System-specific conventions

Agents read both `AGENTS.md` (generic workflow) and `README.md` (user-specific config).
