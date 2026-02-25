#!/usr/bin/env python3
"""HTTP posting helper for channel blackbox probe."""

from __future__ import annotations

import urllib.error
import urllib.request


def post_webhook_update(
    url: str,
    payload: str,
    secret_token: str | None,
    *,
    secret_header: str = "X-Telegram-Bot-Api-Secret-Token",
) -> tuple[int, str]:
    """Post one webhook update payload and return status/body."""
    data = payload.encode("utf-8")
    request = urllib.request.Request(url=url, data=data, method="POST")
    request.add_header("Content-Type", "application/json")
    if secret_token:
        request.add_header(secret_header, secret_token)
    try:
        with urllib.request.urlopen(request, timeout=15) as response:
            body = response.read().decode("utf-8", errors="replace")
            return response.status, body
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf-8", errors="replace")
        return int(error.code), body
    except urllib.error.URLError as error:
        return 0, f"connection_error: {error.reason}"
