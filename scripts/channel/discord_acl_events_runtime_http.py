#!/usr/bin/env python3
"""HTTP and payload helpers for Discord ACL runtime probes."""

from __future__ import annotations

import json
import os
import re
import secrets
import time
import urllib.error
import urllib.request
from typing import Any


def now_event_id() -> str:
    """Generate unique synthetic Discord ingress event id."""
    base_ms = int(time.time() * 1000)
    pid_component = os.getpid() % 10_000
    rand_component = secrets.randbelow(100)
    return str((base_ms * 1_000_000) + (pid_component * 100) + rand_component)


def build_ingress_payload(config: Any, event_id: str, prompt: str) -> str:
    """Build Discord ingress payload JSON string."""
    payload: dict[str, object] = {
        "id": event_id,
        "content": prompt,
        "channel_id": config.channel_id,
        "author": {"id": config.user_id},
    }
    if config.username:
        payload["author"] = {"id": config.user_id, "username": config.username}
    if config.guild_id:
        payload["guild_id"] = config.guild_id
        if config.role_ids:
            payload["member"] = {"roles": list(config.role_ids)}
    return json.dumps(payload, ensure_ascii=False)


def post_ingress_event(
    url: str,
    payload: str,
    secret_token: str | None,
    *,
    secret_header_name: str,
) -> tuple[int, str]:
    """Post one synthetic event to Discord ingress endpoint."""
    data = payload.encode("utf-8")
    request = urllib.request.Request(url=url, data=data, method="POST")
    request.add_header("content-type", "application/json")
    if secret_token:
        request.add_header(secret_header_name, secret_token)
    try:
        with urllib.request.urlopen(request, timeout=15) as response:
            body = response.read().decode("utf-8", errors="replace")
            return response.status, body
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf-8", errors="replace")
        return int(error.code), body
    except urllib.error.URLError as error:
        return 0, f"connection_error: {error.reason}"


def compile_patterns(patterns: tuple[str, ...]) -> list[re.Pattern[str]]:
    """Compile regex patterns."""
    return [re.compile(pattern) for pattern in patterns]
