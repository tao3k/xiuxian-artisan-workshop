#!/usr/bin/env python3
"""Runtime/identity argument sections for memory CI gate parser."""

from __future__ import annotations

import os
from typing import Any


def add_runtime_args(parser: Any) -> None:
    """Register runtime and execution control arguments."""
    parser.add_argument(
        "--profile",
        choices=("quick", "nightly"),
        default="quick",
        help="Gate profile (default: quick).",
    )
    parser.add_argument(
        "--agent-bin",
        default="",
        help=(
            "Optional path to prebuilt omni-agent binary. "
            "When set, startup uses '<agent-bin> channel ...' instead of "
            "'cargo run -p omni-agent -- channel ...'."
        ),
    )
    parser.add_argument("--webhook-port", type=int, default=18081)
    parser.add_argument("--telegram-api-port", type=int, default=18080)
    parser.add_argument("--valkey-port", type=int, default=6379)
    parser.add_argument(
        "--valkey-url",
        default="",
        help="Optional explicit Valkey URL (default: redis://<resolved-host>:<valkey-port>/<db>).",
    )
    parser.add_argument(
        "--valkey-prefix",
        default="",
        help=(
            "Optional explicit Valkey key prefix for CI session/memory isolation. "
            "Default: an auto-generated per-run prefix."
        ),
    )
    parser.add_argument("--username", default=os.environ.get("OMNI_TEST_USERNAME", "ci-user"))
    parser.add_argument(
        "--webhook-secret",
        default=os.environ.get("TELEGRAM_WEBHOOK_SECRET", ""),
        help=(
            "Telegram webhook secret token. "
            "Defaults to $TELEGRAM_WEBHOOK_SECRET, otherwise an ephemeral token is generated."
        ),
    )
    parser.add_argument("--chat-id", type=int, default=1304799691)
    parser.add_argument(
        "--chat-b",
        type=int,
        default=int(os.environ["OMNI_TEST_CHAT_B"]) if "OMNI_TEST_CHAT_B" in os.environ else None,
    )
    parser.add_argument(
        "--chat-c",
        type=int,
        default=int(os.environ["OMNI_TEST_CHAT_C"]) if "OMNI_TEST_CHAT_C" in os.environ else None,
    )
    parser.add_argument("--user-id", type=int, default=1304799692)
    parser.add_argument(
        "--user-b",
        type=int,
        default=int(os.environ["OMNI_TEST_USER_B"]) if "OMNI_TEST_USER_B" in os.environ else None,
    )
    parser.add_argument(
        "--user-c",
        type=int,
        default=int(os.environ["OMNI_TEST_USER_C"]) if "OMNI_TEST_USER_C" in os.environ else None,
    )
    parser.add_argument("--runtime-startup-timeout-secs", type=int, default=90)
    parser.add_argument("--quick-max-wait", type=int, default=45)
    parser.add_argument("--quick-max-idle", type=int, default=25)
    parser.add_argument("--full-max-wait", type=int, default=90)
    parser.add_argument("--full-max-idle", type=int, default=40)
    parser.add_argument("--matrix-max-wait", type=int, default=45)
    parser.add_argument("--matrix-max-idle", type=int, default=30)
    parser.add_argument("--benchmark-iterations", type=int, default=3)
    parser.add_argument("--skip-matrix", action="store_true")
    parser.add_argument("--skip-benchmark", action="store_true")
    parser.add_argument("--skip-evolution", action="store_true")
    parser.add_argument("--skip-rust-regressions", action="store_true")
    parser.add_argument("--skip-discover-cache-gate", action="store_true")
    parser.add_argument("--skip-reflection-quality-gate", action="store_true")
    parser.add_argument("--skip-trace-reconstruction-gate", action="store_true")
    parser.add_argument("--skip-cross-group-complex-gate", action="store_true")
