#!/usr/bin/env python3
"""Benchmark quality gate checks for memory CI pipeline."""

from __future__ import annotations

import json
from typing import Any


def assert_benchmark_quality(cfg: Any) -> None:
    """Validate benchmark mode summaries and fallback budgets."""
    if not cfg.benchmark_report_json.exists():
        raise RuntimeError(f"missing benchmark report: {cfg.benchmark_report_json}")
    report = json.loads(cfg.benchmark_report_json.read_text(encoding="utf-8"))
    mode_summaries = report.get("mode_summaries")
    if not isinstance(mode_summaries, dict) or not mode_summaries:
        raise RuntimeError("benchmark report missing mode_summaries")

    failures: list[str] = []
    mode_fallback_observations: list[str] = []
    for mode in ("baseline", "adaptive"):
        summary = mode_summaries.get(mode)
        if not isinstance(summary, dict):
            failures.append(f"missing mode summary: {mode}")
            continue
        query_turns = int(summary.get("query_turns", 0))
        mcp_error_turns = int(summary.get("mcp_error_turns", 0))
        timeout_fallback_turns = int(summary.get("embedding_timeout_fallback_turns", 0))
        cooldown_fallback_turns = int(summary.get("embedding_cooldown_fallback_turns", 0))
        unavailable_fallback_turns = int(summary.get("embedding_unavailable_fallback_turns", 0))
        fallback_turns_total = int(summary.get("embedding_fallback_turns_total", 0))
        if query_turns <= 0:
            failures.append(f"{mode}.query_turns={query_turns} <= 0")
        if mcp_error_turns > 0:
            failures.append(f"{mode}.mcp_error_turns={mcp_error_turns} > 0")
        if timeout_fallback_turns > cfg.max_embedding_timeout_fallback_turns:
            failures.append(
                f"{mode}.embedding_timeout_fallback_turns={timeout_fallback_turns} > "
                f"{cfg.max_embedding_timeout_fallback_turns}"
            )
        if cooldown_fallback_turns > cfg.max_embedding_cooldown_fallback_turns:
            failures.append(
                f"{mode}.embedding_cooldown_fallback_turns={cooldown_fallback_turns} > "
                f"{cfg.max_embedding_cooldown_fallback_turns}"
            )
        if unavailable_fallback_turns > cfg.max_embedding_unavailable_fallback_turns:
            failures.append(
                f"{mode}.embedding_unavailable_fallback_turns={unavailable_fallback_turns} > "
                f"{cfg.max_embedding_unavailable_fallback_turns}"
            )
        if fallback_turns_total > cfg.max_embedding_fallback_turns_total:
            failures.append(
                f"{mode}.embedding_fallback_turns_total={fallback_turns_total} > "
                f"{cfg.max_embedding_fallback_turns_total}"
            )
        mode_fallback_observations.append(
            f"{mode}:timeout={timeout_fallback_turns},cooldown={cooldown_fallback_turns},"
            f"unavailable={unavailable_fallback_turns},total={fallback_turns_total}"
        )

    if failures:
        raise RuntimeError("benchmark quality gates failed: " + "; ".join(failures))

    print(
        "Benchmark quality gates passed "
        "(query_turns > 0, mcp_error_turns == 0, embedding fallback budgets respected): "
        + "; ".join(mode_fallback_observations),
        flush=True,
    )
