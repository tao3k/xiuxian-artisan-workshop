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

import importlib
import sys
from pathlib import Path
from typing import Any

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

_parser_module = importlib.import_module("trace_reconstruction_parser")
_summary_module = importlib.import_module("trace_reconstruction_summary")
_render_module = importlib.import_module("trace_reconstruction_render")
_args_module = importlib.import_module("reconstruct_omni_agent_trace_args")
_runtime_module = importlib.import_module("reconstruct_omni_agent_trace_runtime")

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


def _parse_args() -> Any:
    return _args_module.parse_args(
        stage_choices=tuple(STAGE_TO_FLAG.keys()),
        default_required_stages=DEFAULT_REQUIRED_STAGES,
    )


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

    payload = _runtime_module.build_payload(
        args=args,
        entries=entries,
        summary=summary,
        errors=errors,
        required_stages=required_stages,
    )
    _runtime_module.write_outputs(
        args=args,
        payload=payload,
        entries=entries,
        summary=summary,
        render_markdown_report_fn=render_markdown_report,
    )
    _runtime_module.print_summary(
        summary=summary,
        errors=errors,
        warnings=warnings,
    )
    return _runtime_module.exit_code_from_results(
        strict=bool(args.strict),
        errors=errors,
        warnings=warnings,
    )


if __name__ == "__main__":
    raise SystemExit(main())
