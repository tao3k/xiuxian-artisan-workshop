#!/usr/bin/env python3
"""Verify native context pipeline integration.

Usage:
    uv run python scripts/verify_pipeline.py
"""

from __future__ import annotations

import asyncio
from pathlib import Path

from omni.core.context import create_planner_orchestrator
from omni.foundation.config.settings import get_settings


def _print_section(title: str) -> None:
    print("\n" + "=" * 60)
    print(title)
    print("=" * 60)


async def verify_pipeline() -> bool:
    """Run pipeline checks against the native context orchestrator."""
    _print_section("NATIVE CONTEXT PIPELINE VERIFICATION")

    settings = get_settings()
    project_root = Path(settings.get("general.project_root", "."))
    print(f"Project root: {project_root}")

    # 1) Rust bridge surface check
    try:
        import omni_core_rs as rust
    except ImportError as exc:
        print(f"[FAIL] Failed to import omni_core_rs: {exc}")
        return False

    if not hasattr(rust, "ContextAssembler"):
        print("[FAIL] omni_core_rs.ContextAssembler is not available")
        return False
    print("[PASS] Rust ContextAssembler is available")

    # 2) Build context through orchestrator
    fake_state = {
        "active_skill": "researcher",
        "request": "Analyze the omni-io crate architecture",
        "project_root": str(project_root),
        "messages": [],
    }
    orchestrator = create_planner_orchestrator()

    try:
        start = asyncio.get_running_loop().time()
        system_prompt = await orchestrator.build_context(fake_state)
        elapsed_ms = (asyncio.get_running_loop().time() - start) * 1000.0
    except Exception as exc:
        print(f"[FAIL] Failed to build context: {exc}")
        return False

    print(f"[PASS] Context built in {elapsed_ms:.2f} ms")

    # 3) Validate output content
    checks = [
        ("<active_protocol>", "contains opening active protocol tag"),
        ("</active_protocol>", "contains closing active protocol tag"),
        ("researcher", "contains researcher context"),
    ]
    all_passed = True
    for token, label in checks:
        if token in system_prompt:
            print(f"[PASS] Output {label}")
        else:
            print(f"[FAIL] Output missing: {label}")
            all_passed = False

    print(f"Prompt length: {len(system_prompt)} chars (~{len(system_prompt) // 4} tokens)")
    return all_passed


def main() -> int:
    """Script entry point."""
    try:
        ok = asyncio.run(verify_pipeline())
    except KeyboardInterrupt:
        print("Interrupted.")
        return 1
    except Exception as exc:
        print(f"[FAIL] Unhandled error: {exc}")
        return 1

    if ok:
        print("\nAll pipeline checks passed.")
        return 0
    print("\nPipeline verification failed.")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
