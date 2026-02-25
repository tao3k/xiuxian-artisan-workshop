#!/usr/bin/env python3
"""CLI argument parsing helpers for memory benchmark runner."""

from __future__ import annotations

import argparse
import os
from pathlib import Path


def default_report_path(filename: str) -> Path:
    """Build a report path under PRJ runtime report directory."""
    runtime_root = Path(os.environ.get("PRJ_RUNTIME_DIR", ".run"))
    if not runtime_root.is_absolute():
        project_root = Path(os.environ.get("PRJ_ROOT", Path.cwd()))
        runtime_root = project_root / runtime_root
    return runtime_root / "reports" / filename


def parse_args(
    *,
    script_dir: Path,
    default_log_file: str,
    default_max_wait: int,
    default_max_idle_secs: int,
) -> argparse.Namespace:
    """Parse command-line arguments for memory benchmark script."""
    parser = argparse.ArgumentParser(
        description=(
            "Run live A/B memory benchmark against local omni-agent webhook runtime "
            "(baseline vs adaptive feedback mode)."
        )
    )
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
    parser.add_argument(
        "--max-wait",
        type=int,
        default=default_max_wait,
        help=f"Per probe max wait in seconds (default: {default_max_wait}).",
    )
    parser.add_argument(
        "--max-idle-secs",
        type=int,
        default=default_max_idle_secs,
        help=f"Per probe max idle seconds (default: {default_max_idle_secs}).",
    )
    parser.add_argument(
        "--mode",
        action="append",
        choices=("baseline", "adaptive"),
        help="Benchmark mode (repeatable). Default: baseline+adaptive.",
    )
    parser.add_argument(
        "--iterations",
        type=int,
        default=1,
        help="Dataset replay count per mode (default: 1).",
    )
    parser.add_argument(
        "--skip-reset",
        action="store_true",
        help="Do not issue /reset between scenarios.",
    )
    parser.add_argument(
        "--output-json",
        default=str(default_report_path("omni-agent-memory-benchmark.json")),
        help="Output JSON report path.",
    )
    parser.add_argument(
        "--output-markdown",
        default=str(default_report_path("omni-agent-memory-benchmark.md")),
        help="Output Markdown report path.",
    )
    parser.add_argument(
        "--fail-on-mcp-error",
        action="store_true",
        help=(
            "Fail immediately when logs include `tools/call: Mcp error`. "
            "Default behavior records this as interference."
        ),
    )
    parser.add_argument(
        "--feedback-policy",
        choices=("strict", "deadband"),
        default="deadband",
        help=(
            "Adaptive feedback policy. "
            "`strict`: success=up, failure=down. "
            "`deadband`: success=up, strong failure only=down, else neutral."
        ),
    )
    parser.add_argument(
        "--feedback-down-threshold",
        type=float,
        default=0.34,
        help=(
            "When feedback-policy=deadband, send `down` only when keyword hit ratio "
            "is less than or equal to this threshold (default: 0.34)."
        ),
    )
    return parser.parse_args()
