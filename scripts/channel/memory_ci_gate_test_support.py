#!/usr/bin/env python3
"""Compatibility facade for memory CI gate shared test helpers."""

from __future__ import annotations

from typing import Any

from memory_ci_gate_test_support_fixture import build_cfg as _build_cfg_impl
from memory_ci_gate_test_support_reports import passing_report as _passing_report_impl
from memory_ci_gate_test_support_reports import write_report as _write_report_impl


def build_cfg(tmp_path: Any) -> Any:
    """Build baseline nightly GateConfig for tests."""
    return _build_cfg_impl(tmp_path)


def write_report(cfg: Any, payload: dict[str, object]) -> None:
    """Write JSON report at session matrix report path."""
    _write_report_impl(cfg, payload)


def passing_report(cfg: Any) -> dict[str, object]:
    """Build passing session matrix report payload."""
    return _passing_report_impl(cfg)
