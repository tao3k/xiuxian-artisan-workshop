#!/usr/bin/env python3
"""Trace reconstruction quality gate assertions for memory CI."""

from __future__ import annotations

from typing import Any

from memory_ci_gate_quality_common import load_json


def assert_trace_reconstruction_quality(cfg: Any) -> None:
    """Validate trace reconstruction stage coverage and score."""
    report = load_json(cfg.trace_report_json)
    summary_obj = report.get("summary")
    summary = summary_obj if isinstance(summary_obj, dict) else {}
    errors_obj = report.get("errors")
    errors = errors_obj if isinstance(errors_obj, list) else []

    events_total = int(summary.get("events_total", 0))
    quality_score = float(summary.get("quality_score", 0.0))
    stage_flags_obj = summary.get("stage_flags")
    stage_flags = stage_flags_obj if isinstance(stage_flags_obj, dict) else {}
    required_flags = (
        ("has_route", "has_injection", "has_injection_mode", "has_reflection", "has_memory")
        if cfg.profile == "nightly"
        else ("has_memory",)
    )
    required_hits = sum(1 for flag in required_flags if bool(stage_flags.get(flag, False)))
    required_quality_score = (
        (float(required_hits) / float(len(required_flags))) * 100.0 if required_flags else 100.0
    )

    failures: list[str] = []
    if events_total <= 0:
        failures.append("events_total must be > 0")
    if errors:
        failures.append(f"errors present: {errors}")
    if required_quality_score < cfg.trace_min_quality_score:
        failures.append(
            "required_quality_score="
            f"{required_quality_score:.2f} < trace_min_quality_score={cfg.trace_min_quality_score:.2f}"
        )
    for required_flag in required_flags:
        if not bool(stage_flags.get(required_flag, False)):
            failures.append(f"stage flag missing: {required_flag}")

    if failures:
        raise RuntimeError("trace reconstruction quality gates failed: " + "; ".join(failures))

    print(
        "Trace reconstruction gate passed: "
        f"events_total={events_total}, quality_score={quality_score:.2f}, "
        f"required_quality_score={required_quality_score:.2f}",
        flush=True,
    )
