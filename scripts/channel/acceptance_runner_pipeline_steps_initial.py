#!/usr/bin/env python3
"""Initial and session-baseline pipeline steps for acceptance runner."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from acceptance_runner_pipeline_steps_models import PipelineStepSpec


def build_initial_step_specs(
    cfg: Any,
    *,
    default_matrix_json: str,
    default_matrix_markdown: str,
    python_executable: str,
) -> list[PipelineStepSpec]:
    """Build capture, command-event, dedup, concurrent, and session-matrix steps."""
    steps: list[PipelineStepSpec] = [
        PipelineStepSpec(
            step="capture_groups",
            title="Capture Telegram group profile",
            cmd=(
                python_executable,
                "scripts/channel/capture_telegram_group_profile.py",
                "--titles",
                cfg.titles,
                "--log-file",
                str(cfg.log_file),
                "--output-json",
                str(cfg.group_profile_json),
                "--output-env",
                str(cfg.group_profile_env),
            ),
            expected_outputs=(cfg.group_profile_json, cfg.group_profile_env),
        ),
        PipelineStepSpec(
            step="command_events",
            title="Run command event probes",
            cmd=(
                "bash",
                "scripts/channel/test-omni-agent-command-events.sh",
                "--max-wait",
                str(cfg.max_wait),
                "--max-idle-secs",
                str(cfg.max_idle_secs),
            ),
            expected_outputs=(),
        ),
    ]

    if cfg.group_thread_id is not None and cfg.group_thread_id_b is not None:
        steps.append(
            PipelineStepSpec(
                step="command_events_topic_isolation",
                title="Run command event topic-isolation probes",
                cmd=(
                    "bash",
                    "scripts/channel/test-omni-agent-command-events.sh",
                    "--suite",
                    "admin",
                    "--assert-admin-topic-isolation",
                    "--group-thread-id",
                    str(cfg.group_thread_id),
                    "--group-thread-id-b",
                    str(cfg.group_thread_id_b),
                    "--max-wait",
                    str(cfg.max_wait),
                    "--max-idle-secs",
                    str(cfg.max_idle_secs),
                ),
                expected_outputs=(),
            )
        )

    session_matrix_cmd = [
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
        session_matrix_cmd.extend(["--thread-a", str(cfg.group_thread_id)])
    if cfg.group_thread_id_b is not None:
        session_matrix_cmd.extend(["--thread-b", str(cfg.group_thread_id_b)])

    steps.extend(
        [
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
    )
    return steps
