#!/usr/bin/env python3
"""Compatibility facade for complex scenario step-result builders."""

from __future__ import annotations

from typing import Any

from complex_scenarios_runtime_runner_step_result_skipped import (
    build_skipped_step_result as _build_skipped_step_result_impl,
)
from complex_scenarios_runtime_runner_step_result_success import (
    build_step_result as _build_step_result_impl,
)


def build_step_result(**kwargs: Any) -> Any:
    """Build successful/failed (non-skipped) step result payload."""
    return _build_step_result_impl(**kwargs)


def build_skipped_step_result(**kwargs: Any) -> Any:
    """Build skipped step result (dependency blocked / unreachable)."""
    return _build_skipped_step_result_impl(**kwargs)
