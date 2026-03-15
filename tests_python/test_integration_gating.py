"""Integration tests for _tool_gating via MCP stdio transport.

Spawns the pman-mcp server as a subprocess, connects via the MCP SDK's
stdio client, and verifies:
  1. tools/list includes _tool_gating alongside the 6 pman tools
  2. _tool_gating returns correct claim/exclude verdicts
  3. Ambiguous messages produce minimal verdicts
"""

from __future__ import annotations

import asyncio
import json
import sys
import unittest

from mcp.client.session import ClientSession
from mcp.client.stdio import stdio_client, StdioServerParameters


PMAN_MCP_PYTHON = sys.executable

SERVER_PARAMS = StdioServerParameters(
    command=PMAN_MCP_PYTHON,
    args=["-m", "pman_mcp", "--transport", "stdio"],
)


async def _list_tools() -> set[str]:
    async with stdio_client(SERVER_PARAMS) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()
            result = await session.list_tools()
            return {t.name for t in result.tools}


async def _call_gating(message: str) -> dict:
    async with stdio_client(SERVER_PARAMS) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()
            result = await session.call_tool(
                "_tool_gating", {"message": message}
            )
            return json.loads(result.content[0].text)


def _run(coro):
    loop = asyncio.new_event_loop()
    try:
        return loop.run_until_complete(coro)
    finally:
        loop.close()


class TestToolsList(unittest.TestCase):
    def test_tool_gating_in_tools_list(self):
        names = _run(_list_tools())
        expected = {
            "notes_read",
            "notes_write",
            "notes_edit",
            "project_list",
            "project_new",
            "project_archive",
            "_tool_gating",
        }
        self.assertEqual(names, expected, f"Expected 7 tools, got: {names}")


class TestGatingClaims(unittest.TestCase):
    def test_slash_projects_claim(self):
        resp = _run(_call_gating("/projects"))
        verdicts = resp["verdicts"]
        self.assertEqual(len(verdicts), 1)
        self.assertEqual(verdicts[0]["tool"], "project_list")
        self.assertEqual(verdicts[0]["action"], "claim")

    def test_slash_list_claim(self):
        resp = _run(_call_gating("/list"))
        verdicts = resp["verdicts"]
        self.assertEqual(len(verdicts), 1)
        self.assertEqual(verdicts[0]["tool"], "project_list")
        self.assertEqual(verdicts[0]["action"], "claim")

    def test_slash_new_claim(self):
        resp = _run(_call_gating("/new My Cool Feature"))
        verdicts = resp["verdicts"]
        self.assertEqual(len(verdicts), 1)
        self.assertEqual(verdicts[0]["tool"], "project_new")
        self.assertEqual(verdicts[0]["action"], "claim")
        self.assertEqual(verdicts[0]["arguments"], {"name": "my cool feature"})

    def test_slash_archive_claim(self):
        resp = _run(_call_gating("/archive proj-99"))
        verdicts = resp["verdicts"]
        self.assertEqual(len(verdicts), 1)
        self.assertEqual(verdicts[0]["tool"], "project_archive")
        self.assertEqual(verdicts[0]["action"], "claim")
        self.assertEqual(verdicts[0]["arguments"], {"project": "proj-99"})


class TestGatingExcludes(unittest.TestCase):
    def _excluded_tools(self, message: str) -> set[str]:
        resp = _run(_call_gating(message))
        return {
            v["tool"]
            for v in resp["verdicts"]
            if v["action"] == "exclude"
        }

    def test_read_only_excludes_write_tools(self):
        excluded = self._excluded_tools("What does proj-145 say?")
        self.assertIn("notes_write", excluded)
        self.assertIn("notes_edit", excluded)
        self.assertIn("project_new", excluded)
        self.assertIn("project_archive", excluded)
        self.assertNotIn("notes_read", excluded)
        self.assertNotIn("project_list", excluded)

    def test_write_intent_includes_write_tools(self):
        excluded = self._excluded_tools("Create a new project for X")
        self.assertNotIn("notes_write", excluded)
        self.assertNotIn("notes_edit", excluded)
        self.assertNotIn("project_new", excluded)

    def test_archive_intent_includes_archive(self):
        excluded = self._excluded_tools("Archive proj-42")
        self.assertNotIn("project_archive", excluded)

    def test_gating_tool_never_in_verdicts(self):
        for msg in ["hello", "/projects", "edit the file", "archive proj-1"]:
            resp = _run(_call_gating(msg))
            tools = [v["tool"] for v in resp["verdicts"]]
            self.assertNotIn("_tool_gating", tools, f"_tool_gating leaked for: {msg}")


class TestGatingAmbiguous(unittest.TestCase):
    def test_ambiguous_message(self):
        resp = _run(_call_gating("hello"))
        excluded = {v["tool"] for v in resp["verdicts"] if v["action"] == "exclude"}
        self.assertIn("notes_write", excluded)
        self.assertIn("project_archive", excluded)
        self.assertNotIn("notes_read", excluded)
        self.assertNotIn("project_list", excluded)

    def test_empty_message(self):
        resp = _run(_call_gating(""))
        self.assertIn("verdicts", resp)
        self.assertIsInstance(resp["verdicts"], list)


if __name__ == "__main__":
    unittest.main()
