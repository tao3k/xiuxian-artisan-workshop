#!/usr/bin/env python3

from __future__ import annotations

import json
from typing import TYPE_CHECKING

import pytest
from memory_ci_gate_test_support import build_cfg, passing_report, write_report
from test_omni_agent_memory_ci_gate import (
    assert_evolution_slow_response_quality,
    assert_session_matrix_quality,
    assert_trace_reconstruction_quality,
)

if TYPE_CHECKING:
    from pathlib import Path


def test_assert_session_matrix_quality_accepts_full_matrix(tmp_path: Path) -> None:
    cfg = build_cfg(tmp_path)
    write_report(cfg, passing_report(cfg))
    assert_session_matrix_quality(cfg)


def test_assert_session_matrix_quality_rejects_missing_cross_group(tmp_path: Path) -> None:
    cfg = build_cfg(tmp_path)
    report = passing_report(cfg)
    report["steps"] = [step for step in report["steps"] if step["name"] != "concurrent_cross_group"]
    report["summary"] = {"total": len(report["steps"]), "failed": 0}
    write_report(cfg, report)
    with pytest.raises(RuntimeError, match="concurrent_cross_group"):
        assert_session_matrix_quality(cfg)


def test_assert_session_matrix_quality_accepts_chat_partition_baseline_cross_chat(
    tmp_path: Path,
) -> None:
    cfg = build_cfg(tmp_path)
    steps = [{"name": f"step-{index}", "passed": True} for index in range(1, 16)]
    steps.extend(
        [
            {"name": "concurrent_baseline_cross_chat", "passed": True},
            {"name": "mixed_reset_session_a", "passed": True},
            {"name": "mixed_resume_status_session_b", "passed": True},
            {"name": "mixed_plain_session_c", "passed": True},
        ]
    )
    write_report(
        cfg,
        {
            "overall_passed": True,
            "summary": {"total": len(steps), "failed": 0},
            "config": {
                "chat_id": cfg.chat_id,
                "chat_b": cfg.chat_b,
                "chat_c": cfg.chat_c,
            },
            "steps": steps,
        },
    )
    assert len(steps) == 19
    assert_session_matrix_quality(cfg)


def test_assert_trace_reconstruction_quality_accepts_valid_report(tmp_path: Path) -> None:
    cfg = build_cfg(tmp_path)
    cfg.trace_report_json.write_text(
        json.dumps(
            {
                "summary": {
                    "events_total": 8,
                    "quality_score": 100.0,
                    "stage_flags": {
                        "has_route": True,
                        "has_injection": True,
                        "has_injection_mode": True,
                        "has_reflection": True,
                        "has_memory": True,
                    },
                },
                "errors": [],
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    assert_trace_reconstruction_quality(cfg)


def test_assert_trace_reconstruction_quality_rejects_low_quality(tmp_path: Path) -> None:
    cfg = build_cfg(tmp_path)
    cfg.trace_report_json.write_text(
        json.dumps(
            {
                "summary": {
                    "events_total": 4,
                    "quality_score": 75.0,
                    "stage_flags": {
                        "has_route": True,
                        "has_injection": True,
                        "has_injection_mode": False,
                        "has_reflection": True,
                        "has_memory": False,
                    },
                },
                "errors": ["missing memory lifecycle events"],
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    with pytest.raises(RuntimeError, match="trace reconstruction quality gates failed"):
        assert_trace_reconstruction_quality(cfg)


def test_assert_trace_reconstruction_quality_requires_injection_mode_for_nightly(
    tmp_path: Path,
) -> None:
    cfg = build_cfg(tmp_path)
    cfg.trace_report_json.write_text(
        json.dumps(
            {
                "summary": {
                    "events_total": 6,
                    "quality_score": 100.0,
                    "stage_flags": {
                        "has_route": True,
                        "has_injection": True,
                        "has_injection_mode": False,
                        "has_reflection": True,
                        "has_memory": True,
                    },
                },
                "errors": [],
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    with pytest.raises(RuntimeError, match="stage flag missing: has_injection_mode"):
        assert_trace_reconstruction_quality(cfg)


def test_assert_evolution_slow_response_quality_accepts_long_horizon_report(tmp_path: Path) -> None:
    cfg = build_cfg(tmp_path)
    cfg.evolution_report_json.write_text(
        json.dumps(
            {
                "overall_passed": True,
                "scenarios": [
                    {
                        "scenario_id": "memory_self_correction_high_complexity_dag",
                        "duration_ms": 32000,
                        "steps": [
                            {"step_id": "a", "duration_ms": 1600},
                            {"step_id": "b", "duration_ms": 900},
                            {"step_id": "c", "duration_ms": 1700},
                        ],
                    }
                ],
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    assert_evolution_slow_response_quality(cfg)


def test_assert_evolution_slow_response_quality_rejects_short_report(tmp_path: Path) -> None:
    cfg = build_cfg(tmp_path)
    cfg.evolution_report_json.write_text(
        json.dumps(
            {
                "overall_passed": True,
                "scenarios": [
                    {
                        "scenario_id": "memory_self_correction_high_complexity_dag",
                        "duration_ms": 8000,
                        "steps": [{"step_id": "a", "duration_ms": 500}],
                    }
                ],
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    with pytest.raises(RuntimeError, match="slow-response resilience gate failed"):
        assert_evolution_slow_response_quality(cfg)
