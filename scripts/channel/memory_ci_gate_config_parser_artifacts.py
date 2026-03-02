#!/usr/bin/env python3
"""Compatibility facade for memory CI gate artifact argument sections."""

from __future__ import annotations

from typing import Any

from memory_ci_gate_config_parser_artifacts_cross_group import add_cross_group_args
from memory_ci_gate_config_parser_artifacts_reports import add_report_path_args


def add_artifact_args(parser: Any, *, project_root: Any) -> None:
    """Register artifact and report path arguments."""
    add_report_path_args(parser)
    add_cross_group_args(parser, project_root=project_root)
