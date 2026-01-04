# pman Installation

Instructions for Claude Code to set up the pman workflow in the current working directory.

**Important**: Before creating any file or directory, check if it already exists. Never overwrite existing files or directories. Skip any step where the target already exists.

## Directory Structure

Create these directories relative to the current working directory:

```
./Notes/
  Projects/
  Areas/
  Resources/
  Archives/
    Projects/
```

## Registry File

Create `./Notes/Projects/_registry.md` with this content:

```markdown
# Project Registry

Flat list of project notes. IDs are chronological and unique across all projects.

| ID | Name | Status | Created | Note |
| --- | --- | --- | --- | --- |
```

## CLAUDE.md

Download from:
```
https://raw.githubusercontent.com/divanvisagie/pman/master/resources/CLAUDE.md
```

Place at: `./CLAUDE.md`

## Skills

Download the skills directories from:
```
https://github.com/divanvisagie/pman/tree/master/resources/skills/para-notes
https://github.com/divanvisagie/pman/tree/master/resources/skills/project-structure
```

Or clone the repo and copy:
```bash
git clone https://github.com/divanvisagie/pman.git /tmp/pman
mkdir -p ./.claude/skills
cp -r /tmp/pman/resources/skills/* ./.claude/skills/
rm -rf /tmp/pman
```

Install to: `./.claude/skills/` (workspace-local)

## CLI (Recommended)

Install the pman CLI for creating and archiving projects:

```bash
cargo install --git https://github.com/divanvisagie/pman
```

## Verification

After setup, verify:

1. `./Notes/Projects/_registry.md` exists
2. `./CLAUDE.md` exists
3. `./.claude/skills/para-notes/SKILL.md` exists
4. `./.claude/skills/project-structure/SKILL.md` exists
5. `pman --help` works (if CLI installed)

## User README

The user should create their own `./README.md` documenting:

- Directory layout and organization
- Custom tools and commands
- System-specific conventions

Claude reads both `CLAUDE.md` (generic workflow) and `README.md` (user-specific config).
