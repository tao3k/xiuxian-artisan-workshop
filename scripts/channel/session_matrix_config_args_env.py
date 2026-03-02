#!/usr/bin/env python3
"""Environment-driven default resolvers for session matrix CLI args."""

from __future__ import annotations

import os


def env_int(name: str) -> int | None:
    """Read optional integer from environment."""
    if name not in os.environ:
        return None
    return int(os.environ[name])


def default_wait_secs() -> int:
    """Default max wait seconds from env override."""
    return int(os.environ.get("OMNI_BLACKBOX_MAX_WAIT_SECS", "35"))


def default_idle_secs() -> int:
    """Default max idle seconds from env override."""
    return int(os.environ.get("OMNI_BLACKBOX_MAX_IDLE_SECS", "25"))


def default_log_file() -> str:
    """Default channel runtime log file path."""
    return os.environ.get("OMNI_CHANNEL_LOG_FILE", ".run/logs/omni-agent-webhook.log")


def default_username() -> str | None:
    """Default Telegram username for allowlist checks."""
    return os.environ.get("OMNI_TEST_USERNAME")


def default_secret_token() -> str | None:
    """Default webhook secret from environment."""
    return os.environ.get("TELEGRAM_WEBHOOK_SECRET")
