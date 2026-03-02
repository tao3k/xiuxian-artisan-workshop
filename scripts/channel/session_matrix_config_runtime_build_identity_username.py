#!/usr/bin/env python3
"""Username resolution for session matrix runtime config build."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def resolve_username(
    args: Any,
    *,
    log_file: Path,
    username_from_settings_fn: Any,
    username_from_runtime_log_fn: Any,
) -> str | None:
    """Resolve username from args, settings, then runtime logs."""
    username = args.username.strip() if args.username else None
    if not username:
        username = username_from_settings_fn()
    if not username:
        username = username_from_runtime_log_fn(log_file)
    return username
