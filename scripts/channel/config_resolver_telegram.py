#!/usr/bin/env python3
"""Telegram-specific config resolution helpers."""

from __future__ import annotations

import os
import re
from pathlib import Path

from config_resolver_core import (
    read_telegram_acl_allow_users,
    read_telegram_key_from_yaml,
    repo_root_from,
    settings_candidates,
)
from config_resolver_profiles import env_or_dotenv_value
from config_resolver_runtime import normalize_telegram_session_partition_mode

WEBHOOK_BIND_PORT_RE = re.compile(r":(\d{1,5})$")
DEFAULT_TELEGRAM_WEBHOOK_PORT = 18081


def telegram_webhook_secret_token(repo_root: Path | None = None) -> str | None:
    """Resolve webhook secret from env/.env, then settings."""
    root = repo_root or repo_root_from(Path(__file__).resolve())
    secret = env_or_dotenv_value("TELEGRAM_WEBHOOK_SECRET", root)
    if secret:
        return secret

    for settings_path in settings_candidates(root):
        configured = read_telegram_key_from_yaml(settings_path, "webhook_secret_token")
        if configured is None:
            continue
        normalized = configured.strip()
        if normalized and normalized not in {"null", "None", "~"}:
            return normalized
    return None


def _parse_port(raw: str, *, source: str) -> int:
    token = raw.strip()
    if not token:
        raise ValueError(f"{source} webhook port cannot be empty.")
    try:
        port = int(token)
    except ValueError as error:
        raise ValueError(f"{source} webhook port must be an integer, got '{raw}'.") from error
    if port <= 0 or port > 65535:
        raise ValueError(f"{source} webhook port out of range: {port}.")
    return port


def _port_from_bind(bind: str) -> int | None:
    match = WEBHOOK_BIND_PORT_RE.search(bind.strip())
    if not match:
        return None
    raw_port = match.group(1)
    try:
        return _parse_port(raw_port, source="settings")
    except ValueError:
        return None


def telegram_webhook_bind(repo_root: Path | None = None) -> str | None:
    """Resolve webhook bind from env or settings."""
    explicit_bind = os.environ.get("WEBHOOK_BIND", "").strip()
    if explicit_bind:
        return explicit_bind

    root = repo_root or repo_root_from(Path(__file__).resolve())
    for settings_path in settings_candidates(root):
        configured = read_telegram_key_from_yaml(settings_path, "webhook_bind")
        if configured is None:
            continue
        normalized = configured.strip()
        if normalized and normalized not in {"null", "None", "~"}:
            return normalized
    return None


def telegram_webhook_port(repo_root: Path | None = None) -> int:
    """Resolve webhook port from WEBHOOK_PORT, bind, or default."""
    explicit_port = os.environ.get("WEBHOOK_PORT", "").strip()
    if explicit_port:
        return _parse_port(explicit_port, source="WEBHOOK_PORT")

    bind = telegram_webhook_bind(repo_root)
    if bind:
        parsed = _port_from_bind(bind)
        if parsed is not None:
            return parsed
    return DEFAULT_TELEGRAM_WEBHOOK_PORT


def default_telegram_webhook_url(repo_root: Path | None = None) -> str:
    """Build local webhook URL from resolved webhook port."""
    return f"http://127.0.0.1:{telegram_webhook_port(repo_root)}/telegram/webhook"


def telegram_session_partition_mode(repo_root: Path | None = None) -> str | None:
    """Resolve canonical telegram session partition mode from env/settings."""
    root = repo_root or repo_root_from(Path(__file__).resolve())
    env_mode = os.environ.get("OMNI_AGENT_TELEGRAM_SESSION_PARTITION", "").strip()
    normalized_env_mode = normalize_telegram_session_partition_mode(env_mode)
    if normalized_env_mode:
        return normalized_env_mode

    for settings_path in settings_candidates(root):
        configured = read_telegram_key_from_yaml(settings_path, "session_partition")
        if configured is None:
            continue
        normalized = normalize_telegram_session_partition_mode(configured)
        if normalized:
            return normalized
    return None


def allowed_users_from_settings(repo_root: Path | None = None) -> list[str]:
    """Resolve `telegram.acl.allow.users` from merged settings (user overrides system)."""
    root = repo_root or repo_root_from(Path(__file__).resolve())
    for settings_path in settings_candidates(root):
        allowed_users = read_telegram_acl_allow_users(settings_path)
        if allowed_users is not None:
            return allowed_users
    return []


def username_from_settings(repo_root: Path | None = None) -> str | None:
    """Resolve first explicit username from allowlist."""
    for first in allowed_users_from_settings(repo_root):
        if first in {"*", "''", '""'}:
            return None
        return first
    return None
