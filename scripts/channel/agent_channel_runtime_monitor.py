#!/usr/bin/env python3
"""Structured runtime monitor for long-running channel processes."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

_CLI = importlib.import_module("agent_channel_runtime_monitor_cli")
_COMMON = importlib.import_module("agent_channel_runtime_monitor_common")
_MODELS = importlib.import_module("agent_channel_runtime_monitor_models")
_RUNTIME = importlib.import_module("agent_channel_runtime_monitor_runtime")

parse_args = _CLI.parse_args
run_monitored_process = _RUNTIME.run_monitored_process

classify_exit = _COMMON.classify_exit
extract_event_token = _COMMON.extract_event_token
normalize_exit_code = _COMMON.normalize_exit_code
now_utc_iso = _COMMON.now_utc_iso
write_report = _COMMON.write_report
ERROR_MARKERS = _MODELS.ERROR_MARKERS
EVENT_TOKEN_RE = _MODELS.EVENT_TOKEN_RE
MonitorStats = _MODELS.MonitorStats
MonitorTerminationState = _MODELS.MonitorTerminationState


def main() -> int:
    args = parse_args()
    report_jsonl = Path(args.report_jsonl) if args.report_jsonl else None
    return run_monitored_process(
        command=args.command,
        log_file=Path(args.log_file),
        report_file=Path(args.report_file),
        report_jsonl=report_jsonl,
        tail_lines=args.tail_lines,
    )


if __name__ == "__main__":
    raise SystemExit(main())
