#!/usr/bin/env python3
"""Post-loop outcome handling for agent channel blackbox runtime probe."""

from __future__ import annotations

import sys
from typing import Any

from agent_channel_blackbox_runtime_outcome_failure import handle_no_bot_outcome
from agent_channel_blackbox_runtime_outcome_success import handle_success_outcome


def handle_post_loop_outcome(
    *,
    cfg: Any,
    state: Any,
    finish_fn: Any,
    tail_lines_fn: Any,
    helpers_module: Any,
    trace_mode: bool,
    seen_trace: bool,
    seen_user_dispatch: bool,
    seen_bot: bool,
    bot_line: str,
    error_line: str,
    dedup_duplicate_line: str,
    dispatch_session_mismatch_line: str,
    webhook_seen: bool,
    trace_id: str,
) -> int:
    """Render outcome diagnostics and return probe exit code."""
    print("")
    if dedup_duplicate_line and not seen_user_dispatch:
        print("Probe failed: webhook update was dropped as duplicate.", file=sys.stderr)
        print("Related log line:", file=sys.stderr)
        print(f"  {dedup_duplicate_line}", file=sys.stderr)
        return finish_fn(4)

    if dispatch_session_mismatch_line:
        print("Probe failed: observed session_key does not match target session.", file=sys.stderr)
        print(f"  expected_session_keys={list(state.expected_sessions)}", file=sys.stderr)
        print(f"  line={dispatch_session_mismatch_line}", file=sys.stderr)
        return finish_fn(10)

    if seen_bot:
        return handle_success_outcome(
            cfg=cfg,
            state=state,
            finish_fn=finish_fn,
            helpers_module=helpers_module,
            bot_line=bot_line,
        )

    return handle_no_bot_outcome(
        cfg=cfg,
        state=state,
        finish_fn=finish_fn,
        tail_lines_fn=tail_lines_fn,
        helpers_module=helpers_module,
        trace_mode=trace_mode,
        seen_trace=seen_trace,
        seen_user_dispatch=seen_user_dispatch,
        error_line=error_line,
        webhook_seen=webhook_seen,
        trace_id=trace_id,
    )


__all__ = ["handle_post_loop_outcome"]
