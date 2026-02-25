#!/usr/bin/env python3
"""Thread/topic inference helpers for command-events runtime context."""

from __future__ import annotations

import re
from dataclasses import replace
from typing import Any

from command_events_runtime_context_env import read_log_tail_lines, runtime_log_file

ANSI_ESCAPE_RE = re.compile(r"\x1b\[[0-9;]*m")
SESSION_KEY_RE = re.compile(r"\bsession_key\s*=\s*(?:\"|')?([-\d]+(?::[-\d]+){1,2})(?:\"|')?")
PARSED_MESSAGE_CHAT_ID_RE = re.compile(r"\bchat_id\s*=\s*Some\((-?\d+)\)")
MESSAGE_THREAD_ID_RE = re.compile(r"\bmessage_thread_id\s*=\s*Some\((\d+)\)")


def infer_group_thread_id_from_runtime_log(
    chat_id: int | None,
    *,
    read_log_tail_lines_fn: Any,
) -> int | None:
    """Infer topic thread id for group chat from runtime log entries."""
    if chat_id is None:
        return None
    log_path = runtime_log_file()
    if not log_path.exists():
        return None

    lines = read_log_tail_lines(log_path, read_log_tail_lines_fn=read_log_tail_lines_fn)
    for raw_line in reversed(lines):
        line = ANSI_ESCAPE_RE.sub("", raw_line)
        if "Parsed message, forwarding to agent" not in line:
            continue
        chat_match = PARSED_MESSAGE_CHAT_ID_RE.search(line)
        if chat_match and int(chat_match.group(1)) != int(chat_id):
            continue
        session_key_match = SESSION_KEY_RE.search(line)
        if session_key_match:
            parts = session_key_match.group(1).split(":")
            if len(parts) == 3:
                try:
                    thread_id = int(parts[1])
                except ValueError:
                    thread_id = 0
                if thread_id > 0:
                    return thread_id
        message_thread_match = MESSAGE_THREAD_ID_RE.search(line)
        if message_thread_match:
            thread_id = int(message_thread_match.group(1))
            if thread_id > 0:
                return thread_id
    return None


def apply_runtime_partition_defaults(case: Any, partition_mode: str | None) -> Any:
    """Apply default thread when chat_thread_user partitioning is active."""
    if partition_mode != "chat_thread_user":
        return case
    if case.thread_id is not None:
        return case
    return replace(case, thread_id=0)


def resolve_topic_thread_pair(
    *,
    primary_thread_id: int | None,
    secondary_thread_id: int | None,
) -> tuple[int, int] | None:
    """Resolve distinct topic threads for isolation assertions."""
    if primary_thread_id is None and secondary_thread_id is None:
        return None
    if primary_thread_id is None:
        raise ValueError(
            "--group-thread-id-b requires --group-thread-id "
            "(or OMNI_TEST_GROUP_THREAD_ID) to be set."
        )

    resolved_secondary = secondary_thread_id
    if resolved_secondary is None:
        resolved_secondary = primary_thread_id + 1
    if int(resolved_secondary) == int(primary_thread_id):
        raise ValueError(
            f"group topic isolation requires distinct thread ids; got both={primary_thread_id}."
        )
    return int(primary_thread_id), int(resolved_secondary)
