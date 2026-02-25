#!/usr/bin/env python3
"""HTTP payload and transport helpers for concurrent session probes."""

from __future__ import annotations

import json
import urllib.error
import urllib.request


def build_payload(
    *,
    update_id: int,
    chat_id: int,
    user_id: int,
    username: str | None,
    prompt: str,
    thread_id: int | None,
) -> bytes:
    """Build synthetic Telegram update payload for webhook probe."""
    from_user: dict[str, object] = {"id": user_id, "is_bot": False, "first_name": "ConcurrentProbe"}
    if username:
        from_user["username"] = username
    message: dict[str, object] = {
        "message_id": update_id % 2_000_000_000,
        "date": update_id // 1_000_000,
        "text": prompt,
        "chat": {"id": chat_id, "type": "private" if chat_id > 0 else "group"},
        "from": from_user,
    }
    if thread_id is not None:
        message["message_thread_id"] = thread_id
    body = {"update_id": update_id, "message": message}
    return json.dumps(body, ensure_ascii=False).encode("utf-8")


def post_webhook(url: str, payload: bytes, secret_token: str | None) -> tuple[int, str]:
    """Post one payload to Telegram webhook endpoint."""
    request = urllib.request.Request(url=url, data=payload, method="POST")
    request.add_header("Content-Type", "application/json")
    if secret_token:
        request.add_header("X-Telegram-Bot-Api-Secret-Token", secret_token)
    try:
        with urllib.request.urlopen(request, timeout=15) as response:
            return int(response.status), response.read().decode("utf-8", errors="replace")
    except urllib.error.HTTPError as error:
        return int(error.code), error.read().decode("utf-8", errors="replace")
    except urllib.error.URLError as error:
        return 0, f"connection_error: {error.reason}"
