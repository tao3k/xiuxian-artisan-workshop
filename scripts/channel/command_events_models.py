#!/usr/bin/env python3
"""Datamodels for command-events probe."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class ProbeCase:
    case_id: str
    prompt: str
    event_name: str
    suites: tuple[str, ...]
    extra_args: tuple[str, ...] = ()
    user_id: int | None = None
    chat_id: int | None = None
    thread_id: int | None = None


@dataclass(frozen=True)
class ProbeAttemptRecord:
    mode: str
    case_id: str
    prompt: str
    event_name: str
    suites: tuple[str, ...]
    chat_id: int | None
    user_id: int | None
    thread_id: int | None
    attempt: int
    max_attempts: int
    returncode: int
    passed: bool
    duration_ms: int
    retry_scheduled: bool
