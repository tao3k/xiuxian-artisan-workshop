#!/usr/bin/env python3
"""Warning-budget quality gates for memory CI runtime logs."""

from __future__ import annotations

from typing import Any


def assert_mcp_waiting_warning_budget(cfg: Any, *, count_log_event_fn: Any) -> None:
    """Validate warning budgets for MCP waiting events."""
    if not cfg.runtime_log_file.exists():
        raise RuntimeError(f"missing runtime log file: {cfg.runtime_log_file}")

    call_waiting = count_log_event_fn(cfg.runtime_log_file, "mcp.pool.call.waiting")
    connect_waiting = count_log_event_fn(cfg.runtime_log_file, "mcp.pool.connect.waiting")
    waiting_total = call_waiting + connect_waiting

    failures: list[str] = []
    if call_waiting > cfg.max_mcp_call_waiting_events:
        failures.append(f"mcp.pool.call.waiting={call_waiting} > {cfg.max_mcp_call_waiting_events}")
    if connect_waiting > cfg.max_mcp_connect_waiting_events:
        failures.append(
            f"mcp.pool.connect.waiting={connect_waiting} > {cfg.max_mcp_connect_waiting_events}"
        )
    if waiting_total > cfg.max_mcp_waiting_events_total:
        failures.append(
            f"mcp_waiting_events_total={waiting_total} > {cfg.max_mcp_waiting_events_total}"
        )

    if failures:
        raise RuntimeError("mcp waiting warning budget exceeded: " + "; ".join(failures))

    print(
        "MCP waiting warning budget passed: "
        f"call_waiting={call_waiting}, "
        f"connect_waiting={connect_waiting}, "
        f"total={waiting_total}",
        flush=True,
    )


def assert_memory_stream_warning_budget(cfg: Any, *, count_log_event_fn: Any) -> None:
    """Validate warning budgets for memory stream read failures."""
    if not cfg.runtime_log_file.exists():
        raise RuntimeError(f"missing runtime log file: {cfg.runtime_log_file}")

    read_failed = count_log_event_fn(
        cfg.runtime_log_file, "agent.memory.stream_consumer.read_failed"
    )
    if read_failed > cfg.max_memory_stream_read_failed_events:
        raise RuntimeError(
            "memory stream warning budget exceeded: "
            f"agent.memory.stream_consumer.read_failed={read_failed} > "
            f"{cfg.max_memory_stream_read_failed_events}"
        )

    print(
        "Memory stream warning budget passed: "
        f"agent.memory.stream_consumer.read_failed={read_failed}",
        flush=True,
    )
