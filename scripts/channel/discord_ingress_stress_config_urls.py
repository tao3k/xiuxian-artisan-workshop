#!/usr/bin/env python3
"""Ingress URL defaults for Discord ingress stress probe config."""

from __future__ import annotations

import os

from resolve_mcp_endpoint import resolve_mcp_endpoint


def _default_ingress_bind() -> str:
    """Resolve default ingress bind host:port for local stress probes."""
    host = str(resolve_mcp_endpoint()["host"])
    return f"{host}:18082"


def normalize_ingress_bind_for_local_url(bind_addr: str) -> str:
    """Normalize bind host:port into loopback-safe local URL host:port."""
    token = bind_addr.strip()
    if not token:
        return _default_ingress_bind()
    host, sep, port = token.rpartition(":")
    if not sep:
        default_host = str(resolve_mcp_endpoint()["host"])
        return f"{default_host}:{token}"
    normalized_host = host.strip("[]").strip()
    if normalized_host in {"", "0.0.0.0", "::"}:
        normalized_host = str(resolve_mcp_endpoint()["host"])
    return f"{normalized_host}:{port.strip()}"


def default_ingress_url() -> str:
    """Resolve ingress URL from explicit env or bind/path fallback."""
    explicit = os.environ.get("OMNI_DISCORD_INGRESS_URL", "").strip()
    if explicit:
        return explicit
    bind_addr = os.environ.get("OMNI_AGENT_DISCORD_INGRESS_BIND", _default_ingress_bind())
    ingress_path = os.environ.get("OMNI_AGENT_DISCORD_INGRESS_PATH", "/discord/ingress").strip()
    if not ingress_path:
        ingress_path = "/discord/ingress"
    if not ingress_path.startswith("/"):
        ingress_path = f"/{ingress_path}"
    return f"http://{normalize_ingress_bind_for_local_url(bind_addr)}{ingress_path}"
