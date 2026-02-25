#!/usr/bin/env python3
"""Blackbox step execution for session-matrix probes."""

from __future__ import annotations

import re
import sys
from typing import Any


def run_blackbox_step(
    script_dir: Any,
    cfg: Any,
    step: Any,
    *,
    expected_session_key_fn: Any,
    run_command_with_restart_retry_fn: Any,
    tail_text_fn: Any,
    step_result_cls: Any,
) -> Any:
    """Execute one single-session blackbox probe."""
    session_key = expected_session_key_fn(
        step.chat_id,
        step.user_id,
        step.thread_id,
        cfg.session_partition,
    )
    escaped_key = re.escape(session_key)
    allowed_chat_ids = tuple(dict.fromkeys((cfg.chat_id, cfg.chat_b, cfg.chat_c)))
    cmd = [
        sys.executable,
        str(script_dir / "agent_channel_blackbox.py"),
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
        str(step.chat_id),
        "--user-id",
        str(step.user_id),
        "--expect-log-regex",
        rf'session_key="?{escaped_key}"?',
    ]
    for allowed_chat_id in allowed_chat_ids:
        cmd.extend(["--allow-chat-id", str(allowed_chat_id)])
    if step.event is not None:
        cmd.extend(["--expect-event", step.event])
    if step.thread_id is not None:
        cmd.extend(["--thread-id", str(step.thread_id)])
    if cfg.username:
        cmd.extend(["--username", cfg.username])
    if cfg.secret_token:
        cmd.extend(["--secret-token", cfg.secret_token])
    if cfg.session_partition:
        cmd.extend(["--session-partition", cfg.session_partition])
    for field in step.expect_reply_json_fields:
        cmd.extend(["--expect-reply-json-field", field])
    for pattern in cfg.forbid_log_regexes:
        cmd.extend(["--forbid-log-regex", pattern])

    returncode, duration_ms, stdout, stderr = run_command_with_restart_retry_fn(cmd)
    passed = returncode == 0
    return step_result_cls(
        name=step.name,
        kind="blackbox",
        session_key=session_key,
        prompt=step.prompt,
        event=step.event,
        command=tuple(cmd),
        returncode=returncode,
        duration_ms=duration_ms,
        passed=passed,
        stdout_tail=tail_text_fn(stdout),
        stderr_tail=tail_text_fn(stderr),
    )
