#!/usr/bin/env python3
"""Discover-cache gate helper for memory CI runtime."""

from __future__ import annotations

from typing import Any


def run_discover_cache_gate(
    cfg: Any,
    *,
    cwd: Any,
    env: dict[str, str],
    run_command_fn: Any,
) -> None:
    """Run discover-cache latency gate."""
    if cfg.skip_discover_cache_gate:
        print("Skipping discover cache gate (--skip-discover-cache-gate).", flush=True)
        return
    gate_env = env.copy()
    gate_env["OMNI_AGENT_DISCOVER_CACHE_HIT_P95_MS"] = f"{cfg.discover_cache_hit_p95_ms}"
    gate_env["OMNI_AGENT_DISCOVER_CACHE_MISS_P95_MS"] = f"{cfg.discover_cache_miss_p95_ms}"
    gate_env["OMNI_AGENT_DISCOVER_CACHE_BENCH_ITERATIONS"] = str(
        cfg.discover_cache_bench_iterations
    )
    run_command_fn(
        [
            "cargo",
            "test",
            "-p",
            "omni-agent",
            "--test",
            "mcp_discover_cache",
            "discover_calls_use_valkey_read_through_cache_when_configured",
            "--",
            "--ignored",
            "--exact",
        ],
        title="Discover cache latency gate (A3)",
        cwd=cwd,
        env=gate_env,
    )
