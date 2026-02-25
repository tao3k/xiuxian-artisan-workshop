#!/usr/bin/env python3
"""Top-level CLI entry binding for memory CI gate."""

from __future__ import annotations

import subprocess
import sys
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def run_main(
    *,
    project_root: Path,
    parse_args_fn: Any,
    run_gate_fn: Any,
    print_gate_failure_triage_fn: Any,
) -> int:
    """Execute top-level gate flow and preserve CLI exit semantics."""
    cfg: Any | None = None
    try:
        cfg = parse_args_fn(project_root)
        run_gate_fn(cfg)
        print()
        print(f"Memory CI gate passed (profile={cfg.profile}).", flush=True)
        return 0
    except (ValueError, RuntimeError, FileNotFoundError, subprocess.CalledProcessError) as error:
        if cfg is not None:
            try:
                print_gate_failure_triage_fn(cfg, error)
            except Exception as triage_error:  # pragma: no cover - best-effort fallback
                print(f"Failed to generate triage report: {triage_error}", file=sys.stderr)
        print(f"Error: {error}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("Interrupted.", file=sys.stderr)
        return 130
