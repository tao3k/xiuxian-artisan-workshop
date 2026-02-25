#!/usr/bin/env python3
"""Datamodels for agent blackbox runtime polling."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class ProbeLoopState:
    """Mutable polling state while scanning runtime logs."""

    seen_trace: bool
    seen_user_dispatch: bool
    seen_bot: bool
    bot_line: str
    error_line: str
    dedup_duplicate_line: str
    dispatch_session_mismatch_line: str
    webhook_seen: bool
    last_log_activity: float
    retry_grace_until: float


@dataclass(frozen=True)
class ProbeLoopOutcome:
    """Outcome snapshot produced by the probe polling loop."""

    exit_code: int | None
    allow_no_bot_success: bool
    trace_mode: bool
    seen_trace: bool
    seen_user_dispatch: bool
    seen_bot: bool
    bot_line: str
    error_line: str
    dedup_duplicate_line: str
    dispatch_session_mismatch_line: str
    webhook_seen: bool


def build_initial_state(*, now_monotonic: float) -> ProbeLoopState:
    """Build initial mutable state for probe polling."""
    return ProbeLoopState(
        seen_trace=False,
        seen_user_dispatch=False,
        seen_bot=False,
        bot_line="",
        error_line="",
        dedup_duplicate_line="",
        dispatch_session_mismatch_line="",
        webhook_seen=False,
        last_log_activity=now_monotonic,
        retry_grace_until=0.0,
    )


def outcome_from_state(
    state: ProbeLoopState,
    *,
    trace_mode: bool,
    exit_code: int | None,
    allow_no_bot_success: bool,
) -> ProbeLoopOutcome:
    """Build immutable outcome from mutable loop state."""
    return ProbeLoopOutcome(
        exit_code=exit_code,
        allow_no_bot_success=allow_no_bot_success,
        trace_mode=trace_mode,
        seen_trace=state.seen_trace,
        seen_user_dispatch=state.seen_user_dispatch,
        seen_bot=state.seen_bot,
        bot_line=state.bot_line,
        error_line=state.error_line,
        dedup_duplicate_line=state.dedup_duplicate_line,
        dispatch_session_mismatch_line=state.dispatch_session_mismatch_line,
        webhook_seen=state.webhook_seen,
    )
