---
name: project-structure
description: Understanding workspace organization. Use when navigating between projects, understanding project boundaries, or working with cross-project references.
allowed-tools: Bash(ls:*), Bash(fd:*), Read
---

# Project Structure

The workspace contains multiple independent projects. See the workspace README.md for the specific directory layout.

## Core Concepts

- Each project subdirectory is its own git repository
- The workspace root is not a git repository
- Projects can reference each other via relative paths

## Before Working on a Project

1. Read the project's README.md
2. Check for a project-specific CLAUDE.md
3. Use the build/test commands specified in the README

## Projects vs Repositories

A **repository** is a codebase.

A **project** (in the PARA sense) is a time-bound effort—adding a feature, fixing a bug, refactoring a module.

Repos don't map one-to-one to projects:
- A single repo may have many projects over its lifetime
- A project might touch multiple repos

When making a change, that change belongs to a project note in `Notes/Projects/`.

## Cross-Project References

Reference other projects via relative paths:

```
../sibling-project/src/
../../other-org/shared-lib/
```

## Finding Projects

```bash
fd README.md              # Find all project READMEs
fd CLAUDE.md              # Find project-specific Claude configs
ls <workspace-path>       # List top-level directories
```
