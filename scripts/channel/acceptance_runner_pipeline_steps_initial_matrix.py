#!/usr/bin/env python3
"""Session-matrix command and tail-step builders for acceptance pipeline."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from acceptance_runner_pipeline_steps_models import PipelineStepSpec


def build_session_matrix_cmd(
    cfg: Any,
    *,
    default_matrix_json: str,
    default_matrix_markdown: str,
) -> list[str]:
    """Build session-matrix command line with optional thread overrides."""
    cmd = [
        "bash",
        "scripts/channel/test-omni-agent-session-matrix.sh",
        "--max-wait",
        str(cfg.max_wait),
        "--max-idle-secs",
        str(cfg.max_idle_secs),
        "--output-json",
        default_matrix_json,
        "--output-markdown",
        default_matrix_markdown,
    ]
    if cfg.group_thread_id is not None:
        cmd.extend(["--thread-a", str(cfg.group_thread_id)])
    if cfg.group_thread_id_b is not None:
        cmd.extend(["--thread-b", str(cfg.group_thread_id_b)])
    return cmd


def build_tail_steps(
    *,
    cfg: Any,
    session_matrix_cmd: list[str],
    default_matrix_json: str,
    default_matrix_markdown: str,
) -> list[PipelineStepSpec]:
    """Build dedup, concurrent, and session-matrix steps."""
    return [
        PipelineStepSpec(
            step="dedup",
            title="Run dedup probe",
            cmd=(
                "bash",
                "scripts/channel/test-omni-agent-dedup-events.sh",
                "--max-wait",
                str(cfg.max_wait),
            ),
            expected_outputs=(),
        ),
        PipelineStepSpec(
            step="concurrent",
            title="Run concurrent probe",
            cmd=(
                "bash",
                "scripts/channel/test-omni-agent-concurrent-sessions.sh",
                "--max-wait",
                str(cfg.max_wait),
            ),
            expected_outputs=(),
        ),
        PipelineStepSpec(
            step="session_matrix",
            title="Run session matrix",
            cmd=tuple(session_matrix_cmd),
            expected_outputs=(Path(default_matrix_json), Path(default_matrix_markdown)),
        ),
    ]
