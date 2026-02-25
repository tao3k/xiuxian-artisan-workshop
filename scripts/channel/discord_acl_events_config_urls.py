#!/usr/bin/env python3
"""Ingress URL normalization helpers for Discord ACL probes."""

from __future__ import annotations

import os


def normalize_ingress_bind_for_local_url(bind_addr: str) -> str:
    """Normalize bind host:port for local loopback URL usage."""
    token = bind_addr.strip()
    if not token:
        return "127.0.0.1:18082"
    host, sep, port = token.rpartition(":")
    if not sep:
        return f"127.0.0.1:{token}"
    normalized_host = host.strip("[]").strip()
    if normalized_host in {"", "0.0.0.0", "::"}:
        normalized_host = "127.0.0.1"
    return f"{normalized_host}:{port.strip()}"


def default_ingress_url() -> str:
    """Resolve Discord ingress URL from env with bind/path fallback."""
    explicit = os.environ.get("OMNI_DISCORD_INGRESS_URL", "").strip()
    if explicit:
        return explicit
    bind_addr = os.environ.get("OMNI_AGENT_DISCORD_INGRESS_BIND", "127.0.0.1:18082")
    path = os.environ.get("OMNI_AGENT_DISCORD_INGRESS_PATH", "/discord/ingress").strip()
    if not path:
        path = "/discord/ingress"
    if not path.startswith("/"):
        path = f"/{path}"
    return f"http://{normalize_ingress_bind_for_local_url(bind_addr)}{path}"
