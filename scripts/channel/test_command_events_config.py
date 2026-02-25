#!/usr/bin/env python3
"""Unit tests for command-events config helpers."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

module = importlib.import_module("command_events_config")


def test_parse_args_supports_suite_filter(monkeypatch) -> None:
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "command_events_config.py",
            "--suite",
            "core",
            "--case",
            "session_status_json",
            "--max-wait",
            "33",
        ],
    )
    args = module.parse_args(suites=("core", "control", "admin", "all"))
    assert args.suite == ["core"]
    assert args.case == ["session_status_json"]
    assert args.max_wait == 33


def test_parse_args_defaults_output_paths(monkeypatch) -> None:
    monkeypatch.setattr(sys, "argv", ["command_events_config.py"])
    args = module.parse_args(suites=("core", "control", "admin", "all"))
    assert args.output_json.endswith("agent-channel-command-events.json")
    assert args.output_markdown.endswith("agent-channel-command-events.md")
