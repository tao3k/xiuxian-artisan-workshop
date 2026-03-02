#!/usr/bin/env python3
"""Report writers/printing for memory CI gate triage."""

from __future__ import annotations

import importlib
from typing import Any

import memory_ci_gate_triage_core as core

_reporting_module = importlib.import_module("memory_ci_gate_triage_reporting")


def build_gate_failure_triage_payload(
    cfg: Any,
    *,
    error: Exception,
    category: str,
    summary: str,
    repro_commands: list[str],
    read_tail_fn: Any,
    shell_quote_command_fn: Any,
) -> dict[str, object]:
    """Build triage payload before writing markdown/json outputs."""
    return _reporting_module.build_gate_failure_triage_payload(
        cfg,
        error=error,
        category=category,
        summary=summary,
        repro_commands=repro_commands,
        read_tail_fn=read_tail_fn,
        shell_quote_command_fn=shell_quote_command_fn,
        artifact_rows_fn=core.artifact_rows,
        is_gate_step_error_fn=core.is_gate_step_error,
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
        default_gate_failure_report_base_path_fn=core.default_gate_failure_report_base_path,
        build_gate_failure_triage_payload_fn=build_gate_failure_triage_payload,
        is_gate_step_error_fn=core.is_gate_step_error,
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
        default_gate_failure_report_base_path_fn=core.default_gate_failure_report_base_path,
        build_gate_failure_triage_payload_fn=build_gate_failure_triage_payload,
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
        build_gate_failure_repro_commands_fn=core.build_gate_failure_repro_commands,
        default_gate_failure_report_base_path_fn=core.default_gate_failure_report_base_path,
        write_gate_failure_triage_report_fn=write_gate_failure_triage_report,
        write_gate_failure_triage_json_report_fn=write_gate_failure_triage_json_report,
    )
