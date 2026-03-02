#!/usr/bin/env python3
"""Compare two CLI runner summary JSON artifacts and detect latency regressions."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

DIFF_SCHEMA = "omni.skills.cli_runner_summary.diff.v1"
DEFAULT_MODES = ("default_warm", "no_reuse", "default_cold")
TIMING_PHASES = ("bootstrap", "daemon_connect", "tool_exec")


def _should_skip_metric(*, mode: str, metric: str) -> bool:
    """Return whether one metric should be skipped from regression decisions."""
    # `default_cold` runs through daemon spawn path where bootstrap is coupled to
    # process/runtime startup jitter and daemon internals. Keep it visible in raw
    # artifacts but do not use it as a blocking regression comparator.
    return mode == "default_cold" and metric == "timing.bootstrap.p50_ms"


def _load_json_file(path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ValueError(f"file not found: {path}") from exc
    except json.JSONDecodeError as exc:
        raise ValueError(f"invalid JSON in {path}: {exc}") from exc
    if not isinstance(payload, dict):
        raise ValueError(f"expected JSON object in {path}")
    return payload


def _extract_cli_summary(payload: dict[str, Any]) -> dict[str, Any]:
    summary = payload.get("cli_runner_summary")
    if isinstance(summary, dict):
        return summary
    if isinstance(payload.get("profiles"), dict):
        return payload
    raise ValueError("input is not a cli_runner_summary artifact")


def _as_number(value: Any) -> float | None:
    if isinstance(value, bool):
        return None
    if isinstance(value, int | float):
        return float(value)
    return None


def _collect_metrics(summary: dict[str, Any]) -> dict[tuple[str, str, str], float]:
    metrics: dict[tuple[str, str, str], float] = {}
    profiles = summary.get("profiles")
    if not isinstance(profiles, dict):
        return metrics

    for profile_name, profile_entry in profiles.items():
        if not isinstance(profile_name, str) or not isinstance(profile_entry, dict):
            continue
        cases = profile_entry.get("cases")
        if not isinstance(cases, dict):
            continue
        for mode in DEFAULT_MODES:
            case = cases.get(mode)
            if not isinstance(case, dict):
                continue
            case_p50 = _as_number(case.get("p50_ms"))
            if case_p50 is not None:
                metrics[(profile_name, mode, "case.p50_ms")] = case_p50

            timing = case.get("timing_breakdown_ms")
            if not isinstance(timing, dict):
                continue
            for phase in TIMING_PHASES:
                phase_payload = timing.get(phase)
                if not isinstance(phase_payload, dict):
                    continue
                phase_p50 = _as_number(phase_payload.get("p50_ms"))
                if phase_p50 is not None:
                    metric_name = f"timing.{phase}.p50_ms"
                    metrics[(profile_name, mode, metric_name)] = phase_p50
    return metrics


def _build_diff(
    *,
    base_summary: dict[str, Any],
    target_summary: dict[str, Any],
    max_regression_ms: float,
    max_regression_ratio: float,
    bootstrap_max_regression_ms: float | None,
    bootstrap_max_regression_ratio: float | None,
) -> dict[str, Any]:
    base_metrics = _collect_metrics(base_summary)
    target_metrics = _collect_metrics(target_summary)
    keys = sorted(set(base_metrics) | set(target_metrics))

    comparisons: list[dict[str, Any]] = []
    missing_metrics: list[dict[str, Any]] = []
    regression_count = 0
    for profile, mode, metric in keys:
        if _should_skip_metric(mode=mode, metric=metric):
            continue
        base_value = base_metrics.get((profile, mode, metric))
        target_value = target_metrics.get((profile, mode, metric))
        if base_value is None or target_value is None:
            missing_metrics.append(
                {
                    "profile": profile,
                    "mode": mode,
                    "metric": metric,
                    "base_ms": round(base_value, 2) if isinstance(base_value, float) else None,
                    "target_ms": round(target_value, 2)
                    if isinstance(target_value, float)
                    else None,
                }
            )
            continue

        delta_ms = target_value - base_value
        ratio = (target_value / base_value) if base_value > 0 else None
        metric_max_regression_ms = max_regression_ms
        metric_max_regression_ratio = max_regression_ratio
        if metric == "timing.bootstrap.p50_ms":
            if isinstance(bootstrap_max_regression_ms, float):
                metric_max_regression_ms = bootstrap_max_regression_ms
            if isinstance(bootstrap_max_regression_ratio, float):
                metric_max_regression_ratio = bootstrap_max_regression_ratio

        regression = False
        if delta_ms > metric_max_regression_ms:
            regression = True if ratio is None else ratio > metric_max_regression_ratio
        if regression:
            regression_count += 1
        comparisons.append(
            {
                "profile": profile,
                "mode": mode,
                "metric": metric,
                "base_ms": round(base_value, 2),
                "target_ms": round(target_value, 2),
                "delta_ms": round(delta_ms, 2),
                "ratio": round(ratio, 3) if isinstance(ratio, float) else None,
                "threshold_ms": round(metric_max_regression_ms, 2),
                "threshold_ratio": round(metric_max_regression_ratio, 3),
                "regression": regression,
            }
        )

    comparisons.sort(
        key=lambda item: float(item["delta_ms"])
        if isinstance(item.get("delta_ms"), int | float)
        else 0.0,
        reverse=True,
    )
    return {
        "schema": DIFF_SCHEMA,
        "thresholds": {
            "max_regression_ms": round(float(max_regression_ms), 2),
            "max_regression_ratio": round(float(max_regression_ratio), 3),
            "bootstrap_max_regression_ms": (
                round(float(bootstrap_max_regression_ms), 2)
                if isinstance(bootstrap_max_regression_ms, float)
                else None
            ),
            "bootstrap_max_regression_ratio": (
                round(float(bootstrap_max_regression_ratio), 3)
                if isinstance(bootstrap_max_regression_ratio, float)
                else None
            ),
        },
        "comparison_count": len(comparisons),
        "regression_count": regression_count,
        "missing_metric_count": len(missing_metrics),
        "comparisons": comparisons,
        "missing_metrics": missing_metrics,
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Compare two CLI runner summary artifacts and detect regressions."
    )
    parser.add_argument("base", help="Base JSON file (baseline).")
    parser.add_argument("target", help="Target JSON file (candidate).")
    parser.add_argument(
        "--max-regression-ms",
        type=float,
        default=0.0,
        help="Allowed regression delta in ms before triggering (default: 0.0).",
    )
    parser.add_argument(
        "--max-regression-ratio",
        type=float,
        default=1.0,
        help="Allowed regression ratio before triggering (default: 1.0).",
    )
    parser.add_argument(
        "--bootstrap-max-regression-ms",
        type=float,
        default=None,
        help=(
            "Optional override for timing.bootstrap.p50_ms regression delta in ms. "
            "Unset means use --max-regression-ms."
        ),
    )
    parser.add_argument(
        "--bootstrap-max-regression-ratio",
        type=float,
        default=None,
        help=(
            "Optional override for timing.bootstrap.p50_ms regression ratio. "
            "Unset means use --max-regression-ratio."
        ),
    )
    parser.add_argument(
        "--fail-on-regression",
        action="store_true",
        help="Return exit code 1 when regressions are detected.",
    )
    args = parser.parse_args()

    base_path = Path(args.base).expanduser().resolve()
    target_path = Path(args.target).expanduser().resolve()
    try:
        base_payload = _load_json_file(base_path)
        target_payload = _load_json_file(target_path)
        base_summary = _extract_cli_summary(base_payload)
        target_summary = _extract_cli_summary(target_payload)
    except ValueError as exc:
        print(
            json.dumps(
                {
                    "schema": DIFF_SCHEMA,
                    "status": "error",
                    "error": str(exc),
                },
                ensure_ascii=False,
                indent=2,
            )
        )
        return 1

    diff = _build_diff(
        base_summary=base_summary,
        target_summary=target_summary,
        max_regression_ms=max(0.0, float(args.max_regression_ms)),
        max_regression_ratio=max(0.0, float(args.max_regression_ratio)),
        bootstrap_max_regression_ms=(
            max(0.0, float(args.bootstrap_max_regression_ms))
            if args.bootstrap_max_regression_ms is not None
            else None
        ),
        bootstrap_max_regression_ratio=(
            max(0.0, float(args.bootstrap_max_regression_ratio))
            if args.bootstrap_max_regression_ratio is not None
            else None
        ),
    )
    diff["base_file"] = str(base_path)
    diff["target_file"] = str(target_path)
    print(json.dumps(diff, ensure_ascii=False, indent=2))

    if bool(args.fail_on_regression) and int(diff.get("regression_count", 0)) > 0:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
