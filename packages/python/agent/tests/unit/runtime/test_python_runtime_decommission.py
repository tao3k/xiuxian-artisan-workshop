"""Contract tests for Python runtime decommission guardrails."""

from __future__ import annotations

import importlib
import sys

import pytest


def test_assert_rust_runtime_or_raise_rejects_non_rust(monkeypatch: pytest.MonkeyPatch) -> None:
    from omni.agent.runtime import decommission as decommission_module

    monkeypatch.delenv(decommission_module.TEST_OVERRIDE_ENV, raising=False)
    monkeypatch.setattr(
        decommission_module,
        "get_setting",
        lambda key, default=None: (
            "python" if key == decommission_module.RUNTIME_ORCHESTRATOR_KEY else default
        ),
    )

    with pytest.raises(RuntimeError, match="decommissioned"):
        decommission_module.assert_rust_runtime_or_raise("test.entry")


def test_assert_rust_runtime_or_raise_allows_test_override(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    from omni.agent.runtime import decommission as decommission_module

    monkeypatch.setenv(decommission_module.TEST_OVERRIDE_ENV, "1")
    monkeypatch.setattr(
        decommission_module,
        "get_setting",
        lambda key, default=None: (
            "python" if key == decommission_module.RUNTIME_ORCHESTRATOR_KEY else default
        ),
    )

    decommission_module.assert_rust_runtime_or_raise("test.entry")


def test_main_module_is_removed(monkeypatch: pytest.MonkeyPatch) -> None:
    from omni.agent.runtime import decommission as decommission_module

    monkeypatch.delenv(decommission_module.TEST_OVERRIDE_ENV, raising=False)
    with pytest.raises(ModuleNotFoundError):
        importlib.import_module("omni.agent.main")


@pytest.mark.asyncio
async def test_omni_loop_module_is_removed(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    from omni.agent.runtime import decommission as decommission_module

    monkeypatch.delenv(decommission_module.TEST_OVERRIDE_ENV, raising=False)
    with pytest.raises(ModuleNotFoundError):
        importlib.import_module("omni.agent.cli.omni_loop")


def test_raise_python_runtime_decommissioned_allows_test_override(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    from omni.agent.runtime import decommission as decommission_module

    monkeypatch.setenv(decommission_module.TEST_OVERRIDE_ENV, "1")
    decommission_module.raise_python_runtime_decommissioned("test.removed.entry", "omni-agent repl")


def test_cli_entry_point_invokes_runtime_guard(monkeypatch: pytest.MonkeyPatch) -> None:
    app_module = importlib.import_module("omni.agent.cli.app")

    monkeypatch.setattr(app_module, "_bootstrap_configuration", lambda *_: None)
    monkeypatch.setattr(
        app_module,
        "assert_rust_runtime_or_raise",
        lambda *_: (_ for _ in ()).throw(RuntimeError("guard_called")),
    )
    monkeypatch.setattr(sys, "argv", ["omni", "version"])

    with pytest.raises(RuntimeError, match="guard_called"):
        app_module.entry_point()
