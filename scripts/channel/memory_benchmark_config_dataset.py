#!/usr/bin/env python3
"""Dataset parsing helpers for memory benchmark runner."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

    from memory_benchmark_models import QuerySpec, ScenarioSpec


def load_scenarios(
    path: Path,
    *,
    query_spec_cls: type[QuerySpec],
    scenario_spec_cls: type[ScenarioSpec],
) -> tuple[ScenarioSpec, ...]:
    """Parse and validate benchmark scenario dataset JSON."""
    raw = json.loads(path.read_text(encoding="utf-8"))
    scenarios_raw = raw.get("scenarios")
    if not isinstance(scenarios_raw, list) or not scenarios_raw:
        raise ValueError("dataset must provide a non-empty 'scenarios' array")

    scenarios: list[ScenarioSpec] = []
    seen_ids: set[str] = set()
    for index, scenario_obj in enumerate(scenarios_raw):
        if not isinstance(scenario_obj, dict):
            raise ValueError(f"scenario at index {index} must be an object")
        scenario_id = str(scenario_obj.get("id", "")).strip()
        if not scenario_id:
            raise ValueError(f"scenario at index {index} has empty id")
        if scenario_id in seen_ids:
            raise ValueError(f"duplicate scenario id: {scenario_id}")
        seen_ids.add(scenario_id)

        description = str(scenario_obj.get("description", "")).strip() or scenario_id

        setup_prompts_raw = scenario_obj.get("setup_prompts", [])
        if not isinstance(setup_prompts_raw, list):
            raise ValueError(f"scenario '{scenario_id}' setup_prompts must be an array")
        setup_prompts = tuple(str(item).strip() for item in setup_prompts_raw if str(item).strip())

        queries_raw = scenario_obj.get("queries", [])
        if not isinstance(queries_raw, list) or not queries_raw:
            raise ValueError(f"scenario '{scenario_id}' must define non-empty queries")
        queries: list[QuerySpec] = []
        for query_index, query_obj in enumerate(queries_raw):
            if not isinstance(query_obj, dict):
                raise ValueError(f"scenario '{scenario_id}' query[{query_index}] must be an object")
            prompt = str(query_obj.get("prompt", "")).strip()
            if not prompt:
                raise ValueError(f"scenario '{scenario_id}' query[{query_index}] has empty prompt")
            keywords_raw = query_obj.get("expected_keywords", [])
            if not isinstance(keywords_raw, list):
                raise ValueError(
                    f"scenario '{scenario_id}' query[{query_index}] expected_keywords must be an array"
                )
            keywords = tuple(str(word).strip() for word in keywords_raw if str(word).strip())
            required_ratio = float(query_obj.get("required_ratio", 1.0))
            if required_ratio <= 0.0 or required_ratio > 1.0:
                raise ValueError(
                    f"scenario '{scenario_id}' query[{query_index}] required_ratio must be in (0, 1]"
                )
            queries.append(
                query_spec_cls(
                    prompt=prompt,
                    expected_keywords=keywords,
                    required_ratio=required_ratio,
                )
            )

        scenarios.append(
            scenario_spec_cls(
                scenario_id=scenario_id,
                description=description,
                setup_prompts=setup_prompts,
                queries=tuple(queries),
                reset_before=bool(scenario_obj.get("reset_before", True)),
                reset_after=bool(scenario_obj.get("reset_after", False)),
            )
        )

    return tuple(scenarios)
