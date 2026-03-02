#!/usr/bin/env python3
"""Compatibility facade for initial acceptance-runner pipeline steps."""

from __future__ import annotations

from typing import Any

from acceptance_runner_pipeline_steps_initial_core import (
    build_initial_step_specs as _build_initial_step_specs_impl,
)


def build_initial_step_specs(*args: Any, **kwargs: Any) -> list[Any]:
    """Build capture, command-event, dedup, concurrent, and session-matrix steps."""
    return _build_initial_step_specs_impl(*args, **kwargs)
