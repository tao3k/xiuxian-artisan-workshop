#!/usr/bin/env python3
"""Section block builders for complex scenario markdown reports."""

from __future__ import annotations

from complex_scenarios_report_sections_blocks_summary import append_scenario_header
from complex_scenarios_report_sections_blocks_tables import (
    append_failure_tails,
    append_mcp_diagnostics,
    append_memory_adaptation,
    append_natural_language_trace,
    append_step_table,
)

__all__ = [
    "append_failure_tails",
    "append_mcp_diagnostics",
    "append_memory_adaptation",
    "append_natural_language_trace",
    "append_scenario_header",
    "append_step_table",
]
