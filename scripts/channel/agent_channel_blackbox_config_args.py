#!/usr/bin/env python3
"""CLI parser for agent channel blackbox config."""

from __future__ import annotations

import argparse
import os
from typing import Any


def parse_args(
    *,
    default_telegram_webhook_url_fn: Any,
    target_session_scope_placeholder: str,
) -> argparse.Namespace:
    """Parse blackbox probe CLI arguments."""
    webhook_url_default = os.environ.get("OMNI_WEBHOOK_URL") or default_telegram_webhook_url_fn()
    parser = argparse.ArgumentParser(
        description="Inject one synthetic Telegram webhook update and wait for bot reply logs."
    )
    parser.add_argument("--prompt", required=True, help="Prompt to inject.")
    parser.add_argument(
        "--max-wait",
        type=int,
        default=None,
        help="Optional overall wait upper-bound in seconds. Default: no hard limit (event-driven).",
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=None,
        help="Deprecated alias for --max-wait.",
    )
    parser.add_argument(
        "--max-idle-secs",
        type=int,
        default=None,
        help="Optional max idle (no new logs) before fail-fast.",
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
        default=None,
        help="Synthetic Telegram chat id (auto-infer from logs when omitted).",
    )
    parser.add_argument(
        "--user-id",
        type=int,
        default=None,
        help="Synthetic Telegram user id (auto-infer from logs when omitted).",
    )
    parser.add_argument(
        "--username",
        default=os.environ.get("OMNI_TEST_USERNAME"),
        help="Synthetic Telegram username for allowlist checks (e.g. `tao3k`).",
    )
    parser.add_argument(
        "--chat-title",
        default=os.environ.get("OMNI_TEST_CHAT_TITLE"),
        help=(
            "Optional synthetic Telegram chat title to include in payload "
            "(useful for chat_id/chat_title log mapping checks)."
        ),
    )
    parser.add_argument(
        "--thread-id",
        type=int,
        default=None,
        help="Synthetic Telegram thread/topic id.",
    )
    parser.add_argument(
        "--session-partition",
        default=os.environ.get("OMNI_TEST_SESSION_PARTITION"),
        help=(
            "Optional session partition mode hint "
            "(`chat`, `chat_user`, `user`, `chat_thread_user`) "
            "used for strict session-key validation."
        ),
    )
    parser.add_argument(
        "--secret-token",
        default=os.environ.get("TELEGRAM_WEBHOOK_SECRET"),
        help="Webhook secret token header value.",
    )
    parser.add_argument(
        "--no-follow",
        action="store_true",
        help="Disable live log streaming while waiting.",
    )
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
    return parser.parse_args()
