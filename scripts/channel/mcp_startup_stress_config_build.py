#!/usr/bin/env python3
"""Validation/build helpers for MCP startup stress config."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from path_resolver import resolve_path
from resolve_mcp_endpoint import resolve_mcp_endpoint

if TYPE_CHECKING:
    import argparse

    from mcp_startup_stress_models import StressConfig


def validate_args(args: argparse.Namespace) -> None:
    """Validate numeric bounds for stress config args."""
    if args.rounds <= 0:
        raise ValueError("--rounds must be positive.")
    if args.parallel <= 0:
        raise ValueError("--parallel must be positive.")
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


def resolve_runtime_paths(args: argparse.Namespace) -> tuple[Path, Path, Path, Path, Path]:
    """Resolve project/executable/config/output paths relative to project root."""
    project_root = resolve_path(args.project_root, Path.cwd())
    executable = resolve_path(args.executable, project_root)
    mcp_config = resolve_path(args.mcp_config, project_root)
    output_json = resolve_path(args.output_json, project_root)
    output_markdown = resolve_path(args.output_markdown, project_root)
    return project_root, executable, mcp_config, output_json, output_markdown


def build_config(
    args: argparse.Namespace,
    *,
    config_cls: type[StressConfig],
    validate_args_fn: object = validate_args,
    resolve_runtime_paths_fn: object = resolve_runtime_paths,
) -> StressConfig:
    """Validate arguments and build stress config."""
    validate_callable = validate_args_fn
    resolve_paths_callable = resolve_runtime_paths_fn
    assert callable(validate_callable)
    assert callable(resolve_paths_callable)

    validate_callable(args)

    project_root, executable, mcp_config, output_json, output_markdown = resolve_paths_callable(
        args
    )
    if not executable.exists():
        raise ValueError(
            f"executable not found: {executable}. Build first: cargo build -p omni-agent"
        )
    if not mcp_config.exists():
        raise ValueError(f"mcp config not found: {mcp_config}")

    restart_cmd = args.restart_mcp_cmd.strip() or None
    health_url = args.health_url.strip() or None

    bind_addr = args.bind_addr.strip()
    if not bind_addr:
        resolved_host = str(resolve_mcp_endpoint()["host"])
        bind_addr = f"{resolved_host}:0"

    if health_url is None:
        health_url = str(resolve_mcp_endpoint()["health_url"])

    return config_cls(
        rounds=int(args.rounds),
        parallel=int(args.parallel),
        startup_timeout_secs=int(args.startup_timeout_secs),
        cooldown_secs=float(args.cooldown_secs),
        executable=executable,
        mcp_config=mcp_config,
        project_root=project_root,
        bind_addr=bind_addr,
        rust_log=args.rust_log.strip(),
        output_json=output_json,
        output_markdown=output_markdown,
        restart_mcp_cmd=restart_cmd,
        restart_mcp_settle_secs=float(args.restart_mcp_settle_secs),
        health_url=health_url,
        strict_health_check=bool(args.strict_health_check),
        health_probe_interval_secs=float(args.health_probe_interval_secs),
        health_probe_timeout_secs=float(args.health_probe_timeout_secs),
    )
