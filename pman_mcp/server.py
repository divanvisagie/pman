from __future__ import annotations

from typing import Optional

from mcp.server.fastmcp import FastMCP

from .bridge import PmanBridge


def build_server(
    bridge: PmanBridge,
    host: str = "127.0.0.1",
    port: int = 8000,
    streamable_http_path: str = "/mcp",
) -> FastMCP:
    mcp = FastMCP(
        "pman-mcp",
        host=host,
        port=port,
        streamable_http_path=streamable_http_path,
    )

    @mcp.tool(description="Read a note file from the Notes directory. Returns the file contents.")
    def notes_read(
        path: str,
        lines: Optional[str] = None,
        numbered: bool = False,
    ) -> str:
        args = [path]
        if lines:
            args.extend(["--lines", lines])
        if numbered:
            args.append("--numbered")
        return bridge.run("read", *args)

    @mcp.tool(description="Write or replace a note file's full contents.")
    def notes_write(path: str, content: str, create_dirs: bool = False) -> str:
        args = [path, f"--content={content}"]
        if create_dirs:
            args.append("--create-dirs")
        return bridge.run("write", *args)

    @mcp.tool(
        description=(
            "Edit a note file by replacing an inclusive line range. Supports an optional "
            "expected-text guard to detect stale edits."
        )
    )
    def notes_edit(
        path: str,
        replace_lines: str,
        with_text: str,
        expect: Optional[str] = None,
    ) -> str:
        args = [
            "--replace-lines",
            replace_lines,
            f"--with={with_text}",
            path,
        ]
        if expect is not None:
            args.append(f"--expect={expect}")
        return bridge.run("edit", *args)

    @mcp.tool(description="List projects from the registry. Defaults to active projects.")
    def project_list(status: Optional[str] = None) -> str:
        args: list[str] = []
        if status is not None:
            args.extend(["--status", status])
        return bridge.run("list", *args)

    @mcp.tool(description="Create a new project note in Notes/Projects.")
    def project_new(name: str, status: str = "active", area: Optional[str] = None) -> str:
        args = [name, "--status", status]
        if area is not None:
            args.extend(["--area", area])
        return bridge.run("new", *args)

    @mcp.tool(description="Archive a project by moving it to Notes/Archives/Projects.")
    def project_archive(project: str) -> str:
        return bridge.run("archive", project)

    return mcp
