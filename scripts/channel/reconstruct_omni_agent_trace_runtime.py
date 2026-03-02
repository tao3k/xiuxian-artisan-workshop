#!/usr/bin/env python3
"""Runtime payload/output helpers for trace reconstruction script."""

from __future__ import annotations

import json
from typing import Any


def build_payload(
    *,
    args: Any,
    entries: list[dict[str, object]],
    summary: dict[str, object],
    errors: list[str],
    required_stages: tuple[str, ...],
) -> dict[str, object]:
    """Build serialized report payload."""
    return {
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


def write_outputs(
    *,
    args: Any,
    payload: dict[str, object],
    entries: list[dict[str, object]],
    summary: dict[str, object],
    render_markdown_report_fn: Any,
) -> None:
    """Write optional JSON/markdown artifacts."""
    if args.json_out is not None:
        args.json_out.parent.mkdir(parents=True, exist_ok=True)
        args.json_out.write_text(
            json.dumps(payload, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
    if args.markdown_out is not None:
        args.markdown_out.parent.mkdir(parents=True, exist_ok=True)
        args.markdown_out.write_text(
            render_markdown_report_fn(entries, summary),
            encoding="utf-8",
        )


def print_summary(
    *,
    summary: dict[str, object],
    errors: list[str],
    warnings: list[str],
) -> None:
    """Print concise runtime reconstruction summary."""
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


def exit_code_from_results(*, strict: bool, errors: list[str], warnings: list[str]) -> int:
    """Return process exit code from health evaluation results."""
    if strict and (errors or warnings):
        return 1
    if errors:
        return 1
    return 0
