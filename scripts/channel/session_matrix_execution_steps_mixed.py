#!/usr/bin/env python3
"""Mixed-concurrency step definitions for session-matrix probes."""

from __future__ import annotations

import time
from concurrent.futures import ThreadPoolExecutor
from typing import Any


def build_mixed_concurrency_steps(cfg: Any, *, matrix_step_cls: Any) -> tuple[Any, ...]:
    """Build the 3-step mixed concurrency batch."""
    return (
        matrix_step_cls(
            name="mixed_reset_session_a",
            prompt="/reset",
            chat_id=cfg.chat_id,
            event="telegram.command.session_reset.replied",
            user_id=cfg.user_a,
            thread_id=cfg.thread_a,
        ),
        matrix_step_cls(
            name="mixed_resume_status_session_b",
            prompt="/resume status",
            chat_id=cfg.chat_b,
            event="telegram.command.session_resume_status.replied",
            user_id=cfg.user_b,
            thread_id=cfg.thread_b,
        ),
        matrix_step_cls(
            name="mixed_plain_session_c",
            prompt=cfg.mixed_plain_prompt,
            chat_id=cfg.chat_c,
            event=None,
            user_id=cfg.user_c,
            thread_id=cfg.thread_c,
        ),
    )


def run_mixed_concurrency_batch(
    script_dir: Any,
    cfg: Any,
    *,
    run_blackbox_step_fn: Any,
    build_mixed_concurrency_steps_fn: Any,
) -> list[Any]:
    """Execute mixed batch in parallel with small startup staggering."""

    def _run_with_stagger(step: Any, delay_secs: float) -> Any:
        if delay_secs > 0:
            time.sleep(delay_secs)
        return run_blackbox_step_fn(script_dir, cfg, step)

    steps = build_mixed_concurrency_steps_fn(cfg)
    with ThreadPoolExecutor(max_workers=len(steps)) as pool:
        futures = [
            pool.submit(_run_with_stagger, step, index * 0.02) for index, step in enumerate(steps)
        ]
        return [future.result() for future in futures]
