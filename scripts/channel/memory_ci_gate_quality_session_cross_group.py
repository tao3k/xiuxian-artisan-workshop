#!/usr/bin/env python3
"""Cross-group quality gate assertions for memory CI."""

from __future__ import annotations

from typing import Any

from memory_ci_gate_quality_common import load_json, safe_int


def assert_cross_group_complex_quality(cfg: Any) -> None:
    """Validate cross-group mixed-concurrency scenario report."""
    report = load_json(cfg.cross_group_report_json)
    if not bool(report.get("overall_passed", False)):
        raise RuntimeError("cross-group complex report indicates overall failure")

    scenarios_obj = report.get("scenarios")
    scenarios = scenarios_obj if isinstance(scenarios_obj, list) else []
    scenario_payload: dict[str, object] | None = None
    for item in scenarios:
        if not isinstance(item, dict):
            continue
        if str(item.get("scenario_id", "")).strip() == cfg.cross_group_scenario_id:
            scenario_payload = item
            break
    if scenario_payload is None:
        raise RuntimeError(
            f"cross-group complex report missing scenario: {cfg.cross_group_scenario_id}"
        )
    if not bool(scenario_payload.get("passed", False)):
        raise RuntimeError("cross-group complex scenario failed")

    steps_obj = scenario_payload.get("steps")
    steps = steps_obj if isinstance(steps_obj, list) else []
    if not steps:
        raise RuntimeError("cross-group complex scenario has no steps")

    aliases = {
        str(step.get("session_alias", "")).strip()
        for step in steps
        if isinstance(step, dict) and str(step.get("session_alias", "")).strip()
    }
    missing_aliases = [alias for alias in ("a", "b", "c") if alias not in aliases]
    if missing_aliases:
        raise RuntimeError(
            f"cross-group complex scenario missing session aliases: {missing_aliases}"
        )

    session_keys = {
        str(step.get("session_key", "")).strip()
        for step in steps
        if isinstance(step, dict) and str(step.get("session_key", "")).strip()
    }
    if len(session_keys) < 3:
        raise RuntimeError(
            "cross-group complex scenario did not produce three distinct session keys: "
            f"session_keys={sorted(session_keys)}"
        )

    wave_counts: dict[int, int] = {}
    for step in steps:
        if not isinstance(step, dict):
            continue
        wave_index = safe_int(step.get("wave_index"), default=-1)
        wave_counts[wave_index] = wave_counts.get(wave_index, 0) + 1
    max_wave_width = max(wave_counts.values(), default=0)
    if max_wave_width < 2:
        raise RuntimeError("cross-group complex scenario did not exercise mixed concurrency waves")

    waiting_steps = sum(
        1 for step in steps if isinstance(step, dict) and bool(step.get("mcp_waiting_seen", False))
    )
    if waiting_steps > 0:
        raise RuntimeError(
            "cross-group complex scenario observed mcp waiting steps: "
            f"waiting_steps={waiting_steps}"
        )

    print(
        "Cross-group mixed-concurrency gate passed: "
        f"scenario_id={cfg.cross_group_scenario_id}, "
        f"steps={len(steps)}, "
        f"session_keys={len(session_keys)}, "
        f"max_wave_width={max_wave_width}",
        flush=True,
    )
