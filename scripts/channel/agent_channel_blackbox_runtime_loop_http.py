#!/usr/bin/env python3
"""Webhook-post helper for agent channel blackbox runtime loop."""

from __future__ import annotations

import sys
from typing import Any


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

    status, body = post_webhook_update_fn(cfg.webhook_url, payload, cfg.secret_token)
    if status == 200:
        return None

    print(f"Error: webhook POST failed (HTTP {status}).", file=sys.stderr)
    print(f"Webhook URL: {cfg.webhook_url}", file=sys.stderr)
    print("Response body:", file=sys.stderr)
    for line in body.splitlines():
        print(f"  {line}", file=sys.stderr)
    print("MCP diagnostics:")
    print("  mcp_last_event=")
    print("  mcp_waiting_seen=false")
    print("  mcp_event_counts={}")
    return 1
