#!/usr/bin/env python3
"""Health preflight and background sampling helpers for MCP startup stress flow."""

from __future__ import annotations

import threading
from typing import Any


def prepare_health_preflight(cfg: Any, *, check_health_fn: Any) -> dict[str, object] | None:
    """Run optional health preflight and enforce strict gate when configured."""
    if not cfg.health_url:
        return None
    ok, detail = check_health_fn(cfg.health_url)
    preflight = {"url": cfg.health_url, "ok": ok, "detail": detail}
    if cfg.strict_health_check and not ok:
        raise RuntimeError(f"health check failed before stress: {detail}")
    return preflight


def start_health_sampler(
    cfg: Any,
    *,
    health_sample_cls: Any,
    collect_health_sample_fn: Any,
    health_samples: list[Any],
    health_samples_lock: threading.Lock,
    health_stop: threading.Event,
) -> threading.Thread | None:
    """Start optional background health sampler thread."""
    if not cfg.health_url or cfg.health_probe_interval_secs <= 0:
        return None

    health_url = cfg.health_url

    def _health_loop() -> None:
        assert health_url is not None
        while not health_stop.is_set():
            sample = collect_health_sample_fn(
                health_url,
                cfg.health_probe_timeout_secs,
                health_sample_cls=health_sample_cls,
            )
            with health_samples_lock:
                health_samples.append(sample)
            if cfg.health_probe_interval_secs <= 0:
                return
            health_stop.wait(cfg.health_probe_interval_secs)

    health_thread = threading.Thread(target=_health_loop, daemon=True)
    health_thread.start()
    return health_thread


def stop_health_sampler(
    cfg: Any,
    *,
    health_stop: threading.Event,
    health_thread: threading.Thread | None,
) -> None:
    """Stop sampler thread and wait bounded time for join."""
    health_stop.set()
    if health_thread is not None:
        health_thread.join(timeout=max(1.0, cfg.health_probe_timeout_secs + 1.0))
