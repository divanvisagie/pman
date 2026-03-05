from __future__ import annotations

import unittest

from pman_mcp.__main__ import _parse_args


class MainArgParseTests(unittest.TestCase):
    def test_parse_defaults(self):
        args = _parse_args([])
        self.assertEqual(args.transport, "stdio")
        self.assertEqual(args.host, "127.0.0.1")
        self.assertEqual(args.port, 8000)
        self.assertEqual(args.streamable_http_path, "/mcp")

    def test_parse_streamable_http_options(self):
        args = _parse_args(
            [
                "--transport",
                "streamable-http",
                "--host",
                "0.0.0.0",
                "--port",
                "9123",
                "--streamable-http-path",
                "/custom-mcp",
            ]
        )
        self.assertEqual(args.transport, "streamable-http")
        self.assertEqual(args.host, "0.0.0.0")
        self.assertEqual(args.port, 9123)
        self.assertEqual(args.streamable_http_path, "/custom-mcp")

if __name__ == "__main__":
    unittest.main()
