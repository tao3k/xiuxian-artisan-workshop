#!/usr/bin/env python3
"""Feedback policy helpers for memory benchmark analysis."""

from __future__ import annotations


def select_feedback_direction(
    *,
    keyword_hit_ratio: float | None,
    keyword_success: bool | None,
    policy: str,
    down_threshold: float,
) -> str | None:
    """Select feedback direction from policy and measured quality."""
    if keyword_success is None:
        return None
    if policy == "strict":
        return "up" if keyword_success else "down"
    if keyword_success:
        return "up"
    if keyword_hit_ratio is not None and keyword_hit_ratio <= down_threshold:
        return "down"
    return None
