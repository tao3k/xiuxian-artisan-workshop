#!/usr/bin/env python3
"""Memory lifecycle checks for event sequence validation."""

from __future__ import annotations

from event_sequence_checks_core import Reporter, check_order, first_line, first_line_any


def check_memory_lifecycle(
    reporter: Reporter,
    lines: list[str],
    *,
    require_memory: bool,
    count_memory_backend_initialized: int,
    count_memory_load_succeeded: int,
    count_memory_load_failed: int,
    count_memory_save_succeeded: int,
    count_memory_save_failed: int,
    count_memory_recall_planned: int,
    count_memory_recall_injected: int,
    count_memory_recall_skipped: int,
) -> None:
    """Validate memory backend lifecycle and recall sequence."""
    if count_memory_backend_initialized <= 0:
        if require_memory:
            reporter.emit_fail(
                "memory lifecycle events are required but backend initialization event is missing"
            )
        else:
            reporter.emit_warn(
                "memory backend initialization event missing (memory may be disabled)"
            )
        return

    reporter.emit_pass(
        f"memory backend initialization events present (count={count_memory_backend_initialized})"
    )
    if (count_memory_load_succeeded + count_memory_load_failed) > 0:
        reporter.emit_pass(
            "memory load lifecycle events present "
            f"(load_ok={count_memory_load_succeeded}, load_fail={count_memory_load_failed})"
        )
    else:
        reporter.emit_fail("memory backend initialized but no memory load lifecycle event found")

    if count_memory_save_failed > 0:
        reporter.emit_fail(f"memory save failures detected (count={count_memory_save_failed})")
    elif count_memory_save_succeeded > 0:
        reporter.emit_pass(
            f"memory save success events present (count={count_memory_save_succeeded})"
        )
    else:
        reporter.emit_warn("no memory save events observed")

    if count_memory_recall_planned > 0:
        reporter.emit_pass(
            f"memory recall planning events present (count={count_memory_recall_planned})"
        )
        if (count_memory_recall_injected + count_memory_recall_skipped) > 0:
            reporter.emit_pass(
                "memory recall decision events present "
                f"(injected={count_memory_recall_injected}, skipped={count_memory_recall_skipped})"
            )
        else:
            reporter.emit_fail("memory recall planned but no recall decision event found")
    else:
        if require_memory:
            reporter.emit_fail("memory recall planning events missing while memory is required")
        else:
            reporter.emit_warn("memory recall planning events missing")

    line_memory_backend = first_line(lines, "agent.memory.backend.initialized")
    line_memory_load = first_line_any(
        lines,
        ["agent.memory.state_load_succeeded", "agent.memory.state_load_failed"],
    )
    line_memory_recall_planned = first_line(lines, "agent.memory.recall.planned")
    line_memory_recall_decision = first_line_any(
        lines,
        ["agent.memory.recall.injected", "agent.memory.recall.skipped"],
    )
    check_order(
        reporter,
        "agent.memory.backend.initialized",
        line_memory_backend,
        "agent.memory.state_load_(succeeded|failed)",
        line_memory_load,
        "memory backend initialization should precede memory load lifecycle event",
    )
    check_order(
        reporter,
        "agent.memory.backend.initialized",
        line_memory_backend,
        "agent.memory.recall.planned",
        line_memory_recall_planned,
        "memory backend initialization should precede memory recall planning",
    )
    check_order(
        reporter,
        "agent.memory.recall.planned",
        line_memory_recall_planned,
        "agent.memory.recall.(injected|skipped)",
        line_memory_recall_decision,
        "memory recall planning should precede recall decision logging",
    )
