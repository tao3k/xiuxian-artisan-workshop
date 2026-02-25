#!/usr/bin/env python3
"""HTTP payload/post helpers for Discord ingress stress runtime."""

from __future__ import annotations

import json
import time
import urllib.error
import urllib.request
from typing import Any

DISCORD_INGRESS_SECRET_HEADER = "x-omni-discord-ingress-token"


def build_ingress_payload(cfg: Any, event_id: str, prompt: str) -> bytes:
    """Build synthetic Discord message event payload."""
    author: dict[str, object] = {"id": cfg.user_id}
    if cfg.username:
        author["username"] = cfg.username
    payload: dict[str, object] = {
        "id": event_id,
        "content": prompt,
        "channel_id": cfg.channel_id,
        "author": author,
    }
    if cfg.guild_id:
        payload["guild_id"] = cfg.guild_id
        if cfg.role_ids:
            payload["member"] = {"roles": list(cfg.role_ids)}
    return json.dumps(payload, ensure_ascii=False).encode("utf-8")


def post_ingress_event(
    url: str,
    payload: bytes,
    secret_token: str | None,
    timeout_secs: float,
) -> tuple[int, str, float]:
    """Post one event to Discord ingress and capture status/body/latency_ms."""
    request = urllib.request.Request(url=url, data=payload, method="POST")
    request.add_header("content-type", "application/json")
    if secret_token:
        request.add_header(DISCORD_INGRESS_SECRET_HEADER, secret_token)

    started = time.perf_counter()
    try:
        with urllib.request.urlopen(request, timeout=timeout_secs) as response:
            latency_ms = (time.perf_counter() - started) * 1000.0
            body = response.read().decode("utf-8", errors="replace")
            return int(response.status), body, latency_ms
    except urllib.error.HTTPError as error:
        latency_ms = (time.perf_counter() - started) * 1000.0
        body = error.read().decode("utf-8", errors="replace")
        return int(error.code), body, latency_ms
    except urllib.error.URLError as error:
        latency_ms = (time.perf_counter() - started) * 1000.0
        return 0, f"connection_error: {error.reason}", latency_ms
