#!/usr/bin/env python3
"""Mode execution flow for memory benchmark runner."""

from __future__ import annotations

from typing import Any


def run_mode(
    config: Any,
    scenarios: tuple[Any, ...],
    mode: str,
    *,
    run_reset_fn: Any,
    run_non_command_turn_fn: Any,
    build_turn_result_fn: Any,
    select_feedback_direction_fn: Any,
    run_feedback_fn: Any,
) -> list[Any]:
    """Run all scenarios for one benchmark mode."""
    print(f"\n=== Running mode: {mode} ===", flush=True)
    all_turns: list[Any] = []

    for iteration in range(1, config.iterations + 1):
        print(f"\n[Iteration {iteration}/{config.iterations}]", flush=True)
        for scenario in scenarios:
            print(f"  - Scenario: {scenario.scenario_id}", flush=True)
            if scenario.reset_before and not config.skip_reset:
                run_reset_fn(config)

            for prompt in scenario.setup_prompts:
                run_non_command_turn_fn(config, prompt)

            for index, query in enumerate(scenario.queries, start=1):
                turn_lines = run_non_command_turn_fn(config, query.prompt)
                feedback_direction: str | None = None
                feedback_lines: list[str] | None = None

                provisional = build_turn_result_fn(
                    mode=mode,
                    iteration=iteration,
                    scenario_id=scenario.scenario_id,
                    query_index=index,
                    query=query,
                    lines=turn_lines,
                )

                if mode == "adaptive" and provisional.keyword_success is not None:
                    feedback_direction = select_feedback_direction_fn(
                        keyword_hit_ratio=provisional.keyword_hit_ratio,
                        keyword_success=provisional.keyword_success,
                        policy=config.feedback_policy,
                        down_threshold=config.feedback_down_threshold,
                    )
                    if feedback_direction is not None:
                        feedback_lines = run_feedback_fn(config, feedback_direction)

                turn_result = build_turn_result_fn(
                    mode=mode,
                    iteration=iteration,
                    scenario_id=scenario.scenario_id,
                    query_index=index,
                    query=query,
                    lines=turn_lines,
                    feedback_direction=feedback_direction,
                    feedback_lines=feedback_lines,
                )
                all_turns.append(turn_result)

            if scenario.reset_after and not config.skip_reset:
                run_reset_fn(config)

    return all_turns
