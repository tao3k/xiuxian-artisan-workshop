"""Unit tests for MCP list-changed notifications."""

from __future__ import annotations

import asyncio
from types import SimpleNamespace
from unittest.mock import AsyncMock

from mcp.types import Tool

from omni.agent.mcp_server.server import AgentMCPServer


def test_send_tool_list_changed_handles_missing_request_context() -> None:
    """No active request context should not raise a ContextVar error."""
    server = AgentMCPServer()
    asyncio.run(server.send_tool_list_changed())


def test_send_tool_list_changed_prefers_transport_broadcast() -> None:
    """When transport.broadcast is available, it should be used."""
    server = AgentMCPServer()
    broadcast = AsyncMock()
    server._transport = SimpleNamespace(broadcast=broadcast)

    asyncio.run(server.send_tool_list_changed())
    broadcast.assert_awaited_once()


def test_send_tool_list_changed_uses_session_when_transport_missing() -> None:
    """When a request session is available, send_notification should be called."""
    server = AgentMCPServer()
    send_notification = AsyncMock()
    server._app = SimpleNamespace(
        request_context=SimpleNamespace(
            session=SimpleNamespace(send_notification=send_notification)
        )
    )

    asyncio.run(server.send_tool_list_changed())
    send_notification.assert_awaited_once()


def test_send_tool_list_changed_invalidates_standard_tools_cache() -> None:
    """Tool cache should be invalidated before notifying clients."""
    server = AgentMCPServer()
    server._standard_tools_cache = [
        Tool(name="demo.echo", description="Echo", inputSchema={"type": "object"})
    ]
    asyncio.run(server.send_tool_list_changed())
    assert server._standard_tools_cache is None
