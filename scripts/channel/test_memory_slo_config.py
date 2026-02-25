#!/usr/bin/env python3
"""Unit tests for memory SLO config helpers."""

from __future__ import annotations

import argparse
import importlib
import sys
from pathlib import Path

import pytest

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

config_module = importlib.import_module("memory_slo_config")
models_module = importlib.import_module("memory_slo_models")


def _base_args(tmp_path: Path) -> argparse.Namespace:
    return argparse.Namespace(
        project_root=str(tmp_path),
        evolution_report_json=".run/reports/evolution.json",
        benchmark_report_json=".run/reports/benchmark.json",
        session_matrix_report_json=".run/reports/matrix.json",
        runtime_log_file="",
        output_json=".run/reports/slo.json",
        output_markdown=".run/reports/slo.md",
        min_planned_hits=1,
        min_successful_corrections=1,
        min_recall_credit_events=0,
        min_quality_score=80.0,
        required_benchmark_modes="baseline,adaptive",
        min_query_turns=1,
        max_mode_mcp_error_turns=0,
        max_total_mcp_error_turns=0,
        min_session_steps=1,
        max_session_failed_steps=0,
        enable_stream_gate=False,
        min_stream_ack_ratio=0.9,
        min_stream_published_events=1,
        max_stream_read_failed=0,
    )


def test_parse_required_modes_requires_non_empty() -> None:
    with pytest.raises(ValueError, match="required-benchmark-modes"):
        config_module.parse_required_modes(" ,, ")


def test_build_config_rejects_invalid_stream_ratio(tmp_path: Path) -> None:
    args = _base_args(tmp_path)
    args.min_stream_ack_ratio = 1.5
    with pytest.raises(ValueError, match="--min-stream-ack-ratio"):
        config_module.build_config(args, config_cls=models_module.SloConfig)


def test_build_config_resolves_runtime_log_when_provided(tmp_path: Path) -> None:
    args = _base_args(tmp_path)
    args.runtime_log_file = ".run/logs/omni-agent.log"
    config = config_module.build_config(args, config_cls=models_module.SloConfig)
    assert config.runtime_log_file == (tmp_path / ".run/logs/omni-agent.log").resolve()
