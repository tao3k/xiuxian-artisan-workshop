#!/usr/bin/env python3
"""Scenario/query spec datamodels for memory benchmark runner."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class QuerySpec:
    """One benchmark query and its keyword quality target."""

    prompt: str
    expected_keywords: tuple[str, ...]
    required_ratio: float


@dataclass(frozen=True)
class ScenarioSpec:
    """One benchmark scenario containing setup prompts and query turns."""

    scenario_id: str
    description: str
    setup_prompts: tuple[str, ...]
    queries: tuple[QuerySpec, ...]
    reset_before: bool = True
    reset_after: bool = False
