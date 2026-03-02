#!/usr/bin/env python3
"""Verify MCP server integration for the native workflow runtime.

Usage:
    uv run python scripts/verify_mcp_interface.py
"""

from __future__ import annotations

import asyncio
import json

from mcp.types import Resource, Tool
from pydantic.networks import AnyUrl

from omni.agent.mcp_server.server import AgentMCPServer
from omni.agent.mcp_server.server import main as mcp_main


def _print_section(title: str) -> None:
    print("\n" + "=" * 60)
    print(title)
    print("=" * 60)


def test_imports() -> bool:
    """Verify core MCP imports are available."""
    _print_section("TEST 1: Import Verification")
    try:
        _ = AgentMCPServer
        _ = Tool
        _ = Resource
        print("[PASS] AgentMCPServer, Tool, and Resource imports are available")
        return True
    except Exception as exc:
        print(f"[FAIL] Import failed: {exc}")
        return False


def test_server_initialization() -> bool:
    """Verify MCP server object initializes and exposes core attributes."""
    _print_section("TEST 2: Server Initialization")
    try:
        server = AgentMCPServer()
        if server._app is None:
            print("[FAIL] _app is None")
            return False
        if not hasattr(server, "_start_time"):
            print("[FAIL] _start_time missing")
            return False
        print("[PASS] AgentMCPServer initialized with _app and _start_time")
        return True
    except Exception as exc:
        print(f"[FAIL] Server initialization failed: {exc}")
        return False


def test_mcp_types_contract() -> bool:
    """Verify MCP type payloads can be constructed with expected schema."""
    _print_section("TEST 3: MCP Type Contracts")
    try:
        tool = Tool(
            name="demo.echo",
            description="Echo tool",
            inputSchema={
                "type": "object",
                "properties": {"message": {"type": "string"}},
                "required": ["message"],
            },
        )
        resource = Resource(
            uri=AnyUrl("omni://system/stats"),
            name="System Stats",
            description="Runtime health payload",
            mimeType="application/json",
        )
        if tool.name != "demo.echo":
            print("[FAIL] Tool name mismatch")
            return False
        if str(resource.uri) != "omni://system/stats":
            print("[FAIL] Resource URI mismatch")
            return False
        print("[PASS] MCP Tool/Resource contracts validated")
        return True
    except Exception as exc:
        print(f"[FAIL] MCP type contract test failed: {exc}")
        return False


def test_prompt_payloads() -> bool:
    """Verify prompt payload helpers return structured content."""
    _print_section("TEST 4: Prompt Payloads")
    try:
        server = AgentMCPServer()
        prompts = [
            server._get_default_prompt(),
            server._get_researcher_prompt(),
            server._get_developer_prompt(),
        ]
        for idx, payload in enumerate(prompts, start=1):
            if not isinstance(payload, dict):
                print(f"[FAIL] Prompt #{idx} is not a dict")
                return False
            if not payload.get("description"):
                print(f"[FAIL] Prompt #{idx} missing description")
                return False
            if not payload.get("content"):
                print(f"[FAIL] Prompt #{idx} missing content")
                return False
        print("[PASS] Default/researcher/developer prompt payloads are valid")
        return True
    except Exception as exc:
        print(f"[FAIL] Prompt payload test failed: {exc}")
        return False


async def test_resource_readers_without_kernel() -> bool:
    """Verify resource readers return sane JSON when kernel is not ready."""
    _print_section("TEST 5: Resource Readers (Kernel Uninitialized)")
    try:
        server = AgentMCPServer()
        server._kernel = None

        project_context = json.loads(server._read_project_context())
        system_stats = json.loads(server._read_system_stats())
        agent_memory = json.loads(await server._read_agent_memory())

        if "contexts" not in project_context and "error" not in project_context:
            print("[FAIL] Project context payload missing expected keys")
            return False
        if "kernel_ready" not in system_stats and "error" not in system_stats:
            print("[FAIL] System stats payload missing expected keys")
            return False
        if "error" not in agent_memory:
            print("[FAIL] Agent memory should report kernel-not-ready error")
            return False

        print("[PASS] Resource readers return valid JSON envelopes")
        return True
    except Exception as exc:
        print(f"[FAIL] Resource reader test failed: {exc}")
        return False


async def test_notification_path_without_transport() -> bool:
    """Verify tool-list notification path does not crash without transport/session."""
    _print_section("TEST 6: listChanged Notification Path")
    try:
        server = AgentMCPServer()
        await server.send_tool_list_changed()
        print("[PASS] send_tool_list_changed completed without transport")
        return True
    except Exception as exc:
        print(f"[FAIL] Notification path failed: {exc}")
        return False


def test_entry_point_callable() -> bool:
    """Verify module entry point is callable."""
    _print_section("TEST 7: CLI Entry Point")
    try:
        if not callable(mcp_main):
            print("[FAIL] mcp_main is not callable")
            return False
        print("[PASS] mcp_main is callable")
        return True
    except Exception as exc:
        print(f"[FAIL] CLI entry point test failed: {exc}")
        return False


def main() -> int:
    """Run all MCP verification tests."""
    print("\n" + "=" * 60)
    print("MCP INTERFACE VERIFICATION")
    print("Native Workflow Runtime")
    print("=" * 60)

    results: list[tuple[str, bool]] = [
        ("Import Verification", test_imports()),
        ("Server Initialization", test_server_initialization()),
        ("MCP Type Contracts", test_mcp_types_contract()),
        ("Prompt Payloads", test_prompt_payloads()),
        (
            "Resource Readers (Kernel Uninitialized)",
            asyncio.run(test_resource_readers_without_kernel()),
        ),
        (
            "listChanged Notification Path",
            asyncio.run(test_notification_path_without_transport()),
        ),
        ("CLI Entry Point", test_entry_point_callable()),
    ]

    _print_section("VERIFICATION SUMMARY")
    all_passed = True
    for name, passed in results:
        status = "PASS" if passed else "FAIL"
        print(f"[{status}] {name}")
        if not passed:
            all_passed = False

    if all_passed:
        print("\nAll MCP checks passed.")
        return 0
    print("\nMCP verification failed.")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
