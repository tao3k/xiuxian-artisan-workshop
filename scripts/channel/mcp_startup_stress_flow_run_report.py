#!/usr/bin/env python3
"""Report payload builder for MCP startup stress flow run."""

from __future__ import annotations

import time
from dataclasses import asdict
from datetime import UTC, datetime
from typing import Any


def build_run_report(
    cfg: Any,
    *,
    started_dt: datetime,
    started_monotonic: float,
    health_preflight: dict[str, object] | None,
    restart_events: list[dict[str, object]],
    summary: dict[str, object],
    results: list[Any],
    health_rows: list[Any],
) -> dict[str, object]:
    """Build final stress-run report payload."""
    finished_dt = datetime.now(UTC)
    return {
        "started_at": started_dt.isoformat(),
        "finished_at": finished_dt.isoformat(),
        "duration_ms": int((time.monotonic() - started_monotonic) * 1000),
        "config": {
            "rounds": cfg.rounds,
            "parallel": cfg.parallel,
            "startup_timeout_secs": cfg.startup_timeout_secs,
            "cooldown_secs": cfg.cooldown_secs,
            "executable": str(cfg.executable),
            "mcp_config": str(cfg.mcp_config),
            "bind_addr": cfg.bind_addr,
            "rust_log": cfg.rust_log,
            "health_url": cfg.health_url,
            "health_probe_interval_secs": cfg.health_probe_interval_secs,
            "health_probe_timeout_secs": cfg.health_probe_timeout_secs,
            "restart_mcp_cmd": cfg.restart_mcp_cmd,
            "restart_mcp_settle_secs": cfg.restart_mcp_settle_secs,
        },
        "health_preflight": health_preflight,
        "restart_events": restart_events,
        "summary": summary,
        "results": [asdict(row) for row in results],
        "health_samples": [asdict(row) for row in health_rows[-200:]],
    }
