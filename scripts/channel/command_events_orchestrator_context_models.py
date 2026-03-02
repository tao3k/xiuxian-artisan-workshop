#!/usr/bin/env python3
"""Datamodels for command event probe orchestration context."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from datetime import datetime
    from pathlib import Path


@dataclass(frozen=True)
class OrchestratorContext:
    """Resolved inputs and runtime state required by orchestration flow."""

    output_json: Path
    output_markdown: Path
    started_dt: datetime
    started_mono: float
    attempts: list[Any]
    suites: tuple[str, ...]
    secret_token: str
    username: str
    admin_user_id: int | None
    allow_chat_ids: tuple[str, ...]
    group_chat_id: int
    group_thread_id: int | None
    topic_thread_pair: tuple[int, int] | None
    runtime_partition_mode: str | None
    blackbox_script: Path
