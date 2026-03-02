"""Minimal JSON-only skill runner for fastest machine output path."""

from __future__ import annotations

import logging
import os
import socket
import subprocess
import sys
import time
from contextlib import suppress
from pathlib import Path
from typing import Any

from omni.foundation.config.dirs import PRJ_RUNTIME
from omni.foundation.utils import json_codec as json
from omni.foundation.utils.asyncio import run_async_blocking

from .json_output import normalize_result_for_json_output

_DAEMON_ENV_KEY = "OMNI_SKILL_RUNNER_DAEMON"
_DAEMON_SERVER_ENV_KEY = "OMNI_SKILL_RUNNER_DAEMON_SERVER"
_DAEMON_SOCKET_ENV_KEY = "OMNI_SKILL_RUNNER_SOCKET"
_DAEMON_REQUEST_TIMEOUT_ENV_KEY = "OMNI_SKILL_RUNNER_REQUEST_TIMEOUT"
_DAEMON_BOOT_TIMEOUT_SECONDS = 3.0
_DEFAULT_DAEMON_REQUEST_TIMEOUT_SECONDS = 1800.0
_DAEMON_OPERATION_RUN = "run"
_DAEMON_OPERATION_PING = "ping"
_DAEMON_OPERATION_SHUTDOWN = "shutdown"
_TIMING_ENV_KEY = "OMNI_SKILL_RUN_TIMING"
_TIMING_PREFIX = "__OMNI_SKILL_TIMING__ "

_LAST_RUN_TIMING: dict[str, Any] = {}
_LAST_DAEMON_TIMING: dict[str, Any] = {}


def _setup_quiet_logging() -> None:
    """Suppress verbose logs for strict JSON stdout mode."""
    logging.getLogger("omni").setLevel(logging.WARNING)
    logging.getLogger("omni.core").setLevel(logging.WARNING)
    logging.getLogger("omni.foundation").setLevel(logging.WARNING)
    logging.getLogger("litellm").setLevel(logging.WARNING)
    logging.getLogger("LiteLLM").setLevel(logging.WARNING)


def _verbose_enabled() -> bool:
    """Return True when CLI verbose mode is active."""
    try:
        from omni.agent.cli.app import _is_verbose

        return bool(_is_verbose())
    except Exception:
        pass
    return os.environ.get("OMNI_CLI_VERBOSE", "").strip().lower() in {"1", "true", "yes", "on"}


def _timing_enabled() -> bool:
    raw = os.environ.get(_TIMING_ENV_KEY, "").strip().lower()
    return raw in {"1", "true", "yes", "on"}


def get_daemon_request_timeout_seconds() -> float:
    """Resolve daemon socket request timeout in seconds."""
    raw = os.environ.get(_DAEMON_REQUEST_TIMEOUT_ENV_KEY, "").strip()
    if raw:
        try:
            return max(5.0, float(raw))
        except ValueError:
            pass

    try:
        from omni.foundation.config.settings import get_setting

        configured = get_setting("mcp.timeout", _DEFAULT_DAEMON_REQUEST_TIMEOUT_SECONDS)
        timeout = float(configured)
        if timeout > 0:
            return max(5.0, timeout)
    except Exception:
        pass

    return _DEFAULT_DAEMON_REQUEST_TIMEOUT_SECONDS


def _normalize_timing_payload(payload: dict[str, Any]) -> dict[str, Any]:
    normalized: dict[str, Any] = {}
    for key, value in payload.items():
        if not isinstance(key, str):
            continue
        if isinstance(value, bool):
            normalized[key] = value
            continue
        if isinstance(value, int | float):
            normalized[key] = round(float(value), 3)
            continue
        if isinstance(value, str):
            normalized[key] = value
    return normalized


def _set_last_run_timing(payload: dict[str, Any]) -> None:
    global _LAST_RUN_TIMING
    _LAST_RUN_TIMING = _normalize_timing_payload(payload)


def get_last_run_timing() -> dict[str, Any]:
    """Return the timing payload for the most recent `run_skills_json` invocation."""
    return dict(_LAST_RUN_TIMING)


def _set_last_daemon_timing(payload: dict[str, Any]) -> None:
    global _LAST_DAEMON_TIMING
    _LAST_DAEMON_TIMING = _normalize_timing_payload(payload)


def _record_local_run_timing(
    *,
    started_at: float,
    tool_exec_ms: float = 0.0,
    close_clients_ms: float = 0.0,
) -> None:
    total_ms = (time.perf_counter() - started_at) * 1000.0
    bootstrap_ms = max(0.0, total_ms - tool_exec_ms - close_clients_ms)
    _set_last_run_timing(
        {
            "mode": "local",
            "total_ms": total_ms,
            "bootstrap_ms": bootstrap_ms,
            "daemon_connect_ms": 0.0,
            "tool_exec_ms": tool_exec_ms,
            "close_clients_ms": close_clients_ms,
        }
    )


def _emit_last_run_timing_if_enabled() -> None:
    if not _timing_enabled():
        return
    payload = get_last_run_timing()
    if not payload:
        return
    serialized = json.dumps(payload, ensure_ascii=False, separators=(",", ":"))
    sys.stderr.write(f"{_TIMING_PREFIX}{serialized}\n")
    sys.stderr.flush()


def _write_json(payload: dict[str, Any]) -> None:
    text = json.dumps(payload, indent=2, ensure_ascii=False)
    sys.stdout.write(text)
    if not text.endswith("\n"):
        sys.stdout.write("\n")
    sys.stdout.flush()


def _close_embedding_client_if_loaded() -> None:
    """Close embedding HTTP session only when the client module is already loaded."""
    if "omni.foundation.embedding_client" not in sys.modules:
        return
    with suppress(Exception):
        from omni.foundation.embedding_client import close_embedding_client

        run_async_blocking(close_embedding_client())


def _close_mcp_embed_http_client_if_loaded() -> None:
    """Close shared MCP HTTP client only when mcp_embed module is already loaded."""
    if "omni.agent.cli.mcp_embed" not in sys.modules:
        return
    with suppress(Exception):
        from omni.agent.cli.mcp_embed import close_shared_http_client

        run_async_blocking(close_shared_http_client())


def _serialize_json_payload(payload: dict[str, Any]) -> str:
    return json.dumps(payload, indent=2, ensure_ascii=False)


def _default_runner_socket_path() -> Path:
    configured = os.environ.get(_DAEMON_SOCKET_ENV_KEY, "").strip()
    if configured:
        return Path(configured).expanduser().resolve()
    return PRJ_RUNTIME.ensure_dir("sockets") / "skill-runner-json.sock"


def _in_runner_daemon_process() -> bool:
    raw = os.environ.get(_DAEMON_SERVER_ENV_KEY, "").strip().lower()
    return raw in {"1", "true", "yes", "on"}


def _reuse_process_enabled(reuse_process: bool) -> bool:
    if reuse_process:
        return True
    raw = os.environ.get(_DAEMON_ENV_KEY, "").strip().lower()
    if raw in {"1", "true", "yes", "on"}:
        return True
    if raw in {"0", "false", "no", "off"}:
        return False
    return False


def _spawn_runner_daemon(socket_path: Path) -> None:
    socket_path.parent.mkdir(parents=True, exist_ok=True)
    with suppress(FileNotFoundError):
        socket_path.unlink()
    env = os.environ.copy()
    env[_DAEMON_SERVER_ENV_KEY] = "1"
    env[_DAEMON_SOCKET_ENV_KEY] = str(socket_path)
    subprocess.Popen(
        [
            sys.executable,
            "-m",
            "omni.agent.cli.runner_json_daemon",
            "--socket",
            str(socket_path),
        ],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        close_fds=True,
        start_new_session=True,
        env=env,
    )


def _request_runner_daemon(
    *,
    request_payload: dict[str, Any],
    socket_path: Path,
) -> tuple[int, str]:
    _set_last_daemon_timing({})
    serialized_request = json.dumps(request_payload, ensure_ascii=False, separators=(",", ":"))
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
        client.settimeout(get_daemon_request_timeout_seconds())
        client.connect(str(socket_path))
        client.sendall(serialized_request.encode("utf-8"))
        client.sendall(b"\n")
        with client.makefile("r", encoding="utf-8") as reader:
            line = reader.readline()
    if not line:
        raise RuntimeError("empty response from skill runner daemon")
    response = json.loads(line)
    if isinstance(response.get("timing"), dict):
        _set_last_daemon_timing(response["timing"])
    exit_code = int(response.get("exit_code", 1))
    payload = str(response.get("payload", ""))
    return exit_code, payload


def _request_runner_daemon_run(
    *,
    commands: list[str],
    quiet: bool,
    socket_path: Path,
) -> tuple[int, str]:
    request_payload = {
        "action": _DAEMON_OPERATION_RUN,
        "commands": list(commands),
        "quiet": bool(quiet),
    }
    return _request_runner_daemon(request_payload=request_payload, socket_path=socket_path)


def _request_runner_daemon_ping(socket_path: Path) -> tuple[int, str]:
    return _request_runner_daemon(
        request_payload={"action": _DAEMON_OPERATION_PING},
        socket_path=socket_path,
    )


def _request_runner_daemon_shutdown(socket_path: Path) -> tuple[int, str]:
    return _request_runner_daemon(
        request_payload={"action": _DAEMON_OPERATION_SHUTDOWN},
        socket_path=socket_path,
    )


def _parse_payload_object(payload: str) -> dict[str, Any]:
    try:
        parsed = json.loads(payload)
    except Exception:
        return {}
    return parsed if isinstance(parsed, dict) else {}


def _wait_until_runner_daemon_ready(socket_path: Path, timeout_seconds: float) -> None:
    deadline = time.monotonic() + max(0.1, timeout_seconds)
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            exit_code, payload = _request_runner_daemon_ping(socket_path)
            parsed = _parse_payload_object(payload)
            success = bool(parsed.get("success", True))
            if exit_code == 0 and success:
                return
            last_error = RuntimeError(
                f"runner daemon ping failed (exit_code={exit_code}, payload={payload})"
            )
        except Exception as exc:
            last_error = exc
        time.sleep(0.05)
    if last_error is not None:
        raise last_error
    raise TimeoutError("skill runner daemon did not become ready")


def _run_skills_json_via_daemon(commands: list[str], *, quiet: bool) -> tuple[int, str]:
    socket_path = _default_runner_socket_path()
    started_at = time.perf_counter()
    daemon_connect_ms = 0.0
    daemon_bootstrap_ms = 0.0
    daemon_spawned = False

    def _request_run() -> tuple[int, str]:
        nonlocal daemon_connect_ms
        connect_started_at = time.perf_counter()
        try:
            return _request_runner_daemon_run(
                commands=commands, quiet=quiet, socket_path=socket_path
            )
        finally:
            daemon_connect_ms += (time.perf_counter() - connect_started_at) * 1000.0

    try:
        exit_code, payload = _request_run()
    except Exception:
        daemon_spawned = True
        daemon_bootstrap_started_at = time.perf_counter()
        _spawn_runner_daemon(socket_path)
        _wait_until_runner_daemon_ready(socket_path, timeout_seconds=_DAEMON_BOOT_TIMEOUT_SECONDS)
        daemon_bootstrap_ms = (time.perf_counter() - daemon_bootstrap_started_at) * 1000.0
        exit_code, payload = _request_run()

    daemon_total_ms = 0.0
    daemon_tool_exec_ms = 0.0
    if isinstance(_LAST_DAEMON_TIMING.get("total_ms"), int | float):
        daemon_total_ms = float(_LAST_DAEMON_TIMING["total_ms"])
    if isinstance(_LAST_DAEMON_TIMING.get("tool_exec_ms"), int | float):
        daemon_tool_exec_ms = float(_LAST_DAEMON_TIMING["tool_exec_ms"])
    total_ms = (time.perf_counter() - started_at) * 1000.0
    bootstrap_ms = max(0.0, total_ms - daemon_connect_ms - daemon_tool_exec_ms)
    _set_last_run_timing(
        {
            "mode": "daemon",
            "total_ms": total_ms,
            "bootstrap_ms": bootstrap_ms,
            "daemon_connect_ms": daemon_connect_ms,
            "daemon_bootstrap_ms": daemon_bootstrap_ms,
            "daemon_total_ms": daemon_total_ms,
            "tool_exec_ms": daemon_tool_exec_ms,
            "daemon_spawned": daemon_spawned,
            "close_clients_ms": 0.0,
        }
    )
    return exit_code, payload


def get_runner_daemon_status() -> dict[str, Any]:
    """Return runner daemon status for CLI management commands."""
    socket_path = _default_runner_socket_path()
    status: dict[str, Any] = {"running": False, "socket": str(socket_path)}
    try:
        exit_code, payload = _request_runner_daemon_ping(socket_path)
    except Exception as exc:
        status["error"] = str(exc)
        return status

    parsed_payload = _parse_payload_object(payload)
    status["running"] = exit_code == 0
    if parsed_payload:
        status["daemon"] = parsed_payload
    else:
        status["daemon_raw"] = payload
    if isinstance(parsed_payload.get("pid"), int):
        status["pid"] = parsed_payload["pid"]
    return status


def start_runner_daemon(*, timeout_seconds: float = _DAEMON_BOOT_TIMEOUT_SECONDS) -> dict[str, Any]:
    """Start runner daemon if needed and return status."""
    status = get_runner_daemon_status()
    if status.get("running") is True:
        status["started"] = False
        return status

    socket_path = _default_runner_socket_path()
    _spawn_runner_daemon(socket_path)
    _wait_until_runner_daemon_ready(socket_path, timeout_seconds=timeout_seconds)
    started = get_runner_daemon_status()
    started["started"] = bool(started.get("running") is True)
    if started["started"] is False and "error" not in started:
        started["error"] = "runner daemon started but readiness check failed"
    return started


def stop_runner_daemon() -> dict[str, Any]:
    """Request runner daemon shutdown and return stop status."""
    socket_path = _default_runner_socket_path()
    status: dict[str, Any] = {"socket": str(socket_path), "stopped": False}
    try:
        exit_code, payload = _request_runner_daemon_shutdown(socket_path)
    except Exception as exc:
        status["error"] = str(exc)
        return status

    parsed_payload = _parse_payload_object(payload)
    status["stopped"] = exit_code == 0
    if parsed_payload:
        status["daemon"] = parsed_payload
    else:
        status["daemon_raw"] = payload
    return status


def _run_skills_json_payload(
    commands: list[str],
    *,
    quiet: bool = True,
    close_clients: bool = True,
) -> tuple[int, str]:
    """Run `skill.command` and return `(exit_code, payload_text)`."""
    started_at = time.perf_counter()
    tool_exec_ms = 0.0
    close_clients_ms = 0.0
    if quiet and not _verbose_enabled():
        _setup_quiet_logging()

    if not commands or commands[0] in ("help", "?"):
        payload = _serialize_json_payload(
            {
                "success": False,
                "status": "error",
                "error": "help is not supported in --json fast path; use `omni skill --help`",
            }
        )
        _record_local_run_timing(started_at=started_at)
        return 1, payload

    cmd = commands[0]
    if "." not in cmd:
        payload = _serialize_json_payload(
            {
                "success": False,
                "status": "error",
                "error": f"Invalid format: {cmd}. Use skill.command",
            }
        )
        _record_local_run_timing(started_at=started_at)
        return 1, payload

    cmd_args: dict[str, Any] = {}
    if len(commands) > 1:
        rest = commands[1].strip()
        if rest.startswith("{"):
            try:
                cmd_args = json.loads(commands[1])
            except json.JSONDecodeError as exc:
                payload = _serialize_json_payload(
                    {
                        "success": False,
                        "status": "error",
                        "error": f"Invalid JSON args: {exc}",
                    }
                )
                _record_local_run_timing(started_at=started_at)
                return 1, payload
        elif rest:
            cmd_args = {"file_path": rest}

    result: Any | None = None
    exit_code = 0
    payload_text: str | None = None
    try:
        from omni.core.skills.runner import run_tool_with_monitor
        from omni.foundation.api.tool_context import run_with_execution_timeout

        exec_started_at = time.perf_counter()
        result, _monitor = run_async_blocking(
            run_with_execution_timeout(
                run_tool_with_monitor(
                    cmd,
                    cmd_args,
                    output_json=True,
                    auto_report=False,
                )
            )
        )
        tool_exec_ms = (time.perf_counter() - exec_started_at) * 1000.0
    except TimeoutError as exc:
        exit_code = 124
        payload_text = _serialize_json_payload(
            {
                "success": False,
                "status": "timeout",
                "error": str(exc),
            }
        )
    except ValueError as exc:
        exit_code = 1
        payload_text = _serialize_json_payload(
            {
                "success": False,
                "status": "error",
                "error": str(exc),
            }
        )
    except Exception as exc:
        exit_code = 1
        payload_text = _serialize_json_payload(
            {
                "success": False,
                "status": "error",
                "error": f"Execution error: {exc}",
            }
        )
    finally:
        close_started_at = time.perf_counter()
        if close_clients:
            # Avoid "Unclosed client session" warnings in short-lived CLI subprocesses.
            # Keep this cheap by skipping import when embedding client was never used.
            _close_embedding_client_if_loaded()
            _close_mcp_embed_http_client_if_loaded()
        close_clients_ms = (time.perf_counter() - close_started_at) * 1000.0

    _record_local_run_timing(
        started_at=started_at,
        tool_exec_ms=tool_exec_ms,
        close_clients_ms=close_clients_ms,
    )
    if payload_text is not None:
        return exit_code, payload_text

    if result is None:
        payload_text = _serialize_json_payload(
            {
                "success": False,
                "status": "error",
                "error": "Execution error: empty result payload",
            }
        )
        return 1, payload_text

    payload = normalize_result_for_json_output(result)
    return 0, payload


def run_skills_json_local(
    commands: list[str],
    *,
    quiet: bool = True,
    close_clients: bool = True,
) -> tuple[int, str]:
    """Execute command locally and return `(exit_code, payload_text)`."""
    return _run_skills_json_payload(commands, quiet=quiet, close_clients=close_clients)


def run_skills_json_payload(
    commands: list[str],
    *,
    quiet: bool = True,
    reuse_process: bool = False,
    close_clients: bool = True,
) -> tuple[int, str]:
    """Run command and return `(exit_code, payload)` without writing stdout."""
    _set_last_run_timing({})
    _set_last_daemon_timing({})
    if _reuse_process_enabled(reuse_process) and not _in_runner_daemon_process():
        return _run_skills_json_via_daemon(commands, quiet=quiet)
    return _run_skills_json_payload(commands, quiet=quiet, close_clients=close_clients)


def run_skills_json(
    commands: list[str],
    *,
    quiet: bool = True,
    reuse_process: bool = False,
) -> int:
    """Run `skill.command` and print strict machine JSON to stdout."""
    started_at = time.perf_counter()
    try:
        exit_code, payload = run_skills_json_payload(
            commands,
            quiet=quiet,
            reuse_process=reuse_process,
            close_clients=True,
        )
    except Exception as exc:
        exit_code = 1
        payload = _serialize_json_payload(
            {
                "success": False,
                "status": "error",
                "error": f"Runner daemon error: {exc}",
            }
        )
        total_ms = (time.perf_counter() - started_at) * 1000.0
        _set_last_run_timing(
            {
                "mode": "error",
                "total_ms": total_ms,
                "bootstrap_ms": total_ms,
                "daemon_connect_ms": 0.0,
                "tool_exec_ms": 0.0,
                "close_clients_ms": 0.0,
            }
        )

    if payload:
        sys.stdout.write(payload)
        if not payload.endswith("\n"):
            sys.stdout.write("\n")
        sys.stdout.flush()
    _emit_last_run_timing_if_enabled()
    return exit_code


__all__ = [
    "get_daemon_request_timeout_seconds",
    "get_last_run_timing",
    "get_runner_daemon_status",
    "run_skills_json",
    "run_skills_json_local",
    "run_skills_json_payload",
    "start_runner_daemon",
    "stop_runner_daemon",
]
