#!/usr/bin/env python3
"""CLI argument parser for session matrix runner."""

from __future__ import annotations

import argparse
import os


def parse_args(*, webhook_url_default: str) -> argparse.Namespace:
    """Parse session matrix CLI arguments."""
    parser = argparse.ArgumentParser(
        description=(
            "Run session isolation matrix against local Telegram webhook runtime "
            "and emit structured JSON/Markdown reports."
        )
    )
    parser.add_argument(
        "--max-wait",
        type=int,
        default=int(os.environ.get("OMNI_BLACKBOX_MAX_WAIT_SECS", "35")),
        help="Max wait per probe in seconds (default: 35).",
    )
    parser.add_argument(
        "--max-idle-secs",
        type=int,
        default=int(os.environ.get("OMNI_BLACKBOX_MAX_IDLE_SECS", "25")),
        help="Max idle seconds per probe (default: 25).",
    )
    parser.add_argument(
        "--webhook-url",
        default=webhook_url_default,
        help="Webhook URL.",
    )
    parser.add_argument(
        "--log-file",
        default=os.environ.get("OMNI_CHANNEL_LOG_FILE", ".run/logs/omni-agent-webhook.log"),
        help="Runtime log file path.",
    )
    parser.add_argument(
        "--chat-id",
        type=int,
        default=int(os.environ["OMNI_TEST_CHAT_ID"]) if "OMNI_TEST_CHAT_ID" in os.environ else None,
        help="Telegram chat id (default: inferred from env/log).",
    )
    parser.add_argument(
        "--chat-b",
        type=int,
        default=int(os.environ["OMNI_TEST_CHAT_B"]) if "OMNI_TEST_CHAT_B" in os.environ else None,
        help="Session B chat id (default: --chat-id or $OMNI_TEST_CHAT_B).",
    )
    parser.add_argument(
        "--chat-c",
        type=int,
        default=int(os.environ["OMNI_TEST_CHAT_C"]) if "OMNI_TEST_CHAT_C" in os.environ else None,
        help="Session C chat id for mixed concurrency probe (default: --chat-id or $OMNI_TEST_CHAT_C).",
    )
    parser.add_argument(
        "--user-a",
        type=int,
        default=int(os.environ["OMNI_TEST_USER_ID"]) if "OMNI_TEST_USER_ID" in os.environ else None,
        help="Session A user id (default: inferred from env/log).",
    )
    parser.add_argument(
        "--user-b",
        type=int,
        default=int(os.environ["OMNI_TEST_USER_B"]) if "OMNI_TEST_USER_B" in os.environ else None,
        help="Session B user id (default: user-a + 1 or $OMNI_TEST_USER_B).",
    )
    parser.add_argument(
        "--user-c",
        type=int,
        default=int(os.environ["OMNI_TEST_USER_C"]) if "OMNI_TEST_USER_C" in os.environ else None,
        help="Session C user id for mixed concurrency probe (default: user-a + 2 or $OMNI_TEST_USER_C).",
    )
    parser.add_argument(
        "--username",
        default=os.environ.get("OMNI_TEST_USERNAME"),
        help="Telegram username for allowlist checks.",
    )
    parser.add_argument(
        "--thread-a",
        type=int,
        default=int(os.environ["OMNI_TEST_THREAD_ID"])
        if "OMNI_TEST_THREAD_ID" in os.environ
        else None,
        help="Optional thread id for session A.",
    )
    parser.add_argument(
        "--thread-b",
        type=int,
        default=int(os.environ["OMNI_TEST_THREAD_B"])
        if "OMNI_TEST_THREAD_B" in os.environ
        else None,
        help=(
            "Optional thread id for session B "
            "(default: thread-a + 1 when thread-a is provided, or $OMNI_TEST_THREAD_B)."
        ),
    )
    parser.add_argument(
        "--thread-c",
        type=int,
        default=int(os.environ["OMNI_TEST_THREAD_C"])
        if "OMNI_TEST_THREAD_C" in os.environ
        else None,
        help="Optional thread id for session C mixed probe (default: $OMNI_TEST_THREAD_C).",
    )
    parser.add_argument(
        "--mixed-plain-prompt",
        default="Please reply with one short sentence for mixed concurrency probe.",
        help="Plain prompt used in mixed concurrency batch.",
    )
    parser.add_argument(
        "--secret-token",
        default=os.environ.get("TELEGRAM_WEBHOOK_SECRET"),
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
    return parser.parse_args()
