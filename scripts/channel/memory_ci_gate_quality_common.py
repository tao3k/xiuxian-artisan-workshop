#!/usr/bin/env python3
"""Common helpers for memory CI gate quality assertions."""

from __future__ import annotations

import json
from typing import Any


def load_json(path: Any) -> dict[str, object]:
    """Load and validate a JSON object payload from disk."""
    if not path.exists():
        raise RuntimeError(f"missing report: {path}")
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise RuntimeError(f"invalid report payload (expected object): {path}")
    return payload


def safe_int(value: object, *, default: int = 0) -> int:
    """Coerce a value to int with a caller-provided fallback."""
    try:
        return int(value)
    except (TypeError, ValueError):
        return default
