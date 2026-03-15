# pman

## Architecture

pman has two layers:

1. **Rust CLI** (`src/`) — the `pman` binary, handles all note/project operations
2. **Python MCP server** (`pman_mcp/`) — FastMCP wrapper that proxies tool calls to the CLI via subprocess

These are separate concerns. The MCP server is a thin bridge, not a reimplementation.

## Building & Testing

```bash
# Rust CLI
cargo build
cargo test

# Python MCP server (venv at .venv/)
.venv/bin/python -m pytest tests_python/ -v
```

If pytest is not installed: `.venv/bin/pip install pytest`

## Python environment

The venv is at `.venv/`. Always use `.venv/bin/python` — system `python3` won't have the MCP dependencies.

## MCP Server Notes

- The MCP server lives in `pman_mcp/`, not `src/`
- `src/mcp.rs` was a legacy Rust MCP server — it has been removed (Phase 0 of PROJ-167)
- FastMCP does not easily expose custom `experimental` capabilities — avoid trying to inject them via `_mcp_server` internals
- Server-side tool gating uses a `_tool_gating` tool (not a resource) — the tool's presence in `tools/list` signals gating support, no capability flag needed
