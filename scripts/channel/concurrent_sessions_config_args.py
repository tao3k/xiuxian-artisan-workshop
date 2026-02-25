#!/usr/bin/env python3
"""CLI argument parsing for concurrent Telegram session probes."""

from __future__ import annotations

import argparse
import os


def parse_args(*, webhook_url_default: str) -> argparse.Namespace:
    """Parse CLI arguments for the concurrent dual-session probe."""
    parser = argparse.ArgumentParser(
        description="Run concurrent dual-session command probe against local webhook runtime."
    )
    parser.add_argument(
        "--max-wait",
        type=int,
        default=int(os.environ.get("OMNI_BLACKBOX_MAX_WAIT_SECS", "30")),
        help="Max wait for probe completion in seconds (default: 30).",
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
        help="Session A chat id (default: $OMNI_TEST_CHAT_ID or inferred from logs).",
    )
    parser.add_argument(
        "--chat-b",
        type=int,
        default=None,
        help="Session B chat id (default: --chat-id).",
    )
    parser.add_argument(
        "--user-a",
        type=int,
        default=int(os.environ["OMNI_TEST_USER_ID"]) if "OMNI_TEST_USER_ID" in os.environ else None,
        help="First user id (default: $OMNI_TEST_USER_ID or inferred from logs).",
    )
    parser.add_argument(
        "--user-b",
        type=int,
        default=None,
        help="Second user id (default: user-a + 1; may equal user-a when --chat-b differs).",
    )
    parser.add_argument(
        "--username",
        default=os.environ.get("OMNI_TEST_USERNAME"),
        help="Username used for allowlist checks (default: env/settings/log fallback).",
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
        default=None,
        help="Optional thread id for session B.",
    )
    parser.add_argument(
        "--secret-token",
        default=os.environ.get("TELEGRAM_WEBHOOK_SECRET"),
        help="Webhook secret header value.",
    )
    parser.add_argument(
        "--prompt",
        default="/session json",
        help="Command prompt to execute concurrently for both sessions.",
    )
    parser.add_argument(
        "--forbid-log-regex",
        action="append",
        default=["tools/call: Mcp error", "Telegram sendMessage failed"],
        help="Regex that must not appear in new logs (repeatable).",
    )
    parser.add_argument(
        "--allow-send-failure",
        action="store_true",
        help=(
            "Allow Telegram send failures for synthetic cross-chat probes where "
            "the bot cannot reply to chat-b. This relaxes reply-event requirements."
        ),
    )
    parser.add_argument(
        "--session-partition",
        default=os.environ.get("OMNI_BLACKBOX_SESSION_PARTITION_MODE", ""),
        help=(
            "Session partition mode override for key matching "
            "(chat|chat_user|user|chat_thread_user)."
        ),
    )
    return parser.parse_args()
