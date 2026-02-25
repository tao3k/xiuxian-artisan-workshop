#!/usr/bin/env python3
"""Core parser sections for complex scenarios config."""

from __future__ import annotations

import argparse
import os
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


def build_parser() -> argparse.ArgumentParser:
    """Build base parser with shared description."""
    return argparse.ArgumentParser(
        description=(
            "Run complex black-box workflow scenarios for omni-agent Telegram runtime "
            "and enforce structural complexity gates."
        )
    )


def add_core_args(
    parser: argparse.ArgumentParser,
    *,
    script_dir: Path,
    webhook_url_default: str,
    default_log_file: str,
    default_max_wait: int,
    default_max_idle_secs: int,
) -> None:
    """Register dataset/runtime baseline arguments."""
    parser.add_argument(
        "--dataset",
        default=str(script_dir / "fixtures" / "complex_blackbox_scenarios.json"),
        help="Complex scenario dataset JSON path.",
    )
    parser.add_argument("--scenario", default=None, help="Optional scenario id filter.")
    parser.add_argument(
        "--blackbox-script",
        default=str(script_dir / "agent_channel_blackbox.py"),
        help="Path to one-turn black-box probe script.",
    )
    parser.add_argument(
        "--webhook-url", default=webhook_url_default, help="Telegram webhook endpoint."
    )
    parser.add_argument(
        "--log-file",
        default=default_log_file,
        help=f"Runtime log path (default: {default_log_file}).",
    )
    parser.add_argument(
        "--username",
        default=os.environ.get("OMNI_TEST_USERNAME", ""),
        help="Synthetic Telegram username for allowlist checks.",
    )
    parser.add_argument(
        "--secret-token",
        default=os.environ.get("TELEGRAM_WEBHOOK_SECRET"),
        help="Webhook secret token.",
    )
    parser.add_argument(
        "--max-wait",
        type=int,
        default=default_max_wait,
        help=f"Per-step max wait in seconds (default: {default_max_wait}).",
    )
    parser.add_argument(
        "--max-idle-secs",
        type=int,
        default=default_max_idle_secs,
        help=f"Per-step max idle seconds (default: {default_max_idle_secs}).",
    )
    parser.add_argument(
        "--max-parallel",
        type=int,
        default=4,
        help="Maximum concurrent probes per execution wave.",
    )
    parser.add_argument(
        "--execute-wave-parallel",
        action="store_true",
        help=(
            "Execute independent steps in the same wave concurrently. "
            "Default is sequential-in-wave for deterministic log attribution."
        ),
    )
