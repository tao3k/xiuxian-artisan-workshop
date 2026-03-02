#!/usr/bin/env python3
"""Environment-backed defaults for Discord ingress stress parser."""

from __future__ import annotations

import os
from pathlib import Path

from path_resolver import default_report_path, project_root_from


def default_secret_token() -> str:
    """Default ingress secret token."""
    return os.environ.get("OMNI_TEST_DISCORD_INGRESS_SECRET", "").strip()


def default_channel_id() -> str:
    """Default synthetic Discord channel_id."""
    return os.environ.get("OMNI_TEST_DISCORD_CHANNEL_ID", "").strip()


def default_user_id() -> str:
    """Default synthetic Discord user_id."""
    return os.environ.get("OMNI_TEST_DISCORD_USER_ID", "").strip()


def default_guild_id() -> str:
    """Default synthetic Discord guild_id."""
    return os.environ.get("OMNI_TEST_DISCORD_GUILD_ID", "").strip()


def default_username() -> str:
    """Default synthetic Discord username."""
    return os.environ.get("OMNI_TEST_DISCORD_USERNAME", "").strip()


def default_log_file() -> str:
    """Default runtime log file path."""
    return os.environ.get("OMNI_CHANNEL_LOG_FILE", ".run/logs/omni-agent-webhook.log")


def default_project_root() -> str:
    """Default project root string for relative-path resolution."""
    return str(project_root_from(Path.cwd()))


def default_output_json() -> str:
    """Default JSON report path."""
    return str(default_report_path("omni-agent-discord-ingress-stress.json"))


def default_output_markdown() -> str:
    """Default markdown report path."""
    return str(default_report_path("omni-agent-discord-ingress-stress.md"))
