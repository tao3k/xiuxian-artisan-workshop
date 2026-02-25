#!/usr/bin/env python3
"""Argument parser construction for memory CI gate config."""

from __future__ import annotations

import argparse
from typing import TYPE_CHECKING

from memory_ci_gate_config_parser_artifacts import add_artifact_args
from memory_ci_gate_config_parser_quality import add_quality_args
from memory_ci_gate_config_parser_runtime import add_runtime_args

if TYPE_CHECKING:
    from pathlib import Path


def build_parser(project_root: Path) -> argparse.ArgumentParser:
    """Build argparse parser for memory CI gate."""
    parser = argparse.ArgumentParser(description="Run omni-agent memory CI gate.")
    add_runtime_args(parser)
    add_artifact_args(parser, project_root=project_root)
    add_quality_args(parser)
    return parser
