#!/usr/bin/env python3
"""Datamodels for concurrent Telegram session probes."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


@dataclass(frozen=True)
class ProbeConfig:
    """Configuration for the dual-session concurrent probe."""

    max_wait: int
    webhook_url: str
    log_file: Path
    chat_id: int
    chat_b: int
    user_a: int
    user_b: int
    username: str | None
    thread_a: int | None
    thread_b: int | None
    secret_token: str | None
    prompt: str
    forbid_log_regexes: tuple[str, ...]
    allow_send_failure: bool
    session_partition: str | None = None


@dataclass(frozen=True)
class Observation:
    """Observed runtime event counters for two concurrent updates."""

    accepted_a: int
    accepted_b: int
    dedup_fail_open_a: int
    dedup_fail_open_b: int
    duplicate_a: int
    duplicate_b: int
    parsed_a: int
    parsed_b: int
    replied_a: int
    replied_b: int
    forbidden_hits: tuple[str, ...]
