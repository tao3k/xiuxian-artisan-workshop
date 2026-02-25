#!/usr/bin/env python3
"""CLI argument parser for Discord ACL probes."""

from __future__ import annotations

import argparse
import os


def parse_args(*, suites: tuple[str, ...], default_ingress_url_value: str) -> argparse.Namespace:
    """Parse CLI arguments for Discord ACL probe runner."""
    parser = argparse.ArgumentParser(
        description=(
            "Run Discord ACL black-box probes against local ingress runtime. "
            "Each probe requires a command-specific reply event."
        )
    )
    parser.add_argument(
        "--ingress-url", default=default_ingress_url_value, help="Discord ingress URL."
    )
    parser.add_argument(
        "--log-file",
        default=os.environ.get("OMNI_CHANNEL_LOG_FILE", ".run/logs/omni-agent-webhook.log"),
        help="Runtime log file path.",
    )
    parser.add_argument(
        "--max-wait",
        type=int,
        default=int(os.environ.get("OMNI_BLACKBOX_MAX_WAIT_SECS", "25")),
        help="Overall wait upper-bound per probe in seconds.",
    )
    parser.add_argument(
        "--max-idle-secs",
        type=int,
        default=int(os.environ.get("OMNI_BLACKBOX_MAX_IDLE_SECS", "25")),
        help="Max idle wait for new logs per probe in seconds.",
    )
    parser.add_argument(
        "--channel-id",
        default=os.environ.get("OMNI_TEST_DISCORD_CHANNEL_ID", "").strip(),
        help="Discord channel_id used for synthetic ingress event.",
    )
    parser.add_argument(
        "--user-id",
        default=os.environ.get("OMNI_TEST_DISCORD_USER_ID", "").strip(),
        help="Discord user_id used for synthetic ingress event.",
    )
    parser.add_argument(
        "--guild-id",
        default=os.environ.get("OMNI_TEST_DISCORD_GUILD_ID", "").strip() or None,
        help="Discord guild_id (optional, defaults to DM scope).",
    )
    parser.add_argument(
        "--username",
        default=os.environ.get("OMNI_TEST_DISCORD_USERNAME", "").strip() or None,
        help="Discord username (optional).",
    )
    parser.add_argument(
        "--role-id",
        action="append",
        default=[],
        help="Discord role id attached to synthetic member.roles (repeatable).",
    )
    parser.add_argument(
        "--secret-token",
        default=os.environ.get("OMNI_TEST_DISCORD_INGRESS_SECRET", "").strip() or None,
        help="Ingress secret token for header x-omni-discord-ingress-token.",
    )
    parser.add_argument(
        "--session-partition",
        default=os.environ.get("OMNI_AGENT_DISCORD_SESSION_PARTITION", "guild_channel_user"),
        help="Discord session partition mode: guild_channel_user|channel|user|guild_user.",
    )
    parser.add_argument(
        "--suite",
        action="append",
        choices=suites,
        default=[],
        help="Run selected suite(s): core, all. Repeatable. Default: all.",
    )
    parser.add_argument(
        "--case",
        action="append",
        default=[],
        help="Run only specific case id(s). Repeatable. Use --list-cases to inspect ids.",
    )
    parser.add_argument(
        "--list-cases", action="store_true", help="List available case ids and exit."
    )
    parser.add_argument(
        "--no-follow",
        action="store_true",
        help="Disable live log streaming while waiting.",
    )
    parser.add_argument(
        "--allow-no-bot",
        action="store_true",
        default=True,
        help="Reserved flag for compatibility with Telegram probe semantics.",
    )
    return parser.parse_args()
