#!/usr/bin/env python3
"""Core runtime-state helpers for channel blackbox probes."""

from __future__ import annotations

import importlib
import re
from collections import Counter
from dataclasses import dataclass
from typing import Any

_INTRO = importlib.import_module("agent_channel_blackbox_runtime_helpers_intro")
print_probe_intro = _INTRO.print_probe_intro


@dataclass
class ProbeRuntimeState:
    """Mutable state for one blackbox probe loop."""

    expect_log_compiled: list[re.Pattern[str]]
    expect_bot_compiled: list[re.Pattern[str]]
    forbid_log_compiled: list[re.Pattern[str]]
    expected_sessions: tuple[str, ...]
    expected_session_scopes: tuple[str, ...]
    expected_session: str
    expected_recipient: str
    matched_expect_events: list[bool]
    matched_expect_reply_json_fields: list[bool]
    matched_expect_log: list[bool]
    matched_expect_bot: list[bool]
    bot_observations: list[str]
    command_reply_observations: list[dict[str, object]]
    json_reply_summary_observations: list[dict[str, str]]
    mcp_event_counts: Counter[str]
    mcp_last_event: str | None
    mcp_waiting_seen: bool


def build_probe_runtime_state(
    cfg: Any,
    *,
    expected_session_keys_fn: Any,
    expected_session_scope_values_fn: Any,
    expected_session_scope_prefixes_fn: Any,
    expected_session_key_fn: Any,
    expected_recipient_key_fn: Any,
) -> ProbeRuntimeState:
    """Build initialized runtime state from config + session-key helpers."""
    return ProbeRuntimeState(
        expect_log_compiled=[re.compile(p) for p in cfg.expect_log_regexes],
        expect_bot_compiled=[re.compile(p) for p in cfg.expect_bot_regexes],
        forbid_log_compiled=[re.compile(p) for p in cfg.forbid_log_regexes],
        expected_sessions=expected_session_keys_fn(
            cfg.chat_id,
            cfg.user_id,
            cfg.thread_id,
            cfg.session_partition,
        ),
        expected_session_scopes=expected_session_scope_values_fn(
            cfg.chat_id,
            cfg.user_id,
            cfg.thread_id,
            cfg.session_partition,
            expected_session_scope_prefixes_fn(cfg.expect_events),
        ),
        expected_session=expected_session_key_fn(
            cfg.chat_id,
            cfg.user_id,
            cfg.thread_id,
            cfg.session_partition,
        ),
        expected_recipient=expected_recipient_key_fn(cfg.chat_id, cfg.thread_id),
        matched_expect_events=[False] * len(cfg.expect_events),
        matched_expect_reply_json_fields=[False] * len(cfg.expect_reply_json_fields),
        matched_expect_log=[False] * len(cfg.expect_log_regexes),
        matched_expect_bot=[False] * len(cfg.expect_bot_regexes),
        bot_observations=[],
        command_reply_observations=[],
        json_reply_summary_observations=[],
        mcp_event_counts=Counter(),
        mcp_last_event=None,
        mcp_waiting_seen=False,
    )
