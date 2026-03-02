#!/usr/bin/env python3
"""Probe/output argument groups for session matrix parser."""

from __future__ import annotations

from typing import Any

import session_matrix_config_args_env as _env


def add_probe_and_output_args(parser: Any) -> None:
    """Add probe prompt and output/report settings."""
    parser.add_argument(
        "--mixed-plain-prompt",
        default="Please reply with one short sentence for mixed concurrency probe.",
        help="Plain prompt used in mixed concurrency batch.",
    )
    parser.add_argument(
        "--secret-token",
        default=_env.default_secret_token(),
        help="Webhook secret header value.",
    )
    parser.add_argument(
        "--output-json",
        default=".run/reports/agent-channel-session-matrix.json",
        help="Structured output JSON path.",
    )
    parser.add_argument(
        "--output-markdown",
        default=".run/reports/agent-channel-session-matrix.md",
        help="Structured output Markdown path.",
    )
    parser.add_argument(
        "--forbid-log-regex",
        action="append",
        default=["tools/call: Mcp error", "Telegram sendMessage failed"],
        help="Regex that must not appear in probe logs (repeatable).",
    )
