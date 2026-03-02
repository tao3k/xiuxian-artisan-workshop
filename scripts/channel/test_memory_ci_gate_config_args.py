#!/usr/bin/env python3
"""Focused config/port tests for memory CI gate."""

from __future__ import annotations

import socket
import sys
from typing import TYPE_CHECKING

from test_omni_agent_memory_ci_gate import can_bind_tcp, parse_args, resolve_runtime_ports

if TYPE_CHECKING:
    from pathlib import Path

LOOPBACK_BIND_HOST = "127.0.0.1"


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind((LOOPBACK_BIND_HOST, 0))
        return int(sock.getsockname()[1])


def test_resolve_runtime_ports_reassigns_when_requested_ports_are_occupied() -> None:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as first:
        first.bind((LOOPBACK_BIND_HOST, 0))
        first.listen(1)
        first_port = int(first.getsockname()[1])
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as second:
            second.bind((LOOPBACK_BIND_HOST, 0))
            second.listen(1)
            second_port = int(second.getsockname()[1])

            webhook_port, telegram_api_port = resolve_runtime_ports(
                webhook_port=first_port,
                telegram_api_port=second_port,
            )

    assert webhook_port != first_port
    assert telegram_api_port != second_port
    assert webhook_port != telegram_api_port
    assert can_bind_tcp(LOOPBACK_BIND_HOST, webhook_port)
    assert can_bind_tcp(LOOPBACK_BIND_HOST, telegram_api_port)


def test_resolve_runtime_ports_reassigns_when_ports_conflict() -> None:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as socket_holder:
        socket_holder.bind((LOOPBACK_BIND_HOST, 0))
        socket_holder.listen(1)
        requested = int(socket_holder.getsockname()[1])

    webhook_port, telegram_api_port = resolve_runtime_ports(
        webhook_port=requested,
        telegram_api_port=requested,
    )

    assert webhook_port != telegram_api_port
    assert can_bind_tcp(LOOPBACK_BIND_HOST, webhook_port)
    assert can_bind_tcp(LOOPBACK_BIND_HOST, telegram_api_port)


def test_parse_args_uses_run_scoped_default_artifacts(monkeypatch, tmp_path: Path) -> None:
    webhook_port = pick_free_port()
    telegram_port = pick_free_port()
    valkey_port = pick_free_port()
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "test_omni_agent_memory_ci_gate.py",
            "--profile",
            "quick",
            "--webhook-port",
            str(webhook_port),
            "--telegram-api-port",
            str(telegram_port),
            "--valkey-port",
            str(valkey_port),
        ],
    )

    cfg = parse_args(tmp_path)

    assert cfg.runtime_log_file.parent == (tmp_path / ".run" / "logs")
    assert cfg.runtime_log_file.name.startswith("omni-agent-webhook-ci-quick-")
    assert cfg.runtime_log_file.suffix == ".log"
    assert cfg.mock_log_file.parent == (tmp_path / ".run" / "logs")
    assert cfg.mock_log_file.name.startswith("omni-agent-mock-telegram-quick-")
    assert cfg.mock_log_file.suffix == ".log"
    assert cfg.evolution_report_json.parent == (tmp_path / ".run" / "reports")
    assert cfg.evolution_report_json.name.startswith("omni-agent-memory-evolution-quick-")
    assert cfg.evolution_report_json.suffix == ".json"
    assert cfg.trace_report_markdown.parent == (tmp_path / ".run" / "reports")
    assert cfg.trace_report_markdown.name.startswith("omni-agent-trace-reconstruction-quick-")
    assert cfg.trace_report_markdown.suffix == ".md"
    assert cfg.cross_group_report_json.parent == (tmp_path / ".run" / "reports")
    assert cfg.cross_group_report_json.name.startswith("agent-channel-cross-group-complex-quick-")
    assert cfg.cross_group_report_json.suffix == ".json"
    assert cfg.cross_group_report_markdown.parent == (tmp_path / ".run" / "reports")
    assert cfg.cross_group_report_markdown.name.startswith(
        "agent-channel-cross-group-complex-quick-"
    )
    assert cfg.cross_group_report_markdown.suffix == ".md"
    assert cfg.benchmark_iterations == 3
    assert cfg.max_mcp_call_waiting_events == 0
    assert cfg.max_mcp_connect_waiting_events == 0
    assert cfg.max_mcp_waiting_events_total == 0


def test_parse_args_honors_explicit_artifact_paths(monkeypatch, tmp_path: Path) -> None:
    webhook_port = pick_free_port()
    telegram_port = pick_free_port()
    valkey_port = pick_free_port()
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "test_omni_agent_memory_ci_gate.py",
            "--profile",
            "nightly",
            "--webhook-port",
            str(webhook_port),
            "--telegram-api-port",
            str(telegram_port),
            "--valkey-port",
            str(valkey_port),
            "--runtime-log-file",
            "custom/runtime.log",
            "--mock-log-file",
            "custom/mock.log",
            "--evolution-report-json",
            "custom/evolution.json",
            "--benchmark-report-json",
            "custom/benchmark.json",
            "--session-matrix-report-json",
            "custom/matrix.json",
            "--session-matrix-report-markdown",
            "custom/matrix.md",
            "--trace-report-json",
            "custom/trace.json",
            "--trace-report-markdown",
            "custom/trace.md",
            "--cross-group-report-json",
            "custom/cross-group.json",
            "--cross-group-report-markdown",
            "custom/cross-group.md",
        ],
    )

    cfg = parse_args(tmp_path)

    assert cfg.runtime_log_file == (tmp_path / "custom/runtime.log").resolve()
    assert cfg.mock_log_file == (tmp_path / "custom/mock.log").resolve()
    assert cfg.evolution_report_json == (tmp_path / "custom/evolution.json").resolve()
    assert cfg.benchmark_report_json == (tmp_path / "custom/benchmark.json").resolve()
    assert cfg.session_matrix_report_json == (tmp_path / "custom/matrix.json").resolve()
    assert cfg.session_matrix_report_markdown == (tmp_path / "custom/matrix.md").resolve()
    assert cfg.trace_report_json == (tmp_path / "custom/trace.json").resolve()
    assert cfg.trace_report_markdown == (tmp_path / "custom/trace.md").resolve()
    assert cfg.cross_group_report_json == (tmp_path / "custom/cross-group.json").resolve()
    assert cfg.cross_group_report_markdown == (tmp_path / "custom/cross-group.md").resolve()


def test_parse_args_sets_skip_rust_regressions(monkeypatch, tmp_path: Path) -> None:
    webhook_port = pick_free_port()
    telegram_port = pick_free_port()
    valkey_port = pick_free_port()
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "test_omni_agent_memory_ci_gate.py",
            "--profile",
            "nightly",
            "--webhook-port",
            str(webhook_port),
            "--telegram-api-port",
            str(telegram_port),
            "--valkey-port",
            str(valkey_port),
            "--skip-rust-regressions",
        ],
    )
    cfg = parse_args(tmp_path)
    assert cfg.skip_rust_regressions is True


def test_parse_args_accepts_agent_bin(monkeypatch, tmp_path: Path) -> None:
    webhook_port = pick_free_port()
    telegram_port = pick_free_port()
    valkey_port = pick_free_port()
    agent_bin = tmp_path / "omni-agent"
    agent_bin.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "test_omni_agent_memory_ci_gate.py",
            "--profile",
            "quick",
            "--webhook-port",
            str(webhook_port),
            "--telegram-api-port",
            str(telegram_port),
            "--valkey-port",
            str(valkey_port),
            "--agent-bin",
            str(agent_bin),
        ],
    )
    cfg = parse_args(tmp_path)
    assert cfg.agent_bin == agent_bin.resolve()
