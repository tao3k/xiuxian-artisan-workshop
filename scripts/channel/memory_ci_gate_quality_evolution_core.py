#!/usr/bin/env python3
"""Evolution quality gate checks for memory CI pipeline."""

from __future__ import annotations

import json
from typing import Any


def assert_evolution_quality(cfg: Any) -> None:
    """Validate evolution scenario quality thresholds."""
    if not cfg.evolution_report_json.exists():
        raise RuntimeError(f"missing evolution report: {cfg.evolution_report_json}")
    report = json.loads(cfg.evolution_report_json.read_text(encoding="utf-8"))
    if not bool(report.get("overall_passed", False)):
        raise RuntimeError("evolution report indicates overall failure")
    scenarios = report.get("scenarios")
    if not isinstance(scenarios, list) or not scenarios:
        raise RuntimeError("evolution report has no scenarios")
    scenario = scenarios[0]
    quality = scenario.get("quality", {})
    planned_hits = int(quality.get("planned_hits", 0))
    successful_corrections = int(quality.get("successful_corrections", 0))
    recall_credit_events = int(quality.get("recall_credit_events", 0))
    quality_score = float(quality.get("quality_score", 0.0))

    failures: list[str] = []
    if planned_hits < cfg.min_planned_hits:
        failures.append(f"planned_hits={planned_hits} < {cfg.min_planned_hits}")
    if successful_corrections < cfg.min_successful_corrections:
        failures.append(
            f"successful_corrections={successful_corrections} < {cfg.min_successful_corrections}"
        )
    if recall_credit_events < cfg.min_recall_credit_events:
        failures.append(
            f"recall_credit_events={recall_credit_events} < {cfg.min_recall_credit_events}"
        )
    if quality_score < cfg.min_quality_score:
        failures.append(f"quality_score={quality_score:.2f} < {cfg.min_quality_score:.2f}")

    if failures:
        raise RuntimeError("evolution quality gates failed: " + "; ".join(failures))

    print(
        "Evolution quality gates passed: "
        f"planned_hits={planned_hits}, "
        f"successful_corrections={successful_corrections}, "
        f"recall_credit_events={recall_credit_events}, "
        f"quality_score={quality_score:.2f}",
        flush=True,
    )
