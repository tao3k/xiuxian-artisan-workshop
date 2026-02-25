#!/usr/bin/env python3
"""
One-click session trace reconstruction for omni-agent runtime logs.

Focus:
- route decision/fallback
- injection snapshot
- reflection lifecycle/hints
- memory recall/gate lifecycle
- optional suggested_link evidence
"""

from __future__ import annotations

import argparse
import importlib
import json
import sys
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

_parser_module = importlib.import_module("trace_reconstruction_parser")
_summary_module = importlib.import_module("trace_reconstruction_summary")
_render_module = importlib.import_module("trace_reconstruction_render")

DEFAULT_EVENT_PREFIXES = _parser_module.DEFAULT_EVENT_PREFIXES
DEFAULT_REQUIRED_STAGES = _summary_module.DEFAULT_REQUIRED_STAGES
STAGE_TO_FLAG = _summary_module.STAGE_TO_FLAG
STAGE_ERROR_MESSAGE = _summary_module.STAGE_ERROR_MESSAGE

strip_ansi = _parser_module.strip_ansi
_extract_timestamp = _parser_module.extract_timestamp
_extract_level = _parser_module.extract_level
_extract_event = _parser_module.extract_event
_extract_fields = _parser_module.extract_fields
_line_matches_session = _parser_module.line_matches_session
_line_matches_chat = _parser_module.line_matches_chat
_event_is_tracked = _parser_module.event_is_tracked

load_trace_entries = _parser_module.load_trace_entries
_first_index = _summary_module.first_index
_collect_injection_modes = _summary_module.collect_injection_modes
build_trace_summary = _summary_module.build_trace_summary
evaluate_trace_health = _summary_module.evaluate_trace_health
render_markdown_report = _render_module.render_markdown_report


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Reconstruct omni-agent runtime trace from log file"
    )
    parser.add_argument("log_file", type=Path, help="Runtime log file path")
    parser.add_argument("--session-id", default="", help="Optional session_id/session_key filter")
    parser.add_argument("--chat-id", type=int, default=None, help="Optional chat_id filter")
    parser.add_argument("--max-events", type=int, default=500, help="Maximum events to include")
    parser.add_argument(
        "--required-stage",
        action="append",
        choices=tuple(STAGE_TO_FLAG.keys()),
        default=[],
        help=(
            "Required lifecycle stage for health evaluation (repeatable). "
            f"Default: {','.join(DEFAULT_REQUIRED_STAGES)}"
        ),
    )
    parser.add_argument(
        "--require-suggested-link",
        action="store_true",
        help="Fail if no suggested_link record appears in filtered trace",
    )
    parser.add_argument(
        "--strict", action="store_true", help="Fail when chain warnings/errors exist"
    )
    parser.add_argument("--json-out", type=Path, default=None, help="Optional JSON output path")
    parser.add_argument(
        "--markdown-out", type=Path, default=None, help="Optional markdown output path"
    )
    return parser.parse_args()


def main() -> int:
    args = _parse_args()
    try:
        entries = load_trace_entries(
            args.log_file,
            session_id=args.session_id.strip() or None,
            chat_id=args.chat_id,
            max_events=int(args.max_events),
        )
    except FileNotFoundError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1

    summary = build_trace_summary(entries)
    required_stages = tuple(args.required_stage) if args.required_stage else DEFAULT_REQUIRED_STAGES
    errors = evaluate_trace_health(
        summary,
        require_suggested_link=bool(args.require_suggested_link),
        required_stages=required_stages,
    )
    warnings = list(summary.get("warnings", []))

    payload = {
        "log_file": str(args.log_file),
        "filters": {
            "session_id": args.session_id.strip() or None,
            "chat_id": args.chat_id,
            "max_events": int(args.max_events),
            "required_stages": list(required_stages),
        },
        "summary": summary,
        "errors": errors,
        "entries": entries,
    }

    if args.json_out is not None:
        args.json_out.parent.mkdir(parents=True, exist_ok=True)
        args.json_out.write_text(
            json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
    if args.markdown_out is not None:
        args.markdown_out.parent.mkdir(parents=True, exist_ok=True)
        args.markdown_out.write_text(render_markdown_report(entries, summary), encoding="utf-8")

    print(f"trace events: {summary['events_total']}")
    print(f"quality score: {summary['quality_score']}")
    print(f"errors: {len(errors)}")
    print(f"warnings: {len(warnings)}")

    if errors:
        for error in errors:
            print(f"[ERROR] {error}")
    if warnings:
        for warning in warnings:
            print(f"[WARN] {warning}")

    if args.strict and (errors or warnings):
        return 1
    if errors:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
