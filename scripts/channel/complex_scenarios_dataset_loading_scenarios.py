#!/usr/bin/env python3
"""Scenario loading helpers for complex scenario datasets."""

from __future__ import annotations

import json
from typing import Any

from complex_scenarios_dataset_loading_steps import parse_step, validate_dependencies
from complex_scenarios_dataset_requirements import (
    parse_quality_requirement,
    parse_requirement,
    required_str_field,
)


def load_scenarios(
    path: Any,
    *,
    scenario_spec_cls: Any,
    step_spec_cls: Any,
    requirement_cls: Any,
    quality_requirement_cls: Any,
    build_execution_waves_fn: Any,
) -> tuple[Any, ...]:
    """Load + validate scenario dataset from JSON file."""
    raw = json.loads(path.read_text(encoding="utf-8"))
    scenarios_raw = raw.get("scenarios")
    if not isinstance(scenarios_raw, list) or not scenarios_raw:
        raise ValueError("dataset must provide a non-empty 'scenarios' array")

    scenarios: list[Any] = []
    seen_scenario_ids: set[str] = set()

    for scenario_index, scenario_obj in enumerate(scenarios_raw):
        if not isinstance(scenario_obj, dict):
            raise ValueError(f"scenario[{scenario_index}] must be an object")

        scenario_id = required_str_field(scenario_obj, "id", ctx=f"scenario[{scenario_index}]")
        if scenario_id in seen_scenario_ids:
            raise ValueError(f"duplicate scenario id: {scenario_id}")
        seen_scenario_ids.add(scenario_id)

        description = str(scenario_obj.get("description", "")).strip() or scenario_id
        required_complexity = parse_requirement(
            scenario_obj.get("required_complexity")
            if isinstance(scenario_obj.get("required_complexity"), dict)
            else None,
            requirement_cls=requirement_cls,
        )
        required_quality = parse_quality_requirement(
            scenario_obj.get("required_quality")
            if isinstance(scenario_obj.get("required_quality"), dict)
            else None,
            quality_requirement_cls=quality_requirement_cls,
        )

        steps_raw = scenario_obj.get("steps")
        if not isinstance(steps_raw, list) or not steps_raw:
            raise ValueError(f"scenario '{scenario_id}' requires non-empty steps")

        steps: list[Any] = []
        seen_step_ids: set[str] = set()
        for step_index, step_obj in enumerate(steps_raw):
            if not isinstance(step_obj, dict):
                raise ValueError(f"scenario '{scenario_id}' step[{step_index}] must be an object")
            steps.append(
                parse_step(
                    scenario_id=scenario_id,
                    step_index=step_index,
                    step_obj=step_obj,
                    seen_step_ids=seen_step_ids,
                    step_spec_cls=step_spec_cls,
                )
            )

        validate_dependencies(scenario_id=scenario_id, steps=steps)

        scenario = scenario_spec_cls(
            scenario_id=scenario_id,
            description=description,
            steps=tuple(steps),
            required_complexity=required_complexity,
            required_quality=required_quality,
        )
        build_execution_waves_fn(scenario)
        scenarios.append(scenario)

    return tuple(scenarios)
