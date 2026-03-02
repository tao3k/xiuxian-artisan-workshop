#!/usr/bin/env python3
"""Idle-timeout helpers for blackbox runtime probe polling."""

from __future__ import annotations

import sys
from typing import Any

from agent_channel_blackbox_runtime_loop_poll_model import ProbeLoopOutcome, outcome_from_state


def handle_idle_timeout(
    cfg: Any,
    *,
    loop_state: Any,
    trace_mode: bool,
    monotonic_fn: Any,
    sleep_fn: Any,
) -> tuple[ProbeLoopOutcome | None, bool]:
    """Handle idle timeout checks and return (outcome, skipped_default_sleep)."""
    if cfg.max_idle_secs is None:
        return None, False
    if (monotonic_fn() - loop_state.last_log_activity) <= cfg.max_idle_secs:
        return None, False
    if loop_state.retry_grace_until and monotonic_fn() <= loop_state.retry_grace_until:
        sleep_fn(0.2)
        return None, True

    print("", file=sys.stderr)
    print("Probe failed: max-idle exceeded with no new logs.", file=sys.stderr)
    print(f"  max_idle_secs={cfg.max_idle_secs}", file=sys.stderr)
    return (
        outcome_from_state(
            loop_state,
            trace_mode=trace_mode,
            exit_code=7,
            allow_no_bot_success=False,
        ),
        False,
    )
