#!/usr/bin/env python3
"""Summary assembly for reconstructed runtime traces."""

from __future__ import annotations

from typing import Any

from trace_reconstruction_summary_flags import collect_injection_modes, first_index


def build_trace_summary(entries: list[dict[str, Any]]) -> dict[str, Any]:
    """Build aggregated summary and stage flags from parsed entries."""
    event_counts: dict[str, int] = {}
    for entry in entries:
        event = str(entry["event"])
        event_counts[event] = event_counts.get(event, 0) + 1

    injection_modes = collect_injection_modes(entries)
    stage_flags = {
        "has_dedup": "telegram.dedup.update_accepted" in event_counts,
        "has_route": (
            "session.route.decision_selected" in event_counts
            or "session.route.fallback_applied" in event_counts
        ),
        "has_injection": "session.injection.snapshot_created" in event_counts,
        "has_injection_mode": bool(injection_modes),
        "has_reflection": any(name.startswith("agent.reflection.") for name in event_counts),
        "has_memory": any(name.startswith("agent.memory.") for name in event_counts),
        "has_suggested_link": "suggested_link" in event_counts,
    }

    warnings: list[str] = []
    route_idx = first_index(entries, "session.route.decision_selected")
    injection_idx = first_index(entries, "session.injection.snapshot_created")
    if route_idx is not None and injection_idx is not None and route_idx > injection_idx:
        warnings.append("route decision appeared after injection snapshot")
    if stage_flags["has_injection"] and not stage_flags["has_injection_mode"]:
        warnings.append("injection snapshot missing injection_mode field")

    reflection_store_idx = first_index(entries, "agent.reflection.policy_hint.stored")
    reflection_apply_idx = first_index(entries, "agent.reflection.policy_hint.applied")
    if (
        reflection_store_idx is not None
        and reflection_apply_idx is not None
        and reflection_store_idx > reflection_apply_idx
    ):
        warnings.append("reflection hint applied before it was stored")

    recall_plan_idx = first_index(entries, "agent.memory.recall.planned")
    recall_decision_idx = None
    for candidate in ("agent.memory.recall.injected", "agent.memory.recall.skipped"):
        idx = first_index(entries, candidate)
        if idx is None:
            continue
        recall_decision_idx = idx if recall_decision_idx is None else min(recall_decision_idx, idx)
    if (
        recall_plan_idx is not None
        and recall_decision_idx is not None
        and recall_plan_idx > recall_decision_idx
    ):
        warnings.append("memory recall decision appeared before recall planning")

    quality_components = [
        int(stage_flags["has_route"]),
        int(stage_flags["has_injection"]),
        int(stage_flags["has_reflection"]),
        int(stage_flags["has_memory"]),
    ]
    quality_score = round((sum(quality_components) / len(quality_components)) * 100.0, 2)

    return {
        "events_total": len(entries),
        "event_counts": event_counts,
        "injection_modes": sorted(injection_modes),
        "stage_flags": stage_flags,
        "warnings": warnings,
        "quality_score": quality_score,
    }
