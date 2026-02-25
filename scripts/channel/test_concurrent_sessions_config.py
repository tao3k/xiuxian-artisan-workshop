#!/usr/bin/env python3
"""Unit tests for concurrent session config helpers."""

from __future__ import annotations

import argparse
import importlib
import sys
from pathlib import Path

import pytest

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

config_module = importlib.import_module("concurrent_sessions_config")
models_module = importlib.import_module("concurrent_sessions_models")


def test_resolve_runtime_partition_mode_prefers_override() -> None:
    mode = config_module.resolve_runtime_partition_mode(
        Path(".run/logs/omni-agent-webhook.log"),
        override="chat-user",
        normalize_partition_mode_fn=lambda value: "chat_user" if value else None,
        session_partition_mode_from_runtime_log_fn=lambda _path: "user",
        telegram_session_partition_mode_fn=lambda: "chat_thread_user",
    )
    assert mode == "chat_user"


def test_expected_session_keys_forwards_to_dependency() -> None:
    keys = config_module.expected_session_keys(
        130,
        7,
        None,
        "chat_user",
        expected_session_keys_fn=lambda chat, user, thread, mode, normalize_partition_fn: (
            f"{chat}:{user}:{thread}",
            normalize_partition_fn(mode) or "",
        ),
        normalize_partition_mode_fn=lambda value: (value or "").upper(),
    )
    assert keys == ("130:7:None", "CHAT_USER")


def test_build_config_rejects_identical_session_targets() -> None:
    args = argparse.Namespace(
        max_wait=30,
        webhook_url="http://127.0.0.1:18081/telegram/webhook",
        log_file=".run/logs/omni-agent-webhook.log",
        chat_id=130,
        chat_b=130,
        user_a=1,
        user_b=1,
        username="tester",
        thread_a=None,
        thread_b=None,
        secret_token=None,
        prompt="/session json",
        forbid_log_regex=["tools/call: Mcp error"],
        allow_send_failure=False,
        session_partition="chat_user",
    )

    with pytest.raises(ValueError, match="same session_key"):
        config_module.build_config(
            args,
            group_profile_int_fn=lambda _key: None,
            session_ids_from_runtime_log_fn=lambda _path: (None, None, None),
            username_from_settings_fn=lambda: "tester",
            username_from_runtime_log_fn=lambda _path: "tester",
            telegram_webhook_secret_token_fn=lambda: "secret",
            expected_session_keys_fn=lambda _chat, _user, _thread, _mode: ("130:1",),
            resolve_runtime_partition_mode_fn=lambda _log_file, *, override: override,
            probe_config_cls=models_module.ProbeConfig,
        )
