#!/usr/bin/env python3
"""CyberXiuXian Bootcamp V2: Observability + Audit runner.

Modes:
- stream (default): real-time, heartbeat-driven observability view.
- direct: fast local nextest probes (no gateway/telegram dependency).
- gateway: full HTTP + Valkey verification.
- telegram: webhook realtime probe via scripts/channel probe API.
- discord: ingress realtime probe via scripts/channel probe API.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import select
import socket
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any
from urllib import error as urlerror
from urllib import parse as urlparse
from urllib import request as urlrequest

try:
    import tomllib
except ModuleNotFoundError as exc:  # pragma: no cover
    raise RuntimeError("Python 3.11+ is required for tomllib support") from exc


DEFAULT_TRIGGER_INTENT = (
    "Schedule 10 heavy coding tasks today, each 2-3 hours, no breaks, and keep all carryover."
)
DEFAULT_EXPECTED_Q_CEILING = 0.5
DEFAULT_MESSAGE_TIMEOUT_SECS = 45
DEFAULT_ZHENFA_TIMEOUT_SECS = 20
DEFAULT_VALKEY_WAIT_SECS = 8.0
DEFAULT_VALKEY_POLL_INTERVAL_SECS = 0.25
DEFAULT_DIRECT_TIMEOUT_SECS = 420
DEFAULT_STREAM_TEST = "bootcamp_runs_embedded_agenda_flow_with_mock_llm"
DEFAULT_HEARTBEAT_SECS = 8.0
DEFAULT_CHANNEL_MAX_WAIT_SECS = 45.0
DEFAULT_CHANNEL_MAX_IDLE_SECS = 12.0
DEFAULT_CHANNEL_LOG_FILE = ".run/logs/omni-agent-webhook.log"
DEFAULT_CHANNEL_SCENARIO = "single"

_SCRIPT_DIR = Path(__file__).resolve().parent
_CHANNEL_DIR = _SCRIPT_DIR / "channel"
if str(_CHANNEL_DIR) not in sys.path:
    sys.path.insert(0, str(_CHANNEL_DIR))

from realtime_probe_api import (  # noqa: E402
    DiscordRealtimeProbeConfig,
    TelegramRealtimeProbeConfig,
    default_telegram_secret,
    resolve_discord_probe_channel_id,
    resolve_discord_probe_secret,
    resolve_discord_probe_url,
    resolve_discord_probe_user_id,
    resolve_telegram_probe_chat_id,
    resolve_telegram_probe_url,
    resolve_telegram_probe_user_id,
    run_discord_realtime_probe,
    run_telegram_realtime_probe,
)

ADVERSARIAL_CONTEXT = {
    "user_message": (
        "I must finish these 3 critical tasks between 9 AM and 9 PM today: "
        "\n1. [DEEP REFACTOR] Fix insidious memory leak in `xiuxian-wendao`. (Est: 6h). "
        "\n2. [SECURITY] Audit `ZhenfaTransmuter` XML parser. (Est: 4h). "
        "\n3. [OPS] Purge legacy folders and update branding in flake.nix. (Est: 2h). "
        "\nNote: prioritize milimeter-level alignment and physical realism."
    ),
    "history": [],
    "wendao_search_results": (
        "<hit>User history shows high failure rate for > 4h deep work.</hit>"
        "<hit>Mandatory standard: milimeter-level alignment, audit trail, traceability.</hit>"
        "<hit>Engineering Guardrail: architectural consistency over raw speed.</hit>"
    ),
}

TRINITY_CHANNEL_STEPS: tuple[dict[str, str], ...] = (
    {
        "id": "student_ambition",
        "role": "student",
        "instruction": (
            "Act as Student_Ambition. Propose an over-ambitious plan for the next 12 hours "
            "with maximum throughput and no guardrails. Start your response with "
            "'**Student_Ambition**'."
        ),
    },
    {
        "id": "steward_logistics",
        "role": "steward",
        "instruction": (
            "Act as Steward_Logistics. Critique feasibility, carryover risk, time budget, and "
            "resource constraints from the previous proposal. Start your response with "
            "'**Steward_Logistics**'."
        ),
    },
    {
        "id": "professor_audit",
        "role": "professor",
        "instruction": (
            "Act as Professor_Audit. Output a Telegram MarkdownV2 report only (no XML tags). "
            "Use this structure exactly: '*Agenda Critique Report*', then '*Score:* <0.00-1.00>', "
            "then '*Critique:*' with bullet points, then '*Verdict:* <pass/fail + reason>', "
            "then a short refined plan. Start your response with '**Professor_Audit**'."
        ),
    },
)

TRINITY_ROLE_MARKERS: dict[str, str] = {
    "student_ambition": "Student_Ambition",
    "steward_logistics": "Steward_Logistics",
    "professor_audit": "Professor_Audit",
}

_PROFESSOR_SCORE_RE = re.compile(
    r"(?:\bscore\b|评分)\s*[\*\s:\-\uFF1A]*\s*(0(?:\.\d+)?|1(?:\.0+)?)",
    flags=re.IGNORECASE,
)


@dataclass(frozen=True)
class RuntimeConfig:
    project_root: Path
    config_path: Path
    gateway_url: str
    zhenfa_url: str | None
    valkey_url: str | None
    memory_prefix: str
    memory_table: str


class _Color:
    RESET = "\033[0m"
    BOLD = "\033[1m"
    DIM = "\033[2m"
    CYAN = "\033[36m"
    BLUE = "\033[34m"
    GREEN = "\033[32m"
    YELLOW = "\033[33m"
    RED = "\033[31m"
    MAGENTA = "\033[35m"


def _supports_color() -> bool:
    term = os.getenv("TERM", "")
    return sys.stdout.isatty() and term not in {"", "dumb"}


def _paint(text: str, color: str) -> str:
    if not _supports_color():
        return text
    return f"{color}{text}{_Color.RESET}"


def _render_logo(mode: str) -> None:
    logo = [
        "   ____      _               __  ___      _       ",
        "  / __ )____(_)___ _____ ___/  |/  /___ _(_)___ _ ",
        " / __  / __/ / __ `/ __ `__  /|_/ / __ `/ / __ `/ ",
        "/ /_/ / /_/ / /_/ / / / / / /  / / /_/ / / /_/ /  ",
        r"\____/\__/_/\__, /_/ /_/ /_/_/  /_/\__,_/_/\__,_/   ",
        "           /____/                                    ",
    ]
    print(_paint("=" * 78, _Color.CYAN))
    for line in logo:
        print(_paint(line, _Color.MAGENTA))
    subtitle = f"CyberXiuXian Bootcamp V2 | mode={mode}"
    print(_paint(subtitle, _Color.BOLD + _Color.CYAN))
    print(_paint("=" * 78, _Color.CYAN))


def _render_panel(title: str, lines: list[str], color: str = _Color.BLUE) -> None:
    border = _paint("-" * 78, color)
    print(border)
    print(_paint(f"[{title}]", _Color.BOLD + color))
    for line in lines:
        print(line)
    print(border)


def _status_tag(ok: bool) -> str:
    if ok:
        return _paint("PASS", _Color.GREEN + _Color.BOLD)
    return _paint("FAIL", _Color.RED + _Color.BOLD)


def _fmt_secs(value: float) -> str:
    return f"{value:.3f}s"


def _render_run_table(runs: list[dict[str, Any]]) -> None:
    if not runs:
        return
    name_width = max(len(str(run.get("name", ""))) for run in runs)
    header = f"{'status':<8} {'name':<{name_width}} duration"
    print(_paint(header, _Color.BOLD + _Color.CYAN))
    for run in runs:
        status = _status_tag(bool(run.get("ok", False)))
        name = str(run.get("name", ""))
        elapsed = _fmt_secs(float(run.get("elapsed_secs", 0.0)))
        print(f"{status:<8} {name:<{name_width}} {elapsed}")


def _preview_line(value: str | None, max_chars: int = 180) -> str:
    text = (value or "").strip()
    if not text:
        return "(missing)"
    if len(text) <= max_chars:
        return text
    return f"{text[:max_chars]}..."


def _render_channel_scenario_summary(checks: dict[str, Any]) -> None:
    scenario = checks.get("telegram_scenario")
    if not isinstance(scenario, dict):
        return
    steps = scenario.get("steps")
    if not isinstance(steps, list) or not steps:
        return

    lines: list[str] = []
    for step in steps:
        if not isinstance(step, dict):
            continue
        step_index = step.get("step_index", "?")
        step_id = step.get("step_id", "unknown")
        role = step.get("role", "unknown")
        ok = bool(step.get("ok", False))
        status = "PASS" if ok else "FAIL"
        bot_preview = _preview_line(step.get("bot_line"))
        lines.append(f"[{status}] step={step_index} id={step_id} role={role}")
        lines.append(f"  bot: {bot_preview}")

    if not lines:
        return
    _render_panel("Channel Scenario Steps", lines, _Color.MAGENTA)


class RedisRespClient:
    """Minimal RESP client for Valkey/Redis (no external dependency)."""

    def __init__(self, redis_url: str, timeout_secs: float = 2.5) -> None:
        parsed = urlparse.urlparse(redis_url)
        if parsed.scheme != "redis":
            raise ValueError(
                f"unsupported valkey url scheme `{parsed.scheme}`; expected `redis://`"
            )
        self._host = parsed.hostname or "127.0.0.1"
        self._port = parsed.port or 6379
        self._db = int(parsed.path.lstrip("/") or "0")
        self._username = urlparse.unquote(parsed.username) if parsed.username else None
        self._password = urlparse.unquote(parsed.password) if parsed.password else None
        self._timeout_secs = timeout_secs
        self._sock: socket.socket | None = None
        self._file = None

    def __enter__(self) -> RedisRespClient:
        self.connect()
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        self.close()

    def connect(self) -> None:
        if self._sock is not None:
            return
        self._sock = socket.create_connection((self._host, self._port), timeout=self._timeout_secs)
        self._sock.settimeout(self._timeout_secs)
        self._file = self._sock.makefile("rb")
        if self._password is not None:
            if self._username:
                self.command("AUTH", self._username, self._password)
            else:
                self.command("AUTH", self._password)
        if self._db:
            self.command("SELECT", str(self._db))

    def close(self) -> None:
        if self._file is not None:
            self._file.close()
            self._file = None
        if self._sock is not None:
            self._sock.close()
            self._sock = None

    def command(self, *args: str) -> Any:
        if self._sock is None or self._file is None:
            self.connect()
        assert self._sock is not None
        assert self._file is not None
        payload = self._encode_command(args)
        self._sock.sendall(payload)
        return self._read_reply()

    def scan_keys(self, pattern: str, count: int = 200) -> list[str]:
        cursor = "0"
        keys: list[str] = []
        while True:
            reply = self.command("SCAN", cursor, "MATCH", pattern, "COUNT", str(count))
            if not isinstance(reply, list) or len(reply) != 2:
                raise RuntimeError(f"unexpected SCAN reply shape: {reply!r}")
            cursor = str(reply[0])
            batch = reply[1]
            if isinstance(batch, list):
                keys.extend(str(item) for item in batch)
            if cursor == "0":
                break
        return keys

    def hget_float(self, key: str, field: str) -> float | None:
        raw = self.command("HGET", key, field)
        if raw is None:
            return None
        try:
            return float(raw)
        except ValueError as exc:
            raise RuntimeError(f"failed to parse HGET value `{raw}` as float") from exc

    def hlen(self, key: str) -> int:
        value = self.command("HLEN", key)
        if isinstance(value, int):
            return value
        return int(str(value))

    def _encode_command(self, args: tuple[str, ...]) -> bytes:
        out = [f"*{len(args)}\r\n".encode()]
        for arg in args:
            encoded = arg.encode("utf-8")
            out.append(f"${len(encoded)}\r\n".encode())
            out.append(encoded + b"\r\n")
        return b"".join(out)

    def _read_reply(self) -> Any:
        assert self._file is not None
        line = self._file.readline()
        if not line:
            raise RuntimeError("redis connection closed while waiting for reply")
        prefix, payload = line[:1], line[1:-2]

        if prefix == b"+":
            return payload.decode("utf-8")
        if prefix == b"-":
            raise RuntimeError(f"redis error: {payload.decode('utf-8')}")
        if prefix == b":":
            return int(payload.decode("utf-8"))
        if prefix == b"$":
            length = int(payload.decode("utf-8"))
            if length < 0:
                return None
            data = self._file.read(length)
            tail = self._file.read(2)
            if data is None or tail != b"\r\n":
                raise RuntimeError("malformed bulk reply from redis")
            return data.decode("utf-8")
        if prefix == b"*":
            size = int(payload.decode("utf-8"))
            if size < 0:
                return None
            return [self._read_reply() for _ in range(size)]
        raise RuntimeError(f"unsupported redis reply prefix: {prefix!r}")


def _project_root() -> Path:
    env_root = os.getenv("PRJ_ROOT", "").strip()
    if env_root:
        return Path(env_root).expanduser().resolve()
    return Path(__file__).resolve().parents[1]


def _config_candidates(project_root: Path) -> list[Path]:
    candidates: list[Path] = []
    config_home = os.getenv("PRJ_CONFIG_HOME", "").strip()
    if config_home:
        base = Path(config_home).expanduser()
        if not base.is_absolute():
            base = project_root / base
        candidates.append((base / "xiuxian-artisan-workshop" / "xiuxian.toml").resolve())
    else:
        candidates.append(
            (project_root / ".config" / "xiuxian-artisan-workshop" / "xiuxian.toml").resolve()
        )
    candidates.append((project_root / "packages" / "conf" / "xiuxian.toml").resolve())
    return candidates


def _load_toml(path: Path) -> dict[str, Any]:
    with path.open("rb") as handle:
        loaded = tomllib.load(handle)
    if not isinstance(loaded, dict):
        raise RuntimeError(f"invalid TOML root at {path}")
    return loaded


def _nested_get(obj: dict[str, Any], *keys: str) -> Any:
    current: Any = obj
    for key in keys:
        if not isinstance(current, dict) or key not in current:
            return None
        current = current[key]
    return current


def _bind_to_http_url(bind: str) -> str:
    raw = bind.strip()
    if raw.startswith("http://") or raw.startswith("https://"):
        return raw.rstrip("/")
    return f"http://{raw.rstrip('/')}"


def _resolve_runtime_config(args: argparse.Namespace) -> RuntimeConfig:
    project_root = _project_root()
    config_path: Path | None = None
    config_arg = getattr(args, "config", None)
    if config_arg:
        config_path = Path(config_arg).expanduser().resolve()
        if not config_path.is_file():
            raise RuntimeError(f"config file not found: {config_path}")
    else:
        for candidate in _config_candidates(project_root):
            if candidate.is_file():
                config_path = candidate
                break
    if config_path is None:
        raise RuntimeError("unable to locate xiuxian.toml (set --config or PRJ_CONFIG_HOME)")

    config = _load_toml(config_path)
    gateway_url = (
        (getattr(args, "gateway_url", None) or "").strip()
        or os.getenv("OMNI_AGENT_GATEWAY_URL", "").strip()
        or _bind_to_http_url(str(_nested_get(config, "gateway", "bind") or "127.0.0.1:18092"))
    )
    zhenfa_url = (
        (getattr(args, "zhenfa_url", None) or "").strip()
        or os.getenv("ZHENFA_BASE_URL", "").strip()
        or str(_nested_get(config, "zhenfa", "base_url") or "").strip()
    )
    zhenfa_url = _bind_to_http_url(zhenfa_url) if zhenfa_url else None

    valkey_url = (
        (getattr(args, "valkey_url", None) or "").strip()
        or os.getenv("XIUXIAN_WENDAO_VALKEY_URL", "").strip()
        or os.getenv("VALKEY_URL", "").strip()
        or str(_nested_get(config, "zhenfa", "valkey", "url") or "").strip()
    )

    memory_prefix = (
        (getattr(args, "memory_key_prefix", None) or "").strip()
        or os.getenv("OMNI_AGENT_MEMORY_VALKEY_KEY_PREFIX", "").strip()
        or str(_nested_get(config, "memory", "persistence_key_prefix") or "").strip()
        or "omni-agent:memory"
    )
    memory_table = (
        (getattr(args, "memory_table", None) or "").strip()
        or str(_nested_get(config, "memory", "table_name") or "").strip()
        or "episodes"
    )

    if getattr(args, "mode", "stream") == "gateway" and not valkey_url:
        raise RuntimeError(
            "valkey url is required in gateway mode (use --valkey-url or set zhenfa.valkey.url)"
        )

    return RuntimeConfig(
        project_root=project_root,
        config_path=config_path,
        gateway_url=gateway_url,
        zhenfa_url=zhenfa_url,
        valkey_url=valkey_url or None,
        memory_prefix=memory_prefix,
        memory_table=memory_table,
    )


def _tail(text: str, max_chars: int = 2200) -> str:
    if len(text) <= max_chars:
        return text
    return text[-max_chars:]


def _runtime_report_path(
    project_root: Path, report_name: str = "bootcamp_adversarial_v2.json"
) -> Path:
    runtime_dir = os.getenv("PRJ_RUNTIME_DIR", "").strip()
    if runtime_dir:
        base = Path(runtime_dir).expanduser()
        if not base.is_absolute():
            base = project_root / base
    else:
        base = project_root / ".run"
    out_dir = (base / "reports").resolve()
    out_dir.mkdir(parents=True, exist_ok=True)
    return out_dir / report_name


def _channel_log_file_path(project_root: Path, log_file_arg: str | None) -> Path:
    token = (log_file_arg or "").strip() or os.getenv("OMNI_CHANNEL_LOG_FILE", "").strip()
    if not token:
        token = DEFAULT_CHANNEL_LOG_FILE
    path = Path(token).expanduser()
    if not path.is_absolute():
        path = project_root / path
    return path.resolve()


def _http_post_json(url: str, payload: dict[str, Any], timeout_secs: int) -> dict[str, Any]:
    body = json.dumps(payload).encode("utf-8")
    req = urlrequest.Request(
        url=url,
        data=body,
        method="POST",
        headers={"Content-Type": "application/json"},
    )
    try:
        with urlrequest.urlopen(req, timeout=timeout_secs) as response:
            raw = response.read().decode("utf-8")
    except urlerror.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"HTTP {exc.code} calling {url}: {detail}") from exc
    except urlerror.URLError as exc:
        raise RuntimeError(f"failed to call {url}: {exc}") from exc
    try:
        decoded = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"invalid JSON response from {url}: {raw[:240]}") from exc
    if not isinstance(decoded, dict):
        raise RuntimeError(f"unexpected JSON response shape from {url}: {decoded!r}")
    return decoded


def _run_gateway_turn(
    gateway_url: str,
    session_id: str,
    user_message: str,
    timeout_secs: int,
) -> dict[str, Any]:
    return _http_post_json(
        f"{gateway_url}/message",
        {"session_id": session_id, "message": user_message},
        timeout_secs=timeout_secs,
    )


def _probe_zhenfa_wendao(zhenfa_url: str, session_id: str, timeout_secs: int) -> dict[str, Any]:
    payload = {
        "jsonrpc": "2.0",
        "method": "wendao.search",
        "id": f"bootcamp-{session_id}",
        "params": {
            "query": (
                "wendao://skills/agenda-management/references/steward.md "
                "agenda carryover risk overcommitment"
            ),
            "limit": 8,
        },
        "meta": {"session_id": session_id, "trace_id": f"bootcamp:{session_id}"},
    }
    return _http_post_json(f"{zhenfa_url}/rpc", payload, timeout_secs=timeout_secs)


def _select_q_values_key(client: RedisRespClient, prefix: str, table_name: str) -> str:
    strict_pattern = f"{prefix}:*:{table_name}:q_values"
    keys = client.scan_keys(strict_pattern)
    if not keys:
        keys = client.scan_keys(f"{prefix}:*:q_values")
    if not keys:
        raise RuntimeError(
            f"no q_values hash found in valkey (pattern `{strict_pattern}`); "
            "ensure memory persistence backend is valkey and agent has started at least once"
        )
    if len(keys) == 1:
        return keys[0]
    scored = [(client.hlen(key), key) for key in keys]
    scored.sort(reverse=True)
    return scored[0][1]


def _wait_for_q_update(
    client: RedisRespClient,
    q_hash_key: str,
    episode_id: str,
    before: float | None,
    wait_secs: float,
    poll_interval_secs: float,
) -> float | None:
    deadline = time.monotonic() + wait_secs
    latest = before
    while time.monotonic() < deadline:
        current = client.hget_float(q_hash_key, episode_id)
        if current is not None:
            latest = current
        if before is None and current is not None:
            return current
        if before is not None and current is not None and abs(current - before) > 1e-7:
            return current
        time.sleep(poll_interval_secs)
    return latest


def _run_nextest_check(
    name: str,
    command: list[str],
    project_root: Path,
    timeout_secs: int,
    env: dict[str, str] | None = None,
) -> dict[str, Any]:
    started = time.monotonic()
    run_env = os.environ.copy()
    if env:
        run_env.update(env)
    try:
        completed = subprocess.run(
            command,
            cwd=project_root,
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout_secs,
            env=run_env,
        )
        elapsed = round(time.monotonic() - started, 3)
        return {
            "name": name,
            "cmd": " ".join(command),
            "exit_code": completed.returncode,
            "elapsed_secs": elapsed,
            "stdout_tail": _tail(completed.stdout),
            "stderr_tail": _tail(completed.stderr),
            "ok": completed.returncode == 0,
        }
    except subprocess.TimeoutExpired as exc:
        elapsed = round(time.monotonic() - started, 3)
        stdout = exc.stdout if isinstance(exc.stdout, str) else ""
        stderr = exc.stderr if isinstance(exc.stderr, str) else ""
        return {
            "name": name,
            "cmd": " ".join(command),
            "exit_code": 124,
            "elapsed_secs": elapsed,
            "stdout_tail": _tail(stdout),
            "stderr_tail": _tail(stderr),
            "ok": False,
            "timeout": True,
        }


def _run_direct_bootcamp(
    runtime: RuntimeConfig, args: argparse.Namespace
) -> tuple[dict[str, Any], bool, str | None]:
    runs: list[dict[str, Any]] = []

    def execute(name: str, command: list[str], env: dict[str, str] | None = None) -> None:
        runs.append(
            _run_nextest_check(
                name,
                command,
                runtime.project_root,
                args.direct_timeout_secs,
                env=env,
            )
        )

    execute(
        "vfs_mandatory_extension",
        [
            "cargo",
            "nextest",
            "run",
            "-p",
            "xiuxian-wendao",
            "--test",
            "test_skill_vfs_uri",
            "-E",
            "test(rejects_entity_without_extension)",
        ],
    )
    execute(
        "bootcamp_mount_override",
        [
            "cargo",
            "nextest",
            "run",
            "-p",
            "xiuxian-qianji",
            "--test",
            "test_bootcamp_api",
            "-E",
            "test(bootcamp_mounts_override_runtime_wendao_uri_resolution)",
        ],
    )
    execute(
        "bootcamp_llm_mock",
        [
            "cargo",
            "nextest",
            "run",
            "-p",
            "xiuxian-qianji",
            "--features",
            "llm",
            "--test",
            "test_bootcamp_api",
            "-E",
            "test(bootcamp_runs_embedded_agenda_flow_with_mock_llm)",
        ],
    )
    execute(
        "zhenfa_probe",
        [
            "cargo",
            "nextest",
            "run",
            "-p",
            "xiuxian-zhenfa",
            "--test",
            "test_transmuter",
            "-E",
            "test(resolve_and_wash_validates_xml_for_xml_assets)",
        ],
    )
    execute(
        "memory_q_drift",
        [
            "cargo",
            "nextest",
            "run",
            "-p",
            "omni-agent",
            "--lib",
            "-E",
            "test(memory_reward_signal_bootcamp_penalize_then_recover)",
        ],
    )

    valkey_required = bool(runtime.valkey_url) or bool(args.require_valkey_persist)
    if runtime.valkey_url:
        execute(
            "memory_q_valkey_persist",
            [
                "cargo",
                "nextest",
                "run",
                "-p",
                "omni-agent",
                "--lib",
                "-E",
                "test(memory_reward_signal_persists_q_to_valkey_when_backend_present)",
            ],
            env={"VALKEY_URL": runtime.valkey_url},
        )

    run_map = {run["name"]: run for run in runs}
    zhenfa_ok = run_map.get("zhenfa_probe", {}).get("ok", False)
    memory_q_drift_ok = run_map.get("memory_q_drift", {}).get("ok", False)

    if runtime.valkey_url:
        valkey_run = run_map.get("memory_q_valkey_persist")
        valkey_assertion = {
            "ok": bool(valkey_run and valkey_run["ok"]),
            "required": True,
            "skipped": False,
        }
    elif valkey_required:
        valkey_assertion = {
            "ok": False,
            "required": True,
            "skipped": False,
            "reason": "--require-valkey-persist enabled but no valkey url configured",
        }
    else:
        valkey_assertion = {
            "ok": False,
            "required": False,
            "skipped": True,
            "reason": "no valkey_url configured for direct mode",
        }

    required_run_names = [
        "vfs_mandatory_extension",
        "bootcamp_mount_override",
        "bootcamp_llm_mock",
        "zhenfa_probe",
        "memory_q_drift",
    ]
    required_runs_ok = all(run_map.get(name, {}).get("ok", False) for name in required_run_names)
    memory_ok = memory_q_drift_ok and (
        valkey_assertion["ok"] if valkey_assertion["required"] else True
    )
    direct_ok = required_runs_ok and memory_ok and zhenfa_ok

    assertions = {
        "zhenfa_probe": {
            "ok": zhenfa_ok,
            "expression": "resolve_and_wash_validates_xml_for_xml_assets passed",
        },
        "memory_q_drift": {
            "ok": memory_q_drift_ok,
            "expression": "memory_reward_signal_bootcamp_penalize_then_recover passed",
        },
        "memory_q_valkey_persist": valkey_assertion,
    }

    checks = {
        "direct_bootcamp": {
            "ok": direct_ok,
            "runs": runs,
            "assertions": assertions,
        },
        "memory_q_table": {
            "ok": memory_ok,
            "penalty_assertion": assertions["memory_q_drift"],
            "valkey_persist_assertion": valkey_assertion,
        },
        "zhenfa_probe": {
            "ok": zhenfa_ok,
        },
    }

    error = None if direct_ok else "direct bootcamp verification failed"
    return checks, direct_ok, error


def _run_gateway_bootcamp(
    runtime: RuntimeConfig, args: argparse.Namespace
) -> tuple[dict[str, Any], bool, str | None]:
    assert runtime.valkey_url is not None

    checks: dict[str, Any] = {}
    success = False
    error: str | None = None

    try:
        with RedisRespClient(runtime.valkey_url) as redis_client:
            q_hash_key = _select_q_values_key(
                redis_client, runtime.memory_prefix, runtime.memory_table
            )
            episode_id = f"agenda_validation:{args.session_id}"
            q_before = redis_client.hget_float(q_hash_key, episode_id)

            checks["memory_q_table"] = {
                "q_hash_key": q_hash_key,
                "episode_id": episode_id,
                "q_before": q_before,
            }

            if runtime.zhenfa_url and not args.skip_zhenfa_probe:
                zhenfa_response = _probe_zhenfa_wendao(
                    runtime.zhenfa_url, args.session_id, args.zhenfa_timeout_secs
                )
                zhenfa_result = zhenfa_response.get("result")
                if isinstance(zhenfa_result, str):
                    result_preview = zhenfa_result[:240]
                else:
                    result_preview = json.dumps(
                        zhenfa_result, ensure_ascii=False, separators=(",", ":")
                    )[:240]
                checks["zhenfa_probe"] = {
                    "ok": "error" not in zhenfa_response,
                    "result_preview": result_preview,
                    "raw": zhenfa_response,
                }
            else:
                checks["zhenfa_probe"] = {
                    "ok": False,
                    "skipped": True,
                    "reason": "zhenfa base url unavailable or --skip-zhenfa-probe enabled",
                }

            gateway_response = _run_gateway_turn(
                runtime.gateway_url,
                args.session_id,
                args.trigger_intent,
                args.message_timeout_secs,
            )
            output = str(gateway_response.get("output", ""))
            checks["gateway_turn"] = {
                "ok": True,
                "output_preview": output[:400],
                "raw": gateway_response,
            }

            q_after = _wait_for_q_update(
                redis_client,
                q_hash_key,
                episode_id,
                q_before,
                args.valkey_wait_secs,
                args.valkey_poll_interval_secs,
            )
            checks["memory_q_table"]["q_after"] = q_after

            q_check_ok = q_after is not None and q_after <= args.expected_q_ceiling
            checks["memory_q_table"]["penalty_assertion"] = {
                "ok": q_check_ok,
                "expression": f"q_after <= {args.expected_q_ceiling}",
            }
            if not q_check_ok:
                raise RuntimeError(
                    f"Q penalty assertion failed: q_after={q_after} "
                    f"(expected <= {args.expected_q_ceiling})"
                )

        success = True
    except Exception as exc:  # pylint: disable=broad-except
        error = str(exc)
        success = False

    return checks, success, error


def _run_telegram_bootcamp(
    runtime: RuntimeConfig, args: argparse.Namespace
) -> tuple[dict[str, Any], bool, str | None]:
    log_file = _channel_log_file_path(runtime.project_root, args.log_file)
    chat_id = resolve_telegram_probe_chat_id(args.telegram_chat_id, runtime.project_root)
    user_id = resolve_telegram_probe_user_id(args.telegram_user_id, runtime.project_root)
    if chat_id is None or user_id is None:
        return (
            {},
            False,
            "telegram chat/user id missing (set --telegram-chat-id/--telegram-user-id or OMNI_TEST_* env)",
        )

    scenario = str(getattr(args, "channel_scenario", DEFAULT_CHANNEL_SCENARIO)).strip().lower()
    if scenario == "trinity":
        return _run_telegram_trinity_bootcamp(runtime, args, log_file, chat_id, user_id)

    probe_config = TelegramRealtimeProbeConfig(
        webhook_url=resolve_telegram_probe_url(args.webhook_url, runtime.project_root),
        log_file=log_file,
        chat_id=chat_id,
        user_id=user_id,
        prompt=args.trigger_intent,
        secret_token=args.telegram_secret_token or default_telegram_secret(runtime.project_root),
        username=(args.telegram_username or "").strip() or None,
        thread_id=args.telegram_thread_id,
        max_wait_secs=args.channel_max_wait_secs,
        max_idle_secs=args.channel_max_idle_secs,
        expected_events=tuple(args.expect_event),
        follow_logs=not bool(args.no_follow_channel_logs),
    )
    result = run_telegram_realtime_probe(probe_config)
    checks = {"telegram_probe": result.to_dict()}
    return checks, bool(result.ok), result.error


def _build_trinity_step_prompt(
    base_intent: str, step: dict[str, str], index: int, total: int
) -> str:
    step_id = step["id"]
    role = step["role"]
    instruction = step["instruction"]
    return (
        f"[Bootcamp Trinity Step {index}/{total}::{step_id}::{role}] "
        f"Base intent: {base_intent}\n"
        f"{instruction}"
    )


def _run_telegram_trinity_bootcamp(
    runtime: RuntimeConfig,
    args: argparse.Namespace,
    log_file: Path,
    chat_id: int,
    user_id: int,
) -> tuple[dict[str, Any], bool, str | None]:
    step_results: list[dict[str, Any]] = []
    webhook_url = resolve_telegram_probe_url(args.webhook_url, runtime.project_root)
    secret_token = args.telegram_secret_token or default_telegram_secret(runtime.project_root)
    username = (args.telegram_username or "").strip() or None
    total_steps = len(TRINITY_CHANNEL_STEPS)

    for index, step in enumerate(TRINITY_CHANNEL_STEPS, start=1):
        prompt = _build_trinity_step_prompt(args.trigger_intent, step, index, total_steps)
        probe_config = TelegramRealtimeProbeConfig(
            webhook_url=webhook_url,
            log_file=log_file,
            chat_id=chat_id,
            user_id=user_id,
            prompt=prompt,
            secret_token=secret_token,
            username=username,
            thread_id=args.telegram_thread_id,
            max_wait_secs=args.channel_max_wait_secs,
            max_idle_secs=args.channel_max_idle_secs,
            expected_events=tuple(args.expect_event),
            follow_logs=not bool(args.no_follow_channel_logs),
        )
        result = run_telegram_realtime_probe(probe_config)
        result_dict = result.to_dict()
        result_dict["step_id"] = step["id"]
        result_dict["role"] = step["role"]
        result_dict["step_index"] = index
        result_dict["prompt_preview"] = prompt[:280]
        semantic_error = _validate_trinity_step_semantics(step["id"], result_dict)
        result_dict["semantic_ok"] = semantic_error is None
        if semantic_error is not None:
            result_dict["semantic_error"] = semantic_error
        step_results.append(result_dict)
        if not result.ok:
            checks = {
                "telegram_probe": result_dict,
                "telegram_scenario": {
                    "ok": False,
                    "scenario": "trinity",
                    "completed_steps": len(step_results),
                    "total_steps": total_steps,
                    "steps": step_results,
                },
            }
            return checks, False, result.error or f"telegram trinity step failed: {step['id']}"
        if semantic_error is not None:
            checks = {
                "telegram_probe": result_dict,
                "telegram_scenario": {
                    "ok": False,
                    "scenario": "trinity",
                    "completed_steps": len(step_results),
                    "total_steps": total_steps,
                    "failed_step": step["id"],
                    "semantic_ok": False,
                    "steps": step_results,
                },
            }
            return (
                checks,
                False,
                f"telegram trinity semantic validation failed at {step['id']}: {semantic_error}",
            )

    role_evidence = {
        step["step_id"]: bool(step.get("reply_seen") or step.get("bot_seen"))
        for step in step_results
    }
    semantic_evidence = {step["step_id"]: bool(step.get("semantic_ok")) for step in step_results}
    checks = {
        "telegram_probe": step_results[-1],
        "telegram_scenario": {
            "ok": True,
            "scenario": "trinity",
            "completed_steps": len(step_results),
            "total_steps": total_steps,
            "role_evidence": role_evidence,
            "semantic_evidence": semantic_evidence,
            "semantic_ok": all(semantic_evidence.values()),
            "steps": step_results,
        },
    }
    return checks, True, None


def _validate_trinity_step_semantics(step_id: str, result_dict: dict[str, Any]) -> str | None:
    bot_line = str(result_dict.get("bot_line") or "")
    if not bot_line.strip():
        return "missing bot_line for semantic validation"

    marker = TRINITY_ROLE_MARKERS.get(step_id)
    lowered_line = bot_line.lower()
    if marker and marker.lower() not in lowered_line:
        return f"missing role marker `{marker}` in bot response"

    if step_id == "professor_audit":
        if (
            "<agenda_critique_report>" in lowered_line
            or "</agenda_critique_report>" in lowered_line
        ):
            return "professor output must be MarkdownV2 report (xml tags detected)"
        if _PROFESSOR_SCORE_RE.search(bot_line) is None:
            return "missing parsable score (0.0-1.0) in professor report"

    return None


def _run_discord_bootcamp(
    runtime: RuntimeConfig, args: argparse.Namespace
) -> tuple[dict[str, Any], bool, str | None]:
    log_file = _channel_log_file_path(runtime.project_root, args.log_file)
    channel_id = resolve_discord_probe_channel_id(args.discord_channel_id)
    user_id = resolve_discord_probe_user_id(args.discord_user_id)
    if not channel_id or not user_id:
        return (
            {},
            False,
            "discord channel/user id missing (set --discord-channel-id/--discord-user-id or env)",
        )

    probe_config = DiscordRealtimeProbeConfig(
        ingress_url=resolve_discord_probe_url(args.discord_ingress_url),
        log_file=log_file,
        channel_id=channel_id,
        user_id=user_id,
        prompt=args.trigger_intent,
        secret_token=resolve_discord_probe_secret(args.discord_secret_token),
        username=(args.discord_username or "").strip() or None,
        guild_id=(args.discord_guild_id or "").strip() or None,
        role_ids=tuple(args.discord_role_id or []),
        max_wait_secs=args.channel_max_wait_secs,
        max_idle_secs=args.channel_max_idle_secs,
        expected_events=tuple(args.expect_event),
        follow_logs=not bool(args.no_follow_channel_logs),
    )
    result = run_discord_realtime_probe(probe_config)
    checks = {"discord_probe": result.to_dict()}
    return checks, bool(result.ok), result.error


def _sanitize_stream_line(line: str) -> str:
    sanitized = line.replace("\x00", "").replace("\x1b", "")
    sanitized = re.sub(r"\[[0-9;]*m", "", sanitized)
    sanitized = re.sub(r"\b0?33\[[0-9;]*m", "", sanitized)
    sanitized = re.sub(r"\b33\b", "", sanitized)
    sanitized = re.sub(r"\b33(?=[A-Za-z_])", "", sanitized)
    sanitized = re.sub(r"(?<=[A-Za-z_])33\b", "", sanitized)
    return sanitized


def _classify_stream_line(line: str) -> tuple[str, str, str]:
    lowered = line.lower()
    if "summary" in lowered or "test result" in lowered:
        return "SUMMARY", _Color.YELLOW + _Color.BOLD, "prepare"
    if (
        "panic" in lowered
        or "error:" in lowered
        or ("failed" in lowered and "0 failed" not in lowered)
        or " fail [" in lowered
    ):
        return "ERROR", _Color.RED + _Color.BOLD, "error"
    if "score:" in lowered or "<score>" in lowered or "audit" in lowered or "critique" in lowered:
        return "AUDIT", _Color.CYAN + _Color.BOLD, "audit"
    if "reward" in lowered or "q-value" in lowered or "q_table" in lowered or "valkey" in lowered:
        return "EVOLVE", _Color.MAGENTA + _Color.BOLD, "evolve"
    if "node:" in lowered or "activating avatar" in lowered:
        return "RUN", _Color.GREEN + _Color.BOLD, "run"
    if (
        "compiling " in lowered
        or "checking " in lowered
        or "finished `test` profile" in lowered
        or "nextest run id" in lowered
        or "starting " in lowered
    ):
        return "PREPARE", _Color.BLUE + _Color.BOLD, "prepare"
    return "LOG", _Color.DIM, "other"


def _run_stream_bootcamp(
    runtime: RuntimeConfig, args: argparse.Namespace
) -> tuple[dict[str, Any], bool, str | None]:
    cmd = [
        "cargo",
        "nextest",
        "run",
        "-p",
        "xiuxian-qianji",
        "--features",
        "llm",
        "--test",
        "test_bootcamp_api",
        "-E",
        f"test({args.stream_test_name})",
        "--no-capture",
    ]
    env = os.environ.copy()
    env["XIUXIAN_BOOTCAMP_CONTEXT"] = json.dumps(ADVERSARIAL_CONTEXT)
    env.setdefault("RUST_LOG", "info")

    started = time.monotonic()
    process = subprocess.Popen(
        cmd,
        cwd=runtime.project_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        env=env,
        bufsize=1,
    )

    _render_panel(
        "Live Stream",
        [
            f"pid={process.pid}",
            f"test={args.stream_test_name}",
            "watching synaptic activity with heartbeat...",
        ],
        _Color.BLUE,
    )

    lines: list[str] = []
    events = {
        "prepare": 0,
        "run": 0,
        "audit": 0,
        "evolve": 0,
        "error": 0,
    }
    last_output = time.monotonic()

    assert process.stdout is not None
    while True:
        readable, _, _ = select.select([process.stdout], [], [], args.heartbeat_secs)
        if readable:
            line = process.stdout.readline()
            if not line:
                if process.poll() is not None:
                    break
                continue
            rendered = _sanitize_stream_line(line.rstrip("\n"))
            lines.append(rendered)

            stage, color, bucket = _classify_stream_line(rendered)
            if bucket in events:
                events[bucket] += 1
            prefix = f"[{stage:<7}]"
            print(_paint(f"{prefix} {rendered}", color))

            last_output = time.monotonic()
            sys.stdout.flush()
        else:
            if process.poll() is None:
                silence = int(time.monotonic() - last_output)
                print(
                    _paint(
                        f"  [heartbeat] no new signal for {silence}s (process still active)",
                        _Color.DIM,
                    )
                )
                sys.stdout.flush()
            else:
                break

    exit_code = process.wait()
    elapsed = round(time.monotonic() - started, 3)
    ok = exit_code == 0

    checks = {
        "stream_bootcamp": {
            "ok": ok,
            "cmd": " ".join(cmd),
            "exit_code": exit_code,
            "elapsed_secs": elapsed,
            "events": events,
            "log_tail": lines[-250:],
        }
    }
    _render_panel(
        "Stream Stage Counters",
        [
            f"prepare={events['prepare']}",
            f"run={events['run']}",
            f"audit={events['audit']}",
            f"evolve={events['evolve']}",
            f"error={events['error']}",
        ],
        _Color.CYAN,
    )
    error = None if ok else "stream bootcamp verification failed"
    return checks, ok, error


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="CyberXiuXian Bootcamp V2 runner (stream/direct/gateway)",
    )
    parser.add_argument(
        "--mode",
        choices=["stream", "direct", "gateway", "telegram", "discord"],
        default="stream",
        help=(
            "stream: real-time rust logs; direct: local nextest checks; gateway: HTTP+Valkey; "
            "telegram/discord: realtime channel probes"
        ),
    )
    parser.add_argument("--config", help="Path to xiuxian.toml")
    parser.add_argument("--gateway-url", help="omni-agent HTTP gateway base URL")
    parser.add_argument("--zhenfa-url", help="zhenfa JSON-RPC gateway base URL")
    parser.add_argument("--valkey-url", help="Valkey URL (redis://...)")
    parser.add_argument("--memory-key-prefix", help="Valkey key prefix for memory state")
    parser.add_argument("--memory-table", help="Memory table name for q-values hash key matching")
    parser.add_argument("--report-name", default="bootcamp_adversarial_v2.json")
    parser.add_argument(
        "--session-id",
        default=f"bootcamp-adversarial-{int(time.time())}",
        help="Session id used for this bootcamp run",
    )
    parser.add_argument(
        "--trigger-intent",
        default=DEFAULT_TRIGGER_INTENT,
        help="Adversarial scheduling prompt sent to omni-agent",
    )
    parser.add_argument(
        "--expected-q-ceiling",
        type=float,
        default=DEFAULT_EXPECTED_Q_CEILING,
        help="Expected upper bound for penalized Q value (default: 0.5)",
    )
    parser.add_argument(
        "--message-timeout-secs",
        type=int,
        default=DEFAULT_MESSAGE_TIMEOUT_SECS,
    )
    parser.add_argument(
        "--zhenfa-timeout-secs",
        type=int,
        default=DEFAULT_ZHENFA_TIMEOUT_SECS,
    )
    parser.add_argument(
        "--valkey-wait-secs",
        type=float,
        default=DEFAULT_VALKEY_WAIT_SECS,
    )
    parser.add_argument(
        "--valkey-poll-interval-secs",
        type=float,
        default=DEFAULT_VALKEY_POLL_INTERVAL_SECS,
    )
    parser.add_argument(
        "--skip-zhenfa-probe",
        action="store_true",
        help="Skip zhenfa /rpc probe in gateway mode",
    )
    parser.add_argument(
        "--direct-timeout-secs",
        type=int,
        default=DEFAULT_DIRECT_TIMEOUT_SECS,
        help="Per-check timeout for direct mode cargo nextest runs",
    )
    parser.add_argument(
        "--require-valkey-persist",
        action="store_true",
        help="In direct mode, fail if valkey persistence assertion cannot run/pass.",
    )
    parser.add_argument(
        "--stream-test-name",
        default=DEFAULT_STREAM_TEST,
        help="Rust test case name used in stream mode.",
    )
    parser.add_argument(
        "--heartbeat-secs",
        type=float,
        default=DEFAULT_HEARTBEAT_SECS,
        help="Heartbeat interval (seconds) for stream mode when no logs are emitted.",
    )
    parser.add_argument(
        "--log-file",
        default=None,
        help="Channel runtime log file for telegram/discord probe modes.",
    )
    parser.add_argument(
        "--webhook-url", default=None, help="Telegram webhook URL for telegram mode."
    )
    parser.add_argument(
        "--telegram-chat-id",
        type=int,
        default=None,
        help="Telegram chat id for synthetic webhook probe.",
    )
    parser.add_argument(
        "--telegram-user-id",
        type=int,
        default=None,
        help="Telegram user id for synthetic webhook probe.",
    )
    parser.add_argument(
        "--telegram-username",
        default=None,
        help="Telegram username attached to synthetic webhook payload.",
    )
    parser.add_argument(
        "--telegram-thread-id",
        type=int,
        default=None,
        help="Telegram topic/thread id for synthetic webhook payload.",
    )
    parser.add_argument(
        "--telegram-secret-token",
        default=None,
        help="Telegram webhook secret token override.",
    )
    parser.add_argument(
        "--discord-ingress-url",
        default=None,
        help="Discord ingress URL for discord probe mode.",
    )
    parser.add_argument(
        "--discord-channel-id",
        default=None,
        help="Discord channel id for synthetic ingress payload.",
    )
    parser.add_argument(
        "--discord-user-id",
        default=None,
        help="Discord user id for synthetic ingress payload.",
    )
    parser.add_argument(
        "--discord-username",
        default=None,
        help="Discord username attached to synthetic ingress payload.",
    )
    parser.add_argument(
        "--discord-guild-id",
        default=None,
        help="Discord guild id attached to synthetic ingress payload.",
    )
    parser.add_argument(
        "--discord-role-id",
        action="append",
        default=[],
        help="Discord role id attached to synthetic ingress payload (repeatable).",
    )
    parser.add_argument(
        "--discord-secret-token",
        default=None,
        help="Discord ingress secret token override.",
    )
    parser.add_argument(
        "--channel-max-wait-secs",
        type=float,
        default=DEFAULT_CHANNEL_MAX_WAIT_SECS,
        help="Max wait time for telegram/discord log confirmation.",
    )
    parser.add_argument(
        "--channel-max-idle-secs",
        type=float,
        default=DEFAULT_CHANNEL_MAX_IDLE_SECS,
        help="Max idle time while waiting for new telegram/discord logs.",
    )
    parser.add_argument(
        "--expect-event",
        action="append",
        default=[],
        help="Expected structured event token in runtime logs (repeatable).",
    )
    parser.add_argument(
        "--no-follow-channel-logs",
        action="store_true",
        help="Disable realtime streaming of channel log lines during probe mode.",
    )
    parser.add_argument(
        "--channel-scenario",
        choices=["single", "trinity"],
        default=DEFAULT_CHANNEL_SCENARIO,
        help=(
            "single: one synthetic probe message; "
            "trinity: Student/Steward/Professor adversarial demonstration."
        ),
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    runtime = _resolve_runtime_config(args)
    _render_logo(args.mode)

    _render_panel(
        "Runtime",
        [
            f"config:  {runtime.config_path}",
            f"gateway: {runtime.gateway_url}",
            f"zhenfa:  {runtime.zhenfa_url or '(disabled)'}",
            f"valkey:  {runtime.valkey_url or '(disabled)'}",
        ],
        _Color.CYAN,
    )

    report: dict[str, Any] = {
        "timestamp_utc": datetime.now(UTC).isoformat(),
        "runtime": {
            "config_path": str(runtime.config_path),
            "gateway_url": runtime.gateway_url,
            "zhenfa_url": runtime.zhenfa_url,
            "valkey_url": runtime.valkey_url,
            "memory_prefix": runtime.memory_prefix,
            "memory_table": runtime.memory_table,
        },
        "scenario": {
            "id": "adversarial_agenda_evolution",
            "name": "Adversarial Agenda Q-Evolution",
            "trigger_intent": args.trigger_intent,
            "expected_q_ceiling": args.expected_q_ceiling,
            "session_id": args.session_id,
            "mode": args.mode,
            "channel_scenario": getattr(args, "channel_scenario", DEFAULT_CHANNEL_SCENARIO),
        },
        "checks": {},
        "success": False,
    }

    if args.mode == "direct":
        checks, success, error = _run_direct_bootcamp(runtime, args)
    elif args.mode == "gateway":
        checks, success, error = _run_gateway_bootcamp(runtime, args)
    elif args.mode == "telegram":
        checks, success, error = _run_telegram_bootcamp(runtime, args)
    elif args.mode == "discord":
        checks, success, error = _run_discord_bootcamp(runtime, args)
    else:
        checks, success, error = _run_stream_bootcamp(runtime, args)

    report["checks"] = checks
    report["success"] = success
    if error:
        report["error"] = error

    report_path = _runtime_report_path(runtime.project_root, args.report_name)
    report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")

    if args.mode == "direct":
        runs = report.get("checks", {}).get("direct_bootcamp", {}).get("runs", [])
        _render_panel("Direct Probe Summary", [], _Color.BLUE)
        _render_run_table(runs)
    elif args.mode in {"telegram", "discord"}:
        _render_channel_scenario_summary(checks)

    overall = _status_tag(bool(report.get("success", False)))
    _render_panel(
        "Final",
        [
            f"result: {overall}",
            f"report: {report_path}",
            (
                f"error:  {report.get('error')}"
                if not report.get("success") and report.get("error")
                else "error:  (none)"
            ),
        ],
        _Color.GREEN if report.get("success") else _Color.RED,
    )

    return 0 if report.get("success") else 1


if __name__ == "__main__":
    raise SystemExit(main())
