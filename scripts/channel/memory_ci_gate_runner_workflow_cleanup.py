#!/usr/bin/env python3
"""Cleanup helpers for memory CI gate workflow."""

from __future__ import annotations

import subprocess
from typing import Any


def cleanup_runtime_stack(
    cfg: Any,
    *,
    env: dict[str, str],
    terminate_process_fn: Any,
    agent_process: Any,
    mock_process: Any,
    agent_handle: Any,
    mock_handle: Any,
    valkey_preexisting: bool,
    valkey_stop: Any,
) -> None:
    """Terminate processes and stop valkey when this run started it."""
    terminate_process_fn(agent_process, name="omni-agent runtime")
    terminate_process_fn(mock_process, name="mock Telegram API")
    if agent_handle is not None:
        agent_handle.close()
    if mock_handle is not None:
        mock_handle.close()
    if not valkey_preexisting:
        subprocess.run(
            ["bash", str(valkey_stop), str(cfg.valkey_port)],
            cwd=str(cfg.project_root),
            env=env,
            check=False,
        )
    else:
        print(
            "Skip valkey-stop: existing Valkey instance was already running before CI gate.",
            flush=True,
        )
