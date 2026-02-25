#!/usr/bin/env python3
"""Matrix flow wrappers for session-matrix execution."""

from __future__ import annotations

import importlib
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path

_flow_module = importlib.import_module("session_matrix_execution_flow")


def run_concurrent_step(
    script_dir: Path,
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
    return _flow_module.run_concurrent_step(
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
    script_dir: Path,
    cfg: Any,
    step: Any,
    *,
    expected_session_key_fn: Any,
    run_command_with_restart_retry_fn: Any,
    tail_text_fn: Any,
    step_result_cls: Any,
) -> Any:
    """Execute one single-session blackbox probe."""
    return _flow_module.run_blackbox_step(
        script_dir,
        cfg,
        step,
        expected_session_key_fn=expected_session_key_fn,
        run_command_with_restart_retry_fn=run_command_with_restart_retry_fn,
        tail_text_fn=tail_text_fn,
        step_result_cls=step_result_cls,
    )


def build_mixed_concurrency_steps(cfg: Any, *, matrix_step_cls: Any) -> tuple[Any, ...]:
    """Build the 3-step mixed concurrency batch."""
    return _flow_module.build_mixed_concurrency_steps(
        cfg,
        matrix_step_cls=matrix_step_cls,
    )


def run_mixed_concurrency_batch(
    script_dir: Path,
    cfg: Any,
    *,
    run_blackbox_step_fn: Any,
    build_mixed_concurrency_steps_fn: Any,
) -> list[Any]:
    """Execute mixed batch in parallel with small startup staggering."""
    return _flow_module.run_mixed_concurrency_batch(
        script_dir,
        cfg,
        run_blackbox_step_fn=run_blackbox_step_fn,
        build_mixed_concurrency_steps_fn=build_mixed_concurrency_steps_fn,
    )


def run_matrix(
    cfg: Any,
    *,
    script_dir: Path,
    build_report_fn: Any,
    build_matrix_steps_fn: Any,
    run_concurrent_step_fn: Any,
    run_blackbox_step_fn: Any,
    run_mixed_concurrency_batch_fn: Any,
) -> tuple[bool, dict[str, object]]:
    """Run the full session matrix and return overall status + report."""
    return _flow_module.run_matrix(
        cfg,
        script_dir=script_dir,
        build_report_fn=build_report_fn,
        build_matrix_steps_fn=build_matrix_steps_fn,
        run_concurrent_step_fn=run_concurrent_step_fn,
        run_blackbox_step_fn=run_blackbox_step_fn,
        run_mixed_concurrency_batch_fn=run_mixed_concurrency_batch_fn,
    )
