#!/usr/bin/env python3
"""Unit tests for memory CI gate parser construction."""

from __future__ import annotations

from typing import TYPE_CHECKING

from memory_ci_gate_config_parser import build_parser

if TYPE_CHECKING:
    from pathlib import Path


def test_build_parser_sets_project_scoped_dataset_default(tmp_path: Path) -> None:
    parser = build_parser(tmp_path)
    args = parser.parse_args([])

    assert args.profile == "quick"
    assert args.cross_group_dataset == str(
        tmp_path / "scripts" / "channel" / "fixtures" / "complex_blackbox_scenarios.json"
    )
    assert args.require_cross_group_step is True
    assert args.require_mixed_batch_steps is True


def test_build_parser_allows_disabling_cross_group_requirements(tmp_path: Path) -> None:
    parser = build_parser(tmp_path)
    args = parser.parse_args(["--no-require-cross-group-step", "--no-require-mixed-batch-steps"])

    assert args.require_cross_group_step is False
    assert args.require_mixed_batch_steps is False
