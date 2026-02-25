#!/usr/bin/env python3
"""Step model definitions for acceptance runner pipeline planning."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


@dataclass(frozen=True)
class PipelineStepSpec:
    """One acceptance pipeline command invocation."""

    step: str
    title: str
    cmd: tuple[str, ...]
    expected_outputs: tuple[Path, ...]
