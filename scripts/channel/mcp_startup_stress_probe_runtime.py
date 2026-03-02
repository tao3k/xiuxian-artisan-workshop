#!/usr/bin/env python3
"""Runtime execution loop for one MCP startup stress probe."""

from __future__ import annotations

import select
import subprocess
import time
from typing import Any


def execute_probe(
    *,
    cmd: list[str],
    cwd: str,
    env: dict[str, str],
    startup_timeout_secs: float,
    classify_reason_fn: Any,
) -> dict[str, Any]:
    """Run one gateway probe process and collect handshake telemetry."""
    started = time.monotonic()
    process = subprocess.Popen(
        cmd,
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )

    lines: list[str] = []
    ready_seen = False
    handshake_timeout_seen = False
    connect_failed_seen = False
    timed_out = False
    return_code: int | None = None
    mcp_connect_succeeded = 0
    mcp_connect_failed = 0
    deadline = time.monotonic() + startup_timeout_secs

    try:
        assert process.stdout is not None
        while True:
            if time.monotonic() > deadline:
                timed_out = True
                break

            if process.poll() is not None:
                return_code = process.returncode
                break

            ready, _, _ = select.select([process.stdout], [], [], 0.2)
            if not ready:
                continue
            raw_line = process.stdout.readline()
            if raw_line == "":
                if process.poll() is not None:
                    return_code = process.returncode
                    break
                continue

            line = raw_line.rstrip("\n")
            lines.append(line)

            if 'event="mcp.pool.connect.succeeded"' in line:
                mcp_connect_succeeded += 1
            if 'event="mcp.pool.connect.failed"' in line:
                mcp_connect_failed += 1
            if "MCP handshake timeout" in line:
                handshake_timeout_seen = True
            if "MCP connect failed after" in line:
                connect_failed_seen = True
            if "gateway listening on" in line:
                ready_seen = True
                break
    finally:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
        if return_code is None:
            return_code = process.returncode
        tail_extra = ""
        if process.stdout is not None:
            try:
                tail_extra = process.stdout.read()
            except Exception:
                tail_extra = ""
        if tail_extra:
            lines.extend(tail_extra.splitlines())

    duration_ms = int((time.monotonic() - started) * 1000)
    reason = classify_reason_fn(
        ready_seen=ready_seen,
        handshake_timeout_seen=handshake_timeout_seen,
        connect_failed_seen=connect_failed_seen,
        process_exited=return_code is not None,
        timed_out=timed_out,
    )
    return {
        "success": reason == "ok",
        "reason": reason,
        "startup_duration_ms": duration_ms,
        "return_code": return_code,
        "mcp_connect_succeeded": mcp_connect_succeeded,
        "mcp_connect_failed": mcp_connect_failed,
        "handshake_timeout_seen": handshake_timeout_seen,
        "connect_failed_seen": connect_failed_seen,
        "ready_seen": ready_seen,
        "tail": "\n".join(lines[-40:]),
    }
