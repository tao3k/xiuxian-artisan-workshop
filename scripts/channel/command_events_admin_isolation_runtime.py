#!/usr/bin/env python3
"""Compatibility wrappers for admin isolation runtime assertions."""

from __future__ import annotations

from command_events_admin_isolation_matrix import run_admin_isolation_assertions
from command_events_admin_isolation_topic import run_admin_topic_isolation_assertions

__all__ = [
    "run_admin_isolation_assertions",
    "run_admin_topic_isolation_assertions",
]
