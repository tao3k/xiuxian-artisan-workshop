#!/usr/bin/env python3
"""Config construction helpers for MCP startup suite runner."""

from __future__ import annotations

from types import SimpleNamespace
from typing import TYPE_CHECKING, Any

from mcp_startup_suite_config_build_modes import resolve_restart_and_mode_flags
from mcp_startup_suite_config_build_paths import resolve_quality_baseline, resolve_required_paths
from mcp_startup_suite_config_build_validation import validate_numeric_constraints
from path_resolver import resolve_path
from resolve_mcp_endpoint import resolve_mcp_endpoint

if TYPE_CHECKING:
    from mcp_startup_suite_models import SuiteConfig


def build_config(args: Any, *, config_cls: type[SuiteConfig]) -> SuiteConfig:
    """Validate arguments and construct typed suite config."""
    resolved_endpoint = resolve_mcp_endpoint()
    mcp_host = str(getattr(args, "mcp_host", "")).strip() or str(resolved_endpoint["host"])
    raw_port = int(getattr(args, "mcp_port", 0))
    if raw_port < 0:
        raise ValueError("--mcp-port must be positive.")
    mcp_port = raw_port if raw_port > 0 else int(resolved_endpoint["port"])

    args_payload = dict(vars(args))
    args_payload["mcp_host"] = mcp_host
    args_payload["mcp_port"] = mcp_port
    args_for_validation = SimpleNamespace(**args_payload)
    validate_numeric_constraints(args_for_validation)
    project_root, mcp_config, stress_script, restart_script = resolve_required_paths(
        args_for_validation
    )

    health_url = args.health_url.strip() or f"http://{mcp_host}:{mcp_port}/health"
    strict_health_check = not bool(args.no_strict_health_check)
    if args.strict_health_check:
        strict_health_check = True

    restart_mcp_cmd, allow_mcp_restart, skip_hot, skip_cold = resolve_restart_and_mode_flags(args)
    baseline_path = resolve_quality_baseline(args, project_root)

    return config_cls(
        hot_rounds=int(args.hot_rounds),
        hot_parallel=int(args.hot_parallel),
        cold_rounds=int(args.cold_rounds),
        cold_parallel=int(args.cold_parallel),
        startup_timeout_secs=int(args.startup_timeout_secs),
        cooldown_secs=float(args.cooldown_secs),
        mcp_host=mcp_host,
        mcp_port=int(mcp_port),
        mcp_config=mcp_config,
        health_url=health_url,
        strict_health_check=bool(strict_health_check),
        health_probe_interval_secs=float(args.health_probe_interval_secs),
        health_probe_timeout_secs=float(args.health_probe_timeout_secs),
        restart_mcp_cmd=restart_mcp_cmd,
        allow_mcp_restart=allow_mcp_restart,
        restart_mcp_settle_secs=float(args.restart_mcp_settle_secs),
        restart_health_timeout_secs=int(args.restart_health_timeout_secs),
        restart_no_embedding=bool(args.restart_no_embedding),
        skip_hot=skip_hot,
        skip_cold=skip_cold,
        quality_max_failed_probes=int(args.quality_max_failed_probes),
        quality_max_hot_p95_ms=float(args.quality_max_hot_p95_ms),
        quality_max_cold_p95_ms=float(args.quality_max_cold_p95_ms),
        quality_min_health_samples=int(args.quality_min_health_samples),
        quality_max_health_failure_rate=float(args.quality_max_health_failure_rate),
        quality_max_health_p95_ms=float(args.quality_max_health_p95_ms),
        quality_baseline_json=baseline_path,
        quality_max_hot_p95_regression_ratio=float(args.quality_max_hot_p95_regression_ratio),
        quality_max_cold_p95_regression_ratio=float(args.quality_max_cold_p95_regression_ratio),
        project_root=project_root,
        stress_script=stress_script,
        restart_script=restart_script,
        output_json=resolve_path(args.output_json, project_root),
        output_markdown=resolve_path(args.output_markdown, project_root),
    )
