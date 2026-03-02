#!/usr/bin/env python3
"""Unit tests for MCP startup suite config helpers."""

from __future__ import annotations

import argparse
import importlib
import sys
from pathlib import Path

import pytest

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

config_module = importlib.import_module("mcp_startup_suite_config")
models_module = importlib.import_module("mcp_startup_suite_models")
endpoints = importlib.import_module("channel_test_endpoints")


def _base_args(tmp_path: Path) -> argparse.Namespace:
    mcp_config = tmp_path / ".mcp.json"
    mcp_config.write_text("{}", encoding="utf-8")
    return argparse.Namespace(
        hot_rounds=1,
        hot_parallel=1,
        cold_rounds=1,
        cold_parallel=1,
        startup_timeout_secs=60,
        cooldown_secs=0.1,
        mcp_host=endpoints.DEFAULT_LOCAL_HOST,
        mcp_port=3002,
        mcp_config=str(mcp_config),
        health_url="",
        strict_health_check=False,
        no_strict_health_check=False,
        health_probe_interval_secs=0.2,
        health_probe_timeout_secs=1.0,
        restart_mcp_cmd="",
        allow_mcp_restart=False,
        restart_mcp_settle_secs=0.2,
        restart_health_timeout_secs=30,
        restart_no_embedding=False,
        skip_hot=False,
        skip_cold=False,
        quality_max_failed_probes=0,
        quality_max_hot_p95_ms=1200.0,
        quality_max_cold_p95_ms=1500.0,
        quality_min_health_samples=1,
        quality_max_health_failure_rate=0.02,
        quality_max_health_p95_ms=350.0,
        quality_baseline_json="",
        quality_max_hot_p95_regression_ratio=0.5,
        quality_max_cold_p95_regression_ratio=0.5,
        project_root=str(tmp_path),
        output_json=str(tmp_path / "suite.json"),
        output_markdown=str(tmp_path / "suite.md"),
    )


def test_build_config_rejects_negative_port(tmp_path: Path) -> None:
    args = _base_args(tmp_path)
    args.mcp_port = -1
    with pytest.raises(ValueError, match="--mcp-port must be positive"):
        config_module.build_config(args, config_cls=models_module.SuiteConfig)


def test_build_config_sets_skip_cold_when_restart_not_allowed(tmp_path: Path) -> None:
    args = _base_args(tmp_path)
    cfg = config_module.build_config(args, config_cls=models_module.SuiteConfig)
    assert cfg.skip_cold is True
