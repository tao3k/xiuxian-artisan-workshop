#!/usr/bin/env python3
"""CLI parsing helpers for MCP startup stress config."""

from __future__ import annotations

import argparse
from pathlib import Path

from path_resolver import default_report_path, project_root_from
from resolve_mcp_endpoint import resolve_mcp_endpoint


def _default_health_url() -> str:
    """Resolve default MCP health endpoint from settings."""
    return str(resolve_mcp_endpoint()["health_url"])


def parse_args() -> argparse.Namespace:
    """Parse command-line arguments for stress probe."""
    parser = argparse.ArgumentParser(
        description=(
            "Stress MCP startup by repeatedly spawning omni-agent gateway and "
            "checking handshake logs."
        )
    )
    parser.add_argument("--rounds", type=int, default=6, help="Number of rounds (default: 6).")
    parser.add_argument(
        "--parallel",
        type=int,
        default=3,
        help="Concurrent startups per round (default: 3).",
    )
    parser.add_argument(
        "--startup-timeout-secs",
        type=int,
        default=45,
        help="Max seconds waiting for gateway ready marker (default: 45).",
    )
    parser.add_argument(
        "--cooldown-secs",
        type=float,
        default=0.2,
        help="Delay between rounds (default: 0.2).",
    )
    parser.add_argument(
        "--executable",
        default="target/debug/omni-agent",
        help="Path to omni-agent executable (default: target/debug/omni-agent).",
    )
    parser.add_argument(
        "--mcp-config",
        default=".mcp.json",
        help="Path to MCP config file (default: .mcp.json).",
    )
    parser.add_argument(
        "--project-root",
        default=str(project_root_from(Path.cwd())),
        help="Project root for process cwd (default: auto-detect from .git).",
    )
    parser.add_argument(
        "--bind-addr",
        default="",
        help="Gateway bind address (default: auto, prefer settings-derived host with ephemeral port).",
    )
    parser.add_argument(
        "--rust-log",
        default=(
            "omni_agent::gateway::http=info,"
            "omni_agent::mcp_pool=debug,"
            "omni_agent::main_agent_builder=info"
        ),
        help="RUST_LOG used by spawned probe process.",
    )
    parser.add_argument(
        "--restart-mcp-cmd",
        default="",
        help="Optional shell command to restart MCP server between rounds.",
    )
    parser.add_argument(
        "--restart-mcp-settle-secs",
        type=float,
        default=2.0,
        help="Sleep after restart command before next round (default: 2.0).",
    )
    parser.add_argument(
        "--health-url",
        default=_default_health_url(),
        help="MCP health endpoint checked before stress (default: resolved from settings).",
    )
    parser.add_argument(
        "--strict-health-check",
        action="store_true",
        help="Fail immediately if health check is unavailable.",
    )
    parser.add_argument(
        "--health-probe-interval-secs",
        type=float,
        default=0.2,
        help="Background /health sampling interval during stress (default: 0.2, 0 disables).",
    )
    parser.add_argument(
        "--health-probe-timeout-secs",
        type=float,
        default=1.0,
        help="Timeout for each background /health probe (default: 1.0).",
    )
    parser.add_argument(
        "--output-json",
        default=str(default_report_path("omni-agent-mcp-startup-stress.json")),
        help="Output JSON report path.",
    )
    parser.add_argument(
        "--output-markdown",
        default=str(default_report_path("omni-agent-mcp-startup-stress.md")),
        help="Output Markdown report path.",
    )
    return parser.parse_args()
