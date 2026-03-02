#!/usr/bin/env python3
"""Compatibility facade for session-matrix report helpers."""

from __future__ import annotations

from typing import Any

from session_matrix_report_markdown import render_markdown as _render_markdown_impl
from session_matrix_report_output import write_outputs as _write_outputs_impl
from session_matrix_report_payload import build_report as _build_report_impl


def render_markdown(report: dict[str, object]) -> str:
    """Render markdown report for session matrix runs."""
    return _render_markdown_impl(report)


def build_report(
    cfg: Any,
    results: list[Any],
    started_dt: Any,
    started_mono: float,
) -> dict[str, object]:
    """Build structured report payload."""
    return _build_report_impl(cfg, results, started_dt, started_mono)


def write_outputs(report: dict[str, object], output_json: Any, output_markdown: Any) -> None:
    """Write JSON + Markdown artifacts."""
    _write_outputs_impl(report, output_json, output_markdown)
