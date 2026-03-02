#!/usr/bin/env python3
"""Case execution helpers for command-events runtime probe."""

from __future__ import annotations

import subprocess
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def run_case(
    blackbox_script: Path,
    case: Any,
    username: str,
    allow_chat_ids: tuple[str, ...],
    max_wait: int,
    max_idle_secs: int,
    secret_token: str,
    runtime_partition_mode: str | None,
    *,
    forbidden_log_pattern: str,
    python_executable: str,
    subprocess_run_fn: Any = subprocess.run,
) -> int:
    """Run one black-box probe case and return exit code."""
    print()
    print(f">>> Probe[{case.case_id}]: prompt='{case.prompt}' expect-event='{case.event_name}'")
    case_max_wait = getattr(case, "max_wait_secs", None)
    case_max_idle_secs = getattr(case, "max_idle_secs", None)
    effective_max_wait = int(case_max_wait) if case_max_wait is not None else max_wait
    effective_max_idle_secs = (
        int(case_max_idle_secs) if case_max_idle_secs is not None else max_idle_secs
    )
    cmd = [
        python_executable,
        str(blackbox_script),
        "--prompt",
        case.prompt,
        "--expect-event",
        case.event_name,
        "--forbid-log-regex",
        forbidden_log_pattern,
        "--max-wait",
        str(effective_max_wait),
        "--max-idle-secs",
        str(effective_max_idle_secs),
    ]
    for allowed_chat_id in allow_chat_ids:
        cmd.extend(["--allow-chat-id", allowed_chat_id])
    if case.chat_id is not None:
        cmd.extend(["--chat-id", str(case.chat_id), "--allow-chat-id", str(case.chat_id)])
    if case.thread_id is not None:
        cmd.extend(["--thread-id", str(case.thread_id)])
    if username:
        cmd.extend(["--username", username])
    if case.user_id is not None:
        cmd.extend(["--user-id", str(case.user_id)])
    if runtime_partition_mode:
        cmd.extend(["--session-partition", runtime_partition_mode])
    if secret_token:
        cmd.extend(["--secret-token", secret_token])
    cmd.extend(case.extra_args)

    completed = subprocess_run_fn(cmd, check=False)
    return completed.returncode


def is_transient_matrix_failure(
    returncode: int, transient_exit_codes: set[int] | frozenset[int]
) -> bool:
    """Check whether exit code is retryable in admin matrix mode."""
    return returncode in transient_exit_codes
