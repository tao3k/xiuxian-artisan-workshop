#!/usr/bin/env python3
"""Command construction helpers for complex scenario step runs."""

from __future__ import annotations

import sys
from typing import Any


def build_step_command(
    *,
    cfg: Any,
    step: Any,
    session: Any,
    expected_session_log_regex_fn: Any,
) -> tuple[list[str], str]:
    """Build blackbox command and expected session key regex for one step."""
    allowed_chat_ids = tuple(dict.fromkeys(identity.chat_id for identity in cfg.sessions))
    expected_regex = expected_session_log_regex_fn(
        session.chat_id,
        session.user_id,
        session.thread_id,
        cfg.runtime_partition_mode,
    )
    cmd = [
        sys.executable,
        str(cfg.blackbox_script),
        "--prompt",
        step.prompt,
        "--max-wait",
        str(cfg.max_wait),
        "--max-idle-secs",
        str(cfg.max_idle_secs),
        "--webhook-url",
        cfg.webhook_url,
        "--log-file",
        str(cfg.log_file),
        "--chat-id",
        str(session.chat_id),
        "--user-id",
        str(session.user_id),
        "--expect-log-regex",
        expected_regex,
    ]
    for allowed_chat_id in allowed_chat_ids:
        cmd.extend(["--allow-chat-id", str(allowed_chat_id)])
    if cfg.runtime_partition_mode:
        cmd.extend(["--session-partition", cfg.runtime_partition_mode])
    if step.expect_event:
        cmd.extend(["--expect-event", step.expect_event])
    if session.thread_id is not None:
        cmd.extend(["--thread-id", str(session.thread_id)])
    if cfg.username:
        cmd.extend(["--username", cfg.username])
    if session.chat_title:
        cmd.extend(["--chat-title", session.chat_title])
    if cfg.secret_token:
        cmd.extend(["--secret-token", cfg.secret_token])
    for field in step.expect_reply_json_fields:
        cmd.extend(["--expect-reply-json-field", field])
    for pattern in step.expect_log_regexes:
        cmd.extend(["--expect-log-regex", pattern])
    for pattern in step.expect_bot_regexes:
        cmd.extend(["--expect-bot-regex", pattern])
    for pattern in cfg.forbid_log_regexes:
        cmd.extend(["--forbid-log-regex", pattern])
    for pattern in step.forbid_log_regexes:
        cmd.extend(["--forbid-log-regex", pattern])
    if step.allow_no_bot:
        cmd.append("--allow-no-bot")
    return cmd, expected_regex
