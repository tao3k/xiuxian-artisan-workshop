#!/usr/bin/env python3
"""CyberXiuXian Swarm Evolution: The Collective Forge.

This script demonstrates true "Collective Mind" evolution by spawning
3 concurrent Soul-Forger threads that must reach consensus on the
new persona's identity before manifestation.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path

SWARM_CONTEXT = {
    "failure_trace": "Node execution failed: Context insufficient: CCS=0.5. Temporal Deadlock.",
    "target_domain": "agenda-management",
    "identities": [
        {"agent_id": "forger_alpha", "role_class": "soul-forger", "weight": 1.0},
        {"agent_id": "forger_beta", "role_class": "soul-forger", "weight": 1.0},
        {"agent_id": "forger_gamma", "role_class": "soul-forger", "weight": 1.0},
    ],
    "redis_url": "redis://127.0.0.1:6379/0",
}


def _project_root() -> Path:
    return Path(__file__).resolve().parents[1]


def run_swarm_evolution():
    print("\n" + "=" * 80)
    print("--- INITIATING COLLECTIVE SOUL FORGE: SWARM MODE ---")
    print("=" * 80)

    # We use the new test harness that supports SwarmEngine execution
    cmd = [
        "cargo",
        "test",
        "-p",
        "xiuxian-qianji",
        "--test",
        "test_swarm_orchestration",
        "swarm_engine_executes_workers_concurrently",
        "--features",
        "llm",
        "--",
        "--nocapture",
    ]

    env = os.environ.copy()
    env["XIUXIAN_SWARM_CONTEXT"] = json.dumps(SWARM_CONTEXT)

    print("[*] Spawning 3 concurrent Forger Threads in Rust...")
    result = subprocess.run(cmd, cwd=_project_root(), env=env)

    if result.returncode == 0:
        print("\n" + "=" * 80)
        print("[大捷] COLLECTIVE EVOLUTION SUCCESSFUL! Consensus reached by Swarm.")
        print("=" * 80)
    else:
        print("\n" + "=" * 80)
        print("[FAIL] Swarm failed to reach consensus. Conflict in the Collective Mind.")
        print("=" * 80)


if __name__ == "__main__":
    run_swarm_evolution()
