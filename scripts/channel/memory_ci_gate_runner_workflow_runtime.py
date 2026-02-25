#!/usr/bin/env python3
"""Runtime bootstrap helpers for memory CI gate workflow."""

from __future__ import annotations

import sys
from typing import Any


def build_agent_command(cfg: Any) -> list[str]:
    """Build omni-agent runtime command for CI gate."""
    if cfg.agent_bin is not None:
        return [
            str(cfg.agent_bin),
            "channel",
            "--provider",
            "telegram",
            "--mode",
            "webhook",
            "--webhook-bind",
            f"127.0.0.1:{cfg.webhook_port}",
            "--webhook-secret-token",
            cfg.webhook_secret,
            "--verbose",
        ]
    return [
        "cargo",
        "run",
        "-p",
        "omni-agent",
        "--",
        "channel",
        "--provider",
        "telegram",
        "--mode",
        "webhook",
        "--webhook-bind",
        f"127.0.0.1:{cfg.webhook_port}",
        "--webhook-secret-token",
        cfg.webhook_secret,
        "--verbose",
    ]


def start_runtime_stack(
    cfg: Any,
    *,
    env: dict[str, str],
    script_paths: dict[str, Any],
    valkey_reachable_fn: Any,
    run_command_fn: Any,
    start_background_process_fn: Any,
    wait_for_mock_health_fn: Any,
    wait_for_log_regex_fn: Any,
) -> dict[str, Any]:
    """Start valkey, mock telegram API, and omni-agent runtime."""
    valkey_start = script_paths["valkey_start"]
    valkey_preexisting = valkey_reachable_fn(cfg.valkey_url)

    print(
        "CI gate Valkey isolation: "
        f"url={cfg.valkey_url} prefix={cfg.valkey_prefix} preexisting={valkey_preexisting}",
        flush=True,
    )
    run_command_fn(
        ["bash", str(valkey_start), str(cfg.valkey_port)],
        title="Start Valkey",
        cwd=cfg.project_root,
        env=env,
    )

    mock_process, mock_handle = start_background_process_fn(
        [
            sys.executable,
            str(script_paths["mock_server"]),
            "--host",
            "127.0.0.1",
            "--port",
            str(cfg.telegram_api_port),
        ],
        cwd=cfg.project_root,
        env=env,
        log_file=cfg.mock_log_file,
        title="Start mock Telegram API",
    )
    wait_for_mock_health_fn("127.0.0.1", cfg.telegram_api_port)

    agent_cmd = build_agent_command(cfg)
    agent_process, agent_handle = start_background_process_fn(
        agent_cmd,
        cwd=cfg.project_root,
        env=env,
        log_file=cfg.runtime_log_file,
        title="Start omni-agent webhook runtime (CI gate)",
    )
    wait_for_log_regex_fn(
        cfg.runtime_log_file,
        r"Telegram webhook listening on",
        timeout_secs=cfg.runtime_startup_timeout_secs,
        process=agent_process,
    )

    return {
        "valkey_preexisting": valkey_preexisting,
        "valkey_stop": script_paths["valkey_stop"],
        "agent_process": agent_process,
        "agent_handle": agent_handle,
        "mock_process": mock_process,
        "mock_handle": mock_handle,
    }
