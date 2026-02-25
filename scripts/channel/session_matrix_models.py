#!/usr/bin/env python3
"""Datamodels for session matrix runner."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


@dataclass(frozen=True)
class ProbeConfig:
    max_wait: int
    max_idle_secs: int
    webhook_url: str
    log_file: Path
    chat_id: int
    chat_b: int
    chat_c: int
    user_a: int
    user_b: int
    user_c: int
    username: str | None
    thread_a: int | None
    thread_b: int | None
    thread_c: int | None
    mixed_plain_prompt: str
    secret_token: str | None
    output_json: Path
    output_markdown: Path
    forbid_log_regexes: tuple[str, ...]
    session_partition: str | None = None


@dataclass(frozen=True)
class MatrixStep:
    name: str
    prompt: str
    chat_id: int
    event: str | None
    user_id: int
    thread_id: int | None
    expect_reply_json_fields: tuple[str, ...] = ()


@dataclass(frozen=True)
class StepResult:
    name: str
    kind: str
    session_key: str | None
    prompt: str | None
    event: str | None
    command: tuple[str, ...]
    returncode: int
    duration_ms: int
    passed: bool
    stdout_tail: str
    stderr_tail: str
