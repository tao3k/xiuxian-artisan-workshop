#!/usr/bin/env python3
"""Main validation flow for event-sequence checks."""

from __future__ import annotations

from event_sequence_checks_backend import check_expected_memory_backend
from event_sequence_checks_core import Reporter, count_event
from event_sequence_checks_memory import check_memory_lifecycle
from event_sequence_checks_retry import check_valkey_retry
from event_sequence_checks_session import check_session_gate_and_windows


def run_checks(
    lines: list[str],
    stripped_lines: list[str],
    strict: bool,
    require_memory: bool,
    expect_memory_backend: str,
) -> int:
    """Run full observability event-sequence validation."""
    reporter = Reporter()

    count_dedup_evaluated = count_event(lines, "telegram.dedup.evaluated")
    count_dedup_accepted = count_event(lines, "telegram.dedup.update_accepted")
    count_gate_backend_initialized = count_event(lines, "session.gate.backend.initialized")
    count_gate_acquired = count_event(lines, "session.gate.lease.acquired")
    count_gate_released = count_event(lines, "session.gate.lease.released")
    count_window_loaded = count_event(lines, "session.window_slots.loaded")
    count_window_appended = count_event(lines, "session.window_slots.appended")
    count_retry_failed = count_event(lines, "session.valkey.command.retry_failed")
    count_retry_succeeded = count_event(lines, "session.valkey.command.retry_succeeded")

    count_memory_backend_initialized = count_event(lines, "agent.memory.backend.initialized")
    count_memory_load_succeeded = count_event(lines, "agent.memory.state_load_succeeded")
    count_memory_load_failed = count_event(lines, "agent.memory.state_load_failed")
    count_memory_save_succeeded = count_event(lines, "agent.memory.state_save_succeeded")
    count_memory_save_failed = count_event(lines, "agent.memory.state_save_failed")
    count_memory_recall_planned = count_event(lines, "agent.memory.recall.planned")
    count_memory_recall_injected = count_event(lines, "agent.memory.recall.injected")
    count_memory_recall_skipped = count_event(lines, "agent.memory.recall.skipped")

    if count_dedup_evaluated > 0:
        reporter.emit_pass(f"dedup evaluation events present (count={count_dedup_evaluated})")
    else:
        reporter.emit_fail("dedup evaluation events missing (expected: telegram.dedup.evaluated)")

    check_session_gate_and_windows(
        reporter,
        lines,
        stripped_lines,
        count_dedup_accepted=count_dedup_accepted,
        count_gate_backend_initialized=count_gate_backend_initialized,
        count_gate_acquired=count_gate_acquired,
        count_gate_released=count_gate_released,
        count_window_loaded=count_window_loaded,
        count_window_appended=count_window_appended,
    )
    check_valkey_retry(
        reporter,
        count_retry_failed=count_retry_failed,
        count_retry_succeeded=count_retry_succeeded,
    )
    check_memory_lifecycle(
        reporter,
        lines,
        require_memory=require_memory,
        count_memory_backend_initialized=count_memory_backend_initialized,
        count_memory_load_succeeded=count_memory_load_succeeded,
        count_memory_load_failed=count_memory_load_failed,
        count_memory_save_succeeded=count_memory_save_succeeded,
        count_memory_save_failed=count_memory_save_failed,
        count_memory_recall_planned=count_memory_recall_planned,
        count_memory_recall_injected=count_memory_recall_injected,
        count_memory_recall_skipped=count_memory_recall_skipped,
    )
    check_expected_memory_backend(
        reporter,
        stripped_lines,
        expect_memory_backend=expect_memory_backend,
    )

    if strict and reporter.warnings > 0:
        reporter.emit_fail(
            f"strict mode enabled: warnings are treated as failures (warnings={reporter.warnings})"
        )

    print()
    print(f"Summary: pass={reporter.passes} warn={reporter.warnings} fail={reporter.failures}")
    return 1 if reporter.failures > 0 else 0
