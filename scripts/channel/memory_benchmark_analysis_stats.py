#!/usr/bin/env python3
"""Statistical helper functions for memory benchmark analysis."""

from __future__ import annotations

import statistics


def keyword_hit_ratio(bot_line: str | None, expected_keywords: tuple[str, ...]) -> float | None:
    """Compute keyword-hit ratio for one model reply."""
    if not expected_keywords:
        return None
    if not bot_line:
        return 0.0
    lowered = bot_line.lower()
    hits = sum(1 for keyword in expected_keywords if keyword.lower() in lowered)
    return hits / len(expected_keywords)


def maybe_mean(values: list[float]) -> float:
    """Mean with safe default when list is empty."""
    if not values:
        return 0.0
    return float(statistics.fmean(values))


def maybe_mean_int(values: list[int]) -> float:
    """Mean for integer-valued lists with safe empty default."""
    if not values:
        return 0.0
    return float(statistics.fmean(values))
