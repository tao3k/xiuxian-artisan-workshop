#!/usr/bin/env python3
"""Compatibility wrappers for admin isolation helpers."""

from __future__ import annotations

from command_events_admin_isolation_cases import (
    build_admin_list_isolation_case,
    build_admin_list_topic_isolation_case,
)
from command_events_admin_isolation_runtime import (
    run_admin_isolation_assertions,
    run_admin_topic_isolation_assertions,
)

__all__ = [
    "build_admin_list_isolation_case",
    "build_admin_list_topic_isolation_case",
    "run_admin_isolation_assertions",
    "run_admin_topic_isolation_assertions",
]
