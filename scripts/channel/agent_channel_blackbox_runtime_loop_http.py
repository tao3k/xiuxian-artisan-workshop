#!/usr/bin/env python3
"""Webhook-post helper for agent channel blackbox runtime loop."""

from __future__ import annotations

import sys
import time
from typing import Any


def _is_retryable_webhook_error(status: int, body: str) -> bool:
    if status == 0:
        return True
    if status in (502, 503, 504):
        return True
    lowered = body.lower()
    return "connection refused" in lowered or "timed out" in lowered


def handle_webhook_post(
    cfg: Any,
    *,
    update_id: int,
    message_text: str,
    build_update_payload_fn: Any,
    post_webhook_update_fn: Any,
) -> int | None:
    """Post synthetic webhook update and return error code on failure."""
    payload = build_update_payload_fn(
        update_id=update_id,
        chat_id=cfg.chat_id,
        user_id=cfg.user_id,
        username=cfg.username,
        chat_title=cfg.chat_title,
        text=message_text,
        thread_id=cfg.thread_id,
    )

    attempts = 6
    status = 0
    body = ""
    for attempt in range(1, attempts + 1):
        status, body = post_webhook_update_fn(cfg.webhook_url, payload, cfg.secret_token)
        if status == 200:
            return None
        if attempt >= attempts or not _is_retryable_webhook_error(status, body):
            break
        time.sleep(min(0.25 * attempt, 1.0))

    print(f"Error: webhook POST failed (HTTP {status}).", file=sys.stderr)
    if attempts > 1:
        print(f"Attempts: {attempt}", file=sys.stderr)
    print(f"Webhook URL: {cfg.webhook_url}", file=sys.stderr)
    print("Response body:", file=sys.stderr)
    for line in body.splitlines():
        print(f"  {line}", file=sys.stderr)
    print("MCP diagnostics:")
    print("  mcp_last_event=")
    print("  mcp_waiting_seen=false")
    print("  mcp_event_counts={}")
    return 1
