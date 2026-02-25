#!/usr/bin/env python3
"""Artifact path and ACL settings helpers for memory CI gate runtime."""

from __future__ import annotations

import json
from typing import Any


def default_artifact_relpath(
    *,
    category: str,
    stem: str,
    profile: str,
    run_suffix: str,
    extension: str,
) -> str:
    """Build default run-scoped artifact path."""
    return f".run/{category}/{stem}-{profile}-{run_suffix}.{extension}"


def _yaml_inline_list(values: list[str]) -> str:
    return "[" + ", ".join(json.dumps(value) for value in values) + "]"


def write_ci_channel_acl_settings(cfg: Any, *, config_home: Any) -> Any:
    """Write run-scoped ACL settings for isolated CI gate execution."""
    settings_path = config_home / "omni-dev-fusion" / "settings.yaml"
    settings_path.parent.mkdir(parents=True, exist_ok=True)
    users = [str(cfg.user_id), str(cfg.user_b), str(cfg.user_c)]
    user_list = _yaml_inline_list(users)
    settings_payload = (
        "telegram:\n"
        "  acl:\n"
        "    allow:\n"
        f"      users: {user_list}\n"
        '      groups: ["*"]\n'
        "    admin:\n"
        f"      users: {user_list}\n"
        "    control:\n"
        "      allow_from:\n"
        f"        users: {user_list}\n"
        "    slash:\n"
        "      global:\n"
        f"        users: {user_list}\n"
        "embedding:\n"
        "  batch_max_size: 128\n"
        "  batch_max_concurrency: 1\n"
    )
    settings_path.write_text(settings_payload, encoding="utf-8")
    return settings_path
