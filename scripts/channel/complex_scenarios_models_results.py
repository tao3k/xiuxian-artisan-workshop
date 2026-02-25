#!/usr/bin/env python3
"""Run result datamodels for complex scenarios."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from complex_scenarios_models_core import ComplexityRequirement, QualityRequirement
    from complex_scenarios_models_profiles import ComplexityProfile, QualityProfile


@dataclass(frozen=True)
class StepRunResult:
    scenario_id: str
    step_id: str
    session_alias: str
    session_key: str
    wave_index: int
    depends_on: tuple[str, ...]
    prompt: str
    event: str | None
    command: tuple[str, ...]
    returncode: int
    duration_ms: int
    passed: bool
    skipped: bool
    skip_reason: str | None
    bot_excerpt: str | None
    memory_planned_seen: bool
    memory_injected_seen: bool
    memory_skipped_seen: bool
    memory_feedback_updated_seen: bool
    memory_recall_credit_seen: bool
    memory_decay_seen: bool
    memory_recall_credit_count: int
    memory_decay_count: int
    memory_planned_bias: float | None
    memory_decision: str | None
    mcp_last_event: str | None
    mcp_waiting_seen: bool
    mcp_event_counts: dict[str, int]
    feedback_command_bias_before: float | None
    feedback_command_bias_after: float | None
    feedback_command_bias_delta: float | None
    feedback_heuristic_bias_before: float | None
    feedback_heuristic_bias_after: float | None
    feedback_heuristic_bias_delta: float | None
    stdout_tail: str
    stderr_tail: str


@dataclass(frozen=True)
class ScenarioRunResult:
    scenario_id: str
    description: str
    requirement: ComplexityRequirement
    complexity: ComplexityProfile
    complexity_passed: bool
    complexity_failures: tuple[str, ...]
    quality_requirement: QualityRequirement
    quality: QualityProfile
    quality_passed: bool
    quality_failures: tuple[str, ...]
    duration_ms: int
    steps: tuple[StepRunResult, ...]
    passed: bool
