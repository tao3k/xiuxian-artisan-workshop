#!/usr/bin/env python3
"""Unit tests for memory CI gate quality helpers."""

from __future__ import annotations

import importlib
import json
import sys
from pathlib import Path

import pytest

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

_quality_module = importlib.import_module("memory_ci_gate_quality")
load_json = _quality_module.load_json
safe_int = _quality_module.safe_int


def test_safe_int_returns_default_for_invalid_values() -> None:
    assert safe_int("abc", default=7) == 7
    assert safe_int(None, default=9) == 9


def test_load_json_reads_object_payload(tmp_path: Path) -> None:
    payload_path = tmp_path / "report.json"
    payload_path.write_text(json.dumps({"ok": True}), encoding="utf-8")
    assert load_json(payload_path) == {"ok": True}


def test_load_json_rejects_non_object_payload(tmp_path: Path) -> None:
    payload_path = tmp_path / "report.json"
    payload_path.write_text(json.dumps(["not", "an", "object"]), encoding="utf-8")
    with pytest.raises(RuntimeError, match="invalid report payload"):
        load_json(payload_path)
