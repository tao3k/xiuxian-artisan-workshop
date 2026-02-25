#!/usr/bin/env python3
"""Cross-group scenario gate helper for memory CI runtime."""

from __future__ import annotations

import sys
from typing import Any


def run_cross_group_complex_gate(
    cfg: Any,
    *,
    cwd: Any,
    env: dict[str, str],
    run_command_fn: Any,
    assert_cross_group_complex_quality_fn: Any,
) -> None:
    """Run cross-group mixed-concurrency scenario gate."""
    if cfg.skip_cross_group_complex_gate:
        print("Skipping cross-group complex gate (--skip-cross-group-complex-gate).", flush=True)
        return
    if not cfg.cross_group_dataset.exists():
        raise FileNotFoundError(f"cross-group dataset not found: {cfg.cross_group_dataset}")

    script = cfg.script_dir / "test_omni_agent_complex_scenarios.py"
    if not script.exists():
        raise FileNotFoundError(f"missing complex scenario script: {script}")

    cross_group_chat_a = max(cfg.chat_id, cfg.chat_b, cfg.chat_c) + 1000
    cross_group_chat_b = cross_group_chat_a + 1
    cross_group_chat_c = cross_group_chat_a + 2

    cmd = [
        sys.executable,
        str(script),
        "--dataset",
        str(cfg.cross_group_dataset),
        "--scenario",
        cfg.cross_group_scenario_id,
        "--max-wait",
        str(cfg.cross_group_max_wait),
        "--max-idle-secs",
        str(cfg.cross_group_max_idle),
        "--max-parallel",
        str(cfg.cross_group_max_parallel),
        "--execute-wave-parallel",
        "--chat-a",
        str(cross_group_chat_a),
        "--chat-b",
        str(cross_group_chat_b),
        "--chat-c",
        str(cross_group_chat_c),
        "--user-a",
        str(cfg.user_id),
        "--user-b",
        str(cfg.user_b),
        "--user-c",
        str(cfg.user_c),
        "--min-error-signals",
        "0",
        "--min-negative-feedback-events",
        "0",
        "--min-correction-checks",
        "0",
        "--min-successful-corrections",
        "0",
        "--min-planned-hits",
        "0",
        "--min-natural-language-steps",
        "0",
        "--min-recall-credit-events",
        "0",
        "--min-decay-events",
        "0",
        "--output-json",
        str(cfg.cross_group_report_json),
        "--output-markdown",
        str(cfg.cross_group_report_markdown),
    ]
    if cfg.username.strip():
        cmd.extend(["--username", cfg.username.strip()])

    run_command_fn(
        cmd,
        title="Nightly gate: cross-group mixed-concurrency stress scenario (A4)",
        cwd=cwd,
        env=env,
    )
    assert_cross_group_complex_quality_fn(cfg)
