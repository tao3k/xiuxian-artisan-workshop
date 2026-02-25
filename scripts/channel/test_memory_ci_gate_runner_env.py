#!/usr/bin/env python3
"""Unit tests for memory CI gate runner environment helpers."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import SimpleNamespace

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

_env_module = importlib.import_module("memory_ci_gate_runner_env")


def test_resolve_script_paths_uses_expected_filenames(tmp_path: Path) -> None:
    paths = _env_module.resolve_script_paths(tmp_path)
    assert paths["valkey_start"] == tmp_path / "valkey-start.sh"
    assert paths["valkey_stop"] == tmp_path / "valkey-stop.sh"
    assert paths["mock_server"] == tmp_path / "mock_telegram_api.py"
    assert paths["memory_suite"] == tmp_path / "test_omni_agent_memory_suite.py"
    assert paths["session_matrix"] == tmp_path / "test_omni_agent_session_matrix.py"
    assert paths["memory_benchmark"] == tmp_path / "test_omni_agent_memory_benchmark.py"


def test_build_runtime_env_sets_ci_variables(tmp_path: Path) -> None:
    cfg = SimpleNamespace(
        valkey_url="redis://127.0.0.1:16379/0",
        valkey_prefix="xiuxian_wendao:test",
        webhook_secret="secret",
        telegram_api_port=19191,
        webhook_port=19192,
        runtime_log_file=tmp_path / "runtime.log",
        chat_id=1,
        chat_b=2,
        chat_c=3,
        user_id=10,
        user_b=11,
        user_c=12,
        username="tester",
        project_root=tmp_path,
    )
    captured: dict[str, Path] = {}

    def _write_settings(_cfg: object, *, config_home: Path) -> Path:
        captured["config_home"] = config_home
        return config_home / "omni-dev-fusion" / "settings.yaml"

    env, settings_path = _env_module.build_runtime_env(
        cfg,
        default_run_suffix_fn=lambda: "run-01",
        write_ci_channel_acl_settings_fn=_write_settings,
    )

    assert env["VALKEY_URL"] == "redis://127.0.0.1:16379/0"
    assert env["OMNI_AGENT_SESSION_VALKEY_PREFIX"] == "xiuxian_wendao:test"
    assert env["OMNI_AGENT_MEMORY_VALKEY_KEY_PREFIX"] == "xiuxian_wendao:test:memory"
    assert env["OMNI_WEBHOOK_URL"] == "http://127.0.0.1:19192/telegram/webhook"
    assert env["OMNI_CHANNEL_LOG_FILE"] == str(tmp_path / "runtime.log")
    assert env["OMNI_TEST_USER_ID"] == "10"
    assert env["PRJ_CONFIG_HOME"] == str(tmp_path / ".run" / "config" / "memory-ci-gate" / "run-01")
    assert captured["config_home"] == tmp_path / ".run" / "config" / "memory-ci-gate" / "run-01"
    assert settings_path == captured["config_home"] / "omni-dev-fusion" / "settings.yaml"
