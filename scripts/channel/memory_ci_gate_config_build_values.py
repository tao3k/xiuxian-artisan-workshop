#!/usr/bin/env python3
"""GateConfig value assembly helpers for memory CI gate."""

from __future__ import annotations

from pathlib import Path
from typing import Any


def build_gate_config(
    args: Any,
    *,
    gate_config_cls: Any,
    project_root: Path,
    agent_bin: Path | None,
    webhook_port: int,
    telegram_api_port: int,
    valkey_url: str,
    valkey_prefix: str,
    webhook_secret: str,
    ids: dict[str, int],
    artifact_relpaths: dict[str, str],
) -> Any:
    """Build final GateConfig object from normalized inputs."""
    return gate_config_cls(
        profile=args.profile,
        project_root=project_root,
        script_dir=project_root / "scripts" / "channel",
        agent_bin=agent_bin,
        webhook_port=webhook_port,
        telegram_api_port=telegram_api_port,
        valkey_port=args.valkey_port,
        valkey_url=valkey_url,
        valkey_prefix=valkey_prefix,
        username=args.username.strip(),
        webhook_secret=webhook_secret,
        chat_id=ids["chat_id"],
        chat_b=ids["chat_b"],
        chat_c=ids["chat_c"],
        user_id=ids["user_id"],
        user_b=ids["user_b"],
        user_c=ids["user_c"],
        runtime_log_file=(project_root / artifact_relpaths["runtime_log_file"]).resolve(),
        mock_log_file=(project_root / artifact_relpaths["mock_log_file"]).resolve(),
        runtime_startup_timeout_secs=int(args.runtime_startup_timeout_secs),
        quick_max_wait=int(args.quick_max_wait),
        quick_max_idle=int(args.quick_max_idle),
        full_max_wait=int(args.full_max_wait),
        full_max_idle=int(args.full_max_idle),
        matrix_max_wait=int(args.matrix_max_wait),
        matrix_max_idle=int(args.matrix_max_idle),
        benchmark_iterations=int(args.benchmark_iterations),
        skip_matrix=bool(args.skip_matrix),
        skip_benchmark=bool(args.skip_benchmark),
        skip_evolution=bool(args.skip_evolution),
        skip_rust_regressions=bool(args.skip_rust_regressions),
        skip_discover_cache_gate=bool(args.skip_discover_cache_gate),
        skip_reflection_quality_gate=bool(args.skip_reflection_quality_gate),
        skip_trace_reconstruction_gate=bool(args.skip_trace_reconstruction_gate),
        skip_cross_group_complex_gate=bool(args.skip_cross_group_complex_gate),
        evolution_report_json=(project_root / artifact_relpaths["evolution_report_json"]).resolve(),
        benchmark_report_json=(project_root / artifact_relpaths["benchmark_report_json"]).resolve(),
        session_matrix_report_json=(
            project_root / artifact_relpaths["session_matrix_report_json"]
        ).resolve(),
        session_matrix_report_markdown=(
            project_root / artifact_relpaths["session_matrix_report_markdown"]
        ).resolve(),
        trace_report_json=(project_root / artifact_relpaths["trace_report_json"]).resolve(),
        trace_report_markdown=(project_root / artifact_relpaths["trace_report_markdown"]).resolve(),
        cross_group_report_json=(
            project_root / artifact_relpaths["cross_group_report_json"]
        ).resolve(),
        cross_group_report_markdown=(
            project_root / artifact_relpaths["cross_group_report_markdown"]
        ).resolve(),
        cross_group_dataset=Path(args.cross_group_dataset).expanduser().resolve(),
        cross_group_scenario_id=args.cross_group_scenario.strip(),
        min_planned_hits=int(args.min_planned_hits),
        min_successful_corrections=int(args.min_successful_corrections),
        min_recall_credit_events=int(args.min_recall_credit_events),
        min_quality_score=float(args.min_quality_score),
        slow_response_min_duration_ms=int(args.slow_response_min_duration_ms),
        slow_response_long_step_ms=int(args.slow_response_long_step_ms),
        slow_response_min_long_steps=int(args.slow_response_min_long_steps),
        trace_min_quality_score=float(args.trace_min_quality_score),
        trace_max_events=int(args.trace_max_events),
        min_session_steps=int(args.min_session_steps),
        require_cross_group_step=bool(args.require_cross_group_step),
        require_mixed_batch_steps=bool(args.require_mixed_batch_steps),
        cross_group_max_wait=int(args.cross_group_max_wait),
        cross_group_max_idle=int(args.cross_group_max_idle),
        cross_group_max_parallel=int(args.cross_group_max_parallel),
        discover_cache_hit_p95_ms=float(args.discover_cache_hit_p95_ms),
        discover_cache_miss_p95_ms=float(args.discover_cache_miss_p95_ms),
        discover_cache_bench_iterations=int(args.discover_cache_bench_iterations),
        max_mcp_call_waiting_events=int(args.max_mcp_call_waiting_events),
        max_mcp_connect_waiting_events=int(args.max_mcp_connect_waiting_events),
        max_mcp_waiting_events_total=int(args.max_mcp_waiting_events_total),
        max_memory_stream_read_failed_events=int(args.max_memory_stream_read_failed_events),
        max_embedding_timeout_fallback_turns=int(args.max_embedding_timeout_fallback_turns),
        max_embedding_cooldown_fallback_turns=int(args.max_embedding_cooldown_fallback_turns),
        max_embedding_unavailable_fallback_turns=int(args.max_embedding_unavailable_fallback_turns),
        max_embedding_fallback_turns_total=int(args.max_embedding_fallback_turns_total),
    )
