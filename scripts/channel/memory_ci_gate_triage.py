#!/usr/bin/env python3
"""Failure triage helpers for omni-agent memory CI gate."""

from __future__ import annotations

from typing import Any

import memory_ci_gate_triage_core as _core
import memory_ci_gate_triage_reports as _reports

_is_gate_step_error = _core.is_gate_step_error
classify_gate_failure = _core.classify_gate_failure
artifact_rows = _core.artifact_rows
default_gate_failure_report_base_path = _core.default_gate_failure_report_base_path
build_gate_failure_repro_commands = _core.build_gate_failure_repro_commands

_build_gate_failure_triage_payload = _reports.build_gate_failure_triage_payload
write_gate_failure_triage_report = _reports.write_gate_failure_triage_report
write_gate_failure_triage_json_report = _reports.write_gate_failure_triage_json_report
print_gate_failure_triage = _reports.print_gate_failure_triage


def _typecheck_exports(_: Any = None) -> None:
    """Keep static analyzers aware that this module intentionally re-exports symbols."""
    del _
