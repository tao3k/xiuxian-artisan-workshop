#!/usr/bin/env python3
"""Validation helpers for session matrix runtime config build."""

from __future__ import annotations


def validate_runtime_args(*, max_wait: int, max_idle_secs: int) -> None:
    """Validate runtime wait/idle bounds."""
    if max_wait <= 0:
        raise ValueError("--max-wait must be a positive integer.")
    if max_idle_secs <= 0:
        raise ValueError("--max-idle-secs must be a positive integer.")


def ensure_distinct_session_keys(key_a: str, key_b: str, key_c: str) -> None:
    """Validate three resolved session keys are distinct."""
    unique_keys = {key_a, key_b, key_c}
    if len(unique_keys) != 3:
        raise ValueError(
            "session matrix requires three distinct session identities "
            f"(got keys: {key_a}, {key_b}, {key_c}). Adjust chat/user/thread parameters."
        )
