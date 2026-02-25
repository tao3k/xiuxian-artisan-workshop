#!/usr/bin/env python3
"""Health probe and metrics helpers for MCP startup stress runs."""

from __future__ import annotations

import statistics
import time
import urllib.error
import urllib.request
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Iterable


def check_health(url: str, timeout_secs: float = 2.0) -> tuple[bool, str]:
    """Issue one HTTP health check request."""
    request = urllib.request.Request(url=url, method="GET")
    try:
        with urllib.request.urlopen(request, timeout=timeout_secs) as response:
            body = response.read().decode("utf-8", errors="replace")
            return True, f"status={response.status} body={body[:180]}"
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf-8", errors="replace")
        return False, f"http_error={error.code} body={body[:180]}"
    except Exception as error:
        return False, f"error={error}"


def classify_reason(
    *,
    ready_seen: bool,
    handshake_timeout_seen: bool,
    connect_failed_seen: bool,
    process_exited: bool,
    timed_out: bool,
) -> str:
    """Classify probe failure reason from runtime observations."""
    if ready_seen:
        return "ok"
    if handshake_timeout_seen:
        return "handshake_timeout"
    if connect_failed_seen:
        return "connect_failed"
    if timed_out:
        return "startup_timeout"
    if process_exited:
        return "process_exited_before_ready"
    return "unknown"


def p95(values: list[float]) -> float:
    """Compute p95 using index-based percentile for small sample stability."""
    if not values:
        return 0.0
    if len(values) == 1:
        return float(values[0])
    sorted_values = sorted(values)
    index = int(0.95 * (len(sorted_values) - 1))
    return float(sorted_values[index])


def summarize_health_samples(samples: Iterable[Any]) -> dict[str, object]:
    """Summarize health samples into aggregate metrics."""
    rows = list(samples)
    total = len(rows)
    ok = sum(1 for row in rows if row.ok)
    failed = total - ok
    latencies = [row.latency_ms for row in rows if row.ok]
    error_counts: dict[str, int] = {}
    for row in rows:
        if row.ok:
            continue
        key = row.detail[:120]
        error_counts[key] = error_counts.get(key, 0) + 1
    top_errors = sorted(error_counts.items(), key=lambda item: item[1], reverse=True)[:5]
    return {
        "health_samples_total": total,
        "health_samples_ok": ok,
        "health_samples_failed": failed,
        "health_failure_rate": (failed / total) if total else 0.0,
        "health_avg_latency_ms": statistics.fmean(latencies) if latencies else 0.0,
        "health_p95_latency_ms": p95(latencies),
        "health_max_latency_ms": max(latencies) if latencies else 0.0,
        "health_error_top": [
            {"detail": detail, "count": count} for detail, count in top_errors if detail.strip()
        ],
    }


def collect_health_sample(url: str, timeout_secs: float, *, health_sample_cls: Any) -> Any:
    """Collect one typed health sample."""
    started = time.monotonic()
    ok, detail = check_health(url, timeout_secs=timeout_secs)
    latency_ms = (time.monotonic() - started) * 1000.0
    return health_sample_cls(ok=ok, latency_ms=latency_ms, detail=detail)
