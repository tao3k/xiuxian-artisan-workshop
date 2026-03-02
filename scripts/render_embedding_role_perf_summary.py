#!/usr/bin/env python3
"""Render omni-agent embedding role perf JSON report into markdown summary."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

SCHEMA = "omni_agent.embedding.role_perf_smoke.v1"


def _as_float(value: Any) -> float:
    if isinstance(value, (int, float)):
        return float(value)
    if isinstance(value, str):
        text = value.strip()
        if text:
            return float(text)
    raise ValueError(f"expected numeric value, got {value!r}")


def _as_int(value: Any) -> int:
    if isinstance(value, int):
        return value
    if isinstance(value, str):
        text = value.strip()
        if text:
            return int(text)
    raise ValueError(f"expected integer value, got {value!r}")


def _safe_metric(value: Any, decimals: int = 2) -> str:
    try:
        return f"{_as_float(value):.{decimals}f}"
    except ValueError:
        return "-"


def _safe_int_metric(value: Any) -> str:
    try:
        return str(_as_int(value))
    except ValueError:
        return "-"


def _role_error_count(role_payload: dict[str, Any]) -> int:
    total = 0
    for key in ("single", "batch8", "concurrent_single"):
        stats = role_payload.get(key)
        if isinstance(stats, dict):
            try:
                total += _as_int(stats.get("err", 0))
            except ValueError:
                continue
    return total


def render_summary(
    payload: dict[str, Any], *, profile: str = "", runner_os: str = "", report_path: str = ""
) -> str:
    schema = str(payload.get("schema", "")).strip()
    if schema != SCHEMA:
        raise ValueError(f"schema mismatch: expected {SCHEMA!r}, got {schema!r}")

    status = str(payload.get("status", "unknown")).strip() or "unknown"
    duration_secs = _safe_metric(payload.get("duration_secs"), decimals=2)
    upstream_base_url = str(payload.get("upstream_base_url", "")).strip() or "-"
    embedding_model = str(payload.get("embedding_model", "")).strip() or "-"
    single_runs = _safe_int_metric(payload.get("single_runs"))
    batch_runs = _safe_int_metric(payload.get("batch_runs"))
    concurrent_total = _safe_int_metric(payload.get("concurrent_total"))
    concurrent_width = _safe_int_metric(payload.get("concurrent_width"))

    title_suffix_parts = []
    if profile.strip():
        title_suffix_parts.append(profile.strip())
    if runner_os.strip():
        title_suffix_parts.append(runner_os.strip())
    title_suffix = f" ({', '.join(title_suffix_parts)})" if title_suffix_parts else ""

    lines = [f"## Embedding Role Perf{title_suffix}"]
    lines.append(f"- Status: `{status}`")
    lines.append(f"- Duration: `{duration_secs}s`")
    lines.append(f"- Upstream: `{upstream_base_url}`")
    lines.append(f"- Model: `{embedding_model}`")
    lines.append(
        "- Load: "
        f"`single={single_runs}` "
        f"`batch8={batch_runs}` "
        f"`concurrent={concurrent_total}/{concurrent_width}`"
    )
    if report_path.strip():
        lines.append(f"- Report: `{report_path.strip()}`")

    lines.append(
        "| Role | single p95 (ms) | batch8 p95 (ms) | concurrent RPS | "
        "concurrent p95 (ms) | errors |"
    )
    lines.append("|---|---:|---:|---:|---:|---:|")

    roles = payload.get("roles")
    if not isinstance(roles, list) or not roles:
        lines.append("| (none) | - | - | - | - | - |")
    else:
        for item in roles:
            if not isinstance(item, dict):
                continue
            role = str(item.get("role", "")).strip() or "(unknown)"
            single = item.get("single", {}) if isinstance(item.get("single"), dict) else {}
            batch8 = item.get("batch8", {}) if isinstance(item.get("batch8"), dict) else {}
            concurrent = (
                item.get("concurrent_single", {})
                if isinstance(item.get("concurrent_single"), dict)
                else {}
            )
            lines.append(
                "| "
                f"{role} | "
                f"{_safe_metric(single.get('p95_ms'))} | "
                f"{_safe_metric(batch8.get('p95_ms'))} | "
                f"{_safe_metric(concurrent.get('rps'))} | "
                f"{_safe_metric(concurrent.get('p95_ms'))} | "
                f"{_role_error_count(item)} |"
            )

    failures = payload.get("failures")
    if isinstance(failures, list) and failures:
        lines.append("")
        lines.append("### Failures")
        for entry in failures:
            lines.append(f"- {entry}")

    return "\n".join(lines) + "\n"


def _load_report(path: Path) -> dict[str, Any]:
    raw = path.read_text(encoding="utf-8")
    payload = json.loads(raw)
    if not isinstance(payload, dict):
        raise ValueError("report payload must be a JSON object")
    return payload


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Render omni-agent embedding role perf report into markdown."
    )
    parser.add_argument("--input", required=True, help="Path to embedding role perf JSON report.")
    parser.add_argument(
        "--output-markdown",
        default="",
        help="Optional path to write markdown summary; stdout is always written.",
    )
    parser.add_argument("--profile", default="", help="Optional profile label for summary title.")
    parser.add_argument("--runner-os", default="", help="Optional runner OS label.")
    args = parser.parse_args()

    input_path = Path(args.input).expanduser().resolve()
    payload = _load_report(input_path)
    markdown = render_summary(
        payload,
        profile=str(args.profile),
        runner_os=str(args.runner_os),
        report_path=str(input_path),
    )
    print(markdown, end="")

    if str(args.output_markdown).strip():
        output_path = Path(str(args.output_markdown)).expanduser().resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(markdown, encoding="utf-8")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
