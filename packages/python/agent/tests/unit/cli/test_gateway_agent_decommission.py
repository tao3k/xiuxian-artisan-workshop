"""Tests for decommissioned Python gateway helper loops."""

from __future__ import annotations

import pytest

from omni.agent.cli.commands import gateway_agent


@pytest.mark.asyncio
async def test_webhook_loop_is_decommissioned() -> None:
    with pytest.raises(RuntimeError, match="decommissioned"):
        await gateway_agent._webhook_loop(port=19001, host="127.0.0.1")


@pytest.mark.asyncio
async def test_stdio_loop_is_decommissioned() -> None:
    with pytest.raises(RuntimeError, match="decommissioned"):
        await gateway_agent._stdio_loop(session_id="test-session")
