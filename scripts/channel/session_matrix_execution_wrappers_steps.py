#!/usr/bin/env python3
"""Step-level wrapper helpers for session-matrix execution."""

from __future__ import annotations

from typing import Any

from session_matrix_execution_matrix import (
    run_blackbox_step as _run_blackbox_step,
)
from session_matrix_execution_matrix import (
    run_concurrent_step as _run_concurrent_step,
)
from session_matrix_execution_retry import (
    run_command_with_restart_retry as _run_command_with_restart_retry,
)
from session_matrix_execution_retry import (
    tail_text as _tail_text,
)


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
    run_command_with_restart_retry_fn: Any = _run_command_with_restart_retry,
    tail_text_fn: Any = _tail_text,
    step_result_cls: Any,
) -> Any:
    """Execute one dual-session concurrent probe."""
    return _run_concurrent_step(
        script_dir,
        cfg,
        name=name,
        chat_a=chat_a,
        user_a=user_a,
        thread_a=thread_a,
        chat_b=chat_b,
        user_b=user_b,
        thread_b=thread_b,
        prompt=prompt,
        allow_send_failure=allow_send_failure,
        expected_session_key_fn=expected_session_key_fn,
        run_command_with_restart_retry_fn=run_command_with_restart_retry_fn,
        tail_text_fn=tail_text_fn,
        step_result_cls=step_result_cls,
    )


def run_blackbox_step(
    script_dir: Any,
    cfg: Any,
    step: Any,
    *,
    expected_session_key_fn: Any,
    run_command_with_restart_retry_fn: Any = _run_command_with_restart_retry,
    tail_text_fn: Any = _tail_text,
    step_result_cls: Any,
) -> Any:
    """Execute one single-session blackbox probe."""
    return _run_blackbox_step(
        script_dir,
        cfg,
        step,
        expected_session_key_fn=expected_session_key_fn,
        run_command_with_restart_retry_fn=run_command_with_restart_retry_fn,
        tail_text_fn=tail_text_fn,
        step_result_cls=step_result_cls,
    )
