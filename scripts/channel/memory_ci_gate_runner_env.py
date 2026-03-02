#!/usr/bin/env python3
"""Environment and script path helpers for memory CI gate runner."""

from __future__ import annotations

import os
from typing import Any

from resolve_mcp_endpoint import resolve_mcp_endpoint


def resolve_script_paths(script_dir: Any) -> dict[str, Any]:
    """Resolve runtime scripts used by the CI gate runner."""
    return {
        "valkey_start": script_dir / "valkey-start.sh",
        "valkey_stop": script_dir / "valkey-stop.sh",
        "mock_server": script_dir / "mock_telegram_api.py",
        "memory_suite": script_dir / "test_omni_agent_memory_suite.py",
        "session_matrix": script_dir / "test_omni_agent_session_matrix.py",
        "memory_benchmark": script_dir / "test_omni_agent_memory_benchmark.py",
    }


def build_runtime_env(
    cfg: Any,
    *,
    default_run_suffix_fn: Any,
    write_ci_channel_acl_settings_fn: Any,
) -> tuple[dict[str, str], Any]:
    """Build isolated runtime environment and write run-scoped settings."""
    local_host = str(resolve_mcp_endpoint()["host"])
    env = os.environ.copy()
    env["XIUXIAN_WENDAO_VALKEY_URL"] = cfg.valkey_url
    env["OMNI_AGENT_SESSION_VALKEY_PREFIX"] = cfg.valkey_prefix
    env["OMNI_AGENT_MEMORY_VALKEY_KEY_PREFIX"] = f"{cfg.valkey_prefix}:memory"
    env["TELEGRAM_BOT_TOKEN"] = env.get("TELEGRAM_BOT_TOKEN", "ci-telegram-token")
    env["TELEGRAM_WEBHOOK_SECRET"] = cfg.webhook_secret
    env["OMNI_AGENT_TELEGRAM_API_BASE_URL"] = f"http://{local_host}:{cfg.telegram_api_port}"
    env["OMNI_WEBHOOK_URL"] = f"http://{local_host}:{cfg.webhook_port}/telegram/webhook"
    env["OMNI_CHANNEL_LOG_FILE"] = str(cfg.runtime_log_file)
    env["OMNI_TEST_CHAT_ID"] = str(cfg.chat_id)
    env["OMNI_TEST_CHAT_B"] = str(cfg.chat_b)
    env["OMNI_TEST_CHAT_C"] = str(cfg.chat_c)
    env["OMNI_TEST_USER_ID"] = str(cfg.user_id)
    env["OMNI_TEST_USER_B"] = str(cfg.user_b)
    env["OMNI_TEST_USER_C"] = str(cfg.user_c)
    env["OMNI_TEST_USERNAME"] = cfg.username
    env["RUST_LOG"] = env.get("RUST_LOG", "omni_agent=debug")
    env["RUST_BACKTRACE"] = env.get("RUST_BACKTRACE", "1")
    config_home = cfg.project_root / ".run" / "config" / "memory-ci-gate" / default_run_suffix_fn()
    settings_path = write_ci_channel_acl_settings_fn(cfg, config_home=config_home)
    env["PRJ_CONFIG_HOME"] = str(config_home)
    return env, settings_path
