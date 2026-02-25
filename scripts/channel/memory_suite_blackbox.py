#!/usr/bin/env python3
"""Compatibility facade for memory-suite black-box helpers."""

from __future__ import annotations

from memory_suite_blackbox_cases import (
    TARGET_SESSION_SCOPE_PLACEHOLDER,
    BlackboxCase,
    blackbox_cases,
)
from memory_suite_blackbox_evolution import run_memory_evolution_scenario
from memory_suite_blackbox_runtime import resolve_allowed_chat_ids, run_blackbox_suite

# Backward-compatible private alias kept for monkeypatch/tests.
_resolve_allowed_chat_ids = resolve_allowed_chat_ids

__all__ = [
    "TARGET_SESSION_SCOPE_PLACEHOLDER",
    "BlackboxCase",
    "_resolve_allowed_chat_ids",
    "blackbox_cases",
    "resolve_allowed_chat_ids",
    "run_blackbox_suite",
    "run_memory_evolution_scenario",
]
