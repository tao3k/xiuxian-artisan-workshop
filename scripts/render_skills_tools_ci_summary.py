#!/usr/bin/env python3
"""Render unified skills-tools CI status summary from benchmark report artifacts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

STATUS_SCHEMA = "omni.skills.tools.ci_status.v1"
COMPONENT_ORDER = (
    "remote_fetch",
    "deterministic_gate",
    "cli_diff_gate",
    "baseline",
    "network_observability",
)


def _as_int(value: Any) -> int:
    if isinstance(value, bool):
        return 0
    if isinstance(value, int):
        return value
    if isinstance(value, float):
        return int(value)
    if isinstance(value, str) and value.strip():
        try:
            return int(value.strip())
        except ValueError:
            return 0
    return 0


def _load_json(path: Path) -> tuple[dict[str, Any] | None, str | None]:
    if not path.exists():
        return None, "missing"
    try:
        parsed = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        return None, f"parse_error: {exc}"
    if not isinstance(parsed, dict):
        return None, "invalid_payload"
    return parsed, None


def _status_rank(status: str) -> int:
    normalized = status.strip().lower()
    if normalized == "error":
        return 3
    if normalized == "warn":
        return 2
    if normalized in {"skipped", "missing"}:
        return 1
    return 0


def _component_deterministic(path: Path) -> dict[str, Any]:
    payload, error = _load_json(path)
    if error is not None:
        return {"status": "missing" if error == "missing" else "error", "details": error}
    errors = payload.get("errors")
    errors_count = len(errors) if isinstance(errors, list) else 0
    snapshot = payload.get("snapshot")
    snapshot_anomaly_count = (
        _as_int(snapshot.get("anomaly_count")) if isinstance(snapshot, dict) else 0
    )
    runner_gate = payload.get("cli_runner_gate")
    ordering_violation_count = (
        _as_int(runner_gate.get("violation_count")) if isinstance(runner_gate, dict) else 0
    )
    status = (
        "ok"
        if errors_count == 0 and snapshot_anomaly_count == 0 and ordering_violation_count == 0
        else "error"
    )
    return {
        "status": status,
        "details": (
            f"errors={errors_count}, snapshot_anomalies={snapshot_anomaly_count}, "
            f"ordering_violations={ordering_violation_count}"
        ),
        "errors_count": errors_count,
        "snapshot_anomaly_count": snapshot_anomaly_count,
        "ordering_violation_count": ordering_violation_count,
    }


def _component_cli_diff(path: Path) -> dict[str, Any]:
    payload, error = _load_json(path)
    if error is not None:
        return {"status": "missing" if error == "missing" else "error", "details": error}
    status_field = str(payload.get("status", "")).strip().lower()
    if status_field == "skipped":
        reason = str(payload.get("reason", "")).strip()
        return {"status": "skipped", "details": f"reason={reason or 'n/a'}"}
    if status_field == "error":
        return {"status": "error", "details": str(payload.get("error", "unknown error"))}
    regression_count = _as_int(payload.get("regression_count"))
    missing_metric_count = _as_int(payload.get("missing_metric_count"))
    status = "ok" if regression_count == 0 else "error"
    return {
        "status": status,
        "details": (f"regressions={regression_count}, missing_metrics={missing_metric_count}"),
        "regression_count": regression_count,
        "missing_metric_count": missing_metric_count,
    }


def _component_remote_fetch(path: Path) -> dict[str, Any]:
    payload, error = _load_json(path)
    if error is not None:
        return {"status": "missing" if error == "missing" else "error", "details": error}
    status_field = str(payload.get("status", "")).strip().lower()
    if status_field == "ok":
        run_id = _as_int(payload.get("run_id"))
        member_name = str(payload.get("member_name", "")).strip() or "n/a"
        return {"status": "ok", "details": f"run_id={run_id}, member={member_name}"}
    if status_field == "skipped":
        reason = str(payload.get("reason", "")).strip()
        return {"status": "skipped", "details": f"reason={reason or 'n/a'}"}
    if status_field == "error":
        return {"status": "error", "details": str(payload.get("error", "unknown error"))}
    return {"status": "warn", "details": f"unknown_status={status_field or 'n/a'}"}


def _component_network(path: Path) -> dict[str, Any]:
    payload, error = _load_json(path)
    if error is not None:
        return {"status": "missing" if error == "missing" else "error", "details": error}
    success = payload.get("success")
    status_field = str(payload.get("status", "")).strip().lower()
    if success is False or status_field == "network_observability_failed":
        exit_code = _as_int(payload.get("exit_code"))
        return {
            "status": "warn",
            "details": f"status={status_field or 'failed'}, exit_code={exit_code}",
        }
    return {"status": "ok", "details": "advisory lane passed"}


def _component_baseline(*, baseline_file: Path, artifact_file: Path) -> dict[str, Any]:
    baseline_exists = baseline_file.exists()
    artifact_exists = artifact_file.exists()
    if baseline_exists:
        return {
            "status": "ok",
            "details": f"baseline_exists=true, artifact_exists={str(artifact_exists).lower()}",
        }
    return {
        "status": "missing",
        "details": f"baseline_exists=false, artifact_exists={str(artifact_exists).lower()}",
    }


def _overall_status(components: dict[str, dict[str, Any]]) -> str:
    max_rank = 0
    for component in components.values():
        max_rank = max(max_rank, _status_rank(str(component.get("status", ""))))
    if max_rank >= 3:
        return "error"
    if max_rank == 2:
        return "warn"
    if max_rank == 1:
        return "ok"
    return "ok"


def _normalize_status(value: Any) -> str | None:
    if value is None:
        return None
    raw = str(value).strip().lower()
    return raw if raw else None


def _extract_previous_statuses(
    payload: dict[str, Any] | None,
) -> tuple[str | None, dict[str, str | None]]:
    if not isinstance(payload, dict):
        return None, {}
    previous_overall = _normalize_status(payload.get("overall_status"))
    component_statuses: dict[str, str | None] = {}
    components = payload.get("components")
    if isinstance(components, dict):
        for key, component in components.items():
            if not isinstance(key, str) or not isinstance(component, dict):
                continue
            component_statuses[key] = _normalize_status(component.get("status"))
    return previous_overall, component_statuses


def _extract_previous_streak(
    previous_trend_payload: dict[str, Any] | None,
    *,
    component_key: str | None = None,
) -> int:
    if not isinstance(previous_trend_payload, dict):
        return 0
    if component_key is None:
        overall = previous_trend_payload.get("overall")
        if not isinstance(overall, dict):
            return 0
        return max(0, _as_int(overall.get("regression_streak")))

    components = previous_trend_payload.get("components")
    if not isinstance(components, dict):
        return 0
    component = components.get(component_key)
    if not isinstance(component, dict):
        return 0
    return max(0, _as_int(component.get("regression_streak")))


def _build_trend_entry(
    *,
    current_status: str,
    previous_status: str | None,
    previous_regression_streak: int = 0,
) -> dict[str, Any]:
    normalized_current = _normalize_status(current_status) or "unknown"
    normalized_previous = _normalize_status(previous_status)
    if normalized_previous is None:
        change = "unknown"
    else:
        current_rank = _status_rank(normalized_current)
        previous_rank = _status_rank(normalized_previous)
        if current_rank < previous_rank:
            change = "improved"
        elif current_rank > previous_rank:
            change = "regressed"
        else:
            change = "unchanged"
    previous_streak = max(0, int(previous_regression_streak))
    current_is_regression = _status_rank(normalized_current) >= 2
    previous_is_regression = (
        _status_rank(normalized_previous) >= 2 if normalized_previous is not None else False
    )
    if current_is_regression:
        if previous_is_regression:
            regression_streak = previous_streak + 1 if previous_streak > 0 else 1
        else:
            regression_streak = 1
    else:
        regression_streak = 0
    return {
        "change": change,
        "current_status": normalized_current,
        "previous_status": normalized_previous,
        "regression_streak": regression_streak,
    }


def _build_trend(
    *,
    current_overall_status: str,
    current_components: dict[str, dict[str, Any]],
    previous_status_payload: dict[str, Any] | None,
    previous_status_error: str | None,
    previous_status_file: Path | None,
) -> dict[str, Any]:
    previous_overall_status, previous_component_statuses = _extract_previous_statuses(
        previous_status_payload
    )
    previous_trend_payload = (
        previous_status_payload.get("trend") if isinstance(previous_status_payload, dict) else None
    )
    components: dict[str, dict[str, Any]] = {}
    for key in COMPONENT_ORDER:
        component = current_components.get(key)
        if not isinstance(component, dict):
            continue
        current_status = _normalize_status(component.get("status")) or "unknown"
        components[key] = _build_trend_entry(
            current_status=current_status,
            previous_status=previous_component_statuses.get(key),
            previous_regression_streak=_extract_previous_streak(
                previous_trend_payload,
                component_key=key,
            ),
        )
    return {
        "previous_status_file": str(previous_status_file) if previous_status_file else "",
        "previous_status_available": previous_status_payload is not None,
        "previous_status_error": previous_status_error or "",
        "overall": _build_trend_entry(
            current_status=current_overall_status,
            previous_status=previous_overall_status,
            previous_regression_streak=_extract_previous_streak(previous_trend_payload),
        ),
        "components": components,
    }


def _build_trend_alert(
    *,
    trend_payload: dict[str, Any],
    max_overall_regression_streak: int,
    max_component_regression_streak: int,
) -> dict[str, Any]:
    overall_threshold = max(0, int(max_overall_regression_streak))
    component_threshold = max(0, int(max_component_regression_streak))
    enabled = overall_threshold > 0 or component_threshold > 0
    violations: list[dict[str, Any]] = []

    if enabled:
        overall = trend_payload.get("overall")
        if isinstance(overall, dict):
            overall_streak = max(0, _as_int(overall.get("regression_streak")))
            if overall_threshold > 0 and overall_streak >= overall_threshold:
                violations.append(
                    {
                        "scope": "overall",
                        "threshold": overall_threshold,
                        "regression_streak": overall_streak,
                        "current_status": str(overall.get("current_status", "")),
                        "previous_status": str(overall.get("previous_status", "")),
                    }
                )

        components = trend_payload.get("components")
        if isinstance(components, dict) and component_threshold > 0:
            for key in COMPONENT_ORDER:
                component = components.get(key)
                if not isinstance(component, dict):
                    continue
                streak = max(0, _as_int(component.get("regression_streak")))
                if streak >= component_threshold:
                    violations.append(
                        {
                            "scope": "component",
                            "component": key,
                            "threshold": component_threshold,
                            "regression_streak": streak,
                            "current_status": str(component.get("current_status", "")),
                            "previous_status": str(component.get("previous_status", "")),
                        }
                    )

    return {
        "enabled": enabled,
        "thresholds": {
            "max_overall_regression_streak": overall_threshold,
            "max_component_regression_streak": component_threshold,
        },
        "violation_count": len(violations),
        "violations": violations,
    }


def _render_markdown(*, status_payload: dict[str, Any]) -> str:
    components = status_payload.get("components", {})
    if not isinstance(components, dict):
        components = {}
    trend = status_payload.get("trend", {})
    if not isinstance(trend, dict):
        trend = {}
    overall_trend = trend.get("overall")
    if not isinstance(overall_trend, dict):
        overall_trend = {}
    trend_alert = status_payload.get("trend_alert")
    if not isinstance(trend_alert, dict):
        trend_alert = {}
    lines: list[str] = [
        "### Skills Tools Benchmark Status",
        "",
        f"- overall: `{status_payload.get('overall_status', 'unknown')}`",
        f"- trend_overall: `{overall_trend.get('change', 'unknown')}` "
        f"(previous=`{overall_trend.get('previous_status', '') or 'n/a'}`, "
        f"current=`{overall_trend.get('current_status', '') or 'n/a'}`, "
        f"streak=`{overall_trend.get('regression_streak', 0)}`)",
        f"- trend_alert: enabled=`{trend_alert.get('enabled', False)}` "
        f"violations=`{trend_alert.get('violation_count', 0)}`",
        "",
        "| Component | Status | Trend | Details |",
        "| --- | --- | --- | --- |",
    ]
    trend_components = trend.get("components")
    if not isinstance(trend_components, dict):
        trend_components = {}
    for key in COMPONENT_ORDER:
        component = components.get(key)
        if not isinstance(component, dict):
            continue
        status = str(component.get("status", "unknown"))
        component_trend = trend_components.get(key)
        if not isinstance(component_trend, dict):
            component_trend = {}
        trend_change = str(component_trend.get("change", "unknown"))
        details = str(component.get("details", "")).replace("\n", " ").strip()
        lines.append(f"| `{key}` | `{status}` | `{trend_change}` | {details} |")
    return "\n".join(lines) + "\n"


def _write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Render skills tools CI status summary.")
    parser.add_argument("--deterministic-report", required=True)
    parser.add_argument("--cli-diff-report", required=True)
    parser.add_argument("--remote-fetch-report", required=True)
    parser.add_argument("--network-report", required=True)
    parser.add_argument("--baseline-file", required=True)
    parser.add_argument("--artifact-file", required=True)
    parser.add_argument(
        "--previous-status-json",
        default="",
        help="Optional previous skills_tools_ci_status.json for trend computation.",
    )
    parser.add_argument("--output-json", required=True)
    parser.add_argument("--output-markdown", required=True)
    parser.add_argument("--strict", action="store_true")
    parser.add_argument(
        "--max-overall-regression-streak",
        type=int,
        default=0,
        help="Trend alert threshold for overall regression streak (0 disables).",
    )
    parser.add_argument(
        "--max-component-regression-streak",
        type=int,
        default=0,
        help="Trend alert threshold for component regression streak (0 disables).",
    )
    parser.add_argument(
        "--strict-trend-alert",
        action="store_true",
        help="Exit non-zero when trend alert threshold violations are detected.",
    )
    args = parser.parse_args()

    deterministic_report = Path(args.deterministic_report).expanduser().resolve()
    cli_diff_report = Path(args.cli_diff_report).expanduser().resolve()
    remote_fetch_report = Path(args.remote_fetch_report).expanduser().resolve()
    network_report = Path(args.network_report).expanduser().resolve()
    baseline_file = Path(args.baseline_file).expanduser().resolve()
    artifact_file = Path(args.artifact_file).expanduser().resolve()
    previous_status_file = (
        Path(args.previous_status_json).expanduser().resolve()
        if str(args.previous_status_json).strip()
        else None
    )
    output_json = Path(args.output_json).expanduser().resolve()
    output_markdown = Path(args.output_markdown).expanduser().resolve()

    components = {
        "remote_fetch": _component_remote_fetch(remote_fetch_report),
        "deterministic_gate": _component_deterministic(deterministic_report),
        "cli_diff_gate": _component_cli_diff(cli_diff_report),
        "baseline": _component_baseline(
            baseline_file=baseline_file,
            artifact_file=artifact_file,
        ),
        "network_observability": _component_network(network_report),
    }
    overall_status = _overall_status(components)
    previous_status_payload: dict[str, Any] | None = None
    previous_status_error: str | None = None
    if previous_status_file is not None:
        previous_status_payload, previous_status_error = _load_json(previous_status_file)

    status_payload: dict[str, Any] = {
        "schema": STATUS_SCHEMA,
        "overall_status": overall_status,
        "components": components,
        "trend": _build_trend(
            current_overall_status=overall_status,
            current_components=components,
            previous_status_payload=previous_status_payload,
            previous_status_error=previous_status_error,
            previous_status_file=previous_status_file,
        ),
        "paths": {
            "deterministic_report": str(deterministic_report),
            "cli_diff_report": str(cli_diff_report),
            "remote_fetch_report": str(remote_fetch_report),
            "network_report": str(network_report),
            "baseline_file": str(baseline_file),
            "artifact_file": str(artifact_file),
            "previous_status_file": str(previous_status_file) if previous_status_file else "",
        },
    }
    trend_payload = status_payload["trend"] if isinstance(status_payload.get("trend"), dict) else {}
    status_payload["trend_alert"] = _build_trend_alert(
        trend_payload=trend_payload,
        max_overall_regression_streak=max(0, int(args.max_overall_regression_streak)),
        max_component_regression_streak=max(0, int(args.max_component_regression_streak)),
    )
    markdown = _render_markdown(status_payload=status_payload)
    _write(output_json, json.dumps(status_payload, ensure_ascii=False, indent=2) + "\n")
    _write(output_markdown, markdown)
    print(json.dumps(status_payload, ensure_ascii=False))

    if args.strict and overall_status == "error":
        return 1
    trend_alert_payload = (
        status_payload.get("trend_alert")
        if isinstance(status_payload.get("trend_alert"), dict)
        else {}
    )
    trend_alert_violations = (
        _as_int(trend_alert_payload.get("violation_count"))
        if isinstance(trend_alert_payload, dict)
        else 0
    )
    if args.strict_trend_alert and trend_alert_violations > 0:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
