#!/usr/bin/env python3
"""CyberXiuXian Epic Forge: Cross-Cluster Sovereignty.

This script demonstrates planetary-scale evolution by spawning two
distinct Swarm Clusters (Alpha and Beta). Cluster Alpha initiates
the Forge, while Cluster Beta provides remote specialized reasoning.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path


def _project_root() -> Path:
    return Path(__file__).resolve().parents[1]


def spawn_cluster(cluster_id: str, role_classes: list[str]):
    print(f"[*] Igniting Cluster: {cluster_id} (Roles: {role_classes})...")
    env = os.environ.copy()
    env["CLUSTER_ID"] = cluster_id
    env["RUST_LOG"] = "info"

    # Each cluster is a separate process running its own SwarmEngine
    cmd = [
        "cargo",
        "test",
        "-p",
        "xiuxian-qianji",
        "--test",
        "test_swarm_discovery",
        "test_remote_possession_handshake",
        "--features",
        "llm",
        "--",
        "--nocapture",
    ]
    return subprocess.Popen(
        cmd,
        cwd=_project_root(),
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )


def run_epic_forge():
    print("\n" + "=" * 80)
    print("--- INITIATING PLANETARY EPIC FORGE: CROSS-CLUSTER MODE ---")
    print("=" * 80)

    # 1. Spawn Cluster Beta (The Specialist Auditor)
    p_beta = spawn_cluster("Cluster_Beta", ["auditor"])
    time.sleep(2)  # Give Beta time to publish heartbeats to registry

    # 2. Spawn Cluster Alpha (The Orchestrator)
    p_alpha = spawn_cluster("Cluster_Alpha", ["soul-forger", "manager"])

    print("[*] Clusters are now vibrating in the Valkey Nebula. Observing Synaptic Handshake...\n")

    # Monitor outputs
    while True:
        line_a = p_alpha.stdout.readline()
        if line_a:
            print(f"\033[1;34m[Alpha]\033[0m {line_a.strip()}")

        line_b = p_beta.stdout.readline()
        if line_b:
            print(f"\033[1;35m[Beta ]\033[0m {line_b.strip()}")

        if p_alpha.poll() is not None and p_beta.poll() is not None:
            break

    if p_alpha.returncode == 0:
        print("\n" + "=" * 80)
        print("[大捷] PLANETARY FORGE SUCCESSFUL! Sovereignty established across clusters.")
        print("=" * 80)
    else:
        print(
            f"\n[FAIL] Cosmic Bus disconnected. Exit codes: A={p_alpha.returncode}, B={p_beta.returncode}"
        )


if __name__ == "__main__":
    run_epic_forge()
