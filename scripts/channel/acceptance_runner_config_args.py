#!/usr/bin/env python3
"""CLI argument parsing helpers for acceptance runner config."""

from __future__ import annotations

import argparse
import os


def parse_args(
    *,
    default_webhook_log: str,
    default_report_json: str,
    default_report_markdown: str,
    default_group_profile_json: str,
    default_group_profile_env: str,
) -> argparse.Namespace:
    """Parse CLI args for end-to-end acceptance runner."""
    parser = argparse.ArgumentParser(
        description="Run end-to-end Telegram channel black-box acceptance pipeline."
    )
    parser.add_argument(
        "--titles",
        default="Test1,Test2,Test3",
        help="Group titles for capture step (default: Test1,Test2,Test3).",
    )
    parser.add_argument(
        "--log-file",
        default=default_webhook_log,
        help=f"Webhook runtime log file path (default: {default_webhook_log}).",
    )
    parser.add_argument(
        "--output-json",
        default=default_report_json,
        help=f"Acceptance summary JSON path (default: {default_report_json}).",
    )
    parser.add_argument(
        "--output-markdown",
        default=default_report_markdown,
        help=f"Acceptance summary markdown path (default: {default_report_markdown}).",
    )
    parser.add_argument(
        "--group-profile-json",
        default=default_group_profile_json,
        help=f"Captured group profile JSON path (default: {default_group_profile_json}).",
    )
    parser.add_argument(
        "--group-profile-env",
        default=default_group_profile_env,
        help=f"Captured group profile env path (default: {default_group_profile_env}).",
    )
    parser.add_argument(
        "--max-wait",
        type=int,
        default=int(os.environ.get("OMNI_BLACKBOX_MAX_WAIT_SECS", "40")),
        help="Max wait seconds for standard black-box steps (default: 40).",
    )
    parser.add_argument(
        "--max-idle-secs",
        type=int,
        default=int(os.environ.get("OMNI_BLACKBOX_MAX_IDLE_SECS", "25")),
        help="Max idle seconds for standard black-box steps (default: 25).",
    )
    parser.add_argument(
        "--group-thread-id",
        type=int,
        default=None,
        help=(
            "Optional Telegram topic thread id for thread-aware acceptance checks. "
            "Falls back to $OMNI_TEST_GROUP_THREAD_ID."
        ),
    )
    parser.add_argument(
        "--group-thread-id-b",
        type=int,
        default=None,
        help=(
            "Optional secondary topic thread id for cross-topic checks. "
            "Falls back to $OMNI_TEST_GROUP_THREAD_B; defaults to thread A + 1."
        ),
    )
    parser.add_argument(
        "--evolution-max-wait",
        type=int,
        default=90,
        help="Max wait seconds for memory evolution scenario (default: 90).",
    )
    parser.add_argument(
        "--evolution-max-idle-secs",
        type=int,
        default=60,
        help="Max idle seconds for memory evolution scenario (default: 60).",
    )
    parser.add_argument(
        "--evolution-max-parallel",
        type=int,
        default=4,
        help="Max parallel wave probes for memory evolution scenario (default: 4).",
    )
    parser.add_argument(
        "--retries",
        type=int,
        default=2,
        help="Retry attempts per step on failure (default: 2).",
    )
    return parser.parse_args()
