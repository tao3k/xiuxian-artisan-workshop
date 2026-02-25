#!/usr/bin/env python3
"""Session-gate and dedup sequence checks."""

from __future__ import annotations

from event_sequence_checks_core import Reporter, check_order, first_line


def check_session_gate_and_windows(
    reporter: Reporter,
    lines: list[str],
    stripped_lines: list[str],
    *,
    count_dedup_accepted: int,
    count_gate_backend_initialized: int,
    count_gate_acquired: int,
    count_gate_released: int,
    count_window_loaded: int,
    count_window_appended: int,
) -> None:
    """Validate dedup acceptance, session-gate, and window append sequence."""
    count_gate_backend_valkey = sum(
        1
        for line in stripped_lines
        if "session.gate.backend.initialized" in line and 'backend="valkey"' in line
    )
    count_gate_backend_memory = sum(
        1
        for line in stripped_lines
        if "session.gate.backend.initialized" in line and 'backend="memory"' in line
    )

    if count_dedup_accepted <= 0:
        reporter.emit_warn("no accepted updates found (telegram.dedup.update_accepted)")
        return

    reporter.emit_pass(f"dedup accepted-update events present (count={count_dedup_accepted})")
    if (count_window_loaded + count_window_appended) > 0:
        reporter.emit_pass(
            "session window activity events present "
            f"(loaded={count_window_loaded}, appended={count_window_appended})"
        )
    else:
        reporter.emit_warn("no session window load/append events found after accepted updates")

    if count_gate_acquired <= 0:
        if count_gate_backend_valkey > 0:
            reporter.emit_fail(
                "session gate backend is valkey but no lease acquire events were observed"
            )
        elif count_gate_backend_memory > 0:
            reporter.emit_warn(
                "session gate backend is memory; no lease events observed "
                "(expected in command-only or single-process flows)"
            )
        elif count_gate_backend_initialized > 0:
            reporter.emit_warn(
                "session gate backend initialized but backend mode could not be inferred; "
                "skipping lease checks"
            )
        else:
            reporter.emit_warn(
                "session gate backend initialization event missing; skipping lease checks"
            )
        return

    reporter.emit_pass(f"session gate acquire events present (count={count_gate_acquired})")
    if count_gate_released > 0:
        reporter.emit_pass(f"session gate release events present (count={count_gate_released})")
    else:
        reporter.emit_warn(
            "session gate release events missing; check in-flight shutdowns or lease cleanup"
        )

    line_dedup_accepted = first_line(lines, "telegram.dedup.update_accepted")
    line_gate_acquired = first_line(lines, "session.gate.lease.acquired")
    line_window_appended = first_line(lines, "session.window_slots.appended")

    check_order(
        reporter,
        "telegram.dedup.update_accepted",
        line_dedup_accepted,
        "session.gate.lease.acquired",
        line_gate_acquired,
        "dedup acceptance should precede session gate acquisition",
    )
    check_order(
        reporter,
        "session.gate.lease.acquired",
        line_gate_acquired,
        "session.window_slots.appended",
        line_window_appended,
        "session gate acquisition should precede window append",
    )
