#!/usr/bin/env python3
"""CLI argument parsing helpers for dedup probe config."""

from __future__ import annotations

import argparse
import os


def parse_args(*, webhook_url_default: str) -> argparse.Namespace:
    """Parse CLI args for deterministic dedup probe."""
    parser = argparse.ArgumentParser(
        description=(
            "Post the same Telegram update_id twice to local webhook runtime and assert "
            "accepted/duplicate dedup events."
        )
    )
    parser.add_argument(
        "--max-wait",
        type=int,
        default=int(os.environ.get("OMNI_BLACKBOX_MAX_WAIT_SECS", "25")),
        help="Max wait for dedup logs in seconds (default: 25).",
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
        help="Telegram chat id (default: $OMNI_TEST_CHAT_ID).",
    )
    parser.add_argument(
        "--user-id",
        type=int,
        default=int(os.environ["OMNI_TEST_USER_ID"]) if "OMNI_TEST_USER_ID" in os.environ else None,
        help="Telegram user id (default: $OMNI_TEST_USER_ID).",
    )
    parser.add_argument(
        "--username",
        default=os.environ.get("OMNI_TEST_USERNAME"),
        help="Telegram username (default: $OMNI_TEST_USERNAME).",
    )
    parser.add_argument(
        "--thread-id",
        type=int,
        default=int(os.environ["OMNI_TEST_THREAD_ID"])
        if "OMNI_TEST_THREAD_ID" in os.environ
        else None,
        help="Optional Telegram thread/topic id.",
    )
    parser.add_argument(
        "--secret-token",
        default=os.environ.get("TELEGRAM_WEBHOOK_SECRET"),
        help="Webhook secret header value.",
    )
    parser.add_argument(
        "--text",
        default="/session json",
        help="Message text payload (default: /session json).",
    )
    return parser.parse_args()
