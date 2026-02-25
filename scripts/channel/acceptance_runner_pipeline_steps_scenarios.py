#!/usr/bin/env python3
"""Scenario-oriented pipeline steps for acceptance runner."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from acceptance_runner_pipeline_steps_models import PipelineStepSpec


def build_scenario_step_specs(
    cfg: Any,
    *,
    default_complex_json: str,
    default_complex_markdown: str,
    default_memory_json: str,
    default_memory_markdown: str,
    default_complex_dataset: str,
    default_memory_dataset: str,
    default_memory_scenario: str,
) -> list[PipelineStepSpec]:
    """Build complex control-plane and memory-evolution scenario steps."""
    return [
        PipelineStepSpec(
            step="complex_scenario",
            title="Run complex control-plane scenario",
            cmd=(
                "bash",
                "scripts/channel/test-omni-agent-complex-scenarios.sh",
                "--dataset",
                default_complex_dataset,
                "--max-wait",
                str(cfg.max_wait),
                "--max-idle-secs",
                str(cfg.max_idle_secs),
                "--max-parallel",
                "4",
                "--min-steps",
                "14",
                "--min-dependency-edges",
                "14",
                "--min-critical-path",
                "6",
                "--min-parallel-waves",
                "3",
                "--min-error-signals",
                "0",
                "--min-negative-feedback-events",
                "0",
                "--min-correction-checks",
                "0",
                "--min-successful-corrections",
                "0",
                "--min-planned-hits",
                "0",
                "--min-natural-language-steps",
                "0",
                "--output-json",
                default_complex_json,
                "--output-markdown",
                default_complex_markdown,
            ),
            expected_outputs=(Path(default_complex_json), Path(default_complex_markdown)),
        ),
        PipelineStepSpec(
            step="memory_evolution",
            title="Run memory evolution DAG scenario",
            cmd=(
                "bash",
                "scripts/channel/test-omni-agent-complex-scenarios.sh",
                "--dataset",
                default_memory_dataset,
                "--scenario",
                default_memory_scenario,
                "--max-wait",
                str(cfg.evolution_max_wait),
                "--max-idle-secs",
                str(cfg.evolution_max_idle_secs),
                "--max-parallel",
                str(cfg.evolution_max_parallel),
                "--output-json",
                default_memory_json,
                "--output-markdown",
                default_memory_markdown,
            ),
            expected_outputs=(Path(default_memory_json), Path(default_memory_markdown)),
        ),
    ]
