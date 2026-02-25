#!/usr/bin/env python3
"""Reproduction command helpers for memory CI gate triage."""

from __future__ import annotations

import shlex
from typing import Any


def build_gate_failure_repro_commands(
    cfg: Any,
    *,
    category: str,
    error: Exception,
    shell_quote_command_fn: Any,
    is_gate_step_error_fn: Any,
) -> list[str]:
    """Build deduplicated repro commands based on failure category."""
    commands: list[str] = [
        f"tail -n 200 {shlex.quote(str(cfg.runtime_log_file))}",
        f"tail -n 120 {shlex.quote(str(cfg.mock_log_file))}",
    ]

    if is_gate_step_error_fn(error):
        commands.append(shell_quote_command_fn(error.cmd))

    gate_script = cfg.script_dir / "test_omni_agent_memory_ci_gate.py"
    suite_script = cfg.script_dir / "test_omni_agent_memory_suite.py"
    trace_script = cfg.script_dir / "reconstruct_omni_agent_trace.py"
    agent_bin = cfg.agent_bin or (cfg.project_root / "target" / "debug" / "omni-agent")

    if category in {"runtime_startup_timeout", "runtime_startup_process"}:
        commands.append(
            shell_quote_command_fn(
                [
                    "python3",
                    str(gate_script),
                    "--profile",
                    cfg.profile,
                    "--agent-bin",
                    str(agent_bin),
                    "--runtime-startup-timeout-secs",
                    "180",
                ]
            )
        )

    if category == "memory_suite_subprocess":
        suite_cmd = [
            "python3",
            str(suite_script),
            "--suite",
            "full",
            "--max-wait",
            str(cfg.quick_max_wait if cfg.profile == "quick" else cfg.full_max_wait),
            "--max-idle-secs",
            str(cfg.quick_max_idle if cfg.profile == "quick" else max(cfg.full_max_idle, 80)),
            "--username",
            cfg.username,
        ]
        if cfg.profile == "quick" or cfg.skip_evolution:
            suite_cmd.append("--skip-evolution")
        if cfg.skip_rust_regressions:
            suite_cmd.append("--skip-rust")
        commands.append(shell_quote_command_fn(suite_cmd))

    if category == "reflection_gate_subprocess":
        commands.append(
            "cargo test -p omni-agent --lib reflective_runtime_long_horizon_quality_thresholds"
        )
    if category == "discover_cache_gate_subprocess":
        commands.append(
            "cargo test -p omni-agent --test mcp_discover_cache "
            "discover_calls_use_valkey_read_through_cache_when_configured -- --ignored --exact"
        )
    if category in {"trace_reconstruction_subprocess", "trace_reconstruction_quality"}:
        required_stages = (
            ("route", "injection", "injection_mode", "reflection", "memory")
            if cfg.profile == "nightly"
            else ("memory",)
        )
        commands.append(
            shell_quote_command_fn(
                [
                    "python3",
                    str(trace_script),
                    str(cfg.runtime_log_file),
                    "--session-id",
                    f"telegram:{cfg.chat_id}",
                    "--max-events",
                    str(cfg.trace_max_events),
                    *[item for stage in required_stages for item in ("--required-stage", stage)],
                    "--json-out",
                    str(cfg.trace_report_json),
                    "--markdown-out",
                    str(cfg.trace_report_markdown),
                ]
            )
        )
    if category in {"mcp_waiting_budget", "memory_suite_subprocess"}:
        commands.append(
            f'rg -n "mcp\\.pool\\.(call|connect)\\.waiting" {shlex.quote(str(cfg.runtime_log_file))}'
        )
    if category in {"memory_stream_budget", "memory_suite_subprocess"}:
        commands.append(
            'rg -n "agent.memory.stream_consumer.read_failed" '
            + shlex.quote(str(cfg.runtime_log_file))
        )
    if category in {"evolution_quality", "slow_response_quality"}:
        commands.append(f"python3 -m json.tool {shlex.quote(str(cfg.evolution_report_json))}")
    if category == "benchmark_quality":
        commands.append(f"python3 -m json.tool {shlex.quote(str(cfg.benchmark_report_json))}")
    if category == "session_matrix_quality":
        commands.append(f"python3 -m json.tool {shlex.quote(str(cfg.session_matrix_report_json))}")
    if category == "cross_group_quality":
        commands.append(f"python3 -m json.tool {shlex.quote(str(cfg.cross_group_report_json))}")

    deduped: list[str] = []
    seen: set[str] = set()
    for command in commands:
        if command in seen:
            continue
        seen.add(command)
        deduped.append(command)
    return deduped
