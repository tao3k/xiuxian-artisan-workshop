#!/usr/bin/env python3
"""Shared helpers for runtime monitor."""

from __future__ import annotations

import json
import signal
from datetime import UTC, datetime
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path

from agent_channel_runtime_monitor_models import EVENT_TOKEN_RE


def now_utc_iso() -> str:
    """Return current UTC timestamp in ISO-like format."""
    return datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ")


def normalize_exit_code(returncode: int) -> int:
    """Normalize negative signal returncode into shell-compatible exit code."""
    if returncode < 0:
        return 128 + abs(returncode)
    return returncode


def classify_exit(returncode: int) -> dict[str, str | int | None]:
    """Classify process exit into structured kind/exit/signal payload."""
    if returncode == 0:
        return {"kind": "ok", "exit_code": 0, "signal": None, "signal_name": None}
    if returncode < 0:
        signal_num = abs(returncode)
        try:
            signal_name = signal.Signals(signal_num).name
        except ValueError:
            signal_name = f"SIG{signal_num}"
        return {
            "kind": "signal",
            "exit_code": 128 + signal_num,
            "signal": signal_num,
            "signal_name": signal_name,
        }
    signal_num = returncode - 128 if returncode >= 128 else None
    signal_name = None
    if signal_num:
        try:
            signal_name = signal.Signals(signal_num).name
        except ValueError:
            signal_name = f"SIG{signal_num}"
    return {
        "kind": "nonzero",
        "exit_code": returncode,
        "signal": signal_num,
        "signal_name": signal_name,
    }


def extract_event_token(line: str) -> str | None:
    """Extract event token from one log line."""
    match = EVENT_TOKEN_RE.search(line)
    if match:
        return match.group(1)
    return None


def write_report(report_file: Path, report_jsonl: Path | None, report: dict[str, Any]) -> None:
    """Write report JSON and optional JSONL append output."""
    report_file.parent.mkdir(parents=True, exist_ok=True)
    report_file.write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    if report_jsonl is None:
        return
    report_jsonl.parent.mkdir(parents=True, exist_ok=True)
    with report_jsonl.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(report, ensure_ascii=False) + "\n")
