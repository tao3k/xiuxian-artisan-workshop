#!/usr/bin/env python3
"""Unit tests for blackbox config helpers."""

from __future__ import annotations

import argparse
import importlib
import json
import sys
from types import SimpleNamespace

import pytest

module = importlib.import_module("agent_channel_blackbox_config")


def test_parse_args_reads_prompt_and_defaults(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(sys, "argv", ["agent_channel_blackbox_config.py", "--prompt", "hello"])
    args = module.parse_args(
        default_telegram_webhook_url_fn=lambda: "http://127.0.0.1:9000/telegram/webhook",
        target_session_scope_placeholder="__target_session_scope__",
    )
    assert args.prompt == "hello"
    assert "telegram/webhook" in args.webhook_url


def test_build_probe_message_preserves_slash_command() -> None:
    assert module.build_probe_message("/session json", "trace-1") == "/session json"
    assert module.build_probe_message("hello", "trace-1").startswith("[trace-1] ")


def test_build_update_payload_includes_thread_and_username() -> None:
    payload = module.build_update_payload(
        12345,
        -1001,
        2002,
        "tester",
        "group-name",
        "hello",
        3003,
    )
    data = json.loads(payload)
    assert data["update_id"] == 12345
    assert data["message"]["from"]["username"] == "tester"
    assert data["message"]["message_thread_id"] == 3003
    assert data["message"]["chat"]["title"] == "group-name"


def test_build_config_resolves_ids_waits_and_defaults(
    tmp_path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("OMNI_BLACKBOX_MAX_IDLE_SECS", "25")
    args = argparse.Namespace(
        prompt="hello",
        max_wait=None,
        timeout=30,
        max_idle_secs=None,
        webhook_url="http://127.0.0.1:9000/telegram/webhook",
        log_file=str(tmp_path / "runtime.log"),
        chat_id=None,
        user_id=None,
        username="",
        chat_title="group",
        thread_id=None,
        session_partition="chat_user",
        secret_token="",
        no_follow=False,
        expect_log_regex=[],
        expect_event=["telegram.command.session_status_json.replied"],
        expect_reply_json_field=["json_kind=session_status"],
        expect_bot_regex=[],
        forbid_log_regex=[],
        no_fail_fast_error_log=False,
        allow_no_bot=False,
        allow_chat_id=["1001"],
    )

    cfg = module.build_config(
        args,
        probe_config_cls=lambda **kwargs: SimpleNamespace(**kwargs),
        session_ids_from_runtime_log_fn=lambda _log: (1001, 2001, 3001),
        username_from_settings_fn=lambda: "from-settings",
        username_from_runtime_log_fn=lambda _log: "from-log",
        parse_expected_field_fn=lambda value: tuple(value.split("=", 1)),
        parse_allow_chat_ids_fn=lambda values: tuple(int(v) for v in values if str(v).strip()),
        normalize_session_partition_fn=lambda value: value,
        telegram_webhook_secret_token_fn=lambda: "secret-token",
    )

    assert cfg.chat_id == 1001
    assert cfg.user_id == 2001
    assert cfg.thread_id == 3001
    assert cfg.max_wait_secs == 30
    assert cfg.max_idle_secs == 25
    assert cfg.secret_token == "secret-token"
    assert cfg.username == "from-settings"
    assert cfg.expect_reply_json_fields == (("json_kind", "session_status"),)


def test_build_config_rejects_chat_outside_allowlist(tmp_path) -> None:
    args = argparse.Namespace(
        prompt="hello",
        max_wait=10,
        timeout=None,
        max_idle_secs=10,
        webhook_url="http://127.0.0.1:9000/telegram/webhook",
        log_file=str(tmp_path / "runtime.log"),
        chat_id=1001,
        user_id=2001,
        username="tester",
        chat_title=None,
        thread_id=None,
        session_partition=None,
        secret_token=None,
        no_follow=False,
        expect_log_regex=[],
        expect_event=[],
        expect_reply_json_field=[],
        expect_bot_regex=[],
        forbid_log_regex=[],
        no_fail_fast_error_log=False,
        allow_no_bot=False,
        allow_chat_id=["1002"],
    )

    with pytest.raises(ValueError, match="allowlist"):
        module.build_config(
            args,
            probe_config_cls=lambda **kwargs: SimpleNamespace(**kwargs),
            session_ids_from_runtime_log_fn=lambda _log: (1001, 2001, None),
            username_from_settings_fn=lambda: "tester",
            username_from_runtime_log_fn=lambda _log: "tester",
            parse_expected_field_fn=lambda value: tuple(value.split("=", 1)),
            parse_allow_chat_ids_fn=lambda values: tuple(int(v) for v in values if str(v).strip()),
            normalize_session_partition_fn=lambda value: value,
            telegram_webhook_secret_token_fn=lambda: "secret-token",
        )
