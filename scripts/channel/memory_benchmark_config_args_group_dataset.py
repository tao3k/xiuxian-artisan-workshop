#!/usr/bin/env python3
"""Dataset/path and identity argument groups for memory benchmark parser."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def add_dataset_and_paths_args(
    parser: Any,
    *,
    script_dir: Path,
    default_log_file: str,
) -> None:
    """Add dataset and script path arguments."""
    parser.add_argument(
        "--dataset",
        default=str(script_dir / "fixtures" / "memory_benchmark_scenarios.json"),
        help="Scenario dataset JSON path.",
    )
    parser.add_argument(
        "--log-file",
        default=default_log_file,
        help=f"Runtime log file path (default: {default_log_file}).",
    )
    parser.add_argument(
        "--blackbox-script",
        default=str(script_dir / "agent_channel_blackbox.py"),
        help="Path to black-box probe script.",
    )


def add_identity_args(parser: Any) -> None:
    """Add synthetic identity arguments."""
    parser.add_argument(
        "--username",
        default=os.environ.get("OMNI_TEST_USERNAME", ""),
        help="Synthetic Telegram username for allowlist checks.",
    )
    parser.add_argument(
        "--chat-id",
        type=int,
        default=None,
        help="Pinned synthetic Telegram chat id. Default: infer once from runtime log.",
    )
    parser.add_argument(
        "--user-id",
        type=int,
        default=None,
        help="Pinned synthetic Telegram user id. Default: infer once from runtime log.",
    )
    parser.add_argument(
        "--thread-id",
        type=int,
        default=None,
        help="Pinned synthetic Telegram thread id (optional).",
    )
