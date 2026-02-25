#!/usr/bin/env python3
"""Report output tests for memory CI gate triage."""

from __future__ import annotations

import json

from test_memory_ci_gate import build_cfg
from test_omni_agent_memory_ci_gate import (
    print_gate_failure_triage,
    write_gate_failure_triage_json_report,
    write_gate_failure_triage_report,
)


def test_write_gate_failure_triage_report_writes_expected_sections(tmp_path) -> None:
    cfg = build_cfg(tmp_path)
    cfg.runtime_log_file.write_text(
        '2026-02-22T00:00:00Z WARN event="mcp.pool.call.waiting"\n',
        encoding="utf-8",
    )
    report = write_gate_failure_triage_report(
        cfg,
        error=RuntimeError("mcp waiting warning budget exceeded"),
        category="mcp_waiting_budget",
        summary="mcp waiting warning budget exceeded",
        repro_commands=["echo triage"],
    )
    content = report.read_text(encoding="utf-8")
    assert "Omni Agent Memory CI Failure Triage" in content
    assert "category: `mcp_waiting_budget`" in content
    assert "## Repro Commands" in content
    assert "## Runtime Log Tail" in content


def test_write_gate_failure_triage_json_report_writes_expected_payload(tmp_path) -> None:
    cfg = build_cfg(tmp_path)
    cfg.runtime_log_file.write_text(
        '2026-02-22T00:00:00Z WARN event="mcp.pool.call.waiting"\n',
        encoding="utf-8",
    )
    report = write_gate_failure_triage_json_report(
        cfg,
        error=RuntimeError("mcp waiting warning budget exceeded"),
        category="mcp_waiting_budget",
        summary="mcp waiting warning budget exceeded",
        repro_commands=["echo triage-json"],
    )
    payload = json.loads(report.read_text(encoding="utf-8"))
    assert payload["profile"] == "nightly"
    assert payload["category"] == "mcp_waiting_budget"
    assert payload["summary"] == "mcp waiting warning budget exceeded"
    assert payload["repro_commands"] == ["echo triage-json"]
    artifacts = payload.get("artifacts")
    assert isinstance(artifacts, list)
    assert any(item.get("name") == "runtime_log" for item in artifacts if isinstance(item, dict))
    assert "runtime_log_tail" in payload


def test_print_gate_failure_triage_returns_report_path(tmp_path) -> None:
    cfg = build_cfg(tmp_path)
    cfg.runtime_log_file.write_text(
        '2026-02-22T00:00:00Z WARN event="agent.memory.stream_consumer.read_failed"\n',
        encoding="utf-8",
    )
    report = print_gate_failure_triage(cfg, RuntimeError("memory stream warning budget exceeded"))
    assert report.exists()
    assert report.name.startswith("omni-agent-memory-ci-failure-nightly-")
    report_json = report.with_suffix(".json")
    assert report_json.exists()
    payload = json.loads(report_json.read_text(encoding="utf-8"))
    assert payload.get("category") == "memory_stream_budget"
