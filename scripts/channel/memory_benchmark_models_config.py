#!/usr/bin/env python3
"""Runtime configuration datamodel for memory benchmark runner."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


@dataclass
class BenchmarkConfig:
    """Runtime configuration for memory benchmark execution."""

    dataset_path: Path
    log_file: Path
    blackbox_script: Path
    chat_id: int
    user_id: int
    thread_id: int | None
    runtime_partition_mode: str | None
    username: str
    max_wait: int
    max_idle_secs: int
    modes: tuple[str, ...]
    iterations: int
    skip_reset: bool
    output_json: Path
    output_markdown: Path
    fail_on_mcp_error: bool
    feedback_policy: str
    feedback_down_threshold: float
