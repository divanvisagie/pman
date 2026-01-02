---
name: project-structure
description: Managing the ~/src reverse-domain project structure. Use when creating new projects, understanding project organization, navigating between projects, or working with cross-project resource sharing.
allowed-tools: Bash(ls:*), Bash(fd:*), Bash(mkdir:*), Read
---

# ~/src Project Structure

This directory uses a reverse-domain layout inspired by Java package conventions, organizing projects by ownership and domain.

## Directory Layout

```
~/src/
├── com/
│   ├── yourdomain/       # Personal projects (your domain)
│   ├── github/           # GitHub-hosted repos by owner
│   │   └── yourusername/ # Your GitHub repos
│   └── orgname/          # Organization projects
├── localhost/            # Local-only projects
└── Notes/                # PARA vault
```

Adapt the structure to your domains and organizations.

## Philosophy

1. **Clear Ownership**: Projects organized by owner and host
2. **Domain Mapping**: Paths mirror web domains (e.g., `com/github/user/project`)
3. **Cross-Project References**: Siblings reference each other via relative paths
4. **Namespace Collision Prevention**: Different orgs can have same-named projects
5. **Scalability**: Easy to add new organizations or domains

## Creating New Projects

Create a new directory under the appropriate domain:

```bash
mkdir -p ~/src/com/github/yourusername/new-project
cd ~/src/com/github/yourusername/new-project
git init
```

## Cross-Project Resource Sharing

Reference other projects via relative paths:

```
../../other-org/shared-lib/
../sibling-project/src/
```

## Per-Project Documentation

Each project subdirectory is its own git repository. Always read:

1. **README.md** - Primary documentation (build, test, deploy)
2. **CLAUDE.md** (if present) - Claude Code configuration

The `~/src` root itself is NOT a git repository—it's a filesystem organization strategy.

## Projects vs Repositories

A **repository** is a codebase. A **project** (in the PARA sense) is a time-bound effort—adding a feature, fixing a bug, refactoring a module. Repos don't map one-to-one to projects; a single repo may have many projects over its lifetime.

When making a change, that change belongs to a project note in `Notes/Projects/`, not to the repository itself.
