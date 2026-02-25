#!/usr/bin/env python3
"""Environment parsing helpers for complex scenario config."""

from __future__ import annotations

import os


def env_int(name: str) -> int | None:
    """Parse integer env var value, returning None when unset/blank."""
    raw = os.environ.get(name)
    if raw is None:
        return None
    raw = raw.strip()
    if not raw:
        return None
    return int(raw)
