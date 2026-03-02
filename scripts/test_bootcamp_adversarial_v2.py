from __future__ import annotations

import argparse
import importlib.util
import sys
from pathlib import Path

import pytest

_MODULE_PATH = Path(__file__).resolve().with_name("bootcamp_adversarial_v2.py")
_SPEC = importlib.util.spec_from_file_location("bootcamp_adversarial_v2", _MODULE_PATH)
assert _SPEC and _SPEC.loader
_MODULE = importlib.util.module_from_spec(_SPEC)
sys.modules[_SPEC.name] = _MODULE
_SPEC.loader.exec_module(_MODULE)


def test_bind_to_http_url_normalizes_bind() -> None:
    assert _MODULE._bind_to_http_url("127.0.0.1:18092") == "http://127.0.0.1:18092"
    assert _MODULE._bind_to_http_url("http://127.0.0.1:18092/") == "http://127.0.0.1:18092"
    assert _MODULE._bind_to_http_url("https://example.com/") == "https://example.com"


def test_config_candidates_respect_prj_config_home(monkeypatch: pytest.MonkeyPatch) -> None:
    project_root = Path("/tmp/xiuxian-artisan-workshop")
    monkeypatch.setenv("PRJ_CONFIG_HOME", "custom-config")

    candidates = _MODULE._config_candidates(project_root)

    assert (
        candidates[0]
        == (project_root / "custom-config/xiuxian-artisan-workshop/xiuxian.toml").resolve()
    )
    assert candidates[1] == (project_root / "packages/conf/xiuxian.toml").resolve()


class _FakeRedisClient:
    def __init__(self, key_groups: dict[str, list[str]], hlen_values: dict[str, int]) -> None:
        self._key_groups = key_groups
        self._hlen_values = hlen_values

    def scan_keys(self, pattern: str, count: int = 200) -> list[str]:
        _ = count
        return list(self._key_groups.get(pattern, []))

    def hlen(self, key: str) -> int:
        return self._hlen_values[key]


def test_select_q_values_key_prefers_densest_hash() -> None:
    prefix = "omni-agent:memory"
    table = "episodes"
    strict = f"{prefix}:*:{table}:q_values"
    client = _FakeRedisClient(
        key_groups={strict: ["k:1", "k:2", "k:3"]},
        hlen_values={"k:1": 4, "k:2": 42, "k:3": 15},
    )

    selected = _MODULE._select_q_values_key(client, prefix, table)

    assert selected == "k:2"


def test_select_q_values_key_uses_fallback_pattern() -> None:
    prefix = "omni-agent:memory"
    table = "episodes"
    strict = f"{prefix}:*:{table}:q_values"
    fallback = f"{prefix}:*:q_values"
    client = _FakeRedisClient(
        key_groups={strict: [], fallback: ["fallback:key"]},
        hlen_values={"fallback:key": 1},
    )

    selected = _MODULE._select_q_values_key(client, prefix, table)

    assert selected == "fallback:key"


def test_select_q_values_key_raises_when_missing() -> None:
    client = _FakeRedisClient(key_groups={}, hlen_values={})
    with pytest.raises(RuntimeError, match="no q_values hash found"):
        _MODULE._select_q_values_key(client, "missing", "episodes")


def test_resolve_runtime_config_uses_xiuxian_toml(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    project_root = tmp_path / "repo"
    config_dir = project_root / ".config" / "xiuxian-artisan-workshop"
    config_dir.mkdir(parents=True)
    config_file = config_dir / "xiuxian.toml"
    config_file.write_text(
        """
[gateway]
bind = "127.0.0.1:19092"

[zhenfa]
base_url = "127.0.0.1:19093"

[zhenfa.valkey]
url = "redis://127.0.0.1:6380/0"

[memory]
persistence_key_prefix = "custom-prefix"
table_name = "custom-episodes"
""".strip()
        + "\n",
        encoding="utf-8",
    )

    monkeypatch.setenv("PRJ_ROOT", str(project_root))
    monkeypatch.delenv("PRJ_CONFIG_HOME", raising=False)
    monkeypatch.delenv("OMNI_AGENT_GATEWAY_URL", raising=False)
    monkeypatch.delenv("ZHENFA_BASE_URL", raising=False)
    monkeypatch.delenv("XIUXIAN_WENDAO_VALKEY_URL", raising=False)
    monkeypatch.delenv("VALKEY_URL", raising=False)
    monkeypatch.delenv("OMNI_AGENT_MEMORY_VALKEY_KEY_PREFIX", raising=False)

    args = argparse.Namespace(
        mode="gateway",
        config=None,
        gateway_url=None,
        zhenfa_url=None,
        valkey_url=None,
        memory_key_prefix=None,
        memory_table=None,
    )

    runtime = _MODULE._resolve_runtime_config(args)

    assert runtime.config_path == config_file.resolve()
    assert runtime.gateway_url == "http://127.0.0.1:19092"
    assert runtime.zhenfa_url == "http://127.0.0.1:19093"
    assert runtime.valkey_url == "redis://127.0.0.1:6380/0"
    assert runtime.memory_prefix == "custom-prefix"
    assert runtime.memory_table == "custom-episodes"


def test_resolve_runtime_config_allows_missing_valkey_in_direct_mode(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    project_root = tmp_path / "repo"
    config_dir = project_root / ".config" / "xiuxian-artisan-workshop"
    config_dir.mkdir(parents=True)
    config_file = config_dir / "xiuxian.toml"
    config_file.write_text(
        """
[gateway]
bind = "127.0.0.1:19092"
""".strip()
        + "\n",
        encoding="utf-8",
    )

    monkeypatch.setenv("PRJ_ROOT", str(project_root))
    monkeypatch.delenv("PRJ_CONFIG_HOME", raising=False)
    monkeypatch.delenv("OMNI_AGENT_GATEWAY_URL", raising=False)
    monkeypatch.delenv("ZHENFA_BASE_URL", raising=False)
    monkeypatch.delenv("XIUXIAN_WENDAO_VALKEY_URL", raising=False)
    monkeypatch.delenv("VALKEY_URL", raising=False)
    monkeypatch.delenv("OMNI_AGENT_MEMORY_VALKEY_KEY_PREFIX", raising=False)

    args = argparse.Namespace(
        mode="direct",
        config=None,
        gateway_url=None,
        zhenfa_url=None,
        valkey_url=None,
        memory_key_prefix=None,
        memory_table=None,
    )

    runtime = _MODULE._resolve_runtime_config(args)

    assert runtime.config_path == config_file.resolve()
    assert runtime.gateway_url == "http://127.0.0.1:19092"
    assert runtime.valkey_url is None


def test_sanitize_stream_line_removes_ansi_noise() -> None:
    raw = "\x1b[1;34m[Node] hi\x1b[0m 33avatar33"
    sanitized = _MODULE._sanitize_stream_line(raw)
    assert "\x1b" not in sanitized
    assert "[1;34m" not in sanitized
    assert "33avatar33" not in sanitized
    assert "avatar" in sanitized


def test_classify_stream_line_summary_is_not_error() -> None:
    stage, _, bucket = _MODULE._classify_stream_line(
        "test result: ok. 1 passed; 0 failed; 0 ignored"
    )
    assert stage == "SUMMARY"
    assert bucket == "prepare"


def test_channel_log_file_path_defaults_to_runtime_log(tmp_path: Path) -> None:
    log_path = _MODULE._channel_log_file_path(tmp_path, None)
    assert log_path == (tmp_path / ".run/logs/omni-agent-webhook.log").resolve()


def test_run_telegram_bootcamp_requires_identity(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    runtime = _MODULE.RuntimeConfig(
        project_root=tmp_path,
        config_path=tmp_path / "xiuxian.toml",
        gateway_url="http://127.0.0.1:18092",
        zhenfa_url=None,
        valkey_url=None,
        memory_prefix="omni-agent:memory",
        memory_table="episodes",
    )
    args = argparse.Namespace(
        log_file=None,
        telegram_chat_id=None,
        telegram_user_id=None,
        webhook_url=None,
        trigger_intent="probe",
        telegram_secret_token=None,
        telegram_username=None,
        telegram_thread_id=None,
        channel_max_wait_secs=10.0,
        channel_max_idle_secs=1.0,
        expect_event=[],
        no_follow_channel_logs=True,
    )

    monkeypatch.setattr(_MODULE, "resolve_telegram_probe_chat_id", lambda _value, _root: None)
    monkeypatch.setattr(_MODULE, "resolve_telegram_probe_user_id", lambda _value, _root: None)

    checks, ok, error = _MODULE._run_telegram_bootcamp(runtime, args)
    assert not ok
    assert checks == {}
    assert error is not None and "missing" in error


def test_run_telegram_bootcamp_trinity_runs_three_steps(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    runtime = _MODULE.RuntimeConfig(
        project_root=tmp_path,
        config_path=tmp_path / "xiuxian.toml",
        gateway_url="http://127.0.0.1:18092",
        zhenfa_url=None,
        valkey_url=None,
        memory_prefix="omni-agent:memory",
        memory_table="episodes",
    )
    args = argparse.Namespace(
        channel_scenario="trinity",
        log_file=None,
        telegram_chat_id=123,
        telegram_user_id=456,
        webhook_url="http://127.0.0.1:18081/telegram/webhook",
        trigger_intent="Schedule an impossible 12-hour plan with no breaks.",
        telegram_secret_token=None,
        telegram_username=None,
        telegram_thread_id=None,
        channel_max_wait_secs=10.0,
        channel_max_idle_secs=1.0,
        expect_event=[],
        no_follow_channel_logs=True,
    )

    class _FakeProbeResult:
        def __init__(self, bot_line: str) -> None:
            self.ok = True
            self.error = None
            self._bot_line = bot_line

        def to_dict(self) -> dict[str, object]:
            return {
                "ok": True,
                "reply_seen": False,
                "bot_seen": True,
                "error": None,
                "bot_line": self._bot_line,
            }

    captured_prompts: list[str] = []

    def _fake_probe(config: object) -> _FakeProbeResult:
        captured_prompts.append(config.prompt)
        prompt = str(config.prompt)
        if "student_ambition" in prompt.lower():
            return _FakeProbeResult("**Student_Ambition** 12-hour overload plan with no breaks.")
        if "steward_logistics" in prompt.lower():
            return _FakeProbeResult(
                "**Steward_Logistics** carryover risk and feasibility critique."
            )
        return _FakeProbeResult(
            "**Professor_Audit** *Agenda Critique Report* "
            "*Score:* 0.45 *Critique:* - overloaded day *Verdict:* fail - unrealistic."
        )

    monkeypatch.setattr(_MODULE, "resolve_telegram_probe_chat_id", lambda value, _root: value)
    monkeypatch.setattr(_MODULE, "resolve_telegram_probe_user_id", lambda value, _root: value)
    monkeypatch.setattr(_MODULE, "resolve_telegram_probe_url", lambda value, _root: str(value))
    monkeypatch.setattr(_MODULE, "default_telegram_secret", lambda _root: None)
    monkeypatch.setattr(_MODULE, "run_telegram_realtime_probe", _fake_probe)

    checks, ok, error = _MODULE._run_telegram_bootcamp(runtime, args)

    assert ok
    assert error is None
    assert len(captured_prompts) == 3
    assert checks["telegram_scenario"]["ok"] is True
    assert checks["telegram_scenario"]["scenario"] == "trinity"
    assert checks["telegram_scenario"]["completed_steps"] == 3
    assert checks["telegram_scenario"]["total_steps"] == 3
    assert checks["telegram_scenario"]["semantic_ok"] is True


def test_run_telegram_bootcamp_trinity_fails_on_professor_xml_output(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    runtime = _MODULE.RuntimeConfig(
        project_root=tmp_path,
        config_path=tmp_path / "xiuxian.toml",
        gateway_url="http://127.0.0.1:18092",
        zhenfa_url=None,
        valkey_url=None,
        memory_prefix="omni-agent:memory",
        memory_table="episodes",
    )
    args = argparse.Namespace(
        channel_scenario="trinity",
        log_file=None,
        telegram_chat_id=123,
        telegram_user_id=456,
        webhook_url="http://127.0.0.1:18081/telegram/webhook",
        trigger_intent="Schedule an impossible 12-hour plan with no breaks.",
        telegram_secret_token=None,
        telegram_username=None,
        telegram_thread_id=None,
        channel_max_wait_secs=10.0,
        channel_max_idle_secs=1.0,
        expect_event=[],
        no_follow_channel_logs=True,
    )

    class _FakeProbeResult:
        def __init__(self, bot_line: str) -> None:
            self.ok = True
            self.error = None
            self._bot_line = bot_line

        def to_dict(self) -> dict[str, object]:
            return {
                "ok": True,
                "reply_seen": False,
                "bot_seen": True,
                "error": None,
                "bot_line": self._bot_line,
            }

    def _fake_probe(config: object) -> _FakeProbeResult:
        prompt = str(config.prompt)
        if "student_ambition" in prompt.lower():
            return _FakeProbeResult("**Student_Ambition** overloaded plan.")
        if "steward_logistics" in prompt.lower():
            return _FakeProbeResult("**Steward_Logistics** critique.")
        return _FakeProbeResult(
            "**Professor_Audit** <agenda_critique_report><score>0.3</score></agenda_critique_report>"
        )

    monkeypatch.setattr(_MODULE, "resolve_telegram_probe_chat_id", lambda value, _root: value)
    monkeypatch.setattr(_MODULE, "resolve_telegram_probe_user_id", lambda value, _root: value)
    monkeypatch.setattr(_MODULE, "resolve_telegram_probe_url", lambda value, _root: str(value))
    monkeypatch.setattr(_MODULE, "default_telegram_secret", lambda _root: None)
    monkeypatch.setattr(_MODULE, "run_telegram_realtime_probe", _fake_probe)

    checks, ok, error = _MODULE._run_telegram_bootcamp(runtime, args)

    assert not ok
    assert error is not None and "semantic validation failed" in error
    scenario = checks["telegram_scenario"]
    assert scenario["ok"] is False
    assert scenario["failed_step"] == "professor_audit"
