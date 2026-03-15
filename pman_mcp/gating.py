from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from enum import Enum


class VerdictAction(str, Enum):
    EXCLUDE = "exclude"
    CLAIM = "claim"


@dataclass
class Verdict:
    tool: str
    action: VerdictAction
    arguments: dict | None = None


def evaluate_gating(message: str) -> list[Verdict]:
    msg = message.lower().strip()
    verdicts: list[Verdict] = []

    # --- Claims: deterministic command matching ---
    if msg in ("/projects", "/list") or msg.startswith(("/projects ", "/list ")):
        return [Verdict("project_list", VerdictAction.CLAIM, {})]

    if msg.startswith("/new "):
        name = msg.removeprefix("/new ").strip()
        return [Verdict("project_new", VerdictAction.CLAIM, {"name": name})]

    if msg.startswith("/archive "):
        project = msg.removeprefix("/archive ").strip()
        return [Verdict("project_archive", VerdictAction.CLAIM, {"project": project})]

    # --- Excludes: keyword-based relevance ---
    write_keywords = ["create", "write", "add", "edit", "update", "change", "new", "append"]
    has_write_intent = any(k in msg for k in write_keywords)

    if not has_write_intent:
        verdicts.append(Verdict("notes_write", VerdictAction.EXCLUDE))
        verdicts.append(Verdict("notes_edit", VerdictAction.EXCLUDE))
        verdicts.append(Verdict("project_new", VerdictAction.EXCLUDE))

    archive_keywords = ["archive", "close", "done", "finish", "complete"]
    has_archive_intent = any(k in msg for k in archive_keywords)

    if not has_archive_intent:
        verdicts.append(Verdict("project_archive", VerdictAction.EXCLUDE))

    return verdicts


def gating_response(message: str) -> str:
    verdicts = evaluate_gating(message)
    return json.dumps(
        {"verdicts": [asdict(v) for v in verdicts]},
    )
