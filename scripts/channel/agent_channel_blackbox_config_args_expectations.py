#!/usr/bin/env python3
"""Expectation/filter CLI argument group for blackbox config."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import argparse


def add_expectation_args(
    parser: argparse.ArgumentParser,
    *,
    target_session_scope_placeholder: str,
) -> None:
    """Register expectation/assertion/filter arguments."""
    parser.add_argument(
        "--expect-log-regex",
        action="append",
        default=[],
        help="Regex expected somewhere in new logs (repeatable).",
    )
    parser.add_argument(
        "--expect-event",
        action="append",
        default=[],
        help="Structured `event=` token expected in new logs (repeatable, exact match).",
    )
    parser.add_argument(
        "--expect-reply-json-field",
        action="append",
        default=[],
        help=(
            "Expected key=value from `command reply json summary` logs "
            "(repeatable). Example: --expect-reply-json-field json_kind=session_budget. "
            f"For session scope checks, use "
            f"--expect-reply-json-field json_session_scope={target_session_scope_placeholder}."
        ),
    )
    parser.add_argument(
        "--expect-bot-regex",
        action="append",
        default=[],
        help="Regex expected in `→ Bot:` log line (repeatable).",
    )
    parser.add_argument(
        "--forbid-log-regex",
        action="append",
        default=[],
        help="Regex that must not appear in new logs (repeatable).",
    )
    parser.add_argument(
        "--no-fail-fast-error-log",
        action="store_true",
        help="Do not fail immediately when known error patterns appear.",
    )
    parser.add_argument(
        "--allow-no-bot",
        action="store_true",
        help="Allow success without `→ Bot:` if all expect-log checks are satisfied.",
    )
    parser.add_argument(
        "--allow-chat-id",
        action="append",
        default=[],
        help=(
            "Allowlisted chat id for this probe (repeatable). "
            "When set, probe refuses to post outside this allowlist."
        ),
    )
