"""Persistent daemon for `omni skill run --json --reuse-process`."""

from __future__ import annotations

import argparse
import os
import socket
import time
from contextlib import suppress
from pathlib import Path
from typing import Any

from omni.foundation.config.dirs import PRJ_RUNTIME
from omni.foundation.utils import json_codec as json

from .runner_json import (
    get_daemon_request_timeout_seconds,
    get_last_run_timing,
    run_skills_json_local,
)

_DEFAULT_IDLE_TIMEOUT_SECONDS = 600.0
_DEFAULT_SOCKET_FILE = "skill-runner-json.sock"
_ACTION_RUN = "run"
_ACTION_PING = "ping"
_ACTION_SHUTDOWN = "shutdown"


def _default_socket_path() -> Path:
    return PRJ_RUNTIME.ensure_dir("sockets") / _DEFAULT_SOCKET_FILE


def _socket_path_from_args(args: argparse.Namespace) -> Path:
    raw = str(args.socket or os.environ.get("OMNI_SKILL_RUNNER_SOCKET") or "").strip()
    if raw:
        return Path(raw).expanduser().resolve()
    return _default_socket_path()


def _idle_timeout_seconds() -> float:
    raw = str(os.environ.get("OMNI_SKILL_RUNNER_IDLE_TIMEOUT", "")).strip()
    if not raw:
        return _DEFAULT_IDLE_TIMEOUT_SECONDS
    try:
        return max(5.0, float(raw))
    except ValueError:
        return _DEFAULT_IDLE_TIMEOUT_SECONDS


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="omni skill json runner daemon")
    parser.add_argument("--socket", default="", help="Unix socket path")
    return parser.parse_args()


def _build_error_payload(message: str) -> dict[str, Any]:
    return {
        "exit_code": 1,
        "payload": json.dumps(
            {"success": False, "status": "error", "error": message},
            indent=2,
            ensure_ascii=False,
        ),
    }


def _build_ok_payload(payload: dict[str, Any]) -> dict[str, Any]:
    return {
        "exit_code": 0,
        "payload": json.dumps(payload, indent=2, ensure_ascii=False),
    }


def _handle_request_line(line: str) -> tuple[dict[str, Any], bool]:
    if not line.strip():
        return _build_error_payload("empty request"), False
    try:
        request = json.loads(line)
    except Exception as exc:
        return _build_error_payload(f"invalid request json: {exc}"), False

    action = str(request.get("action") or _ACTION_RUN).strip().lower()
    if action == _ACTION_PING:
        return _build_ok_payload(
            {
                "success": True,
                "status": "ok",
                "daemon": "skill-runner-json",
                "pid": os.getpid(),
            }
        ), False
    if action == _ACTION_SHUTDOWN:
        return _build_ok_payload(
            {
                "success": True,
                "status": "stopping",
                "daemon": "skill-runner-json",
                "pid": os.getpid(),
            }
        ), True
    if action != _ACTION_RUN:
        return _build_error_payload(f"unsupported action: {action}"), False

    commands = request.get("commands")
    if not isinstance(commands, list) or not all(isinstance(item, str) for item in commands):
        return _build_error_payload("commands must be a list[str]"), False
    quiet = bool(request.get("quiet", True))

    try:
        started_at = time.perf_counter()
        exit_code, payload = run_skills_json_local(
            commands,
            quiet=quiet,
            close_clients=False,
        )
        elapsed_ms = (time.perf_counter() - started_at) * 1000.0
    except Exception as exc:
        return _build_error_payload(f"daemon execution error: {exc}"), False

    timing_payload = get_last_run_timing()
    if not timing_payload:
        timing_payload = {
            "mode": "daemon_local",
            "total_ms": round(elapsed_ms, 3),
            "bootstrap_ms": round(elapsed_ms, 3),
            "daemon_connect_ms": 0.0,
            "tool_exec_ms": 0.0,
            "close_clients_ms": 0.0,
        }

    return (
        {
            "exit_code": int(exit_code),
            "payload": str(payload),
            "timing": timing_payload,
        },
        False,
    )


def serve(socket_path: Path) -> int:
    socket_path.parent.mkdir(parents=True, exist_ok=True)
    with suppress(FileNotFoundError):
        socket_path.unlink()

    idle_timeout = _idle_timeout_seconds()
    last_activity = time.monotonic()

    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server:
        server.bind(str(socket_path))
        server.listen(64)
        server.settimeout(1.0)

        while True:
            if time.monotonic() - last_activity >= idle_timeout:
                break
            try:
                conn, _ = server.accept()
            except TimeoutError:
                continue
            except OSError:
                continue

            with conn:
                last_activity = time.monotonic()
                conn.settimeout(get_daemon_request_timeout_seconds())
                try:
                    with conn.makefile("r", encoding="utf-8") as reader:
                        line = reader.readline()
                    response, should_stop = _handle_request_line(line)
                except Exception as exc:
                    response = _build_error_payload(f"daemon request failure: {exc}")
                    should_stop = False

                encoded = json.dumps(response, ensure_ascii=False, separators=(",", ":"))
                with suppress(OSError):
                    conn.sendall(encoded.encode("utf-8"))
                    conn.sendall(b"\n")
                if should_stop:
                    break

    with suppress(FileNotFoundError):
        socket_path.unlink()
    return 0


def main() -> int:
    args = _parse_args()
    socket_path = _socket_path_from_args(args)
    return serve(socket_path)


if __name__ == "__main__":
    raise SystemExit(main())
