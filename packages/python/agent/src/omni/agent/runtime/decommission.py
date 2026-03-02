"""Python runtime decommission guardrails.

This module centralizes fail-fast checks for legacy Python orchestration
entrypoints. Runtime orchestration authority is Rust-only (`omni-agent`).
"""

from __future__ import annotations

import os
from typing import Any

from omni.foundation.config.settings import get_setting

TEST_OVERRIDE_ENV = "OMNI_AGENT_ALLOW_PYTHON_RUNTIME_FOR_TESTS"
RUNTIME_ORCHESTRATOR_KEY = "agent.runtime_orchestrator"
ALLOW_PYTHON_FOR_TESTS_KEY = "agent.allow_python_runtime_for_tests"

_RUST_ORCHESTRATOR_VALUES = {"rust", "rust-only", "rust_only", "omni-agent", "omni_agent"}


def _parse_bool(value: Any, default: bool = False) -> bool:
    if isinstance(value, bool):
        return value
    if value is None:
        return default
    if isinstance(value, (int, float)):
        return bool(value)
    normalized = str(value).strip().lower()
    if normalized in {"1", "true", "yes", "on"}:
        return True
    if normalized in {"0", "false", "no", "off"}:
        return False
    return default


def python_runtime_test_override_enabled() -> bool:
    """Return True only for explicit local test overrides."""
    if TEST_OVERRIDE_ENV in os.environ:
        return _parse_bool(os.environ.get(TEST_OVERRIDE_ENV), default=False)
    return _parse_bool(get_setting(ALLOW_PYTHON_FOR_TESTS_KEY, False), default=False)


def runtime_orchestrator_is_rust() -> bool:
    """Return True when config pins runtime orchestration to Rust."""
    configured = str(get_setting(RUNTIME_ORCHESTRATOR_KEY, "rust")).strip().lower()
    return configured in _RUST_ORCHESTRATOR_VALUES


def assert_rust_runtime_or_raise(entrypoint: str) -> None:
    """Fail fast when configuration drifts away from Rust-only orchestration."""
    if runtime_orchestrator_is_rust():
        return
    if python_runtime_test_override_enabled():
        return
    configured = get_setting(RUNTIME_ORCHESTRATOR_KEY, "rust")
    raise RuntimeError(
        "Python runtime orchestration is decommissioned. "
        f"Entry: {entrypoint}. "
        f"Invalid `{RUNTIME_ORCHESTRATOR_KEY}`={configured!r}; expected 'rust'. "
        f"For test-only overrides, set {TEST_OVERRIDE_ENV}=1."
    )


def raise_python_runtime_decommissioned(entrypoint: str, replacement: str) -> None:
    """Always reject legacy Python runtime entrypoints unless test override is enabled."""
    if python_runtime_test_override_enabled():
        return
    raise RuntimeError(
        "Python runtime is decommissioned. "
        f"Entry: {entrypoint}. "
        f"Use: {replacement}. "
        f"For test-only overrides, set {TEST_OVERRIDE_ENV}=1."
    )


__all__ = [
    "ALLOW_PYTHON_FOR_TESTS_KEY",
    "RUNTIME_ORCHESTRATOR_KEY",
    "TEST_OVERRIDE_ENV",
    "assert_rust_runtime_or_raise",
    "python_runtime_test_override_enabled",
    "raise_python_runtime_decommissioned",
    "runtime_orchestrator_is_rust",
]
