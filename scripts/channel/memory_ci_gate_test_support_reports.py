#!/usr/bin/env python3
"""Report payload helpers for memory CI gate tests."""

from __future__ import annotations

import json
from typing import Any


def write_report(cfg: Any, payload: dict[str, object]) -> None:
    """Write JSON report at session matrix report path."""
    cfg.session_matrix_report_json.parent.mkdir(parents=True, exist_ok=True)
    cfg.session_matrix_report_json.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )


def passing_report(cfg: Any) -> dict[str, object]:
    """Build passing session matrix report payload."""
    steps = [{"name": f"step-{index}", "passed": True} for index in range(1, 18)]
    steps.extend(
        [
            {"name": "concurrent_cross_group", "passed": True},
            {"name": "mixed_reset_session_a", "passed": True},
            {"name": "mixed_resume_status_session_b", "passed": True},
            {"name": "mixed_plain_session_c", "passed": True},
        ]
    )
    return {
        "overall_passed": True,
        "summary": {"total": len(steps), "failed": 0},
        "config": {
            "chat_id": cfg.chat_id,
            "chat_b": cfg.chat_b,
            "chat_c": cfg.chat_c,
        },
        "steps": steps,
    }
