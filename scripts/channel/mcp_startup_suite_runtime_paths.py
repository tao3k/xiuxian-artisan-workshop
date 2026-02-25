#!/usr/bin/env python3
"""Path/command spec helpers for MCP startup suite runtime."""

from __future__ import annotations

import json
import os
import shlex
from pathlib import Path
from typing import Any


def shell_join(parts: list[str]) -> str:
    """Quote and join shell command parts."""
    return " ".join(shlex.quote(part) for part in parts)


def build_restart_command(cfg: Any) -> str:
    """Build command used to restart MCP service for cold-start mode."""
    if cfg.restart_mcp_cmd:
        return cfg.restart_mcp_cmd

    runtime_root = Path(os.environ.get("PRJ_RUNTIME_DIR", ".run"))
    if not runtime_root.is_absolute():
        runtime_root = cfg.project_root / runtime_root
    pid_file = runtime_root / f"omni-mcp-sse-{cfg.mcp_port}.pid"
    log_file = runtime_root / "logs" / f"omni-mcp-sse-{cfg.mcp_port}.log"
    cmd = [
        "bash",
        str(cfg.restart_script),
        "--host",
        cfg.mcp_host,
        "--port",
        str(cfg.mcp_port),
        "--pid-file",
        str(pid_file),
        "--log-file",
        str(log_file),
        "--health-timeout-secs",
        str(cfg.restart_health_timeout_secs),
    ]
    if cfg.restart_no_embedding:
        cmd.append("--no-embedding")
    return shell_join(cmd)


def build_mode_specs(cfg: Any, *, mode_spec_cls: Any) -> tuple[Any, ...]:
    """Resolve hot/cold mode specs based on config flags."""
    specs: list[Any] = []
    if not cfg.skip_hot:
        specs.append(
            mode_spec_cls(
                name="hot",
                rounds=cfg.hot_rounds,
                parallel=cfg.hot_parallel,
                restart_mcp_cmd=None,
            )
        )
    if not cfg.skip_cold:
        specs.append(
            mode_spec_cls(
                name="cold",
                rounds=cfg.cold_rounds,
                parallel=cfg.cold_parallel,
                restart_mcp_cmd=build_restart_command(cfg),
            )
        )
    return tuple(specs)


def mode_report_paths(cfg: Any, mode: str) -> tuple[Path, Path]:
    """Build per-mode JSON/Markdown report output paths."""
    json_path = cfg.output_json.with_name(f"{cfg.output_json.stem}-{mode}{cfg.output_json.suffix}")
    md_path = cfg.output_markdown.with_name(
        f"{cfg.output_markdown.stem}-{mode}{cfg.output_markdown.suffix}"
    )
    return json_path, md_path


def load_summary(path: Path) -> dict[str, object] | None:
    """Load summary section from one stress mode report."""
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None
    summary = payload.get("summary")
    return summary if isinstance(summary, dict) else None
