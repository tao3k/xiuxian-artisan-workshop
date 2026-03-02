#!/usr/bin/env python3
"""Compatibility facade for command-events orchestration path helpers."""

from __future__ import annotations

from typing import Any

from command_events_orchestrator_paths_default_bridge import (
    run_default_mode as _run_default_mode_impl,
)
from command_events_orchestrator_paths_matrix_bridge import (
    run_matrix_mode as _run_matrix_mode_impl,
)
from command_events_orchestrator_paths_topic_bridge import (
    run_admin_topic_isolation_if_requested as _run_admin_topic_isolation_if_requested_impl,
)


def _run_admin_topic_isolation_if_requested(**kwargs: object) -> int | None:
    """Backward-compatible alias used by existing call sites/tests."""
    return _run_admin_topic_isolation_if_requested_impl(**kwargs)


def run_matrix_mode(**kwargs: Any) -> tuple[int, tuple[int, ...]]:
    """Run matrix-mode execution path and return `(exit_code, matrix_chat_ids)`."""
    return _run_matrix_mode_impl(**kwargs)


def run_default_mode(**kwargs: Any) -> int:
    """Run non-matrix execution path and return exit code."""
    return _run_default_mode_impl(**kwargs)
