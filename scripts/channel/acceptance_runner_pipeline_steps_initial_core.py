#!/usr/bin/env python3
"""Core builder for initial acceptance-runner pipeline step specs."""

from __future__ import annotations

from typing import Any

from acceptance_runner_pipeline_steps_initial_matrix import (
    build_session_matrix_cmd,
    build_tail_steps,
)
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

    session_matrix_cmd = build_session_matrix_cmd(
        cfg,
        default_matrix_json=default_matrix_json,
        default_matrix_markdown=default_matrix_markdown,
    )
    steps.extend(
        build_tail_steps(
            cfg=cfg,
            session_matrix_cmd=session_matrix_cmd,
            default_matrix_json=default_matrix_json,
            default_matrix_markdown=default_matrix_markdown,
        )
    )
    return steps
