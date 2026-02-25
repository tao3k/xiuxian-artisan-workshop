#!/usr/bin/env python3
"""Runner config datamodel for complex scenarios."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

    from complex_scenarios_models_core import (
        ComplexityRequirement,
        QualityRequirement,
        SessionIdentity,
    )


@dataclass(frozen=True)
class RunnerConfig:
    dataset_path: Path
    scenario_id: str | None
    blackbox_script: Path
    webhook_url: str
    log_file: Path
    username: str | None
    secret_token: str | None
    max_wait: int
    max_idle_secs: int
    max_parallel: int
    execute_wave_parallel: bool
    runtime_partition_mode: str | None
    sessions: tuple[SessionIdentity, ...]
    output_json: Path
    output_markdown: Path
    forbid_log_regexes: tuple[str, ...]
    global_requirement: ComplexityRequirement
    global_quality_requirement: QualityRequirement
