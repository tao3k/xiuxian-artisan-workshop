#!/usr/bin/env python3
"""Probe intro printers for channel blackbox runtime."""

from __future__ import annotations

from typing import Any


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
