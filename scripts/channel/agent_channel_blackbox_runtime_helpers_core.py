#!/usr/bin/env python3
"""Core runtime-state helpers for channel blackbox probes."""

from __future__ import annotations

import re
from collections import Counter
from dataclasses import dataclass
from typing import Any


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


def print_probe_intro(cfg: Any, *, update_id: int, trace_id: str, message_text: str) -> None:
    """Print initial probe request context."""
    print("Blackbox probe posted.")
    print(f"  update_id={update_id}")
    print(f"  trace_id={trace_id}")
    print(f"  webhook_url={cfg.webhook_url}")
    print(
        f"  chat_id={cfg.chat_id} user_id={cfg.user_id} "
        f"username={cfg.username if cfg.username else '(none)'} "
        f"chat_title={cfg.chat_title if cfg.chat_title else '(none)'} "
        f"thread_id={cfg.thread_id if cfg.thread_id is not None else 'none'}"
    )
    print(f"  session_partition={cfg.session_partition or 'auto'}")
    print(f"  log_file={cfg.log_file}")
    print(f"  trace_mode={'on' if trace_id in message_text else 'off'}")
    if cfg.allow_chat_ids:
        print(f"  allow_chat_ids={list(cfg.allow_chat_ids)}")
    else:
        print("  allow_chat_ids=none (no probe-level restriction)")
    if cfg.max_wait_secs is None:
        print("  max_wait_secs=none (event-driven)")
    else:
        print(f"  max_wait_secs={cfg.max_wait_secs}")
    if cfg.max_idle_secs is None:
        print("  max_idle_secs=none")
    else:
        print(f"  max_idle_secs={cfg.max_idle_secs}")
    if cfg.expect_events:
        print(f"  expect_events={list(cfg.expect_events)}")
    if cfg.expect_reply_json_fields:
        print(
            "  expect_reply_json_fields="
            f"{[f'{key}={value}' for key, value in cfg.expect_reply_json_fields]}"
        )
    if cfg.expect_log_regexes:
        print(f"  expect_log_regexes={list(cfg.expect_log_regexes)}")
    if cfg.expect_bot_regexes:
        print(f"  expect_bot_regexes={list(cfg.expect_bot_regexes)}")
    if cfg.forbid_log_regexes:
        print(f"  forbid_log_regexes={list(cfg.forbid_log_regexes)}")
    print("")


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
