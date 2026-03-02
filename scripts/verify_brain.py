#!/usr/bin/env python3
"""Verify native workflow-state persistence for brain-memory workflows.

Usage: uv run python scripts/verify_brain.py
"""

from __future__ import annotations

import json
import time

from omni.foundation.config.prj import get_runtime_dir
from omni.foundation.workflow_state import (
    delete_workflow_state,
    get_checkpointer,
    get_workflow_history,
    load_workflow_state,
    save_workflow_state,
)


def _print_test_header(name: str) -> None:
    print("\n" + "=" * 60)
    print(name)
    print("=" * 60)


def test_imports() -> bool:
    """Verify native workflow-state APIs import cleanly."""
    _print_test_header("TEST 1: Import Verification")
    try:
        # Reference imported symbols to keep static analyzers explicit.
        _ = (
            get_checkpointer,
            save_workflow_state,
            load_workflow_state,
            get_workflow_history,
            delete_workflow_state,
        )
        print("[PASS] Native workflow-state APIs imported successfully")
        return True
    except Exception as exc:
        print(f"[FAIL] Import failed: {exc}")
        return False


def test_direct_save_load() -> bool:
    """Verify direct save/load/history workflow-state APIs."""
    _print_test_header("TEST 2: Direct Save/Load APIs")
    workflow_type = "verify_brain_direct"
    workflow_id = f"thread_{int(time.time() * 1000)}"
    state = {
        "current_plan": "Research agent memory architecture",
        "step": 1,
        "messages": [{"role": "user", "content": "Research agent memory architecture"}],
    }
    metadata = {"source": "verify_brain", "step": 1}

    try:
        if not save_workflow_state(workflow_type, workflow_id, state, metadata=metadata):
            print("[FAIL] save_workflow_state returned False")
            return False

        loaded = load_workflow_state(workflow_type, workflow_id)
        if loaded != state:
            print("[FAIL] Loaded state mismatch")
            print(f"  Expected: {state}")
            print(f"  Actual:   {loaded}")
            return False

        history = get_workflow_history(workflow_type, workflow_id, limit=5)
        if not history:
            print("[FAIL] History should contain at least one checkpoint")
            return False
        print(f"[PASS] Direct APIs round-trip succeeded; history entries: {len(history)}")
        return True
    except Exception as exc:
        print(f"[FAIL] Direct save/load test failed: {exc}")
        return False
    finally:
        delete_workflow_state(workflow_type, workflow_id)


def test_history_order_and_limit() -> bool:
    """Verify history returns most recent records first and honors limit."""
    _print_test_header("TEST 3: History Ordering and Limit")
    workflow_type = "verify_brain_history"
    workflow_id = f"thread_{int(time.time() * 1000)}"

    try:
        for step in range(5):
            state = {"step": step, "updated_at_ms": int(time.time() * 1000)}
            if not save_workflow_state(workflow_type, workflow_id, state, metadata={"step": step}):
                print(f"[FAIL] Failed to save checkpoint for step={step}")
                return False

        history = get_workflow_history(workflow_type, workflow_id, limit=3)
        if len(history) != 3:
            print(f"[FAIL] Expected 3 history entries, got {len(history)}")
            return False

        steps = [entry.get("state", {}).get("step") for entry in history]
        if steps != [4, 3, 2]:
            print(f"[FAIL] Unexpected history order: {steps}")
            return False

        print(f"[PASS] History ordering verified: {steps}")
        return True
    except Exception as exc:
        print(f"[FAIL] History test failed: {exc}")
        return False
    finally:
        delete_workflow_state(workflow_type, workflow_id)


def test_checkpointer_handle() -> bool:
    """Verify get_checkpointer handle methods for save/get/history/delete."""
    _print_test_header("TEST 4: Checkpointer Handle API")
    table_name = "verify_brain_handle"
    thread_id = f"thread_{int(time.time() * 1000)}"
    handle = get_checkpointer(table_name)
    state = {"phase": "analysis", "step": 7, "confidence": 0.88}

    try:
        payload = json.dumps(state, ensure_ascii=False)
        if not handle.save_checkpoint(
            table_name,
            thread_id,
            payload,
            metadata={"source": "verify_brain_handle"},
        ):
            print("[FAIL] save_checkpoint returned False")
            return False

        latest_payload = handle.get_latest(table_name, thread_id)
        if latest_payload is None:
            print("[FAIL] get_latest returned None")
            return False
        latest = json.loads(latest_payload)
        if latest != state:
            print("[FAIL] get_latest payload mismatch")
            print(f"  Expected: {state}")
            print(f"  Actual:   {latest}")
            return False

        history_payloads = handle.get_history(table_name, thread_id, limit=10)
        if not history_payloads:
            print("[FAIL] get_history returned no entries")
            return False

        if not handle.delete_thread(table_name, thread_id):
            print("[FAIL] delete_thread returned False")
            return False

        deleted_latest = handle.get_latest(table_name, thread_id)
        if deleted_latest is not None:
            print("[FAIL] Expected no checkpoint after delete_thread")
            return False

        print("[PASS] Checkpointer handle API verified")
        return True
    except Exception as exc:
        print(f"[FAIL] Checkpointer handle test failed: {exc}")
        return False
    finally:
        delete_workflow_state(table_name, thread_id)


def main() -> int:
    """Run workflow-state verification tests."""
    print("\n" + "=" * 60)
    print("BRAIN MEMORY VERIFICATION")
    print("Native Workflow-State Runtime")
    print("=" * 60)
    print(f"Runtime state directory: {get_runtime_dir('xiuxian_qianji/workflow_state')}")

    tests = [
        ("Import Verification", test_imports),
        ("Direct Save/Load APIs", test_direct_save_load),
        ("History Ordering and Limit", test_history_order_and_limit),
        ("Checkpointer Handle API", test_checkpointer_handle),
    ]
    results: list[tuple[str, bool]] = []
    for name, fn in tests:
        results.append((name, fn()))

    print("\n" + "=" * 60)
    print("VERIFICATION SUMMARY")
    print("=" * 60)
    all_passed = True
    for name, passed in results:
        status = "PASS" if passed else "FAIL"
        print(f"  [{status}] {name}")
        if not passed:
            all_passed = False

    print("=" * 60)
    if all_passed:
        print("All checks passed: native workflow-state persistence is operational.")
    else:
        print("Some checks failed. Inspect details above.")
    print("=" * 60 + "\n")
    return 0 if all_passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
