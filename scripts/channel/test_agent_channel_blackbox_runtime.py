#!/usr/bin/env python3
"""Unit tests for agent channel blackbox runtime helpers."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import SimpleNamespace

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

module = importlib.import_module("agent_channel_blackbox_runtime")


def _base_cfg(log_file: Path) -> SimpleNamespace:
    return SimpleNamespace(
        prompt="/session json",
        max_wait_secs=5,
        max_idle_secs=5,
        webhook_url="http://127.0.0.1:18081/telegram/webhook",
        log_file=log_file,
        chat_id=100,
        user_id=200,
        username="tester",
        chat_title=None,
        thread_id=None,
        secret_token=None,
        follow_logs=False,
        expect_events=tuple(),
        expect_reply_json_fields=tuple(),
        expect_log_regexes=tuple(),
        expect_bot_regexes=tuple(),
        forbid_log_regexes=tuple(),
        fail_fast_error_logs=False,
        allow_no_bot=True,
        allow_chat_ids=tuple(),
        strong_update_id=True,
        session_partition=None,
    )


def test_run_probe_returns_error_on_non_200_webhook(tmp_path: Path) -> None:
    cfg = _base_cfg(tmp_path / "runtime.log")
    code = module.run_probe(
        cfg,
        count_lines_fn=lambda _path: 0,
        next_update_id_fn=lambda _strong: 123,
        build_probe_message_fn=lambda prompt, _trace_id: prompt,
        build_update_payload_fn=lambda **_kwargs: "{}",
        post_webhook_update_fn=lambda _url, _payload, _secret: (403, "forbidden"),
        expected_session_keys_fn=lambda *_args: ("100:200",),
        expected_session_scope_values_fn=lambda *_args: ("telegram:100:200",),
        expected_session_scope_prefixes_fn=lambda _events: ("telegram:",),
        expected_session_key_fn=lambda *_args: "100:200",
        expected_recipient_key_fn=lambda chat_id, _thread_id: str(chat_id),
        read_new_lines_fn=lambda _path, cursor: (cursor, []),
        tail_lines_fn=lambda _path, _count: [],
        strip_ansi_fn=lambda line: line,
        extract_event_token_fn=lambda _line: None,
        extract_session_key_token_fn=lambda _line: None,
        parse_command_reply_event_line_fn=lambda _line: None,
        parse_command_reply_json_summary_line_fn=lambda _line: None,
        telegram_send_retry_grace_seconds_fn=lambda _line: None,
        parse_log_tokens_fn=lambda _line: {},
        error_patterns=("error",),
        mcp_observability_events=("mcp.pool.connect.waiting",),
        mcp_waiting_events=frozenset({"mcp.pool.connect.waiting"}),
        target_session_scope_placeholder="__target_session_scope__",
    )
    assert code == 1


def test_run_probe_allow_no_bot_succeeds_when_non_bot_expectations_met(tmp_path: Path) -> None:
    cfg = _base_cfg(tmp_path / "runtime.log")
    calls = {"read": 0}

    def _read_new_lines(_path: Path, cursor: int) -> tuple[int, list[str]]:
        calls["read"] += 1
        if calls["read"] == 1:
            return cursor + 1, ['2026-01-01T00:00:00Z INFO ← User: "/session json"']
        return cursor, []

    code = module.run_probe(
        cfg,
        count_lines_fn=lambda _path: 0,
        next_update_id_fn=lambda _strong: 456,
        build_probe_message_fn=lambda prompt, _trace_id: prompt,
        build_update_payload_fn=lambda **_kwargs: "{}",
        post_webhook_update_fn=lambda _url, _payload, _secret: (200, "ok"),
        expected_session_keys_fn=lambda *_args: ("100:200",),
        expected_session_scope_values_fn=lambda *_args: ("telegram:100:200",),
        expected_session_scope_prefixes_fn=lambda _events: ("telegram:",),
        expected_session_key_fn=lambda *_args: "100:200",
        expected_recipient_key_fn=lambda chat_id, _thread_id: str(chat_id),
        read_new_lines_fn=_read_new_lines,
        tail_lines_fn=lambda _path, _count: [],
        strip_ansi_fn=lambda line: line,
        extract_event_token_fn=lambda _line: None,
        extract_session_key_token_fn=lambda _line: None,
        parse_command_reply_event_line_fn=lambda _line: None,
        parse_command_reply_json_summary_line_fn=lambda _line: None,
        telegram_send_retry_grace_seconds_fn=lambda _line: None,
        parse_log_tokens_fn=lambda _line: {},
        error_patterns=("error",),
        mcp_observability_events=("mcp.pool.connect.waiting",),
        mcp_waiting_events=frozenset({"mcp.pool.connect.waiting"}),
        target_session_scope_placeholder="__target_session_scope__",
    )
    assert code == 0
