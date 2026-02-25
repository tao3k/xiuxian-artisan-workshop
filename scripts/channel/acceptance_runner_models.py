#!/usr/bin/env python3
"""Datamodels for channel acceptance runner."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


@dataclass(frozen=True)
class StepResult:
    """Execution result for one acceptance pipeline step."""

    step: str
    title: str
    command: tuple[str, ...]
    returncode: int
    duration_ms: int
    attempts: int
    passed: bool
    expected_outputs: tuple[str, ...]
    missing_outputs: tuple[str, ...]
    stdout_tail: str
    stderr_tail: str


@dataclass(frozen=True)
class AcceptanceConfig:
    """Config for unified Telegram channel acceptance pipeline."""

    titles: str
    log_file: Path
    output_json: Path
    output_markdown: Path
    group_profile_json: Path
    group_profile_env: Path
    max_wait: int
    max_idle_secs: int
    group_thread_id: int | None
    group_thread_id_b: int | None
    evolution_max_wait: int
    evolution_max_idle_secs: int
    evolution_max_parallel: int
    retries: int
