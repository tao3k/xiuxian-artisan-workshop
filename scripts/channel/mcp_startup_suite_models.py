#!/usr/bin/env python3
"""Datamodels for MCP startup suite runner."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


@dataclass(frozen=True)
class SuiteConfig:
    hot_rounds: int
    hot_parallel: int
    cold_rounds: int
    cold_parallel: int
    startup_timeout_secs: int
    cooldown_secs: float
    mcp_host: str
    mcp_port: int
    mcp_config: Path
    health_url: str
    strict_health_check: bool
    health_probe_interval_secs: float
    health_probe_timeout_secs: float
    restart_mcp_cmd: str | None
    allow_mcp_restart: bool
    restart_mcp_settle_secs: float
    restart_health_timeout_secs: int
    restart_no_embedding: bool
    skip_hot: bool
    skip_cold: bool
    quality_max_failed_probes: int
    quality_max_hot_p95_ms: float
    quality_max_cold_p95_ms: float
    quality_min_health_samples: int
    quality_max_health_failure_rate: float
    quality_max_health_p95_ms: float
    quality_baseline_json: Path | None
    quality_max_hot_p95_regression_ratio: float
    quality_max_cold_p95_regression_ratio: float
    project_root: Path
    stress_script: Path
    restart_script: Path
    output_json: Path
    output_markdown: Path


@dataclass(frozen=True)
class ModeSpec:
    name: str
    rounds: int
    parallel: int
    restart_mcp_cmd: str | None
