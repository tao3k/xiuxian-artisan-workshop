#!/usr/bin/env python3
"""Execution workflow for omni-agent memory CI gate."""

from __future__ import annotations

import importlib
from typing import Any

from memory_ci_gate_runner_workflow_cleanup import cleanup_runtime_stack
from memory_ci_gate_runner_workflow_profile import run_profile
from memory_ci_gate_runner_workflow_runtime import start_runtime_stack

_env_module = importlib.import_module("memory_ci_gate_runner_env")
_profiles_module = importlib.import_module("memory_ci_gate_runner_profiles")


def run_gate(
    cfg: Any,
    *,
    ensure_parent_dirs_fn: Any,
    default_run_suffix_fn: Any,
    write_ci_channel_acl_settings_fn: Any,
    valkey_reachable_fn: Any,
    run_command_fn: Any,
    start_background_process_fn: Any,
    wait_for_mock_health_fn: Any,
    wait_for_log_regex_fn: Any,
    run_reflection_quality_gate_fn: Any,
    run_discover_cache_gate_fn: Any,
    run_trace_reconstruction_gate_fn: Any,
    run_cross_group_complex_gate_fn: Any,
    assert_mcp_waiting_warning_budget_fn: Any,
    assert_memory_stream_warning_budget_fn: Any,
    assert_evolution_quality_fn: Any,
    assert_evolution_slow_response_quality_fn: Any,
    assert_session_matrix_quality_fn: Any,
    assert_benchmark_quality_fn: Any,
    terminate_process_fn: Any,
) -> None:
    """Run full memory CI gate workflow for quick or nightly profile."""
    ensure_parent_dirs_fn(cfg.runtime_log_file, cfg.mock_log_file, cfg.evolution_report_json)
    ensure_parent_dirs_fn(
        cfg.benchmark_report_json,
        cfg.session_matrix_report_json,
        cfg.session_matrix_report_markdown,
        cfg.trace_report_json,
        cfg.trace_report_markdown,
        cfg.cross_group_report_json,
        cfg.cross_group_report_markdown,
    )

    script_paths = _env_module.resolve_script_paths(cfg.script_dir)
    mock_server = script_paths["mock_server"]

    if not mock_server.exists():
        raise FileNotFoundError(f"missing mock server script: {mock_server}")

    env, settings_path = _env_module.build_runtime_env(
        cfg,
        default_run_suffix_fn=default_run_suffix_fn,
        write_ci_channel_acl_settings_fn=write_ci_channel_acl_settings_fn,
    )

    valkey_stop = script_paths["valkey_stop"]
    valkey_preexisting = valkey_reachable_fn(cfg.valkey_url)
    mock_process: Any | None = None
    mock_handle: object | None = None
    agent_process: Any | None = None
    agent_handle: object | None = None
    try:
        print(f"CI gate ACL settings: {settings_path}", flush=True)
        runtime_stack = start_runtime_stack(
            cfg,
            env=env,
            script_paths=script_paths,
            valkey_reachable_fn=valkey_reachable_fn,
            run_command_fn=run_command_fn,
            start_background_process_fn=start_background_process_fn,
            wait_for_mock_health_fn=wait_for_mock_health_fn,
            wait_for_log_regex_fn=wait_for_log_regex_fn,
        )
        agent_process = runtime_stack["agent_process"]
        agent_handle = runtime_stack["agent_handle"]
        mock_process = runtime_stack["mock_process"]
        mock_handle = runtime_stack["mock_handle"]
        valkey_preexisting = bool(runtime_stack["valkey_preexisting"])
        valkey_stop = runtime_stack["valkey_stop"]

        run_profile(
            cfg,
            env=env,
            script_paths=script_paths,
            profiles_module=_profiles_module,
            run_command_fn=run_command_fn,
            run_reflection_quality_gate_fn=run_reflection_quality_gate_fn,
            run_discover_cache_gate_fn=run_discover_cache_gate_fn,
            run_trace_reconstruction_gate_fn=run_trace_reconstruction_gate_fn,
            run_cross_group_complex_gate_fn=run_cross_group_complex_gate_fn,
            assert_mcp_waiting_warning_budget_fn=assert_mcp_waiting_warning_budget_fn,
            assert_memory_stream_warning_budget_fn=assert_memory_stream_warning_budget_fn,
            assert_evolution_quality_fn=assert_evolution_quality_fn,
            assert_evolution_slow_response_quality_fn=assert_evolution_slow_response_quality_fn,
            assert_session_matrix_quality_fn=assert_session_matrix_quality_fn,
            assert_benchmark_quality_fn=assert_benchmark_quality_fn,
        )
    finally:
        cleanup_runtime_stack(
            cfg,
            env=env,
            terminate_process_fn=terminate_process_fn,
            agent_process=agent_process,
            mock_process=mock_process,
            agent_handle=agent_handle,
            mock_handle=mock_handle,
            valkey_preexisting=valkey_preexisting,
            valkey_stop=valkey_stop,
        )
