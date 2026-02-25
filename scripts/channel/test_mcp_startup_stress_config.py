#!/usr/bin/env python3
"""Unit tests for MCP startup stress config helpers."""

from __future__ import annotations

import argparse
import importlib
import sys
from pathlib import Path

import pytest

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

config_module = importlib.import_module("mcp_startup_stress_config")
models_module = importlib.import_module("mcp_startup_stress_models")


def test_build_config_validates_positive_rounds(tmp_path: Path) -> None:
    executable = tmp_path / "omni-agent"
    executable.write_text("", encoding="utf-8")
    mcp_config = tmp_path / ".mcp.json"
    mcp_config.write_text("{}", encoding="utf-8")

    args = argparse.Namespace(
        rounds=0,
        parallel=1,
        startup_timeout_secs=30,
        cooldown_secs=0.2,
        executable=str(executable),
        mcp_config=str(mcp_config),
        project_root=str(tmp_path),
        bind_addr="127.0.0.1:0",
        rust_log="x",
        output_json=str(tmp_path / "out.json"),
        output_markdown=str(tmp_path / "out.md"),
        restart_mcp_cmd="",
        restart_mcp_settle_secs=0.0,
        health_url="",
        strict_health_check=False,
        health_probe_interval_secs=0.2,
        health_probe_timeout_secs=1.0,
    )

    with pytest.raises(ValueError, match="--rounds must be positive"):
        config_module.build_config(args, config_cls=models_module.StressConfig)


def test_build_config_resolves_paths(tmp_path: Path) -> None:
    executable = tmp_path / "omni-agent"
    executable.write_text("", encoding="utf-8")
    mcp_config = tmp_path / ".mcp.json"
    mcp_config.write_text("{}", encoding="utf-8")

    args = argparse.Namespace(
        rounds=1,
        parallel=1,
        startup_timeout_secs=30,
        cooldown_secs=0.2,
        executable=str(executable),
        mcp_config=str(mcp_config),
        project_root=str(tmp_path),
        bind_addr="127.0.0.1:0",
        rust_log="x",
        output_json=str(tmp_path / "out.json"),
        output_markdown=str(tmp_path / "out.md"),
        restart_mcp_cmd="",
        restart_mcp_settle_secs=0.0,
        health_url="http://127.0.0.1:3002/health",
        strict_health_check=False,
        health_probe_interval_secs=0.2,
        health_probe_timeout_secs=1.0,
    )

    cfg = config_module.build_config(args, config_cls=models_module.StressConfig)
    assert cfg.rounds == 1
    assert cfg.executable == executable.resolve()
