#!/usr/bin/env python3
"""Runtime orchestration helpers for memory-suite black-box probes."""

from __future__ import annotations

import os
import sys
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def resolve_allowed_chat_ids(env_get_fn: Any) -> list[str]:
    """Resolve allowed chat ids from env with fallback priority."""
    allow_chat_ids = [
        token.strip()
        for token in str(env_get_fn("OMNI_BLACKBOX_ALLOWED_CHAT_IDS", "")).split(",")
        if token.strip()
    ]
    if allow_chat_ids:
        return allow_chat_ids
    single_chat = str(env_get_fn("OMNI_TEST_CHAT_ID", "")).strip()
    if single_chat:
        return [single_chat]
    return []


def run_blackbox_suite(
    script_dir: Path,
    *,
    max_wait: int,
    max_idle_secs: int,
    username: str,
    require_live_turn: bool,
    forbidden_log_pattern: str,
    run_command_fn: Any,
    resolve_runtime_partition_mode_fn: Any,
    blackbox_cases_fn: Any,
    env_get_fn: Any = os.environ.get,
    python_executable: str = sys.executable,
) -> None:
    """Run all black-box probes for memory suite."""
    blackbox_script = script_dir / "agent_channel_blackbox.py"
    if not blackbox_script.exists():
        raise FileNotFoundError(f"black-box script not found: {blackbox_script}")
    runtime_partition_mode = resolve_runtime_partition_mode_fn()
    if runtime_partition_mode:
        print(
            f"Resolved runtime session partition mode for black-box probes: {runtime_partition_mode}",
            flush=True,
        )
    allow_chat_ids = resolve_allowed_chat_ids(env_get_fn)
    for case in blackbox_cases_fn(require_live_turn):
        cmd = [
            python_executable,
            str(blackbox_script),
            "--prompt",
            case.prompt,
            "--expect-event",
            case.expected_event,
            "--forbid-log-regex",
            forbidden_log_pattern,
            "--max-wait",
            str(max_wait),
            "--max-idle-secs",
            str(max_idle_secs),
        ]
        for allowed_chat_id in allow_chat_ids:
            cmd.extend(["--allow-chat-id", allowed_chat_id])
        if username.strip():
            cmd.extend(["--username", username.strip()])
        if runtime_partition_mode:
            cmd.extend(["--session-partition", runtime_partition_mode])
        cmd.extend(case.extra_args)
        run_command_fn(cmd, title=f"Black-box probe: {case.prompt}")
