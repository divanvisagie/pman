from pman_mcp.gating import VerdictAction, evaluate_gating


def _tools(verdicts):
    return {(v.tool, v.action) for v in verdicts}


def _claimed(verdicts):
    return [(v.tool, v.arguments) for v in verdicts if v.action == VerdictAction.CLAIM]


def _excluded(verdicts):
    return [v.tool for v in verdicts if v.action == VerdictAction.EXCLUDE]


# --- Claim tests ---


class TestClaims:
    def test_slash_projects(self):
        vs = evaluate_gating("/projects")
        assert _claimed(vs) == [("project_list", {})]

    def test_slash_list(self):
        vs = evaluate_gating("/list")
        assert _claimed(vs) == [("project_list", {})]

    def test_slash_new(self):
        vs = evaluate_gating("/new My Cool Feature")
        assert _claimed(vs) == [("project_new", {"name": "my cool feature"})]

    def test_slash_archive(self):
        vs = evaluate_gating("/archive proj-99")
        assert _claimed(vs) == [("project_archive", {"project": "proj-99"})]

    def test_claim_returns_single_verdict(self):
        vs = evaluate_gating("/projects")
        assert len(vs) == 1


# --- Exclude tests ---


class TestExcludes:
    def test_read_only_excludes_write_tools(self):
        vs = evaluate_gating("What does proj-145 say?")
        excluded = _excluded(vs)
        assert "notes_write" in excluded
        assert "notes_edit" in excluded
        assert "project_new" in excluded
        assert "project_archive" in excluded

    def test_read_only_does_not_exclude_read_tools(self):
        vs = evaluate_gating("Show me the project list")
        excluded = _excluded(vs)
        assert "notes_read" not in excluded
        assert "project_list" not in excluded

    def test_write_intent_includes_write_tools(self):
        vs = evaluate_gating("Create a new project for X")
        excluded = _excluded(vs)
        assert "notes_write" not in excluded
        assert "notes_edit" not in excluded
        assert "project_new" not in excluded

    def test_archive_intent_includes_archive(self):
        vs = evaluate_gating("Archive proj-42")
        excluded = _excluded(vs)
        assert "project_archive" not in excluded

    def test_gating_tool_never_in_verdicts(self):
        for msg in ["hello", "/projects", "edit the file", "archive proj-1"]:
            vs = evaluate_gating(msg)
            tools = [v.tool for v in vs]
            assert "_tool_gating" not in tools


# --- Default include tests ---


class TestDefaults:
    def test_ambiguous_returns_minimal_excludes(self):
        vs = evaluate_gating("hello")
        excluded = _excluded(vs)
        # Write tools excluded (no write intent), archive excluded (no archive intent)
        assert "notes_write" in excluded
        assert "notes_edit" in excluded
        assert "project_new" in excluded
        assert "project_archive" in excluded
        # Read tools always included
        assert "notes_read" not in excluded
        assert "project_list" not in excluded

    def test_empty_message(self):
        vs = evaluate_gating("")
        # Should not crash, returns excludes for write/archive
        assert isinstance(vs, list)
