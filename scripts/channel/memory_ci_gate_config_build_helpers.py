#!/usr/bin/env python3
"""Shared helper functions for memory CI gate config building."""

from __future__ import annotations

from typing import Any

ARTIFACT_SPECS: tuple[tuple[str, str, str, str], ...] = (
    ("runtime_log_file", "logs", "omni-agent-webhook-ci", "log"),
    ("mock_log_file", "logs", "omni-agent-mock-telegram", "log"),
    ("evolution_report_json", "reports", "omni-agent-memory-evolution", "json"),
    ("benchmark_report_json", "reports", "omni-agent-memory-benchmark", "json"),
    ("session_matrix_report_json", "reports", "agent-channel-session-matrix", "json"),
    ("session_matrix_report_markdown", "reports", "agent-channel-session-matrix", "md"),
    ("trace_report_json", "reports", "omni-agent-trace-reconstruction", "json"),
    ("trace_report_markdown", "reports", "omni-agent-trace-reconstruction", "md"),
    ("cross_group_report_json", "reports", "agent-channel-cross-group-complex", "json"),
    ("cross_group_report_markdown", "reports", "agent-channel-cross-group-complex", "md"),
)


def resolve_artifact_relpaths(
    args: Any,
    *,
    run_suffix: str,
    default_artifact_relpath_fn: Any,
) -> dict[str, str]:
    """Resolve run-scoped artifact paths using explicit overrides when provided."""
    relpaths: dict[str, str] = {}
    for attr, category, stem, extension in ARTIFACT_SPECS:
        raw_value = str(getattr(args, attr)).strip()
        relpaths[attr] = raw_value or default_artifact_relpath_fn(
            category=category,
            stem=stem,
            profile=args.profile,
            run_suffix=run_suffix,
            extension=extension,
        )
    return relpaths


def resolve_numeric_identities(args: Any) -> dict[str, int]:
    """Resolve required/derived chat and user identities."""
    chat_id = int(args.chat_id)
    user_id = int(args.user_id)
    return {
        "chat_id": chat_id,
        "chat_b": int(args.chat_b) if args.chat_b is not None else chat_id + 1,
        "chat_c": int(args.chat_c) if args.chat_c is not None else chat_id + 2,
        "user_id": user_id,
        "user_b": int(args.user_b) if args.user_b is not None else user_id + 1,
        "user_c": int(args.user_c) if args.user_c is not None else user_id + 2,
    }
