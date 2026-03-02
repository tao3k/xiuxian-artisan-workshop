"""Contract tests that lock Python runtime decommission behavior.

These checks are anti-regression guardrails for Rust-only orchestration.
"""

from __future__ import annotations

import importlib
import inspect
from pathlib import Path

import pytest
import yaml

from omni.foundation.runtime.gitops import get_project_root


def test_removed_python_runtime_modules_are_not_importable() -> None:
    """Removed Python runtime modules should stay absent from package imports."""
    with pytest.raises(ModuleNotFoundError):
        importlib.import_module("omni.agent.main")

    with pytest.raises(ModuleNotFoundError):
        importlib.import_module("omni.agent.cli.omni_loop")

    with pytest.raises(ModuleNotFoundError):
        importlib.import_module("omni.agent.workflows.run_entry")


@pytest.mark.asyncio
async def test_gateway_runtime_helpers_remain_blocked() -> None:
    """Remaining Python gateway helper paths must stay blocked."""
    from omni.agent.cli.commands import gateway_agent

    with pytest.raises(RuntimeError, match="decommissioned"):
        await gateway_agent._webhook_loop(port=19001, host="127.0.0.1")

    with pytest.raises(RuntimeError, match="decommissioned"):
        await gateway_agent._stdio_loop(session_id="s1")


def test_cli_entrypoint_source_keeps_rust_runtime_guard_call() -> None:
    """entry_point must keep the Rust runtime guard invocation."""
    app_module = importlib.import_module("omni.agent.cli.app")
    source = inspect.getsource(app_module.entry_point)
    assert "assert_rust_runtime_or_raise" in source
    assert 'assert_rust_runtime_or_raise("omni.cli.entry_point")' in source


def test_omni_loop_module_is_removed() -> None:
    """Python OmniLoop module should be fully removed."""
    with pytest.raises(ModuleNotFoundError):
        importlib.import_module("omni.agent.core.omni.loop")


def test_omega_module_is_removed() -> None:
    """Python Omega module should be fully removed."""
    with pytest.raises(ModuleNotFoundError):
        importlib.import_module("omni.agent.core.omni.omega")


def test_system_default_settings_pin_rust_runtime_orchestrator() -> None:
    """System defaults must keep Rust runtime orchestration authoritative."""
    project_root = Path(get_project_root())
    settings_path = project_root / "packages" / "conf" / "settings.yaml"
    settings = yaml.safe_load(settings_path.read_text(encoding="utf-8"))

    agent_settings = settings.get("agent", {})
    assert agent_settings.get("runtime_orchestrator") == "rust"
    assert agent_settings.get("allow_python_runtime_for_tests") is False


def test_core_omni_public_api_excludes_runtime_symbols() -> None:
    """Public omni facade must not export removed Python runtime classes."""
    omni_module = importlib.import_module("omni.agent.core.omni")

    assert not hasattr(omni_module, "OmniLoopConfig")
    assert not hasattr(omni_module, "OmniLoop")
    assert not hasattr(omni_module, "OmegaRunner")
    assert not hasattr(omni_module, "MissionConfig")
