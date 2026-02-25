#!/usr/bin/env python3
"""Failure triage helpers for omni-agent memory CI gate."""

from __future__ import annotations

import importlib
from typing import Any

_reporting_module = importlib.import_module("memory_ci_gate_triage_reporting")
_classification_module = importlib.import_module("memory_ci_gate_triage_classification")
_repro_module = importlib.import_module("memory_ci_gate_triage_repro")
_artifacts_module = importlib.import_module("memory_ci_gate_triage_artifacts")


def _is_gate_step_error(error: Exception) -> bool:
    return _classification_module.is_gate_step_error(error)


def classify_gate_failure(error: Exception) -> tuple[str, str]:
    """Classify a gate failure into category + short summary."""
    return _classification_module.classify_gate_failure(error)


def artifact_rows(cfg: Any) -> list[tuple[str, Any]]:
    """Return all known gate artifact paths."""
    return _artifacts_module.artifact_rows(cfg)


def default_gate_failure_report_base_path(
    cfg: Any, *, stamp_ms: int | None = None
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
        is_gate_step_error_fn=_is_gate_step_error,
    )


def _build_gate_failure_triage_payload(
    cfg: Any,
    *,
    error: Exception,
    category: str,
    summary: str,
    repro_commands: list[str],
    read_tail_fn: Any,
    shell_quote_command_fn: Any,
) -> dict[str, object]:
    return _reporting_module.build_gate_failure_triage_payload(
        cfg,
        error=error,
        category=category,
        summary=summary,
        repro_commands=repro_commands,
        read_tail_fn=read_tail_fn,
        shell_quote_command_fn=shell_quote_command_fn,
        artifact_rows_fn=artifact_rows,
        is_gate_step_error_fn=_is_gate_step_error,
    )


def write_gate_failure_triage_report(
    cfg: Any,
    *,
    error: Exception,
    category: str,
    summary: str,
    repro_commands: list[str],
    read_tail_fn: Any,
    shell_quote_command_fn: Any,
    report_path: Any | None = None,
) -> Any:
    """Write markdown triage report and return the report path."""
    return _reporting_module.write_gate_failure_triage_report(
        cfg,
        error=error,
        category=category,
        summary=summary,
        repro_commands=repro_commands,
        read_tail_fn=read_tail_fn,
        shell_quote_command_fn=shell_quote_command_fn,
        report_path=report_path,
        default_gate_failure_report_base_path_fn=default_gate_failure_report_base_path,
        build_gate_failure_triage_payload_fn=_build_gate_failure_triage_payload,
        is_gate_step_error_fn=_is_gate_step_error,
    )


def write_gate_failure_triage_json_report(
    cfg: Any,
    *,
    error: Exception,
    category: str,
    summary: str,
    repro_commands: list[str],
    read_tail_fn: Any,
    shell_quote_command_fn: Any,
    report_path: Any | None = None,
) -> Any:
    """Write JSON triage report and return the report path."""
    return _reporting_module.write_gate_failure_triage_json_report(
        cfg,
        error=error,
        category=category,
        summary=summary,
        repro_commands=repro_commands,
        read_tail_fn=read_tail_fn,
        shell_quote_command_fn=shell_quote_command_fn,
        report_path=report_path,
        default_gate_failure_report_base_path_fn=default_gate_failure_report_base_path,
        build_gate_failure_triage_payload_fn=_build_gate_failure_triage_payload,
    )


def print_gate_failure_triage(
    cfg: Any,
    error: Exception,
    *,
    classify_failure_fn: Any,
    read_tail_fn: Any,
    shell_quote_command_fn: Any,
) -> Any:
    """Emit failure summary to stderr and persist markdown/json reports."""
    return _reporting_module.print_gate_failure_triage(
        cfg,
        error=error,
        classify_failure_fn=classify_failure_fn,
        read_tail_fn=read_tail_fn,
        shell_quote_command_fn=shell_quote_command_fn,
        build_gate_failure_repro_commands_fn=build_gate_failure_repro_commands,
        default_gate_failure_report_base_path_fn=default_gate_failure_report_base_path,
        write_gate_failure_triage_report_fn=write_gate_failure_triage_report,
        write_gate_failure_triage_json_report_fn=write_gate_failure_triage_json_report,
    )
