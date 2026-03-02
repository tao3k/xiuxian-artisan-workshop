#!/usr/bin/env python3
"""Unit tests for step-scoped state observation processing."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import SimpleNamespace

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

module = importlib.import_module("agent_channel_blackbox_runtime_loop_poll_chunk_observe")


def _runtime_state() -> SimpleNamespace:
    return SimpleNamespace(expected_sessions=("1304799691:1304799692",))


def _loop_state(*, seen_user_dispatch: bool = False) -> SimpleNamespace:
    return SimpleNamespace(
        webhook_seen=False,
        dedup_duplicate_line="",
        dispatch_session_mismatch_line="",
        seen_trace=False,
        seen_user_dispatch=seen_user_dispatch,
        seen_bot=False,
        bot_line="",
        error_line="",
    )


def _helpers(marked: list[str]) -> SimpleNamespace:
    return SimpleNamespace(
        mark_expect_bot_patterns=lambda _state, line: marked.append(line),
    )


def test_process_state_lines_ignores_pre_dispatch_bot_lines_in_same_chunk() -> None:
    cfg = SimpleNamespace(prompt="Decay probe B2", fail_fast_error_logs=False)
    runtime_state = _runtime_state()
    loop_state = _loop_state()
    marked: list[str] = []

    exit_code = module.process_state_lines(
        cfg=cfg,
        runtime_state=runtime_state,
        loop_state=loop_state,
        update_id=42,
        trace_mode=False,
        trace_id="bbx-42",
        normalized_chunk=[
            '2026-01-01T00:00:01Z INFO omni: → Bot: "DECAY-B1"',
            '2026-01-01T00:00:02Z INFO omni: ← User: "No tools needed. Decay probe B2"',
            '2026-01-01T00:00:03Z INFO omni: → Bot: "DECAY-B2"',
        ],
        extract_session_key_token_fn=lambda _line: None,
        error_patterns=("ERROR",),
        helpers_module=_helpers(marked),
    )

    assert exit_code is None
    assert loop_state.seen_user_dispatch is True
    assert loop_state.seen_bot is True
    assert loop_state.bot_line.endswith('→ Bot: "DECAY-B2"')
    assert marked == ['2026-01-01T00:00:03Z INFO omni: → Bot: "DECAY-B2"']


def test_process_state_lines_keeps_processing_when_dispatch_seen_in_previous_chunk() -> None:
    cfg = SimpleNamespace(prompt="ignored", fail_fast_error_logs=False)
    runtime_state = _runtime_state()
    loop_state = _loop_state(seen_user_dispatch=True)
    marked: list[str] = []

    exit_code = module.process_state_lines(
        cfg=cfg,
        runtime_state=runtime_state,
        loop_state=loop_state,
        update_id=7,
        trace_mode=False,
        trace_id="bbx-7",
        normalized_chunk=['2026-01-01T00:00:03Z INFO omni: → Bot: "READY"'],
        extract_session_key_token_fn=lambda _line: None,
        error_patterns=("ERROR",),
        helpers_module=_helpers(marked),
    )

    assert exit_code is None
    assert loop_state.seen_bot is True
    assert loop_state.bot_line.endswith('→ Bot: "READY"')
    assert marked == ['2026-01-01T00:00:03Z INFO omni: → Bot: "READY"']


def test_process_state_lines_fail_fast_ignores_pre_dispatch_error_and_stops_on_post_dispatch_error() -> (
    None
):
    cfg = SimpleNamespace(prompt="Decay probe C2", fail_fast_error_logs=True)
    runtime_state = _runtime_state()
    loop_state = _loop_state()
    marked: list[str] = []

    exit_code = module.process_state_lines(
        cfg=cfg,
        runtime_state=runtime_state,
        loop_state=loop_state,
        update_id=9,
        trace_mode=False,
        trace_id="bbx-9",
        normalized_chunk=[
            "2026-01-01T00:00:01Z ERROR old failure before dispatch",
            '2026-01-01T00:00:02Z INFO omni: ← User: "No tools needed. Decay probe C2"',
            "2026-01-01T00:00:03Z ERROR current failure after dispatch",
        ],
        extract_session_key_token_fn=lambda _line: None,
        error_patterns=("ERROR",),
        helpers_module=_helpers(marked),
    )

    assert exit_code == 6
    assert loop_state.error_line.endswith("ERROR current failure after dispatch")
    assert marked == []
