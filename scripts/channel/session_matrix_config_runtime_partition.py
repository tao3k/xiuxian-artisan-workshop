#!/usr/bin/env python3
"""Partition-mode resolution helpers for session matrix runtime config."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def resolve_runtime_partition_mode(
    log_file: Path,
    *,
    normalize_telegram_session_partition_mode_fn: Any,
    session_partition_mode_from_runtime_log_fn: Any,
    telegram_session_partition_mode_fn: Any,
) -> str | None:
    """Resolve runtime partition mode from override/log/settings chain."""
    override = os.environ.get("OMNI_BLACKBOX_SESSION_PARTITION_MODE", "").strip()
    normalized_override = normalize_telegram_session_partition_mode_fn(override)
    if normalized_override:
        return normalized_override

    mode_from_log = session_partition_mode_from_runtime_log_fn(log_file)
    if mode_from_log:
        return mode_from_log
    return telegram_session_partition_mode_fn()
