"""Tests for run command surfaces after Python runtime module removal."""

from __future__ import annotations

import pytest


def test_run_command_registration_exists() -> None:
    from omni.agent.cli.commands.run import register_run_command

    assert callable(register_run_command)


def test_run_entry_module_is_removed() -> None:
    with pytest.raises(ModuleNotFoundError):
        __import__("omni.agent.workflows.run_entry")


def test_gateway_agent_commands_registered() -> None:
    from omni.agent.cli.commands.gateway_agent import (
        register_agent_command,
        register_gateway_command,
    )

    assert callable(register_gateway_command)
    assert callable(register_agent_command)
