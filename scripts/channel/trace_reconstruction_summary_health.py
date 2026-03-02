#!/usr/bin/env python3
"""Health evaluation for reconstructed runtime traces."""

from __future__ import annotations

from typing import Any

from trace_reconstruction_summary_flags import (
    DEFAULT_REQUIRED_STAGES,
    STAGE_ERROR_MESSAGE,
    STAGE_TO_FLAG,
)


def evaluate_trace_health(
    summary: dict[str, Any],
    *,
    require_suggested_link: bool = False,
    required_stages: tuple[str, ...] = DEFAULT_REQUIRED_STAGES,
) -> list[str]:
    """Evaluate summary against required stages and optional link evidence."""
    stage_flags = summary.get("stage_flags", {})
    errors: list[str] = []
    for stage in required_stages:
        flag_name = STAGE_TO_FLAG.get(stage)
        if flag_name is None:
            continue
        if not bool(stage_flags.get(flag_name, False)):
            errors.append(STAGE_ERROR_MESSAGE[stage])
    if require_suggested_link and not bool(stage_flags.get("has_suggested_link", False)):
        errors.append("missing suggested_link evidence")
    return errors
