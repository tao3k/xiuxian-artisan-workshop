#!/usr/bin/env python3
"""Artifact path helpers for memory CI gate triage."""

from __future__ import annotations

import time
from typing import Any


def artifact_rows(cfg: Any) -> list[tuple[str, Any]]:
    """Return all known gate artifact paths."""
    return [
        ("runtime_log", cfg.runtime_log_file),
        ("mock_log", cfg.mock_log_file),
        ("evolution_report_json", cfg.evolution_report_json),
        ("benchmark_report_json", cfg.benchmark_report_json),
        ("session_matrix_report_json", cfg.session_matrix_report_json),
        ("session_matrix_report_markdown", cfg.session_matrix_report_markdown),
        ("trace_report_json", cfg.trace_report_json),
        ("trace_report_markdown", cfg.trace_report_markdown),
        ("cross_group_report_json", cfg.cross_group_report_json),
        ("cross_group_report_markdown", cfg.cross_group_report_markdown),
    ]


def default_gate_failure_report_base_path(
    cfg: Any, *, stamp_ms: int | None = None
) -> tuple[Any, int]:
    """Resolve default report base path + timestamp."""
    final_stamp = int(time.time() * 1000) if stamp_ms is None else int(stamp_ms)
    base = (
        cfg.project_root
        / ".run"
        / "reports"
        / f"omni-agent-memory-ci-failure-{cfg.profile}-{final_stamp}"
    )
    return base, final_stamp
