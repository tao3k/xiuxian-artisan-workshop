#!/usr/bin/env python3
"""Per-mode execution helpers for MCP startup suite runtime."""

from __future__ import annotations

import subprocess
import sys
import time
from typing import TYPE_CHECKING, Any

from mcp_startup_suite_runtime_paths import load_summary, mode_report_paths

if TYPE_CHECKING:
    from pathlib import Path


def run_shell_command(command: str, cwd: Path) -> tuple[int, str]:
    """Run shell command and return exit code + merged output."""
    completed = subprocess.run(
        command,
        cwd=str(cwd),
        shell=True,
        capture_output=True,
        text=True,
        check=False,
    )
    output = (completed.stdout or "") + ("\n" + completed.stderr if completed.stderr else "")
    return completed.returncode, output.strip()


def run_mode(cfg: Any, spec: Any) -> dict[str, object]:
    """Execute one mode by invoking stress probe script."""
    report_json, report_markdown = mode_report_paths(cfg, spec.name)
    pre_restart_output = ""
    if spec.restart_mcp_cmd:
        pre_restart_code, pre_restart_output = run_shell_command(
            spec.restart_mcp_cmd,
            cfg.project_root,
        )
        if pre_restart_code != 0:
            return {
                "mode": spec.name,
                "rounds": spec.rounds,
                "parallel": spec.parallel,
                "return_code": pre_restart_code,
                "duration_ms": 0,
                "passed": False,
                "summary": None,
                "json_report": str(report_json),
                "markdown_report": str(report_markdown),
                "stdout_tail": "",
                "stderr_tail": pre_restart_output[-1200:],
                "pre_restart_failed": True,
            }

    cmd = [
        sys.executable,
        str(cfg.stress_script),
        "--rounds",
        str(spec.rounds),
        "--parallel",
        str(spec.parallel),
        "--startup-timeout-secs",
        str(cfg.startup_timeout_secs),
        "--cooldown-secs",
        str(cfg.cooldown_secs),
        "--mcp-config",
        str(cfg.mcp_config),
        "--health-url",
        cfg.health_url,
        "--health-probe-interval-secs",
        str(cfg.health_probe_interval_secs),
        "--health-probe-timeout-secs",
        str(cfg.health_probe_timeout_secs),
        "--output-json",
        str(report_json),
        "--output-markdown",
        str(report_markdown),
    ]
    if cfg.strict_health_check:
        cmd.append("--strict-health-check")
    if spec.restart_mcp_cmd:
        cmd.extend(["--restart-mcp-cmd", spec.restart_mcp_cmd])
        cmd.extend(["--restart-mcp-settle-secs", str(cfg.restart_mcp_settle_secs)])

    started = time.monotonic()
    completed = subprocess.run(
        cmd,
        cwd=str(cfg.project_root),
        capture_output=True,
        text=True,
        check=False,
    )
    duration_ms = int((time.monotonic() - started) * 1000)
    summary = load_summary(report_json)
    summary_failed = int(summary.get("failed", 1)) if summary else 1
    passed = completed.returncode == 0 and summary_failed == 0

    print(
        f"[mode:{spec.name}] return_code={completed.returncode} "
        f"duration_ms={duration_ms} passed={passed}",
        flush=True,
    )

    return {
        "mode": spec.name,
        "rounds": spec.rounds,
        "parallel": spec.parallel,
        "return_code": completed.returncode,
        "duration_ms": duration_ms,
        "passed": passed,
        "summary": summary,
        "json_report": str(report_json),
        "markdown_report": str(report_markdown),
        "stdout_tail": "\n".join((completed.stdout or "").splitlines()[-20:]),
        "stderr_tail": "\n".join((completed.stderr or "").splitlines()[-20:]),
        "pre_restart_output": pre_restart_output[-1200:],
    }
