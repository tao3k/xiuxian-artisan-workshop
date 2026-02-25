#!/usr/bin/env python3
"""Complexity/quality profile datamodels for complex scenarios."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class ComplexityProfile:
    step_count: int
    dependency_edges: int
    critical_path_len: int
    wave_count: int
    parallel_waves: int
    max_wave_width: int
    branch_nodes: int
    complexity_score: float


@dataclass(frozen=True)
class QualityProfile:
    error_signal_steps: int
    negative_feedback_events: int
    correction_check_steps: int
    successful_corrections: int
    planned_hits: int
    natural_language_steps: int
    recall_credit_events: int
    decay_events: int
    quality_score: float
