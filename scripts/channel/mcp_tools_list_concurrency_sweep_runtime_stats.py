#!/usr/bin/env python3
"""Stats/normalization helpers for MCP tools/list concurrency sweep."""

from __future__ import annotations


def normalize_base_url(base_url: str) -> str:
    """Normalize base URL by trimming trailing slash."""
    return base_url.rstrip("/")


def nearest_rank_percentile(sorted_values: list[float], p: float) -> float:
    """Compute nearest-rank percentile from sorted values."""
    if not sorted_values:
        return 0.0
    idx = max(0, min(len(sorted_values) - 1, int(len(sorted_values) * p) - 1))
    return sorted_values[idx]


def parse_concurrency_values(raw: str) -> list[int]:
    """Parse and validate concurrency CSV into a sorted unique integer list."""
    values = []
    for item in raw.split(","):
        token = item.strip()
        if not token:
            continue
        value = int(token)
        if value <= 0:
            raise ValueError("concurrency values must be positive integers")
        values.append(value)
    if not values:
        raise ValueError("at least one concurrency value is required")
    return sorted(set(values))
