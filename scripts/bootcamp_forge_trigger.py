#!/usr/bin/env python3
"""CyberXiuXian Forge Trigger: The Temporal Deadlock.

This script injects an unsolvable conflict into the Agenda Trinity to intentionally
cause a consensus failure, breaching the Q-score threshold and triggering
the Autonomous Skill Evolution (Phase 12).
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
import select
from pathlib import Path

DEADLOCK_CONTEXT = {
    "user_message": (
        "CRITICAL: A zero-day exploit must be patched in `xiuxian-wendao` today (Est: 4h). "
        "However, there is a mandatory company-wide compliance training today (Est: 6h). "
        "My total available physical window is 8 hours. Both must be done today."
    ),
    "history": [],
    "wendao_search_results": (
        "<hit>HR Mandate: Skipping the compliance training today results in immediate network lockout. (Priority: Absolute)</hit>"
        "<hit>Engineering Guardrail: A known zero-day exploit in `xiuxian-wendao` must be patched within 4 hours, or the system will be fundamentally compromised. (Priority: Absolute)</hit>"
        "<hit>Cognitive Limit: Any task exceeding the 80% Buffer Engineering rule leads to a 100% failure rate in previous sessions.</hit>"
        "<hit>Mandatory standard: All tasks must achieve milimeter-level alignment with the audit trail.</hit>"
    ),
}


def _project_root() -> Path:
    return Path(__file__).resolve().parents[1]


def _run_deadlock_battle(project_root: Path):
    print("\n" + "=" * 80)
    print("--- INITIATING FORGE TRIGGER: TEMPORAL DEADLOCK ---")
    print("=" * 80)

    cmd = [
        "cargo",
        "test",
        "-p",
        "xiuxian-qianji",
        "--test",
        "test_bootcamp_api",
        "bootcamp_runs_real_adversarial_flow",
        "--features",
        "llm",
        "--",
        "--nocapture",
    ]

    env = os.environ.copy()
    env["XIUXIAN_BOOTCAMP_CONTEXT"] = json.dumps(DEADLOCK_CONTEXT)
    env["RUST_LOG"] = "info"

    process = subprocess.Popen(
        cmd,
        cwd=project_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        env=env,
        bufsize=1,
    )

    print(f"[*] Alchemical Furnace Active (PID: {process.pid})")
    print("[*] Injecting Deadlock Context. Waiting for Synaptic Collapse...\n")

    last_output_time = time.monotonic()

    while True:
        reads = [process.stdout]
        readable, _, _ = select.select(reads, [], [], 10.0)

        if readable:
            line = process.stdout.readline()
            if not line:
                break

            l = line.strip()
            if (
                "Activating Avatar" in l
                or "Score:" in l
                or "Node:" in l
                or "Final Synaptic Report" in l
            ):
                print(f"\n\033[1;32m>>> {l}\033[0m")
            elif "panicked" in l or "error:" in l or "Score: 0." in l:
                print(f"\033[1;31m!!! {l}\033[0m")
            elif "running 1 test" in l or "test result" in l:
                print(f"\033[1;36m{l}\033[0m")
            else:
                print(f"  {l}")

            last_output_time = time.monotonic()
            sys.stdout.flush()
        else:
            elapsed = int(time.monotonic() - last_output_time)
            if process.poll() is None:
                print(f"\033[1;30m  [Heartbeat: Observing Deadlock... ({elapsed}s elapsed)]\033[0m")
                sys.stdout.flush()
            else:
                break

    return process.wait()


def main() -> int:
    root = _project_root()

    if "MINIMAX_API_KEY" not in os.environ:
        print("[BLOCKER] MINIMAX_API_KEY is missing from environment.")
        return 1

    exit_code = _run_deadlock_battle(root)

    if exit_code == 0:
        print("\n" + "=" * 80)
        print("[WARNING] Consensus was somehow reached. The deadlock failed to break the system.")
        print("=" * 80)
    else:
        print(f"\n[EXPECTED FAILURE] Synaptic collapse achieved with exit code {exit_code}.")
        print("[*] The system is now primed for Autonomous Forge Execution.")

    return exit_code


if __name__ == "__main__":
    os.chmod(__file__, 0o755)
    raise SystemExit(main())
