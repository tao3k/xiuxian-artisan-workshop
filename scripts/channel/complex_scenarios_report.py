#!/usr/bin/env python3
"""Report helpers for complex scenario black-box runs."""

from __future__ import annotations

import json
import time
from dataclasses import asdict
from datetime import UTC, datetime
from typing import Any

from complex_scenarios_report_render import render_markdown


def build_report(
    cfg: Any,
    scenario_results: tuple[Any, ...],
    started_mono: float,
    started_dt: datetime,
) -> dict[str, object]:
    """Build structured JSON report payload for complex scenario runs."""
    finished_dt = datetime.now(UTC)
    duration_ms = int((time.monotonic() - started_mono) * 1000)

    scenario_payloads = []
    passed_count = 0

    for result in scenario_results:
        if result.passed:
            passed_count += 1
        payload = {
            "scenario_id": result.scenario_id,
            "description": result.description,
            "requirement": asdict(result.requirement),
            "complexity": asdict(result.complexity),
            "complexity_passed": result.complexity_passed,
            "complexity_failures": list(result.complexity_failures),
            "quality_requirement": asdict(result.quality_requirement),
            "quality": asdict(result.quality),
            "quality_passed": result.quality_passed,
            "quality_failures": list(result.quality_failures),
            "duration_ms": result.duration_ms,
            "passed": result.passed,
            "steps": [asdict(step) for step in result.steps],
        }
        scenario_payloads.append(payload)

    overall_passed = passed_count == len(scenario_payloads) and len(scenario_payloads) > 0

    return {
        "started_at": started_dt.isoformat(),
        "finished_at": finished_dt.isoformat(),
        "duration_ms": duration_ms,
        "overall_passed": overall_passed,
        "summary": {
            "total": len(scenario_payloads),
            "passed": passed_count,
            "failed": len(scenario_payloads) - passed_count,
        },
        "config": {
            "dataset": str(cfg.dataset_path),
            "scenario_filter": cfg.scenario_id,
            "blackbox_script": str(cfg.blackbox_script),
            "webhook_url": cfg.webhook_url,
            "log_file": str(cfg.log_file),
            "max_wait": cfg.max_wait,
            "max_idle_secs": cfg.max_idle_secs,
            "max_parallel": cfg.max_parallel,
            "execute_wave_parallel": cfg.execute_wave_parallel,
            "runtime_partition_mode": cfg.runtime_partition_mode,
            "username": cfg.username,
            "forbid_log_regexes": list(cfg.forbid_log_regexes),
            "global_requirement": asdict(cfg.global_requirement),
            "global_quality_requirement": asdict(cfg.global_quality_requirement),
            "sessions": [asdict(session) for session in cfg.sessions],
        },
        "scenarios": scenario_payloads,
    }


def write_outputs(report: dict[str, object], output_json: Any, output_markdown: Any) -> None:
    """Write both JSON and Markdown complex-scenario reports."""
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_markdown.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    output_markdown.write_text(render_markdown(report), encoding="utf-8")
