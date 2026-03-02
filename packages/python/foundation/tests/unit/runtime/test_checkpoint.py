"""Tests for workflow_state.py (Qianji workflow persistence)."""

from __future__ import annotations

import importlib
import json

import pytest

DEFAULT_WORKFLOW_TYPE = "unit_workflow"
DEFAULT_WORKFLOW_ID = "session_001"
DEFAULT_STATE = {"step": 1, "payload": {"name": "qianji"}}
STATE_UPDATED = {"step": 2, "payload": {"name": "qianji", "status": "done"}}
DEFAULT_METADATA = {"source": "unit-test", "channel": "runtime"}


@pytest.fixture(autouse=True)
def isolated_workflow_state_runtime(
    tmp_path,
    monkeypatch: pytest.MonkeyPatch,
):
    """Route workflow-state IO to a per-test runtime directory."""
    runtime_root = tmp_path / ".run"
    runtime_root.mkdir(parents=True, exist_ok=True)

    monkeypatch.setattr(
        "omni.foundation.workflow_state.get_runtime_dir",
        lambda *parts: runtime_root.joinpath(*parts),
    )
    return runtime_root


def test_legacy_checkpoint_module_is_removed() -> None:
    """Legacy checkpoint compatibility layer must stay removed."""
    with pytest.raises(ModuleNotFoundError):
        importlib.import_module("omni.foundation.checkpoint")


def test_save_and_load_workflow_state_roundtrip() -> None:
    """save_workflow_state/load_workflow_state should roundtrip one record."""
    from omni.foundation.workflow_state import load_workflow_state, save_workflow_state

    assert save_workflow_state(DEFAULT_WORKFLOW_TYPE, DEFAULT_WORKFLOW_ID, DEFAULT_STATE) is True
    assert load_workflow_state(DEFAULT_WORKFLOW_TYPE, DEFAULT_WORKFLOW_ID) == DEFAULT_STATE


def test_save_rejects_invalid_payload_and_blank_id() -> None:
    """Invalid payloads should not be persisted."""
    from omni.foundation.workflow_state import save_workflow_state

    assert save_workflow_state(DEFAULT_WORKFLOW_TYPE, "", DEFAULT_STATE) is False
    assert save_workflow_state(DEFAULT_WORKFLOW_TYPE, DEFAULT_WORKFLOW_ID, "bad") is False


def test_load_missing_workflow_returns_none() -> None:
    """Loading missing workflow state should return None."""
    from omni.foundation.workflow_state import load_workflow_state

    assert load_workflow_state(DEFAULT_WORKFLOW_TYPE, "missing-id") is None


def test_get_workflow_history_returns_latest_first_with_metadata() -> None:
    """History should include metadata and be ordered from latest to oldest."""
    from omni.foundation.workflow_state import get_workflow_history, save_workflow_state

    assert save_workflow_state(
        DEFAULT_WORKFLOW_TYPE,
        DEFAULT_WORKFLOW_ID,
        DEFAULT_STATE,
        metadata=DEFAULT_METADATA,
    )
    assert save_workflow_state(
        DEFAULT_WORKFLOW_TYPE,
        DEFAULT_WORKFLOW_ID,
        STATE_UPDATED,
        metadata={"source": "unit-test", "channel": "runtime-updated"},
    )

    history = get_workflow_history(DEFAULT_WORKFLOW_TYPE, DEFAULT_WORKFLOW_ID, limit=5)
    assert len(history) == 2
    assert history[0]["state"] == STATE_UPDATED
    assert history[0]["metadata"]["channel"] == "runtime-updated"
    assert history[1]["state"] == DEFAULT_STATE


def test_delete_workflow_state_removes_record_and_history() -> None:
    """Delete should remove both latest record and history file."""
    from omni.foundation.workflow_state import (
        delete_workflow_state,
        get_workflow_history,
        load_workflow_state,
        save_workflow_state,
    )

    assert save_workflow_state(DEFAULT_WORKFLOW_TYPE, DEFAULT_WORKFLOW_ID, DEFAULT_STATE)
    assert save_workflow_state(DEFAULT_WORKFLOW_TYPE, DEFAULT_WORKFLOW_ID, STATE_UPDATED)

    assert delete_workflow_state(DEFAULT_WORKFLOW_TYPE, DEFAULT_WORKFLOW_ID) is True
    assert load_workflow_state(DEFAULT_WORKFLOW_TYPE, DEFAULT_WORKFLOW_ID) is None
    assert get_workflow_history(DEFAULT_WORKFLOW_TYPE, DEFAULT_WORKFLOW_ID, limit=5) == []
    assert delete_workflow_state(DEFAULT_WORKFLOW_TYPE, DEFAULT_WORKFLOW_ID) is False


def test_get_checkpointer_returns_workflow_state_handle() -> None:
    """get_checkpointer should return a workflow-state handle bound to workflow type."""
    from omni.foundation.workflow_state import WorkflowStateHandle, get_checkpointer

    handle = get_checkpointer(DEFAULT_WORKFLOW_TYPE)
    assert isinstance(handle, WorkflowStateHandle)
    assert handle.workflow_type == DEFAULT_WORKFLOW_TYPE


def test_workflow_state_handle_save_get_history_and_delete() -> None:
    """WorkflowStateHandle should support save/get_latest/get_history/delete_thread."""
    from omni.foundation.workflow_state import get_checkpointer

    handle = get_checkpointer(DEFAULT_WORKFLOW_TYPE)
    table_name = DEFAULT_WORKFLOW_TYPE
    thread_id = DEFAULT_WORKFLOW_ID

    assert (
        handle.save_checkpoint(
            table_name,
            thread_id,
            json.dumps(DEFAULT_STATE),
            metadata=DEFAULT_METADATA,
        )
        is True
    )
    assert handle.save_checkpoint(table_name, thread_id, json.dumps(STATE_UPDATED)) is True
    assert handle.save_checkpoint(table_name, thread_id, "{not-json") is False

    latest_raw = handle.get_latest(table_name, thread_id)
    assert latest_raw is not None
    assert json.loads(latest_raw) == STATE_UPDATED

    history_raw = handle.get_history(table_name, thread_id, limit=5)
    assert len(history_raw) == 2
    assert json.loads(history_raw[0])["state"] == STATE_UPDATED
    assert json.loads(history_raw[1])["state"] == DEFAULT_STATE

    assert handle.delete_thread(table_name, thread_id) is True
    assert handle.get_latest(table_name, thread_id) is None


def test_foundation_lazy_exports_route_to_workflow_state() -> None:
    """Foundation package exports should point to workflow_state implementation."""
    from omni import foundation

    assert foundation.save_workflow_state(DEFAULT_WORKFLOW_TYPE, DEFAULT_WORKFLOW_ID, DEFAULT_STATE)
    assert (
        foundation.load_workflow_state(DEFAULT_WORKFLOW_TYPE, DEFAULT_WORKFLOW_ID) == DEFAULT_STATE
    )
    assert foundation.delete_workflow_state(DEFAULT_WORKFLOW_TYPE, DEFAULT_WORKFLOW_ID) is True
