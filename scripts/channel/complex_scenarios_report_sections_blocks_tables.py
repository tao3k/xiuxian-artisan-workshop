#!/usr/bin/env python3
"""Compatibility facade for complex scenario report table builders."""

from __future__ import annotations

from complex_scenarios_report_sections_blocks_tables_core import (
    append_memory_adaptation as _append_memory_adaptation_impl,
)
from complex_scenarios_report_sections_blocks_tables_core import (
    append_natural_language_trace as _append_natural_language_trace_impl,
)
from complex_scenarios_report_sections_blocks_tables_core import (
    append_step_table as _append_step_table_impl,
)
from complex_scenarios_report_sections_blocks_tables_diagnostics import (
    append_failure_tails as _append_failure_tails_impl,
)
from complex_scenarios_report_sections_blocks_tables_diagnostics import (
    append_mcp_diagnostics as _append_mcp_diagnostics_impl,
)


def append_step_table(lines: list[str], scenario: dict[str, object]) -> None:
    """Append per-step status table."""
    _append_step_table_impl(lines, scenario)


def append_natural_language_trace(lines: list[str], scenario: dict[str, object]) -> None:
    """Append natural-language prompt/reply trace table."""
    _append_natural_language_trace_impl(lines, scenario)


def append_memory_adaptation(lines: list[str], scenario: dict[str, object]) -> None:
    """Append memory adaptation evidence table."""
    _append_memory_adaptation_impl(lines, scenario)


def append_mcp_diagnostics(lines: list[str], scenario: dict[str, object]) -> None:
    """Append MCP event diagnostics table."""
    _append_mcp_diagnostics_impl(lines, scenario)


def append_failure_tails(lines: list[str], scenario: dict[str, object]) -> None:
    """Append stderr/stdout tails for failed steps."""
    _append_failure_tails_impl(lines, scenario)
