#!/usr/bin/env python3
"""Rust and Valkey regression runners for memory suite."""

from __future__ import annotations

import os
import shutil
import subprocess
from typing import Any


def run_rust_memory_regressions(*, run_command_fn: Any) -> None:
    """Run Rust regression commands required by full memory suite."""
    run_command_fn(
        [
            "cargo",
            "test",
            "-p",
            "omni-agent",
            "--test",
            "agent_memory_persistence_backend",
            "memory_turn_store_skips_episode_when_embedding_endpoint_is_unavailable",
            "-q",
        ],
        title="Regression: embedding endpoint down fallback behavior",
    )
    run_command_fn(
        [
            "cargo",
            "test",
            "-p",
            "omni-agent",
            "--lib",
            "runtime_handle_inbound_session_memory_reports_latest_snapshot_json",
            "-q",
        ],
        title="Regression: /session memory json payload fields",
    )
    run_command_fn(
        [
            "cargo",
            "test",
            "-p",
            "omni-agent",
            "--lib",
            "runtime_handle_inbound_session_feedback_json",
            "-q",
        ],
        title="Regression: /session feedback json payload fields",
    )
    run_command_fn(
        [
            "cargo",
            "test",
            "-p",
            "omni-agent",
            "--lib",
            "agent::embedding_dimension::tests",
            "-q",
        ],
        title="Regression: embedding dimension auto-repair behavior",
    )
    run_command_fn(
        [
            "cargo",
            "test",
            "-p",
            "omni-agent",
            "--lib",
            "inspect_memory_recall_snapshot_keeps_embedding_repaired_source",
            "-q",
        ],
        title="Regression: session memory snapshot keeps embedding_repaired source",
    )


def ensure_valkey_cli() -> None:
    """Ensure valkey-cli is available in PATH."""
    if shutil.which("valkey-cli") is None:
        raise RuntimeError("valkey-cli not found in PATH")


def check_valkey_connectivity(valkey_url: str) -> None:
    """Verify Valkey reachability using PING."""
    print(f"Checking Valkey connectivity at {valkey_url}...", flush=True)
    subprocess.run(
        ["valkey-cli", "-u", valkey_url, "ping"],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def run_valkey_cross_instance_regression(
    valkey_url: str, valkey_prefix: str, *, run_command_fn: Any
) -> None:
    """Run cross-instance snapshot continuity regression via Valkey."""
    ensure_valkey_cli()
    check_valkey_connectivity(valkey_url)
    env = os.environ.copy()
    env["XIUXIAN_WENDAO_VALKEY_URL"] = valkey_url
    env["OMNI_AGENT_SESSION_VALKEY_PREFIX"] = valkey_prefix
    env["OMNI_AGENT_MEMORY_VALKEY_KEY_PREFIX"] = f"{valkey_prefix}:memory"
    print(f"Valkey isolation prefix: {valkey_prefix}", flush=True)
    run_command_fn(
        [
            "cargo",
            "test",
            "-p",
            "omni-agent",
            "memory_recall_snapshot_is_shared_across_agent_instances_with_valkey",
            "--",
            "--ignored",
            "--nocapture",
        ],
        title="Regression: cross-instance /session memory snapshot continuity with Valkey",
        env=env,
    )
