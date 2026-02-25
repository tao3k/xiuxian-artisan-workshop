#!/usr/bin/env python3
"""Probe execution helpers for memory benchmark runner."""

from __future__ import annotations

import sys
from typing import Any


def run_probe(
    config: Any,
    *,
    prompt: str,
    expect_event: str,
    allow_no_bot: bool = False,
    count_lines_fn: Any,
    read_new_lines_fn: Any,
    strip_ansi_fn: Any,
    has_event_fn: Any,
    control_admin_required_event: str,
    forbidden_log_pattern: str,
    subprocess_run_fn: Any,
    called_process_error_cls: Any,
) -> list[str]:
    """Run one black-box probe and return normalized new runtime log lines."""
    start_cursor = count_lines_fn(config.log_file)
    cmd = [
        sys.executable,
        str(config.blackbox_script),
        "--prompt",
        prompt,
        "--expect-event",
        expect_event,
        "--chat-id",
        str(config.chat_id),
        "--user-id",
        str(config.user_id),
        "--allow-chat-id",
        str(config.chat_id),
        "--max-wait",
        str(config.max_wait),
        "--max-idle-secs",
        str(config.max_idle_secs),
        "--log-file",
        str(config.log_file),
        "--no-follow",
    ]
    if config.thread_id is not None:
        cmd.extend(["--thread-id", str(config.thread_id)])
    if config.runtime_partition_mode:
        cmd.extend(["--session-partition", config.runtime_partition_mode])
    if config.fail_on_mcp_error:
        cmd.extend(["--forbid-log-regex", forbidden_log_pattern])
    else:
        cmd.append("--no-fail-fast-error-log")
    if config.username:
        cmd.extend(["--username", config.username])
    if allow_no_bot:
        cmd.append("--allow-no-bot")

    try:
        subprocess_run_fn(cmd, check=True)
    except called_process_error_cls as error:
        _, lines = read_new_lines_fn(config.log_file, start_cursor)
        normalized_lines = [strip_ansi_fn(line) for line in lines]
        if prompt.startswith("/") and has_event_fn(normalized_lines, control_admin_required_event):
            raise RuntimeError(
                "control command denied (admin_required): "
                "set --user-id to an admin-capable Telegram user for benchmark control flows."
            ) from error
        raise

    _, lines = read_new_lines_fn(config.log_file, start_cursor)
    return [strip_ansi_fn(line) for line in lines]


def run_reset(
    config: Any,
    *,
    run_probe_fn: Any,
    reset_event: str,
) -> None:
    """Execute reset command probe."""
    run_probe_fn(
        config,
        prompt="/reset",
        expect_event=reset_event,
        allow_no_bot=True,
    )


def run_feedback(
    config: Any,
    direction: str,
    *,
    run_probe_fn: Any,
    feedback_event: str,
) -> list[str]:
    """Execute adaptive feedback command probe."""
    prompt = "/session feedback up json" if direction == "up" else "/session feedback down json"
    return run_probe_fn(config, prompt=prompt, expect_event=feedback_event)


def run_non_command_turn(
    config: Any,
    prompt: str,
    *,
    run_probe_fn: Any,
    recall_plan_event: str,
) -> list[str]:
    """Execute one regular non-command prompt turn."""
    return run_probe_fn(config, prompt=prompt, expect_event=recall_plan_event)
