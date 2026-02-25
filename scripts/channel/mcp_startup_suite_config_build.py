#!/usr/bin/env python3
"""Config construction helpers for MCP startup suite runner."""

from __future__ import annotations

import sys
from pathlib import Path
from typing import TYPE_CHECKING, Any

from path_resolver import resolve_path

if TYPE_CHECKING:
    from mcp_startup_suite_models import SuiteConfig


def _validate_numeric_constraints(args: Any) -> None:
    if args.hot_rounds <= 0 or args.hot_parallel <= 0:
        raise ValueError("--hot-rounds and --hot-parallel must be positive.")
    if args.cold_rounds <= 0 or args.cold_parallel <= 0:
        raise ValueError("--cold-rounds and --cold-parallel must be positive.")
    if args.startup_timeout_secs <= 0:
        raise ValueError("--startup-timeout-secs must be positive.")
    if args.cooldown_secs < 0:
        raise ValueError("--cooldown-secs must be >= 0.")
    if args.restart_mcp_settle_secs < 0:
        raise ValueError("--restart-mcp-settle-secs must be >= 0.")
    if args.health_probe_interval_secs < 0:
        raise ValueError("--health-probe-interval-secs must be >= 0.")
    if args.health_probe_timeout_secs <= 0:
        raise ValueError("--health-probe-timeout-secs must be positive.")
    if args.restart_health_timeout_secs <= 0:
        raise ValueError("--restart-health-timeout-secs must be positive.")
    if args.mcp_port <= 0:
        raise ValueError("--mcp-port must be positive.")
    if args.quality_max_failed_probes < 0:
        raise ValueError("--quality-max-failed-probes must be >= 0.")
    if args.quality_max_hot_p95_ms <= 0:
        raise ValueError("--quality-max-hot-p95-ms must be positive.")
    if args.quality_max_cold_p95_ms <= 0:
        raise ValueError("--quality-max-cold-p95-ms must be positive.")
    if args.quality_min_health_samples < 0:
        raise ValueError("--quality-min-health-samples must be >= 0.")
    if args.quality_max_health_failure_rate < 0:
        raise ValueError("--quality-max-health-failure-rate must be >= 0.")
    if args.quality_max_health_p95_ms <= 0:
        raise ValueError("--quality-max-health-p95-ms must be positive.")
    if args.quality_max_hot_p95_regression_ratio < 0:
        raise ValueError("--quality-max-hot-p95-regression-ratio must be >= 0.")
    if args.quality_max_cold_p95_regression_ratio < 0:
        raise ValueError("--quality-max-cold-p95-regression-ratio must be >= 0.")


def _resolve_required_paths(args: Any) -> tuple[Path, Path, Path, Path]:
    project_root = resolve_path(args.project_root, Path.cwd())
    mcp_config = resolve_path(args.mcp_config, project_root)
    if not mcp_config.exists():
        raise ValueError(f"mcp config not found: {mcp_config}")

    script_dir = Path(__file__).resolve().parent
    stress_script = script_dir / "test_omni_agent_mcp_startup_stress.py"
    restart_script = script_dir / "restart-omni-mcp.sh"
    if not stress_script.exists():
        raise ValueError(f"stress runner not found: {stress_script}")
    if not restart_script.exists():
        raise ValueError(f"restart script not found: {restart_script}")
    return project_root, mcp_config, stress_script, restart_script


def _resolve_restart_and_mode_flags(
    args: Any,
) -> tuple[str | None, bool, bool, bool]:
    restart_mcp_cmd = args.restart_mcp_cmd.strip() or None
    allow_mcp_restart = bool(args.allow_mcp_restart or restart_mcp_cmd is not None)
    skip_hot = bool(args.skip_hot)
    skip_cold = bool(args.skip_cold)
    if skip_hot and not skip_cold and not allow_mcp_restart:
        raise ValueError(
            "cold-only startup suite requires MCP restart permission. "
            "Use --allow-mcp-restart or --restart-mcp-cmd."
        )
    if not skip_cold and not allow_mcp_restart:
        print(
            "[mcp-startup-suite] cold mode auto-skipped (restart not allowed). "
            "Use --allow-mcp-restart to enable cold restart checks.",
            file=sys.stderr,
        )
        skip_cold = True
    if skip_hot and skip_cold:
        raise ValueError("At least one mode must run (do not set both --skip-hot and --skip-cold).")
    return restart_mcp_cmd, allow_mcp_restart, skip_hot, skip_cold


def _resolve_quality_baseline(args: Any, project_root: Path) -> Path | None:
    quality_baseline_json = args.quality_baseline_json.strip()
    baseline_path = (
        resolve_path(quality_baseline_json, project_root) if quality_baseline_json else None
    )
    if baseline_path is not None and not baseline_path.exists():
        raise ValueError(f"quality baseline report not found: {baseline_path}")
    return baseline_path


def build_config(args: Any, *, config_cls: type[SuiteConfig]) -> SuiteConfig:
    """Validate arguments and construct typed suite config."""
    _validate_numeric_constraints(args)
    project_root, mcp_config, stress_script, restart_script = _resolve_required_paths(args)

    health_url = args.health_url.strip() or f"http://{args.mcp_host}:{args.mcp_port}/health"
    strict_health_check = not bool(args.no_strict_health_check)
    if args.strict_health_check:
        strict_health_check = True

    restart_mcp_cmd, allow_mcp_restart, skip_hot, skip_cold = _resolve_restart_and_mode_flags(args)
    baseline_path = _resolve_quality_baseline(args, project_root)

    return config_cls(
        hot_rounds=int(args.hot_rounds),
        hot_parallel=int(args.hot_parallel),
        cold_rounds=int(args.cold_rounds),
        cold_parallel=int(args.cold_parallel),
        startup_timeout_secs=int(args.startup_timeout_secs),
        cooldown_secs=float(args.cooldown_secs),
        mcp_host=args.mcp_host.strip(),
        mcp_port=int(args.mcp_port),
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
