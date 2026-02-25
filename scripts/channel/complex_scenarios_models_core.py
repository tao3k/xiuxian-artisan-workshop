#!/usr/bin/env python3
"""Core datamodels for complex scenario runner."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class SessionIdentity:
    alias: str
    chat_id: int
    user_id: int
    thread_id: int | None
    chat_title: str | None


@dataclass(frozen=True)
class ScenarioStepSpec:
    step_id: str
    session_alias: str
    prompt: str
    expect_event: str | None
    expect_reply_json_fields: tuple[str, ...]
    expect_log_regexes: tuple[str, ...]
    expect_bot_regexes: tuple[str, ...]
    forbid_log_regexes: tuple[str, ...]
    allow_no_bot: bool
    tags: tuple[str, ...]
    depends_on: tuple[str, ...]
    order: int


@dataclass(frozen=True)
class ComplexityRequirement:
    steps: int
    dependency_edges: int
    critical_path_len: int
    parallel_waves: int


@dataclass(frozen=True)
class QualityRequirement:
    min_error_signals: int
    min_negative_feedback_events: int
    min_correction_checks: int
    min_successful_corrections: int
    min_planned_hits: int
    min_natural_language_steps: int
    min_recall_credit_events: int
    min_decay_events: int


@dataclass(frozen=True)
class ScenarioSpec:
    scenario_id: str
    description: str
    steps: tuple[ScenarioStepSpec, ...]
    required_complexity: ComplexityRequirement | None
    required_quality: QualityRequirement | None
