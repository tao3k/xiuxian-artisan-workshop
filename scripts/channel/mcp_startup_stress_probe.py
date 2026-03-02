#!/usr/bin/env python3
"""Single-probe execution helpers for MCP startup stress tests."""

from __future__ import annotations

import os
from typing import Any

from mcp_startup_stress_probe_runtime import execute_probe


def run_single_probe(
    cfg: Any,
    round_index: int,
    worker_index: int,
    *,
    probe_result_cls: Any,
    classify_reason_fn: Any,
) -> Any:
    """Run one gateway startup probe and parse key handshake events."""
    env = os.environ.copy()
    env["RUST_LOG"] = cfg.rust_log
    cmd = [
        str(cfg.executable),
        "gateway",
        "--bind",
        cfg.bind_addr,
        "--mcp-config",
        str(cfg.mcp_config),
    ]
    details = execute_probe(
        cmd=cmd,
        cwd=str(cfg.project_root),
        env=env,
        startup_timeout_secs=cfg.startup_timeout_secs,
        classify_reason_fn=classify_reason_fn,
    )

    return probe_result_cls(
        round_index=round_index,
        worker_index=worker_index,
        success=bool(details["success"]),
        reason=str(details["reason"]),
        startup_duration_ms=int(details["startup_duration_ms"]),
        return_code=details["return_code"],
        mcp_connect_succeeded=int(details["mcp_connect_succeeded"]),
        mcp_connect_failed=int(details["mcp_connect_failed"]),
        handshake_timeout_seen=bool(details["handshake_timeout_seen"]),
        connect_failed_seen=bool(details["connect_failed_seen"]),
        ready_seen=bool(details["ready_seen"]),
        tail=str(details["tail"]),
    )
