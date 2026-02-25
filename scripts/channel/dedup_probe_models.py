#!/usr/bin/env python3
"""Datamodels for deterministic webhook dedup probe."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


@dataclass(frozen=True)
class ProbeConfig:
    """Runtime config for one dedup probe execution."""

    max_wait: int
    webhook_url: str
    log_file: Path
    chat_id: int
    user_id: int
    username: str | None
    thread_id: int | None
    secret_token: str | None
    text: str
