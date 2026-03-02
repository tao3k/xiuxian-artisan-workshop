#!/usr/bin/env python3
"""Unified gate for distributed swarm synchronization.

Gate sequence:
1. Run targeted Rust validation: `test_swarm_orchestration`.
2. Run multi-process distributed swarm execution against one shared session.
3. Exit non-zero if any stage fails.
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

DEFAULT_TIMEOUT_SECONDS = 240
DEFAULT_VALKEY_URL = "redis://127.0.0.1:6379/0"


def _project_root() -> Path:
    return Path(__file__).resolve().parents[1]


def _default_manifest(root: Path) -> Path:
    return root / "scripts/fixtures/bootcamp/distributed_swarm_flow.toml"


def _default_context_file(root: Path) -> Path:
    return root / "scripts/fixtures/bootcamp/distributed_swarm_context.json"


def _run_command(cmd: list[str], cwd: Path, env: dict[str, str] | None = None) -> int:
    print(f"[CMD] {' '.join(cmd)}")
    process = subprocess.Popen(cmd, cwd=cwd, env=env)
    return process.wait()


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    """Parse CLI arguments."""
    parser = argparse.ArgumentParser(
        description="Run distributed swarm gate (Rust test + multi-process run)."
    )
    parser.add_argument(
        "--valkey-url",
        type=str,
        default=os.getenv("VALKEY_URL", DEFAULT_VALKEY_URL),
        help="Valkey URL used by distributed run.",
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        default=None,
        help="Flow TOML used by distributed run.",
    )
    parser.add_argument(
        "--context-file",
        type=Path,
        default=None,
        help="Context JSON file used by distributed run.",
    )
    parser.add_argument(
        "--skip-rust-test",
        action="store_true",
        help="Skip `cargo nextest` validation stage.",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=DEFAULT_TIMEOUT_SECONDS,
        help="Distributed run timeout (seconds).",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print commands only.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    root = _project_root()
    manifest = (args.manifest or _default_manifest(root)).resolve()
    context_file = (args.context_file or _default_context_file(root)).resolve()

    if not manifest.exists():
        print(f"[ERROR] manifest missing: {manifest}", file=sys.stderr)
        return 1
    if not context_file.exists():
        print(f"[ERROR] context file missing: {context_file}", file=sys.stderr)
        return 1

    rust_test_cmd = [
        "cargo",
        "nextest",
        "run",
        "-p",
        "xiuxian-qianji",
        "--test",
        "test_swarm_orchestration",
    ]
    distributed_cmd = [
        sys.executable,
        str((root / "scripts/bootcamp_distributed_swarm.py").resolve()),
        "--manifest",
        str(manifest),
        "--context-file",
        str(context_file),
        "--valkey-url",
        args.valkey_url,
        "--timeout-seconds",
        str(args.timeout_seconds),
        "--agent",
        "student_node_1:student:1.0",
        "--agent",
        "steward_node_1:steward:1.0",
        "--agent",
        "teacher_node_1:teacher:1.0",
    ]

    print("=" * 80)
    print("Distributed Swarm Gate")
    print(f"manifest   : {manifest}")
    print(f"context    : {context_file}")
    print(f"valkey_url : {args.valkey_url}")
    print("=" * 80)

    if args.dry_run:
        if not args.skip_rust_test:
            print(f"[DRY-RUN] {' '.join(rust_test_cmd)}")
        print(f"[DRY-RUN] {' '.join(distributed_cmd)}")
        return 0

    if not args.skip_rust_test:
        rust_rc = _run_command(rust_test_cmd, cwd=root)
        if rust_rc != 0:
            print(f"[FAIL] Rust validation failed (rc={rust_rc})", file=sys.stderr)
            return rust_rc
        print("[PASS] Rust validation passed.")

    env = os.environ.copy()
    env["VALKEY_URL"] = args.valkey_url
    distributed_rc = _run_command(distributed_cmd, cwd=root, env=env)
    if distributed_rc != 0:
        print(
            f"[FAIL] Distributed swarm execution failed (rc={distributed_rc})",
            file=sys.stderr,
        )
        return distributed_rc

    print("[PASS] Distributed swarm gate passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
