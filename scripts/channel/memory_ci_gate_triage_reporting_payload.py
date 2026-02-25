#!/usr/bin/env python3
"""Payload builders for memory CI gate failure triage reporting."""

from __future__ import annotations

import time
from typing import Any


def build_gate_failure_triage_payload(
    cfg: Any,
    *,
    error: Exception,
    category: str,
    summary: str,
    repro_commands: list[str],
    read_tail_fn: Any,
    shell_quote_command_fn: Any,
    artifact_rows_fn: Any,
    is_gate_step_error_fn: Any,
) -> dict[str, object]:
    """Build the normalized triage payload for markdown/json reports."""
    payload: dict[str, object] = {
        "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "profile": cfg.profile,
        "category": category,
        "summary": summary,
        "error": str(error),
        "artifacts": [
            {"name": name, "path": str(path), "exists": bool(path.exists())}
            for name, path in artifact_rows_fn(cfg)
        ],
        "repro_commands": list(repro_commands),
    }
    runtime_tail = read_tail_fn(cfg.runtime_log_file, max_lines=80).strip()
    if runtime_tail:
        payload["runtime_log_tail"] = runtime_tail
    if is_gate_step_error_fn(error):
        payload["failed_stage"] = error.title
        payload["failed_exit_code"] = error.returncode
        payload["failed_command"] = shell_quote_command_fn(error.cmd)
    return payload
