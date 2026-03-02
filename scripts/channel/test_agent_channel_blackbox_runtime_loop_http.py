#!/usr/bin/env python3
"""Unit tests for blackbox webhook POST preflight handling."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import SimpleNamespace

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

module = importlib.import_module("agent_channel_blackbox_runtime_loop_http")


def _cfg() -> SimpleNamespace:
    return SimpleNamespace(
        webhook_url="http://127.0.0.1:18081/telegram/webhook",
        chat_id=1,
        user_id=2,
        username="tester",
        chat_title=None,
        thread_id=None,
        secret_token=None,
    )


def test_handle_webhook_post_retries_transient_connection_errors(monkeypatch) -> None:
    cfg = _cfg()
    attempts = {"n": 0}

    def _post(_url: str, _payload: str, _secret: str | None) -> tuple[int, str]:
        attempts["n"] += 1
        if attempts["n"] < 3:
            return 0, "connection_error: [Errno 61] Connection refused"
        return 200, "ok"

    monkeypatch.setattr(module.time, "sleep", lambda _secs: None)
    code = module.handle_webhook_post(
        cfg,
        update_id=7,
        message_text="hello",
        build_update_payload_fn=lambda **_kwargs: "{}",
        post_webhook_update_fn=_post,
    )

    assert code is None
    assert attempts["n"] == 3


def test_handle_webhook_post_does_not_retry_non_retryable_http_error(monkeypatch) -> None:
    cfg = _cfg()
    attempts = {"n": 0}

    def _post(_url: str, _payload: str, _secret: str | None) -> tuple[int, str]:
        attempts["n"] += 1
        return 403, "forbidden"

    monkeypatch.setattr(module.time, "sleep", lambda _secs: None)
    code = module.handle_webhook_post(
        cfg,
        update_id=8,
        message_text="hello",
        build_update_payload_fn=lambda **_kwargs: "{}",
        post_webhook_update_fn=_post,
    )

    assert code == 1
    assert attempts["n"] == 1
