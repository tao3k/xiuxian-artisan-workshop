#!/usr/bin/env python3
"""Console output orchestration for memory CI gate triage reporting."""

from __future__ import annotations

import sys
from typing import Any


def print_gate_failure_triage(
    cfg: Any,
    error: Exception,
    *,
    classify_failure_fn: Any,
    read_tail_fn: Any,
    shell_quote_command_fn: Any,
    build_gate_failure_repro_commands_fn: Any,
    default_gate_failure_report_base_path_fn: Any,
    write_gate_failure_triage_report_fn: Any,
    write_gate_failure_triage_json_report_fn: Any,
) -> Any:
    """Emit failure summary to stderr and persist markdown/json reports."""
    category, summary = classify_failure_fn(error)
    repro_commands = build_gate_failure_repro_commands_fn(
        cfg,
        category=category,
        error=error,
        shell_quote_command_fn=shell_quote_command_fn,
    )
    base, _ = default_gate_failure_report_base_path_fn(cfg)
    markdown_path = base.with_suffix(".md")
    json_path = base.with_suffix(".json")
    report_path = write_gate_failure_triage_report_fn(
        cfg,
        error=error,
        category=category,
        summary=summary,
        repro_commands=repro_commands,
        read_tail_fn=read_tail_fn,
        shell_quote_command_fn=shell_quote_command_fn,
        report_path=markdown_path,
    )
    json_report_path = write_gate_failure_triage_json_report_fn(
        cfg,
        error=error,
        category=category,
        summary=summary,
        repro_commands=repro_commands,
        read_tail_fn=read_tail_fn,
        shell_quote_command_fn=shell_quote_command_fn,
        report_path=json_path,
    )
    print("", file=sys.stderr)
    print("Memory CI gate failure triage:", file=sys.stderr)
    print(f"  profile={cfg.profile}", file=sys.stderr)
    print(f"  category={category}", file=sys.stderr)
    print(f"  summary={summary}", file=sys.stderr)
    print(f"  report={report_path}", file=sys.stderr)
    print(f"  report_json={json_report_path}", file=sys.stderr)
    print("  repro_commands:", file=sys.stderr)
    for command in repro_commands[:8]:
        print(f"    - {command}", file=sys.stderr)
    if len(repro_commands) > 8:
        print(f"    - ... (+{len(repro_commands) - 8} more)", file=sys.stderr)
    return report_path
