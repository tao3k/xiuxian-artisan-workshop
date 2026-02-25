#!/usr/bin/env python3
"""Process and network runtime helpers for memory CI gate."""

from __future__ import annotations

import subprocess
import time
import urllib.error
import urllib.request
from typing import Any


def run_command(
    cmd: list[str],
    *,
    title: str,
    cwd: Any,
    env: dict[str, str] | None = None,
    gate_step_error_cls: Any,
) -> None:
    """Run a command and re-map non-zero exit into GateStepError."""
    print()
    print(f">>> {title}", flush=True)
    print("+ " + " ".join(cmd), flush=True)
    try:
        subprocess.run(cmd, check=True, cwd=str(cwd), env=env)
    except subprocess.CalledProcessError as exc:
        raise gate_step_error_cls(title=title, cmd=cmd, returncode=int(exc.returncode)) from exc


def wait_for_mock_health(host: str, port: int, timeout_secs: int = 20) -> None:
    """Wait until mock Telegram API /health endpoint is ready."""
    url = f"http://{host}:{port}/health"
    deadline = time.monotonic() + timeout_secs
    while time.monotonic() < deadline:
        try:
            with urllib.request.urlopen(url, timeout=2) as response:
                if response.status == 200:
                    return
        except (urllib.error.URLError, urllib.error.HTTPError):
            pass
        time.sleep(0.5)
    raise RuntimeError(f"mock Telegram API health endpoint not ready: {url}")


def terminate_process(process: subprocess.Popen[str] | None, *, name: str) -> None:
    """Terminate a background process gracefully with fallback kill."""
    if process is None or process.poll() is not None:
        return
    process.terminate()
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)
    print(f"{name} stopped (exit={process.returncode})", flush=True)


def start_background_process(
    cmd: list[str],
    *,
    cwd: Any,
    env: dict[str, str],
    log_file: Any,
    title: str,
    ensure_parent_dirs_fn: Any,
) -> tuple[subprocess.Popen[str], object]:
    """Start a background process redirecting stdout/stderr to a log file."""
    ensure_parent_dirs_fn(log_file)
    handle = log_file.open("w", encoding="utf-8")
    print()
    print(f">>> {title}", flush=True)
    print("+ " + " ".join(cmd), flush=True)
    process = subprocess.Popen(
        cmd,
        cwd=str(cwd),
        env=env,
        stdout=handle,
        stderr=subprocess.STDOUT,
        text=True,
        preexec_fn=None,
    )
    return process, handle
