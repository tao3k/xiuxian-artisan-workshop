#!/usr/bin/env python3
"""Datamodels for Discord ACL black-box probes."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


@dataclass(frozen=True)
class ProbeCase:
    """Definition of a single Discord ACL probe case."""

    case_id: str
    prompt: str
    event_name: str
    suites: tuple[str, ...]
    expect_reply_json_fields: tuple[str, ...] = ()


@dataclass(frozen=True)
class ProbeConfig:
    """Runtime config for Discord ACL probes."""

    ingress_url: str
    log_file: Path
    max_wait_secs: int
    max_idle_secs: int
    channel_id: str
    user_id: str
    guild_id: str | None
    username: str | None
    role_ids: tuple[str, ...]
    secret_token: str | None
    session_partition: str
    no_follow: bool
