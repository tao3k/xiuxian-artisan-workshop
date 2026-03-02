#!/usr/bin/env python3
"""Artifact path and ACL config helpers for memory CI gate runtime."""

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


def _toml_inline_list(values: list[str]) -> str:
    return "[" + ", ".join(json.dumps(value) for value in values) + "]"


def write_ci_channel_acl_settings(cfg: Any, *, config_home: Any) -> Any:
    """Write run-scoped ACL config for isolated CI gate execution."""
    config_path = config_home / "xiuxian-artisan-workshop" / "xiuxian.toml"
    config_path.parent.mkdir(parents=True, exist_ok=True)
    users = [str(cfg.user_id), str(cfg.user_b), str(cfg.user_c)]
    user_list = _toml_inline_list(users)
    config_payload = (
        "[telegram.acl.allow]\n"
        f"users = {user_list}\n"
        'groups = ["*"]\n'
        "\n"
        "[telegram.acl.admin]\n"
        f"users = {user_list}\n"
        "\n"
        "[telegram.acl.control.allow_from]\n"
        f"users = {user_list}\n"
        "\n"
        "[telegram.slash.global]\n"
        f"users = {user_list}\n"
        "\n"
        "[embedding]\n"
        "batch_max_size = 128\n"
        "batch_max_concurrency = 1\n"
    )
    config_path.write_text(config_payload, encoding="utf-8")
    return config_path
