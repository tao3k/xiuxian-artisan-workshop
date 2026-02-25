#!/usr/bin/env python3
"""Datamodels for MCP startup stress probe."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


@dataclass(frozen=True)
class StressConfig:
    rounds: int
    parallel: int
    startup_timeout_secs: int
    cooldown_secs: float
    executable: Path
    mcp_config: Path
    project_root: Path
    bind_addr: str
    rust_log: str
    output_json: Path
    output_markdown: Path
    restart_mcp_cmd: str | None
    restart_mcp_settle_secs: float
    health_url: str | None
    strict_health_check: bool
    health_probe_interval_secs: float
    health_probe_timeout_secs: float


@dataclass(frozen=True)
class ProbeResult:
    round_index: int
    worker_index: int
    success: bool
    reason: str
    startup_duration_ms: int
    return_code: int | None
    mcp_connect_succeeded: int
    mcp_connect_failed: int
    handshake_timeout_seen: bool
    connect_failed_seen: bool
    ready_seen: bool
    tail: str


@dataclass(frozen=True)
class HealthSample:
    ok: bool
    latency_ms: float
    detail: str
