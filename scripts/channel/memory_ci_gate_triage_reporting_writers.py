#!/usr/bin/env python3
"""Markdown/JSON writers for memory CI gate failure triage reporting."""

from __future__ import annotations

import json
from typing import Any


def write_gate_failure_triage_report(
    cfg: Any,
    *,
    error: Exception,
    category: str,
    summary: str,
    repro_commands: list[str],
    read_tail_fn: Any,
    shell_quote_command_fn: Any,
    report_path: Any | None = None,
    default_gate_failure_report_base_path_fn: Any,
    build_gate_failure_triage_payload_fn: Any,
    is_gate_step_error_fn: Any,
) -> Any:
    """Write markdown triage report and return the report path."""
    if report_path is None:
        base, _ = default_gate_failure_report_base_path_fn(cfg)
        report_path = base.with_suffix(".md")
    report_path.parent.mkdir(parents=True, exist_ok=True)
    payload = build_gate_failure_triage_payload_fn(
        cfg,
        error=error,
        category=category,
        summary=summary,
        repro_commands=repro_commands,
        read_tail_fn=read_tail_fn,
        shell_quote_command_fn=shell_quote_command_fn,
    )

    lines: list[str] = [
        "# Omni Agent Memory CI Failure Triage",
        "",
        f"- generated_at_utc: `{payload['generated_at_utc']}`",
        f"- profile: `{cfg.profile}`",
        f"- category: `{category}`",
        f"- summary: {summary}",
        f"- error: `{error}`",
    ]
    if is_gate_step_error_fn(error):
        lines.extend(
            [
                f"- failed_stage: `{error.title}`",
                f"- failed_exit_code: `{error.returncode}`",
                f"- failed_command: `{shell_quote_command_fn(error.cmd)}`",
            ]
        )

    lines.extend(["", "## Artifacts", ""])
    artifacts = payload.get("artifacts", [])
    for item in artifacts if isinstance(artifacts, list) else []:
        if not isinstance(item, dict):
            continue
        name = str(item.get("name", "unknown"))
        exists = "yes" if bool(item.get("exists", False)) else "no"
        path = str(item.get("path", ""))
        lines.append(f"- `{name}` exists={exists} path=`{path}`")

    lines.extend(["", "## Repro Commands", ""])
    for command in repro_commands:
        lines.append(f"1. `{command}`")

    runtime_tail_obj = payload.get("runtime_log_tail")
    runtime_tail = str(runtime_tail_obj).strip() if runtime_tail_obj is not None else ""
    if runtime_tail:
        lines.extend(["", "## Runtime Log Tail", "", "```text", runtime_tail, "```"])

    report_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return report_path


def write_gate_failure_triage_json_report(
    cfg: Any,
    *,
    error: Exception,
    category: str,
    summary: str,
    repro_commands: list[str],
    read_tail_fn: Any,
    shell_quote_command_fn: Any,
    report_path: Any | None = None,
    default_gate_failure_report_base_path_fn: Any,
    build_gate_failure_triage_payload_fn: Any,
) -> Any:
    """Write JSON triage report and return the report path."""
    if report_path is None:
        base, _ = default_gate_failure_report_base_path_fn(cfg)
        report_path = base.with_suffix(".json")
    report_path.parent.mkdir(parents=True, exist_ok=True)
    payload = build_gate_failure_triage_payload_fn(
        cfg,
        error=error,
        category=category,
        summary=summary,
        repro_commands=repro_commands,
        read_tail_fn=read_tail_fn,
        shell_quote_command_fn=shell_quote_command_fn,
    )
    report_path.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    return report_path
