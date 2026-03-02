#!/usr/bin/env python3
"""Runtime and suite-mode CLI args for memory suite."""

from __future__ import annotations

import os
from typing import Any


def add_runtime_args(
    parser: Any,
    *,
    default_max_wait: int,
    default_max_idle_secs: int,
) -> None:
    """Register base suite/runtime arguments."""
    parser.add_argument(
        "--suite",
        choices=("quick", "full"),
        default="quick",
        help="Suite mode: quick (black-box only) or full (black-box + cargo regressions).",
    )
    parser.add_argument(
        "--max-wait",
        type=int,
        default=default_max_wait,
        help=f"Per black-box probe max wait in seconds (default: {default_max_wait}).",
    )
    parser.add_argument(
        "--max-idle-secs",
        type=int,
        default=default_max_idle_secs,
        help=f"Per black-box probe max idle seconds (default: {default_max_idle_secs}).",
    )
    parser.add_argument(
        "--username",
        default=os.environ.get("OMNI_TEST_USERNAME", ""),
        help="Synthetic Telegram username for allowlist checks.",
    )
    parser.add_argument(
        "--require-live-turn",
        action="store_true",
        help=(
            "Also probe a normal non-command turn and require memory recall observability "
            "events in logs."
        ),
    )
    parser.add_argument(
        "--skip-blackbox",
        action="store_true",
        help="Skip webhook black-box checks (useful when local webhook runtime is not running).",
    )
    parser.add_argument(
        "--skip-rust",
        action="store_true",
        help="Skip Rust regression checks.",
    )
    parser.add_argument(
        "--skip-evolution",
        action="store_true",
        help=(
            "Skip memory self-evolution DAG black-box scenario in full suite. "
            "By default, full suite includes this scenario."
        ),
    )
