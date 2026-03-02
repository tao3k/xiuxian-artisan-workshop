#!/usr/bin/env python3
"""Compatibility facade for command event probe reporting helpers."""

from __future__ import annotations

from typing import Any

from command_events_report_markdown import render_markdown as _render_markdown_impl
from command_events_report_output import write_outputs as _write_outputs_impl
from command_events_report_payload import build_report as _build_report_impl


def render_markdown(report: dict[str, object]) -> str:
    """Render a markdown report from structured command-events payload."""
    return _render_markdown_impl(report)


def build_report(**kwargs: Any) -> dict[str, object]:
    """Build structured report payload from probe attempts and run config."""
    return _build_report_impl(**kwargs)


def write_outputs(report: dict[str, object], output_json: Any, output_markdown: Any) -> None:
    """Write JSON and Markdown reports to output paths."""
    _write_outputs_impl(report, output_json, output_markdown)
