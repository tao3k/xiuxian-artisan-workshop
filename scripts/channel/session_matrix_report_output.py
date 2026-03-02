#!/usr/bin/env python3
"""Output writing helpers for session-matrix reports."""

from __future__ import annotations

import json
from typing import Any

from session_matrix_report_markdown import render_markdown


def write_outputs(report: dict[str, object], output_json: Any, output_markdown: Any) -> None:
    """Write JSON + Markdown artifacts."""
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_markdown.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    output_markdown.write_text(render_markdown(report), encoding="utf-8")
