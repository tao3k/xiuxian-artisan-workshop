#!/usr/bin/env python3
"""CyberXiuXian Forge Execution: Soul Synthesis.

This script feeds the Failure DNA (Temporal Deadlock) into the
Autonomous Evolution Flow, instructing the Agent to synthesize
a new "Collision Specialist" persona.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
import select
from pathlib import Path

# The DNA harvested from the crash of Task 7.1
FORGE_CONTEXT = {
    "failure_trace": (
        "Node execution failed: Qianhuan annotation failed: Context insufficient: CCS=0.5. "
        "Missing: traceability, architectural consistency. "
        "Conflict: 4h Zero-Day Patch vs 6h Mandatory HR Training within an 8h physical window."
    ),
    "failure_cluster": "temporal-deadlock, physical-constraint-violation, absolute-priority-collision",
    "target_domain": "agenda-management",
    "target_persona_dir": "packages/rust/crates/xiuxian-zhixing/resources/zhixing/skills/autonomous-forged/references",
    "role_id": "collision_specialist",
    "project_root": ".",
    "raw_facts": (
        "The current Trinity (Student, Steward, Professor) enters an infinite loop of vetoes "
        "when multiple 'Absolute Priority' constraints physically exceed the available time window. "
        "No persona currently has the authority to negotiate or override a 'Mandatory' external constraint."
    ),
    "wendao_search_results": (
        "<hit>Auditor's Codex: [ADVERSARIAL-FRICTION] Personas that merely agree are failures.</hit>"
        "<hit>Agenda Methodologies: [1. Buffer Engineering] Any plan utilizing more than 80% is a critical integrity failure.</hit>"
    ),
}


def _project_root() -> Path:
    return Path(__file__).resolve().parents[1]


def _run_forge_battle(project_root: Path, verbose: bool = False):
    print("\n" + "=" * 80)
    print("--- INITIATING AUTONOMOUS SOUL FORGE ---")
    print("=" * 80)

    cmd = [
        "cargo",
        "test",
        "-p",
        "xiuxian-qianji",
        "--test",
        "test_bootcamp_api",
        "bootcamp_runs_real_forge_flow",
        "--features",
        "llm",
        "--",
        "--nocapture",
    ]

    if verbose:
        # Assuming the test harness is updated to check this env var
        os.environ["XIUXIAN_VERBOSE"] = "true"

    env = os.environ.copy()
    env["XIUXIAN_BOOTCAMP_CONTEXT"] = json.dumps(FORGE_CONTEXT)
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
    print("[*] Injecting Failure DNA. Waiting for Soul Synthesis...\n")

    last_output_time = time.monotonic()

    while True:
        reads = [process.stdout]
        readable, _, _ = select.select(reads, [], [], 10.0)

        if readable:
            line = process.stdout.readline()
            if not line:
                break

            l = line.strip()
            if "Activating Avatar" in l or "Score:" in l or "Node:" in l or "forge_guard" in l:
                print(f"\n\033[1;35m[FORGE] {l}\033[0m")
            elif "panicked" in l or "error:" in l:
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
                print(f"\033[1;30m  [Heartbeat: Synthesizing Soul... ({elapsed}s elapsed)]\033[0m")
                sys.stdout.flush()
            else:
                break

    return process.wait()


def main() -> int:
    root = _project_root()

    if "MINIMAX_API_KEY" not in os.environ:
        print("[BLOCKER] MINIMAX_API_KEY is missing from environment.")
        return 1

    exit_code = _run_forge_battle(root)

    if exit_code == 0:
        print("\n" + "=" * 80)
        print("[大捷] Soul Synthesis Complete. The Forge Guard has approved the new Persona.")

        # --- NEW: PHYSICAL ENFORCEMENT ---
        # The Rust output is printed to stdout. We can scrape the final_context JSON from it.
        # But for now, let's look at the result manually and confirm success.
        print("[*] Verifying physical manifestation...")
        target_path = Path(FORGE_CONTEXT["target_persona_dir"]) / f"{FORGE_CONTEXT['role_id']}.md"
        if target_path.exists():
            print(f"[✅] Physical Soul Manifested at: {target_path}")
        else:
            print(f"[🚨] Manifestation Node Failed (Escaping Error). Use manual snapshot.")
        print("=" * 80)
    else:
        print(f"\n[FAIL] Synthesis collapsed with exit code {exit_code}.")

    return exit_code


if __name__ == "__main__":
    os.chmod(__file__, 0o755)
    raise SystemExit(main())
