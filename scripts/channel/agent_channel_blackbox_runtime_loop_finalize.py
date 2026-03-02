#!/usr/bin/env python3
"""Outcome finalization helpers for blackbox runtime loop."""

from __future__ import annotations

from typing import Any


def finalize_probe_outcome(
    *,
    cfg: Any,
    state: Any,
    loop_outcome: Any,
    trace_id: str,
    finish_fn: Any,
    tail_lines_fn: Any,
    helpers_module: Any,
    outcome_module: Any,
) -> int:
    """Convert polling outcome into final process exit code."""
    if loop_outcome.exit_code is not None:
        return finish_fn(loop_outcome.exit_code)
    if loop_outcome.allow_no_bot_success:
        return finish_fn(0)

    return outcome_module.handle_post_loop_outcome(
        cfg=cfg,
        state=state,
        finish_fn=finish_fn,
        tail_lines_fn=tail_lines_fn,
        helpers_module=helpers_module,
        trace_mode=loop_outcome.trace_mode,
        seen_trace=loop_outcome.seen_trace,
        seen_user_dispatch=loop_outcome.seen_user_dispatch,
        seen_bot=loop_outcome.seen_bot,
        bot_line=loop_outcome.bot_line,
        error_line=loop_outcome.error_line,
        dedup_duplicate_line=loop_outcome.dedup_duplicate_line,
        dispatch_session_mismatch_line=loop_outcome.dispatch_session_mismatch_line,
        webhook_seen=loop_outcome.webhook_seen,
        trace_id=trace_id,
    )
