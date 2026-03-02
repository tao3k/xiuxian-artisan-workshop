#!/usr/bin/env python3
"""Path-resolution helpers for MCP startup suite config construction."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from path_resolver import resolve_path


def resolve_required_paths(args: Any) -> tuple[Path, Path, Path, Path]:
    """Resolve required project/config/script paths and validate existence."""
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


def resolve_quality_baseline(args: Any, project_root: Path) -> Path | None:
    """Resolve optional baseline report path if present."""
    quality_baseline_json = args.quality_baseline_json.strip()
    baseline_path = (
        resolve_path(quality_baseline_json, project_root) if quality_baseline_json else None
    )
    if baseline_path is not None and not baseline_path.exists():
        raise ValueError(f"quality baseline report not found: {baseline_path}")
    return baseline_path
