# pman CLI Reference

This document covers the CLI behavior. The workflow manual lives in the README.

## Install

```sh
cargo install --path /path/to/pman
```

Or run directly:

```sh
cargo run -- new "Project Name" --status active
```

## Commands

### new

Create a new project note and registry entry.

```sh
pman new "Project Name" --status active
pman new "Runes Notes" --area religion
```

Creates:
- `Notes/Projects/proj-<n>-<slug>/README.md`
- Appends to `Notes/Projects/_registry.md`

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
- `Notes/Projects/proj-22-*/` -> `Notes/Archives/Projects/proj-22-*/`
- Updates registry status to `archived` with the new path.

Options:
- `--notes-dir <path>` overrides the Notes root.

## Notes

- Slugs are derived from the project name (ASCII alnum, dash-separated).
- Slugs are unique across both `Projects` and `Archives/Projects`.
- Area slugs are optional; when set, they become a prefix in the directory slug.
