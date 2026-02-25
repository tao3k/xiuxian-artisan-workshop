#!/usr/bin/env python3
"""Compatibility facade for Discord ingress stress runtime helpers."""

from __future__ import annotations

from typing import Any

from discord_ingress_stress_runtime_core import (
    init_log_offset,
    p95,
    read_new_log_lines,
    utc_now,
)
from discord_ingress_stress_runtime_core import (
    next_event_id as _next_event_id,
)
from discord_ingress_stress_runtime_flow import (
    run_stress as _run_stress,
)
from discord_ingress_stress_runtime_flow import (
    run_worker_dynamic,
)
from discord_ingress_stress_runtime_http import (
    DISCORD_INGRESS_SECRET_HEADER,
    build_ingress_payload,
    post_ingress_event,
)


def _run_worker_dynamic(cfg: Any, round_index: int, worker_index: int) -> dict[str, Any]:
    return run_worker_dynamic(
        cfg,
        round_index,
        worker_index,
        next_event_id_fn=_next_event_id,
        build_ingress_payload_fn=build_ingress_payload,
        post_ingress_event_fn=post_ingress_event,
    )


def run_stress(cfg: Any, *, round_result_cls: Any) -> dict[str, object]:
    """Execute full Discord ingress stress run and return structured report."""
    return _run_stress(
        cfg,
        round_result_cls=round_result_cls,
        utc_now_fn=utc_now,
        p95_fn=p95,
        init_log_offset_fn=init_log_offset,
        read_new_log_lines_fn=read_new_log_lines,
        run_worker_fn=_run_worker_dynamic,
    )


__all__ = [
    "DISCORD_INGRESS_SECRET_HEADER",
    "_next_event_id",
    "_run_worker_dynamic",
    "build_ingress_payload",
    "init_log_offset",
    "p95",
    "post_ingress_event",
    "read_new_log_lines",
    "run_stress",
    "run_worker_dynamic",
    "utc_now",
]
