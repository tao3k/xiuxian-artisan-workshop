#!/usr/bin/env python3
"""Matrix step templates for session matrix runner."""

from __future__ import annotations

from typing import Any

from session_matrix_steps_session_a_flow import build_session_a_reset_validation_steps
from session_matrix_steps_session_b_flow import build_session_b_reset_validation_steps


def build_matrix_steps(
    cfg: Any,
    *,
    matrix_step_cls: Any,
    session_context_result_fields_fn: Any,
    session_memory_result_fields_fn: Any,
) -> tuple[Any, ...]:
    """Build deterministic matrix step sequence for validation flow."""
    return (
        *build_session_a_reset_validation_steps(
            cfg,
            matrix_step_cls=matrix_step_cls,
            session_context_result_fields_fn=session_context_result_fields_fn,
            session_memory_result_fields_fn=session_memory_result_fields_fn,
        ),
        *build_session_b_reset_validation_steps(
            cfg,
            matrix_step_cls=matrix_step_cls,
            session_context_result_fields_fn=session_context_result_fields_fn,
        ),
    )
