#!/usr/bin/env python3
"""Datamodels for agent channel blackbox probes."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


@dataclass(frozen=True)
class ProbeConfig:
    """Runtime configuration for one blackbox probe execution."""

    prompt: str
    max_wait_secs: int | None
    max_idle_secs: int | None
    webhook_url: str
    log_file: Path
    chat_id: int
    user_id: int
    username: str | None
    chat_title: str | None
    thread_id: int | None
    secret_token: str | None
    follow_logs: bool
    expect_events: tuple[str, ...]
    expect_reply_json_fields: tuple[tuple[str, str], ...]
    expect_log_regexes: tuple[str, ...]
    expect_bot_regexes: tuple[str, ...]
    forbid_log_regexes: tuple[str, ...]
    fail_fast_error_logs: bool
    allow_no_bot: bool
    allow_chat_ids: tuple[int, ...]
    strong_update_id: bool = False
    session_partition: str | None = None
