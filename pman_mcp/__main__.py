from __future__ import annotations

import argparse
from typing import Sequence

from .bridge import BridgeConfig, PmanBridge


def _parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="pman-mcp",
        description="MCP server that proxies tool calls to the Rust pman CLI.",
    )
    parser.add_argument(
        "--transport",
        choices=["stdio", "streamable-http"],
        default="stdio",
        help="MCP transport mode.",
    )
    parser.add_argument(
        "--pman-bin",
        default="pman",
        help="Path or command name for the Rust pman CLI binary.",
    )
    parser.add_argument(
        "--notes-dir",
        default=None,
        help="Optional Notes root override passed through to pman commands.",
    )
    parser.add_argument(
        "--host",
        default="127.0.0.1",
        help="Bind host for HTTP-based transports.",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=8000,
        help="Bind port for HTTP-based transports.",
    )
    parser.add_argument(
        "--streamable-http-path",
        default="/mcp",
        help="HTTP endpoint path for streamable HTTP transport.",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> None:
    args = _parse_args(argv)
    try:
        from .server import build_server
    except ModuleNotFoundError as exc:
        if exc.name == "mcp":
            raise SystemExit(
                "Missing dependency 'mcp'. Install pman-mcp with pipx/uv to pull dependencies."
            ) from exc
        raise

    bridge = PmanBridge(BridgeConfig(pman_bin=args.pman_bin, notes_dir=args.notes_dir))
    mcp = build_server(
        bridge,
        host=args.host,
        port=args.port,
        streamable_http_path=args.streamable_http_path,
    )
    mcp.run(transport=args.transport)


if __name__ == "__main__":
    main()
