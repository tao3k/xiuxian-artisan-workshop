"""MCP tools/list concurrency benchmark snapshot helpers (YAML)."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import UTC, datetime
from typing import TYPE_CHECKING, Any

import yaml

from omni.foundation.config.skills import SKILLS_DIR

if TYPE_CHECKING:
    from pathlib import Path

DEFAULT_SNAPSHOT_SCHEMA = "omni.agent.mcp_tools_list_snapshot.v1"
DEFAULT_REGRESSION_FACTOR = 2.0
DEFAULT_MIN_REGRESSION_DELTA_MS = 40.0


@dataclass(frozen=True, slots=True)
class McpToolsListSnapshotAnomaly:
    """One observed latency regression against a tools/list snapshot baseline."""

    target: str
    concurrency: int
    metric: str
    baseline_ms: float
    observed_ms: float
    threshold_ms: float
    regression_factor: float
    min_regression_delta_ms: float

    @property
    def delta_ms(self) -> float:
        return float(self.observed_ms - self.baseline_ms)

    @property
    def ratio(self) -> float:
        if self.baseline_ms <= 0:
            return 0.0
        return float(self.observed_ms / self.baseline_ms)

    def to_record(self) -> dict[str, Any]:
        """Serialize anomaly to JSON/YAML-friendly record."""
        return {
            "target": self.target,
            "concurrency": int(self.concurrency),
            "metric": self.metric,
            "baseline_ms": round(self.baseline_ms, 3),
            "observed_ms": round(self.observed_ms, 3),
            "threshold_ms": round(self.threshold_ms, 3),
            "delta_ms": round(self.delta_ms, 3),
            "ratio": round(self.ratio, 3),
            "regression_factor": round(self.regression_factor, 3),
            "min_regression_delta_ms": round(self.min_regression_delta_ms, 3),
        }


def default_mcp_tools_list_snapshot_path() -> Path:
    """Return default snapshot path under ``SKILLS_DIR``."""
    return SKILLS_DIR() / "_snapshots" / "benchmark" / "mcp_tools_list.yaml"


def load_mcp_tools_list_snapshot(path: Path) -> dict[str, Any] | None:
    """Load YAML snapshot file if present and valid."""
    if not path.exists():
        return None
    raw = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        return None
    return raw


def save_mcp_tools_list_snapshot(path: Path, payload: dict[str, Any]) -> Path:
    """Persist snapshot payload as YAML."""
    path.parent.mkdir(parents=True, exist_ok=True)
    text = yaml.safe_dump(payload, sort_keys=False, allow_unicode=False)
    path.write_text(text, encoding="utf-8")
    return path


def _as_float(value: Any) -> float | None:
    if isinstance(value, bool):
        return None
    if isinstance(value, int | float):
        return float(value)
    return None


def _as_int(value: Any) -> int | None:
    if isinstance(value, bool):
        return None
    if isinstance(value, int):
        return int(value)
    if isinstance(value, float):
        if value.is_integer():
            return int(value)
        return None
    return None


def _positive_float(value: Any, fallback: float) -> float:
    parsed = _as_float(value)
    if parsed is None or parsed <= 0:
        return float(fallback)
    return float(parsed)


def _optional_positive_float(value: Any) -> float | None:
    parsed = _as_float(value)
    if parsed is None or parsed <= 0:
        return None
    return float(parsed)


def _point_by_concurrency(points: Any) -> dict[int, dict[str, Any]]:
    if not isinstance(points, list):
        return {}
    output: dict[int, dict[str, Any]] = {}
    for raw_point in points:
        if not isinstance(raw_point, dict):
            continue
        concurrency = _as_int(raw_point.get("concurrency"))
        if concurrency is None or concurrency <= 0:
            continue
        output[concurrency] = raw_point
    return output


def build_mcp_tools_list_snapshot_payload(
    *,
    summary: dict[str, Any],
    previous: dict[str, Any] | None = None,
    alpha: float = 0.35,
    default_regression_factor: float = DEFAULT_REGRESSION_FACTOR,
    default_min_regression_delta_ms: float = DEFAULT_MIN_REGRESSION_DELTA_MS,
) -> dict[str, Any]:
    """Build YAML snapshot payload from one tools/list concurrency sweep summary."""
    clamped_alpha = max(0.0, min(1.0, float(alpha)))
    previous_targets = previous.get("targets") if isinstance(previous, dict) else None
    if not isinstance(previous_targets, dict):
        previous_targets = {}

    targets_out: dict[str, dict[str, Any]] = {}
    for target_name, target_payload in previous_targets.items():
        if isinstance(target_name, str) and isinstance(target_payload, dict):
            targets_out[target_name] = dict(target_payload)

    target = str(summary.get("base_url") or "").strip()
    if target:
        previous_target = previous_targets.get(target)
        if not isinstance(previous_target, dict):
            previous_target = {}
        previous_points = previous_target.get("points")
        if not isinstance(previous_points, dict):
            previous_points = {}

        points_out: dict[str, dict[str, Any]] = {}
        for concurrency, point in _point_by_concurrency(summary.get("points")).items():
            p95_ms = _as_float(point.get("p95_ms"))
            p99_ms = _as_float(point.get("p99_ms"))
            if p95_ms is None or p95_ms < 0:
                continue
            if p99_ms is None or p99_ms < 0:
                continue
            prior = previous_points.get(str(concurrency))
            if not isinstance(prior, dict):
                prior = {}

            prior_p95 = _as_float(prior.get("baseline_p95_ms"))
            if prior_p95 is None or prior_p95 <= 0:
                baseline_p95 = float(p95_ms)
            else:
                baseline_p95 = (prior_p95 * (1.0 - clamped_alpha)) + (float(p95_ms) * clamped_alpha)

            prior_p99 = _as_float(prior.get("baseline_p99_ms"))
            if prior_p99 is None or prior_p99 <= 0:
                baseline_p99 = float(p99_ms)
            else:
                baseline_p99 = (prior_p99 * (1.0 - clamped_alpha)) + (float(p99_ms) * clamped_alpha)

            new_point: dict[str, Any] = {
                "baseline_p95_ms": round(baseline_p95, 3),
                "baseline_p99_ms": round(baseline_p99, 3),
                "last_p95_ms": round(float(p95_ms), 3),
                "last_p99_ms": round(float(p99_ms), 3),
            }
            last_rps = _as_float(point.get("rps"))
            if last_rps is not None and last_rps >= 0:
                new_point["last_rps"] = round(float(last_rps), 3)
            last_errors = _as_int(point.get("errors"))
            if last_errors is not None and last_errors >= 0:
                new_point["last_errors"] = int(last_errors)

            for key in ("regression_factor", "min_regression_delta_ms"):
                if key in prior:
                    new_point[key] = prior[key]

            points_out[str(concurrency)] = new_point

        recommendation = summary.get("recommendation")
        recommendation_obj = recommendation if isinstance(recommendation, dict) else {}
        summary_meta = summary.get("summary")
        summary_obj = summary_meta if isinstance(summary_meta, dict) else {}
        target_entry: dict[str, Any] = {
            "recommended_concurrency": recommendation_obj.get("recommended_concurrency"),
            "knee_concurrency": recommendation_obj.get("knee_concurrency"),
            "mean_rps": summary_obj.get("mean_rps"),
            "points": dict(sorted(points_out.items(), key=lambda item: int(item[0]))),
        }
        if isinstance(previous_target, dict):
            for key in ("regression_factor", "min_regression_delta_ms"):
                if key in previous_target:
                    target_entry[key] = previous_target[key]
        targets_out[target] = target_entry

    return {
        "schema": DEFAULT_SNAPSHOT_SCHEMA,
        "updated_at_utc": datetime.now(UTC).isoformat(),
        "benchmark": {
            "total_per_point": int(_as_int(summary.get("total_per_point")) or 0),
            "concurrency_values": list(summary.get("concurrency_values") or []),
            "p95_slo_ms": float(_as_float((summary.get("slo") or {}).get("p95_ms")) or 0.0),
            "p99_slo_ms": float(_as_float((summary.get("slo") or {}).get("p99_ms")) or 0.0),
        },
        "defaults": {
            "regression_factor": float(default_regression_factor),
            "min_regression_delta_ms": float(default_min_regression_delta_ms),
        },
        "targets": dict(sorted(targets_out.items(), key=lambda item: item[0])),
    }


def detect_mcp_tools_list_snapshot_anomalies(
    *,
    summary: dict[str, Any],
    snapshot: dict[str, Any] | None,
    default_regression_factor: float = DEFAULT_REGRESSION_FACTOR,
    default_min_regression_delta_ms: float = DEFAULT_MIN_REGRESSION_DELTA_MS,
) -> list[McpToolsListSnapshotAnomaly]:
    """Detect latency regressions relative to snapshot baselines for one target."""
    if not isinstance(snapshot, dict):
        return []
    target = str(summary.get("base_url") or "").strip()
    if not target:
        return []

    snapshot_targets = snapshot.get("targets")
    if not isinstance(snapshot_targets, dict):
        return []
    tracked_target = snapshot_targets.get(target)
    if not isinstance(tracked_target, dict):
        return []
    tracked_points = tracked_target.get("points")
    if not isinstance(tracked_points, dict):
        return []

    defaults = snapshot.get("defaults")
    defaults_obj = defaults if isinstance(defaults, dict) else {}
    global_factor = _positive_float(
        defaults_obj.get("regression_factor"),
        fallback=default_regression_factor,
    )
    global_delta = _positive_float(
        defaults_obj.get("min_regression_delta_ms"),
        fallback=default_min_regression_delta_ms,
    )
    target_factor = _positive_float(tracked_target.get("regression_factor"), fallback=global_factor)
    target_delta = _positive_float(
        tracked_target.get("min_regression_delta_ms"),
        fallback=global_delta,
    )
    snapshot_benchmark = snapshot.get("benchmark")
    benchmark_obj = snapshot_benchmark if isinstance(snapshot_benchmark, dict) else {}
    snapshot_p95_slo_ms = _optional_positive_float(benchmark_obj.get("p95_slo_ms"))
    snapshot_p99_slo_ms = _optional_positive_float(benchmark_obj.get("p99_slo_ms"))

    anomalies: list[McpToolsListSnapshotAnomaly] = []
    for concurrency, point in _point_by_concurrency(summary.get("points")).items():
        tracked_point = tracked_points.get(str(concurrency))
        if not isinstance(tracked_point, dict):
            continue

        baseline_p95_ms = _as_float(tracked_point.get("baseline_p95_ms"))
        baseline_p99_ms = _as_float(tracked_point.get("baseline_p99_ms"))
        # Gate only SLO-feasible baseline points to avoid noisy blocking on
        # high-concurrency regions that are intentionally outside the latency budget.
        if (
            snapshot_p95_slo_ms is not None
            and baseline_p95_ms is not None
            and baseline_p95_ms > snapshot_p95_slo_ms
        ):
            continue
        if (
            snapshot_p99_slo_ms is not None
            and baseline_p99_ms is not None
            and baseline_p99_ms > snapshot_p99_slo_ms
        ):
            continue

        factor = _positive_float(tracked_point.get("regression_factor"), fallback=target_factor)
        delta = _positive_float(
            tracked_point.get("min_regression_delta_ms"),
            fallback=target_delta,
        )

        observed_metrics = {
            "p95_ms": _as_float(point.get("p95_ms")),
            "p99_ms": _as_float(point.get("p99_ms")),
        }
        baseline_metrics = {
            "p95_ms": baseline_p95_ms,
            "p99_ms": baseline_p99_ms,
        }

        for metric, observed_ms in observed_metrics.items():
            if observed_ms is None or observed_ms < 0:
                continue
            baseline_ms = baseline_metrics.get(metric)
            if baseline_ms is None or baseline_ms <= 0:
                continue
            threshold_ms = max(float(baseline_ms) * factor, float(baseline_ms) + delta)
            if float(observed_ms) > threshold_ms:
                anomalies.append(
                    McpToolsListSnapshotAnomaly(
                        target=target,
                        concurrency=int(concurrency),
                        metric=metric,
                        baseline_ms=float(baseline_ms),
                        observed_ms=float(observed_ms),
                        threshold_ms=float(threshold_ms),
                        regression_factor=float(factor),
                        min_regression_delta_ms=float(delta),
                    )
                )

    anomalies.sort(key=lambda item: (item.concurrency, item.metric))
    return anomalies


__all__ = [
    "DEFAULT_MIN_REGRESSION_DELTA_MS",
    "DEFAULT_REGRESSION_FACTOR",
    "DEFAULT_SNAPSHOT_SCHEMA",
    "McpToolsListSnapshotAnomaly",
    "build_mcp_tools_list_snapshot_payload",
    "default_mcp_tools_list_snapshot_path",
    "detect_mcp_tools_list_snapshot_anomalies",
    "load_mcp_tools_list_snapshot",
    "save_mcp_tools_list_snapshot",
]
