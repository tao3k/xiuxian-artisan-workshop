#!/usr/bin/env python3
"""Quality and warning-budget tests for memory CI gate triage."""

from __future__ import annotations

import json
from dataclasses import replace

import pytest
from test_memory_ci_gate import build_cfg
from test_omni_agent_memory_ci_gate import (
    assert_cross_group_complex_quality,
    assert_mcp_waiting_warning_budget,
)


def test_assert_cross_group_complex_quality_accepts_parallel_isolation_report(tmp_path) -> None:
    cfg = build_cfg(tmp_path)
    cfg.cross_group_report_json.write_text(
        json.dumps(
            {
                "overall_passed": True,
                "scenarios": [
                    {
                        "scenario_id": "cross_group_control_plane_stress",
                        "passed": True,
                        "steps": [
                            {
                                "step_id": "a0",
                                "session_alias": "a",
                                "session_key": "telegram:1001:2001",
                                "wave_index": 0,
                                "mcp_waiting_seen": False,
                            },
                            {
                                "step_id": "b0",
                                "session_alias": "b",
                                "session_key": "telegram:1002:2002",
                                "wave_index": 0,
                                "mcp_waiting_seen": False,
                            },
                            {
                                "step_id": "c0",
                                "session_alias": "c",
                                "session_key": "telegram:1003:2003",
                                "wave_index": 1,
                                "mcp_waiting_seen": False,
                            },
                        ],
                    }
                ],
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    assert_cross_group_complex_quality(cfg)


def test_assert_cross_group_complex_quality_rejects_missing_third_group(tmp_path) -> None:
    cfg = build_cfg(tmp_path)
    cfg.cross_group_report_json.write_text(
        json.dumps(
            {
                "overall_passed": True,
                "scenarios": [
                    {
                        "scenario_id": "cross_group_control_plane_stress",
                        "passed": True,
                        "steps": [
                            {
                                "step_id": "a0",
                                "session_alias": "a",
                                "session_key": "telegram:1001:2001",
                                "wave_index": 0,
                                "mcp_waiting_seen": False,
                            },
                            {
                                "step_id": "b0",
                                "session_alias": "b",
                                "session_key": "telegram:1002:2002",
                                "wave_index": 0,
                                "mcp_waiting_seen": False,
                            },
                        ],
                    }
                ],
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    with pytest.raises(RuntimeError, match="missing session aliases"):
        assert_cross_group_complex_quality(cfg)


def test_assert_mcp_waiting_warning_budget_accepts_clean_runtime_log(tmp_path) -> None:
    cfg = build_cfg(tmp_path)
    cfg.runtime_log_file.write_text(
        "\n".join(
            [
                '2026-02-20T00:00:00Z INFO event="session.route.decision_selected"',
                '2026-02-20T00:00:01Z INFO event="agent.memory.recall.planned"',
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    assert_mcp_waiting_warning_budget(cfg)


def test_assert_mcp_waiting_warning_budget_rejects_over_budget(tmp_path) -> None:
    cfg = build_cfg(tmp_path)
    cfg.runtime_log_file.write_text(
        "\n".join(
            [
                '2026-02-20T00:00:00Z WARN event="mcp.pool.call.waiting"',
                '2026-02-20T00:00:01Z WARN event="mcp.pool.connect.waiting"',
                '2026-02-20T00:00:02Z WARN event="mcp.pool.connect.waiting"',
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    with pytest.raises(RuntimeError, match="mcp waiting warning budget exceeded"):
        assert_mcp_waiting_warning_budget(cfg)


def test_assert_mcp_waiting_warning_budget_allows_configured_budget(tmp_path) -> None:
    cfg = build_cfg(tmp_path)
    cfg = replace(
        cfg,
        max_mcp_call_waiting_events=2,
        max_mcp_connect_waiting_events=3,
        max_mcp_waiting_events_total=5,
    )
    cfg.runtime_log_file.write_text(
        "\n".join(
            [
                '2026-02-20T00:00:00Z WARN event="mcp.pool.call.waiting"',
                '2026-02-20T00:00:01Z WARN event="mcp.pool.connect.waiting"',
                '2026-02-20T00:00:02Z WARN event="mcp.pool.connect.waiting"',
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    assert_mcp_waiting_warning_budget(cfg)
