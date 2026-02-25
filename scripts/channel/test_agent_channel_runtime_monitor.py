#!/usr/bin/env python3
"""Unit tests for runtime monitor helpers."""

from __future__ import annotations

import importlib
import json
import sys
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

monitor_module = importlib.import_module("agent_channel_runtime_monitor")
runtime_module = importlib.import_module("agent_channel_runtime_monitor_runtime")


def test_classify_exit_maps_ok_signal_and_nonzero() -> None:
    assert monitor_module.classify_exit(0)["kind"] == "ok"
    assert monitor_module.classify_exit(-15)["kind"] == "signal"
    assert monitor_module.classify_exit(1)["kind"] == "nonzero"


def test_extract_event_token_parses_event_field() -> None:
    line = '2026-01-01T00:00:00Z INFO event="agent.memory.recall.planned" scope=test'
    assert monitor_module.extract_event_token(line) == "agent.memory.recall.planned"


def test_write_report_writes_json_and_jsonl(tmp_path: Path) -> None:
    report = {"schema_version": 1, "ok": True}
    report_path = tmp_path / "reports" / "report.json"
    jsonl_path = tmp_path / "reports" / "report.jsonl"

    monitor_module.write_report(report_path, jsonl_path, report)

    loaded = json.loads(report_path.read_text(encoding="utf-8"))
    assert loaded["ok"] is True
    lines = jsonl_path.read_text(encoding="utf-8").splitlines()
    assert len(lines) == 1
    assert json.loads(lines[0])["schema_version"] == 1


def test_run_monitored_process_success_records_stats(tmp_path: Path) -> None:
    log_file = tmp_path / "runtime.log"
    report_file = tmp_path / "report.json"
    report_jsonl = tmp_path / "report.jsonl"

    command = [
        sys.executable,
        "-c",
        (
            "print('Webhook received Telegram update');"
            "print('event=\"agent.test.event\"');"
            "print('← User: hello');"
            "print('→ Bot: ok')"
        ),
    ]
    code = runtime_module.run_monitored_process(
        command=command,
        log_file=log_file,
        report_file=report_file,
        report_jsonl=report_jsonl,
        tail_lines=3,
    )
    assert code == 0

    payload = json.loads(report_file.read_text(encoding="utf-8"))
    stats = payload["stats"]
    assert stats["total_lines"] == 4
    assert stats["saw_webhook"] is True
    assert stats["saw_user_dispatch"] is True
    assert stats["saw_bot_reply"] is True
    assert payload["events"]["counts"].get("agent.test.event") == 1
    assert len(payload["tail"]) == 3
    assert log_file.exists()
    assert report_jsonl.exists()


def test_run_monitored_process_spawn_error_generates_report(tmp_path: Path) -> None:
    log_file = tmp_path / "runtime.log"
    report_file = tmp_path / "report.json"

    code = runtime_module.run_monitored_process(
        command=["__definitely_missing_runtime_monitor_binary__"],
        log_file=log_file,
        report_file=report_file,
        report_jsonl=None,
        tail_lines=10,
    )
    assert code == 127
    payload = json.loads(report_file.read_text(encoding="utf-8"))
    assert payload["exit"]["kind"] == "spawn_error"
    assert payload["exit"]["exit_code"] == 127
