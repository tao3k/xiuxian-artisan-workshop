#!/usr/bin/env python3
"""Compatibility facade for complex scenario dataset/runtime helpers."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

_dataset_module = importlib.import_module("complex_scenarios_dataset")
_runtime_module = importlib.import_module("complex_scenarios_runtime")

_required_str_field = _dataset_module.required_str_field
parse_requirement = _dataset_module.parse_requirement
parse_quality_requirement = _dataset_module.parse_quality_requirement
merge_requirements = _dataset_module.merge_requirements
merge_quality_requirements = _dataset_module.merge_quality_requirements
load_scenarios = _dataset_module.load_scenarios
select_scenarios = _dataset_module.select_scenarios

run_cmd = _runtime_module.run_cmd
extract_bot_excerpt = _runtime_module.extract_bot_excerpt
detect_memory_event_flags = _runtime_module.detect_memory_event_flags
_as_float = _runtime_module.as_float
extract_memory_metrics = _runtime_module.extract_memory_metrics
extract_mcp_metrics = _runtime_module.extract_mcp_metrics
run_step = _runtime_module.run_step
skipped_step_result = _runtime_module.skipped_step_result
run_scenario = _runtime_module.run_scenario

__all__ = [
    "_as_float",
    "_required_str_field",
    "detect_memory_event_flags",
    "extract_bot_excerpt",
    "extract_mcp_metrics",
    "extract_memory_metrics",
    "load_scenarios",
    "merge_quality_requirements",
    "merge_requirements",
    "parse_quality_requirement",
    "parse_requirement",
    "run_cmd",
    "run_scenario",
    "run_step",
    "select_scenarios",
    "skipped_step_result",
]
