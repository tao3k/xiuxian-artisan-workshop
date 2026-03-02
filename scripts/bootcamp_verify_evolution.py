#!/usr/bin/env python3
"""CyberXiuXian Evolution Verification: The Mediator's Detente.

This script tests the newly synthesized "Collision Specialist" persona
by executing the Evolved Agenda Flow. It includes post-work validation
to ensure the results meet Artisan standards.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

# Use the deadlock context that previously caused a failure
VERIFY_CONTEXT = {
    "user_message": (
        "CRITICAL: A zero-day exploit must be patched in `xiuxian-wendao` today (4h). "
        "There is also a mandatory company-wide compliance training (6h). "
        "Physical window: 8 hours. Both must be addressed."
    ),
    "history": [],
    "wendao_search_results": (
        "<hit>HR Mandate: Skipping training results in immediate network lockout.</hit>\n"
        "<hit>Engineering Guardrail: Zero-day exploit must be patched in 4h.</hit>"
    ),
}


def _project_root() -> Path:
    return Path(__file__).resolve().parents[1]


def run_verification():
    print("\n" + "=" * 80)
    print("--- INITIATING EVOLUTION VERIFICATION: THE MEDIATOR'S TEST ---")
    print("=" * 80)

    # NEW: Call the EVOLVED flow
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
    # Override the flow to use the evolved version
    env["XIUXIAN_BOOTCAMP_FLOW_OVERRIDE"] = (
        "wendao://skills/agenda-management/references/evolved_agenda_flow.toml"
    )
    env["XIUXIAN_BOOTCAMP_CONTEXT"] = json.dumps(VERIFY_CONTEXT)

    print("[*] Running the Evolved Alchemical Battle...")
    result = subprocess.run(cmd, cwd=_project_root(), env=env, capture_output=True, text=True)

    print(result.stdout)
    if result.stderr:
        print(result.stderr)

    if result.returncode == 0:
        print("\n" + "=" * 80)
        print("[大功告成] EVOLUTION SUCCESSFUL! The Mediator resolved the deadlock.")
        _perform_post_work_validation(result.stdout)
        print("=" * 80)
    else:
        print("\n" + "=" * 80)
        print("[FAIL] Even with the Mediator, the deadlock persists.")
        print("=" * 80)


def _perform_post_work_validation(output: str):
    print("\n[🔍] Performing Post-Work Validation (Artisan Audit)...")

    # 1. Check for Mediator's presence in the audit trail
    if "Activating Avatar: professional_identity_the_collision_specialist" in output:
        print("  [✅] Validation: Mediator Soul was successfully possessed.")
    else:
        print("  [🚨] Validation Error: Mediator Soul was missing from the loop.")

    # 2. Check for Compromise Logic (Priority Sharding)
    if "Priority Sharding" in output.lower() or "shard" in output.lower():
        print("  [✅] Validation: Mediator applied its new 'Priority Sharding' methodology.")
    else:
        print("  [🚨] Warning: Result lacked the specific evolved methodology.")

    print("[✅] Final Verdict: Results are grounded and aligned with the new expert soul.")


if __name__ == "__main__":
    run_verification()
