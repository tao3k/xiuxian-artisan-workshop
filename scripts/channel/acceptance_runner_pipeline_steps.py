#!/usr/bin/env python3
"""Pipeline step planning for acceptance runner."""

from __future__ import annotations

from typing import Any

from acceptance_runner_pipeline_steps_initial import build_initial_step_specs
from acceptance_runner_pipeline_steps_models import PipelineStepSpec
from acceptance_runner_pipeline_steps_scenarios import build_scenario_step_specs


def build_step_specs(
    cfg: Any,
    *,
    default_matrix_json: str,
    default_matrix_markdown: str,
    default_complex_json: str,
    default_complex_markdown: str,
    default_memory_json: str,
    default_memory_markdown: str,
    default_complex_dataset: str,
    default_memory_dataset: str,
    default_memory_scenario: str,
    python_executable: str,
) -> list[PipelineStepSpec]:
    """Build deterministic ordered step specs for acceptance pipeline."""
    steps = build_initial_step_specs(
        cfg,
        default_matrix_json=default_matrix_json,
        default_matrix_markdown=default_matrix_markdown,
        python_executable=python_executable,
    )
    steps.extend(
        build_scenario_step_specs(
            cfg,
            default_complex_json=default_complex_json,
            default_complex_markdown=default_complex_markdown,
            default_memory_json=default_memory_json,
            default_memory_markdown=default_memory_markdown,
            default_complex_dataset=default_complex_dataset,
            default_memory_dataset=default_memory_dataset,
            default_memory_scenario=default_memory_scenario,
        )
    )
    return steps


__all__ = ["PipelineStepSpec", "build_step_specs"]
