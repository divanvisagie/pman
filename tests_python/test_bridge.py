from __future__ import annotations

import tempfile
import unittest
from unittest.mock import patch

from pman_mcp.bridge import BridgeConfig, PmanBridge


class BridgeTests(unittest.TestCase):
    @patch("pman_mcp.bridge.shutil.which", return_value="/usr/bin/pman")
    def test_run_builds_command_with_notes_dir(self, _which):
        bridge = PmanBridge(BridgeConfig(pman_bin="pman", notes_dir="/tmp/Notes"))

        with patch("pman_mcp.bridge.subprocess.run") as run_mock:
            run_mock.return_value.returncode = 0
            run_mock.return_value.stdout = "ok\n"
            run_mock.return_value.stderr = ""

            out = bridge.run("read", "Projects/proj-1/README.md")

        self.assertEqual(out, "ok\n")
        run_mock.assert_called_once_with(
            [
                "pman",
                "read",
                "--notes-dir",
                "/tmp/Notes",
                "Projects/proj-1/README.md",
            ],
            check=False,
            capture_output=True,
            text=True,
        )

    @patch("pman_mcp.bridge.shutil.which", return_value="/usr/bin/pman")
    def test_run_raises_on_failure(self, _which):
        bridge = PmanBridge(BridgeConfig(pman_bin="pman", notes_dir=None))

        with patch("pman_mcp.bridge.subprocess.run") as run_mock:
            run_mock.return_value.returncode = 1
            run_mock.return_value.stdout = ""
            run_mock.return_value.stderr = "boom"

            with self.assertRaises(RuntimeError) as ctx:
                bridge.run("list", "--status", "active")

        self.assertIn("pman command failed", str(ctx.exception))
        self.assertIn("boom", str(ctx.exception))

    @patch("pman_mcp.bridge.shutil.which", return_value=None)
    def test_validate_binary_fails_if_not_found_in_path(self, _which):
        with self.assertRaises(FileNotFoundError):
            PmanBridge(BridgeConfig(pman_bin="missing-pman"))

    def test_validate_binary_accepts_existing_absolute_path(self):
        with tempfile.NamedTemporaryFile() as tmp_file:
            bridge = PmanBridge(BridgeConfig(pman_bin=tmp_file.name))
        self.assertIsInstance(bridge, PmanBridge)


if __name__ == "__main__":
    unittest.main()
