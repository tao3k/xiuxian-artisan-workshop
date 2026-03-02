#!/usr/bin/env python3
"""Core triage classification/artifact/repro helpers."""

from __future__ import annotations

import importlib
from typing import Any

_classification_module = importlib.import_module("memory_ci_gate_triage_classification")
_repro_module = importlib.import_module("memory_ci_gate_triage_repro")
_artifacts_module = importlib.import_module("memory_ci_gate_triage_artifacts")


def is_gate_step_error(error: Exception) -> bool:
    """Return whether an error is a structured gate-step error."""
    return _classification_module.is_gate_step_error(error)


def classify_gate_failure(error: Exception) -> tuple[str, str]:
    """Classify a gate failure into category + short summary."""
    return _classification_module.classify_gate_failure(error)


def artifact_rows(cfg: Any) -> list[tuple[str, Any]]:
    """Return all known gate artifact paths."""
    return _artifacts_module.artifact_rows(cfg)


def default_gate_failure_report_base_path(
    cfg: Any,
    *,
    stamp_ms: int | None = None,
) -> tuple[Any, int]:
    """Resolve default report base path + timestamp."""
    return _artifacts_module.default_gate_failure_report_base_path(cfg, stamp_ms=stamp_ms)


def build_gate_failure_repro_commands(
    cfg: Any,
    *,
    category: str,
    error: Exception,
    shell_quote_command_fn: Any,
) -> list[str]:
    """Build deduplicated repro commands based on failure category."""
    return _repro_module.build_gate_failure_repro_commands(
        cfg,
        category=category,
        error=error,
        shell_quote_command_fn=shell_quote_command_fn,
        is_gate_step_error_fn=is_gate_step_error,
    )
