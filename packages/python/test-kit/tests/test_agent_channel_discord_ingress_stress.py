"""Tests for scripts/channel/test_omni_agent_discord_ingress_stress.py."""

from __future__ import annotations

import argparse
import importlib.util
import sys
from typing import TYPE_CHECKING

import pytest

from omni.foundation.runtime.gitops import get_project_root

if TYPE_CHECKING:
    from pathlib import Path
    from types import ModuleType


def _load_module() -> ModuleType:
    root = get_project_root()
    script_path = root / "scripts" / "channel" / "test_omni_agent_discord_ingress_stress.py"
    spec = importlib.util.spec_from_file_location(
        "omni_agent_discord_ingress_stress",
        script_path,
    )
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def _make_args(tmp_path: Path, **overrides: object) -> argparse.Namespace:
    defaults: dict[str, object] = {
        "rounds": 2,
        "warmup_rounds": 1,
        "parallel": 2,
        "requests_per_worker": 3,
        "timeout_secs": 0.2,
        "cooldown_secs": 0.0,
        "ingress_url": "http://127.0.0.1:18082/discord/ingress",
        "secret_token": "",
        "channel_id": "2001",
        "user_id": "1001",
        "guild_id": "3001",
        "username": "alice",
        "role_id": ["r1"],
        "prompt": "stress",
        "log_file": str(tmp_path / "runtime.log"),
        "project_root": str(tmp_path),
        "output_json": str(tmp_path / "report.json"),
        "output_markdown": str(tmp_path / "report.md"),
        "quality_max_failure_rate": 0.0,
        "quality_max_p95_ms": 0.0,
        "quality_min_rps": 0.0,
    }
    defaults.update(overrides)
    return argparse.Namespace(**defaults)


def test_build_config_requires_channel_id(tmp_path: Path) -> None:
    module = _load_module()
    args = _make_args(tmp_path, channel_id="")
    with pytest.raises(ValueError, match="--channel-id is required"):
        module.build_config(args)


def test_run_stress_delegates_round_result_class(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    module = _load_module()
    cfg = module.build_config(_make_args(tmp_path))
    captured: dict[str, object] = {}

    def _fake_run_stress(cfg_in: object, *, round_result_cls: object) -> dict[str, object]:
        captured["cfg"] = cfg_in
        captured["round_result_cls"] = round_result_cls
        return {"ok": True}

    monkeypatch.setattr(module._runtime_module, "run_stress", _fake_run_stress)
    report = module.run_stress(cfg)

    assert report == {"ok": True}
    assert captured["cfg"] == cfg
    assert captured["round_result_cls"] is module.RoundResult


def test_write_report_creates_json_and_markdown(tmp_path: Path) -> None:
    module = _load_module()
    output_json = tmp_path / "stress.json"
    output_markdown = tmp_path / "stress.md"
    report = {
        "started_at": "2026-02-24T00:00:00Z",
        "finished_at": "2026-02-24T00:00:01Z",
        "duration_ms": 1000,
        "inputs": {
            "ingress_url": "http://127.0.0.1:8082/discord/ingress",
        },
        "summary": {
            "measured_rounds": 1,
            "total_requests": 10,
            "success_requests": 10,
            "failed_requests": 0,
            "failure_rate": 0.0,
            "average_rps": 100.0,
            "max_round_p95_ms": 12.0,
            "parsed_messages": 10,
            "queue_wait_events": 0,
            "foreground_gate_wait_events": 0,
            "inbound_queue_unavailable_events": 0,
            "quality_passed": True,
            "quality_failures": [],
        },
        "rounds": [
            {
                "round_index": 1,
                "warmup": False,
                "total_requests": 10,
                "success_requests": 10,
                "failed_requests": 0,
                "p95_latency_ms": 12.0,
                "rps": 100.0,
                "log_queue_wait_events": 0,
                "log_foreground_gate_wait_events": 0,
                "log_inbound_queue_unavailable_events": 0,
            }
        ],
    }

    module.write_report(report, output_json, output_markdown)

    assert output_json.exists()
    assert output_markdown.exists()
    assert "Discord Ingress Stress Report" in output_markdown.read_text(encoding="utf-8")
