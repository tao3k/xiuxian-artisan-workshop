#!/usr/bin/env python3
"""Scenario selection helpers for complex scenario datasets."""

from __future__ import annotations

from typing import Any


def select_scenarios(scenarios: tuple[Any, ...], scenario_id: str | None) -> tuple[Any, ...]:
    """Filter loaded scenarios by id (or return all)."""
    if scenario_id is None:
        return scenarios
    filtered = tuple(s for s in scenarios if s.scenario_id == scenario_id)
    if not filtered:
        raise ValueError(f"scenario not found: {scenario_id}")
    return filtered
