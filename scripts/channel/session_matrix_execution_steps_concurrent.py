#!/usr/bin/env python3
"""Concurrent step execution for session-matrix probes."""

from __future__ import annotations

import sys
from typing import Any


def run_concurrent_step(
    script_dir: Any,
    cfg: Any,
    *,
    name: str,
    chat_a: int,
    user_a: int,
    thread_a: int | None,
    chat_b: int,
    user_b: int,
    thread_b: int | None,
    prompt: str = "/session json",
    allow_send_failure: bool = False,
    expected_session_key_fn: Any,
    run_command_with_restart_retry_fn: Any,
    tail_text_fn: Any,
    step_result_cls: Any,
) -> Any:
    """Execute one dual-session concurrent probe."""
    session_a = expected_session_key_fn(chat_a, user_a, thread_a, cfg.session_partition)
    session_b = expected_session_key_fn(chat_b, user_b, thread_b, cfg.session_partition)
    cmd = [
        sys.executable,
        str(script_dir / "test_omni_agent_concurrent_sessions.py"),
        "--max-wait",
        str(cfg.max_wait),
        "--webhook-url",
        cfg.webhook_url,
        "--log-file",
        str(cfg.log_file),
        "--chat-id",
        str(chat_a),
        "--chat-b",
        str(chat_b),
        "--user-a",
        str(user_a),
        "--user-b",
        str(user_b),
        "--prompt",
        prompt,
    ]
    if thread_a is not None:
        cmd.extend(["--thread-a", str(thread_a)])
    if thread_b is not None:
        cmd.extend(["--thread-b", str(thread_b)])
    if cfg.username:
        cmd.extend(["--username", cfg.username])
    if cfg.secret_token:
        cmd.extend(["--secret-token", cfg.secret_token])
    if cfg.session_partition:
        cmd.extend(["--session-partition", cfg.session_partition])
    if allow_send_failure:
        cmd.append("--allow-send-failure")
    for pattern in cfg.forbid_log_regexes:
        cmd.extend(["--forbid-log-regex", pattern])

    returncode, duration_ms, stdout, stderr = run_command_with_restart_retry_fn(cmd)
    passed = returncode == 0
    return step_result_cls(
        name=name,
        kind="concurrent",
        session_key=f"{session_a} | {session_b}",
        prompt=f"{prompt} (concurrent)",
        event="telegram.command.session_status_json.replied",
        command=tuple(cmd),
        returncode=returncode,
        duration_ms=duration_ms,
        passed=passed,
        stdout_tail=tail_text_fn(stdout),
        stderr_tail=tail_text_fn(stderr),
    )
