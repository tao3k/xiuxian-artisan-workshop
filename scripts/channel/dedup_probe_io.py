#!/usr/bin/env python3
"""I/O and payload helpers for deterministic dedup probe."""

from __future__ import annotations

import json
import re
import urllib.error
import urllib.request
from typing import Any

ANSI_ESCAPE_RE = re.compile(r"\x1b\[[0-9;]*m")


def strip_ansi(value: str) -> str:
    """Strip ANSI escape codes from one line."""
    return ANSI_ESCAPE_RE.sub("", value)


def count_lines(path: Any, *, init_log_cursor_fn: Any) -> int:
    """Initialize offset cursor for incremental log polling."""
    return init_log_cursor_fn(path, kind="offset").value


def read_new_lines(
    path: Any,
    cursor: int,
    *,
    read_new_log_lines_with_cursor_fn: Any,
    log_cursor_cls: Any,
) -> tuple[int, list[str]]:
    """Read appended log lines and return updated offset cursor."""
    next_cursor, lines = read_new_log_lines_with_cursor_fn(
        path,
        log_cursor_cls(kind="offset", value=cursor),
    )
    return next_cursor.value, lines


def post_webhook_update(url: str, payload: bytes, secret_token: str | None) -> tuple[int, str]:
    """Post one webhook update payload."""
    request = urllib.request.Request(url=url, data=payload, method="POST")
    request.add_header("Content-Type", "application/json")
    if secret_token:
        request.add_header("X-Telegram-Bot-Api-Secret-Token", secret_token)
    try:
        with urllib.request.urlopen(request, timeout=15) as response:
            body = response.read().decode("utf-8", errors="replace")
            return int(response.status), body
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf-8", errors="replace")
        return int(error.code), body
    except urllib.error.URLError as error:
        return 0, f"connection_error: {error.reason}"


def build_payload(cfg: Any, update_id: int) -> bytes:
    """Build Telegram-compatible update payload for dedup probe."""
    from_user: dict[str, object] = {"id": cfg.user_id, "is_bot": False, "first_name": "DedupProbe"}
    if cfg.username:
        from_user["username"] = cfg.username
    message: dict[str, object] = {
        "message_id": update_id % 2_000_000_000,
        "date": update_id // 1_000_000,
        "text": cfg.text,
        "chat": {"id": cfg.chat_id, "type": "private" if cfg.chat_id > 0 else "group"},
        "from": from_user,
    }
    if cfg.thread_id is not None:
        message["message_thread_id"] = cfg.thread_id
    body = {"update_id": update_id, "message": message}
    return json.dumps(body, ensure_ascii=False).encode("utf-8")
