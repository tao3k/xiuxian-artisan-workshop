#!/usr/bin/env python3
"""Port and runtime identifier helpers for memory CI gate config."""

from __future__ import annotations

import os
import socket
import sys
import time
from typing import Any


def default_valkey_prefix(profile: str) -> str:
    """Generate a run-scoped Valkey prefix."""
    safe_profile = profile.strip().lower() or "default"
    return f"omni-agent:session:ci:{safe_profile}:{os.getpid()}:{int(time.time() * 1000)}"


def can_bind_tcp(host: str, port: int) -> bool:
    """Return whether host:port is currently bindable."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        try:
            sock.bind((host, port))
        except OSError:
            return False
    return True


def allocate_free_tcp_port(
    host: str,
    *,
    avoid: set[int] | None = None,
    can_bind_tcp_fn: Any = can_bind_tcp,
) -> int:
    """Allocate a free TCP port on host, avoiding explicit blocked values."""
    blocked = avoid or set()
    for _ in range(32):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.bind((host, 0))
            candidate = int(sock.getsockname()[1])
        if candidate in blocked:
            continue
        if can_bind_tcp_fn(host, candidate):
            return candidate
    raise RuntimeError(f"failed to allocate free TCP port on host={host}")


def resolve_runtime_ports(
    webhook_port: int,
    telegram_api_port: int,
    *,
    host: str = "127.0.0.1",
    can_bind_tcp_fn: Any = can_bind_tcp,
    allocate_free_tcp_port_fn: Any = allocate_free_tcp_port,
) -> tuple[int, int]:
    """Resolve non-conflicting runtime ports for webhook and mock Telegram API."""
    resolved_telegram_api_port = telegram_api_port
    if not can_bind_tcp_fn(host, resolved_telegram_api_port):
        resolved_telegram_api_port = allocate_free_tcp_port_fn(host)
        print(
            "Port occupied; reassigned --telegram-api-port "
            f"{telegram_api_port} -> {resolved_telegram_api_port}",
            file=sys.stderr,
            flush=True,
        )

    resolved_webhook_port = webhook_port
    webhook_blocked = resolved_webhook_port == resolved_telegram_api_port or not can_bind_tcp_fn(
        host, resolved_webhook_port
    )
    if webhook_blocked:
        resolved_webhook_port = allocate_free_tcp_port_fn(host, avoid={resolved_telegram_api_port})
        print(
            "Port occupied/conflict; reassigned --webhook-port "
            f"{webhook_port} -> {resolved_webhook_port}",
            file=sys.stderr,
            flush=True,
        )

    return resolved_webhook_port, resolved_telegram_api_port


def default_run_suffix() -> str:
    """Return stable run suffix used for run-scoped artifacts."""
    return f"{os.getpid()}-{int(time.time() * 1000)}"
