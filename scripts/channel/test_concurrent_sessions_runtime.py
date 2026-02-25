#!/usr/bin/env python3
"""Unit tests for concurrent session runtime helpers."""

from __future__ import annotations

import importlib
import re
import sys
from pathlib import Path
from types import SimpleNamespace

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

models_module = importlib.import_module("concurrent_sessions_models")
runtime_module = importlib.import_module("concurrent_sessions_runtime")


def test_read_new_lines_returns_cursor_and_lines() -> None:
    next_cursor, lines = runtime_module.read_new_lines(
        Path(".run/logs/omni-agent-webhook.log"),
        3,
        log_cursor_cls=SimpleNamespace,
        read_new_log_lines_with_cursor_fn=lambda _path, _cursor: (
            SimpleNamespace(kind="offset", value=17),
            ["line-a", "line-b"],
        ),
    )
    assert next_cursor == 17
    assert lines == ["line-a", "line-b"]


def test_collect_observation_counts_expected_events() -> None:
    lines = [
        'DEBUG event="telegram.dedup.update_accepted" update_id=101',
        'DEBUG event="telegram.dedup.update_accepted" update_id=202',
        "INFO Parsed message, forwarding to agent session_key=130:1",
        "INFO Parsed message, forwarding to agent session_key=130:2",
        (
            'INFO telegram command reply sent event="telegram.command.session_status_json.replied" '
            'session_key="130:1" recipient="130"'
        ),
        (
            'INFO telegram command reply sent event="telegram.command.session_status_json.replied" '
            'session_key="130:2" recipient="130"'
        ),
    ]
    observation = runtime_module.collect_observation(
        lines,
        update_a=101,
        update_b=202,
        key_a_candidates=("130:1",),
        key_b_candidates=("130:2",),
        forbid_log_regexes=(),
        strip_ansi_fn=lambda value: value,
        session_key_re=re.compile(r"\bsession_key(?:=|=\"?)([-\d:]+)"),
        observation_cls=models_module.Observation,
    )
    assert observation.accepted_a == 1
    assert observation.accepted_b == 1
    assert observation.parsed_a == 1
    assert observation.parsed_b == 1
    assert observation.replied_a == 1
    assert observation.replied_b == 1
