#!/usr/bin/env python3
"""Unit tests for command-events case catalog helpers."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path

import pytest

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

catalog_module = importlib.import_module("command_events_case_catalog")
models_module = importlib.import_module("command_events_models")


def test_build_cases_includes_memory_session_scope_placeholder() -> None:
    cases = catalog_module.build_cases(
        admin_user_id=1304799691,
        group_chat_id=-5101776367,
        group_thread_id=42,
        make_probe_case=models_module.ProbeCase,
        target_session_scope_placeholder="__target_session_scope__",
    )
    by_case = {case.case_id: case for case in cases}
    assert "session_memory_json" in by_case
    assert (
        "json_session_scope=__target_session_scope__" in by_case["session_memory_json"].extra_args
    )


def test_select_cases_raises_on_unknown_case() -> None:
    cases = (
        models_module.ProbeCase(
            case_id="session_status_json",
            prompt="/session json",
            event_name="telegram.command.session_status_json.replied",
            suites=("core",),
        ),
    )
    with pytest.raises(ValueError, match="Unknown case id"):
        catalog_module.select_cases(cases, ("all",), ("missing_case",))
