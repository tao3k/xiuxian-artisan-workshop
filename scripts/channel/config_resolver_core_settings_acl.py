#!/usr/bin/env python3
"""ACL readers for channel resolver config (xiuxian.toml)."""

from __future__ import annotations

import tomllib
from typing import TYPE_CHECKING

from config_resolver_core_scalars import parse_scalar_list

if TYPE_CHECKING:
    from pathlib import Path


def _normalize_user_entries(entries: object) -> list[str]:
    if entries is None:
        return []
    if isinstance(entries, list):
        values = entries
    elif isinstance(entries, str):
        values = parse_scalar_list(entries)
    else:
        values = [entries]
    normalized: list[str] = []
    for item in values:
        text = str(item).strip()
        if not text or text in {"null", "None", "~"}:
            continue
        normalized.append(text)
    return normalized


def _read_telegram_acl_allow_users_from_toml(path: Path) -> list[str] | None:
    if not path.exists():
        return None
    try:
        document = tomllib.loads(path.read_text(encoding="utf-8", errors="ignore"))
    except tomllib.TOMLDecodeError:
        return None

    candidates = (
        document.get("telegram", {}).get("acl", {}).get("allow", {}).get("users"),
        document.get("telegram", {}).get("acl", {}).get("allow_users"),
        document.get("telegram", {}).get("acl", {}).get("users"),
        document.get("telegram", {}).get("allow_users"),
    )
    for candidate in candidates:
        if candidate is None:
            continue
        return _normalize_user_entries(candidate)
    return None


def read_telegram_acl_allow_users(path: Path) -> list[str] | None:
    """Read telegram.acl.allow.users from xiuxian TOML candidate."""
    return _read_telegram_acl_allow_users_from_toml(path)
