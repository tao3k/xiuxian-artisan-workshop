#!/usr/bin/env python3
"""Evolution-scenario orchestration for memory-suite black-box probes."""

from __future__ import annotations

import sys
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def run_memory_evolution_scenario(
    script_dir: Path,
    *,
    max_wait: int,
    max_idle_secs: int,
    username: str,
    dataset_path: Path,
    scenario_id: str,
    max_parallel: int,
    output_json: Path,
    output_markdown: Path,
    run_command_fn: Any,
    python_executable: str = sys.executable,
) -> None:
    """Run complex memory self-evolution scenario."""
    scenario_runner = script_dir / "test_omni_agent_complex_scenarios.py"
    if not scenario_runner.exists():
        raise FileNotFoundError(f"complex scenario runner not found: {scenario_runner}")
    if not dataset_path.exists():
        raise FileNotFoundError(f"evolution dataset not found: {dataset_path}")
    if max_parallel <= 0:
        raise ValueError("--evolution-max-parallel must be a positive integer.")

    cmd = [
        python_executable,
        str(scenario_runner),
        "--dataset",
        str(dataset_path),
        "--scenario",
        scenario_id,
        "--max-wait",
        str(max_wait),
        "--max-idle-secs",
        str(max_idle_secs),
        "--max-parallel",
        str(max_parallel),
        "--output-json",
        str(output_json),
        "--output-markdown",
        str(output_markdown),
    ]
    if username.strip():
        cmd.extend(["--username", username.strip()])

    run_command_fn(
        cmd,
        title=(
            "Black-box evolution: memory self-correction + feedback adaptation + "
            "cross-session isolation DAG"
        ),
    )
