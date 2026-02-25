#!/usr/bin/env python3
"""Command builders for memory CI gate profile runners."""

from __future__ import annotations

import sys
from typing import Any


def build_quick_suite_cmd(cfg: Any, memory_suite: Any) -> list[str]:
    """Build quick profile memory-suite command."""
    cmd = [
        sys.executable,
        str(memory_suite),
        "--suite",
        "full",
        "--skip-evolution",
        "--max-wait",
        str(cfg.quick_max_wait),
        "--max-idle-secs",
        str(cfg.quick_max_idle),
        "--username",
        cfg.username,
    ]
    if cfg.skip_rust_regressions:
        cmd.append("--skip-rust")
    return cmd


def build_nightly_suite_cmd(cfg: Any, memory_suite: Any) -> list[str]:
    """Build nightly profile memory-suite command."""
    nightly_suite_idle = max(cfg.full_max_idle, 80)
    cmd = [
        sys.executable,
        str(memory_suite),
        "--suite",
        "full",
        "--max-wait",
        str(cfg.full_max_wait),
        "--max-idle-secs",
        str(nightly_suite_idle),
        "--username",
        cfg.username,
        "--evolution-output-json",
        str(cfg.evolution_report_json),
    ]
    if cfg.skip_evolution:
        cmd.append("--skip-evolution")
    if cfg.skip_rust_regressions:
        cmd.append("--skip-rust")
    return cmd


def build_session_matrix_cmd(cfg: Any, session_matrix: Any) -> list[str]:
    """Build nightly session-matrix command."""
    return [
        sys.executable,
        str(session_matrix),
        "--max-wait",
        str(cfg.matrix_max_wait),
        "--max-idle-secs",
        str(cfg.matrix_max_idle),
        "--username",
        cfg.username,
        "--chat-id",
        str(cfg.chat_id),
        "--chat-b",
        str(cfg.chat_b),
        "--chat-c",
        str(cfg.chat_c),
        "--user-a",
        str(cfg.user_id),
        "--user-b",
        str(cfg.user_b),
        "--user-c",
        str(cfg.user_c),
        "--output-json",
        str(cfg.session_matrix_report_json),
        "--output-markdown",
        str(cfg.session_matrix_report_markdown),
        "--mixed-plain-prompt",
        "/session json",
    ]


def build_benchmark_cmd(cfg: Any, memory_benchmark: Any) -> list[str]:
    """Build nightly memory benchmark command."""
    return [
        sys.executable,
        str(memory_benchmark),
        "--username",
        cfg.username,
        "--chat-id",
        str(cfg.chat_id),
        "--user-id",
        str(cfg.user_id),
        "--iterations",
        str(cfg.benchmark_iterations),
        "--max-wait",
        str(cfg.full_max_wait),
        "--max-idle-secs",
        str(cfg.full_max_idle),
        "--output-json",
        str(cfg.benchmark_report_json),
    ]
