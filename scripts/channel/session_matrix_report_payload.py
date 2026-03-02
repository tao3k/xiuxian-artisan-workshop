#!/usr/bin/env python3
"""Payload building helpers for session-matrix reports."""

from __future__ import annotations

import time
from dataclasses import asdict
from datetime import UTC, datetime
from typing import Any


def build_report(
    cfg: Any,
    results: list[Any],
    started_dt: datetime,
    started_mono: float,
) -> dict[str, object]:
    """Build structured report payload."""
    finished_dt = datetime.now(UTC)
    duration_ms = int((time.monotonic() - started_mono) * 1000)
    passed = sum(1 for result in results if result.passed)
    failed = len(results) - passed
    return {
        "started_at": started_dt.isoformat(),
        "finished_at": finished_dt.isoformat(),
        "duration_ms": duration_ms,
        "overall_passed": failed == 0 and len(results) > 0,
        "summary": {"total": len(results), "passed": passed, "failed": failed},
        "config": {
            "webhook_url": cfg.webhook_url,
            "log_file": str(cfg.log_file),
            "chat_id": cfg.chat_id,
            "chat_b": cfg.chat_b,
            "chat_c": cfg.chat_c,
            "user_a": cfg.user_a,
            "user_b": cfg.user_b,
            "user_c": cfg.user_c,
            "thread_a": cfg.thread_a,
            "thread_b": cfg.thread_b,
            "thread_c": cfg.thread_c,
            "mixed_plain_prompt": cfg.mixed_plain_prompt,
            "forbid_log_regexes": list(cfg.forbid_log_regexes),
        },
        "steps": [asdict(result) for result in results],
    }
