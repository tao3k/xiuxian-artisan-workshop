"""Workflow state persistence for multi-step skill workflows.

This module replaces the removed legacy checkpoint backend with a file-based
runtime store under ``PRJ_RUNTIME_DIR``.
"""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass
from datetime import UTC, datetime
from typing import Any

from omni.foundation.config.prj import get_runtime_dir

_SCHEMA_VERSION = "xiuxian_qianji.workflow_state.v1"
_STATE_ROOT_SUBDIR = "xiuxian_qianji/workflow_state"
_SAFE_COMPONENT_RE = re.compile(r"[^A-Za-z0-9_.-]+")


def _now_iso() -> str:
    return datetime.now(UTC).isoformat()


def _normalize_component(value: str, *, fallback: str) -> str:
    raw = str(value or "").strip().lower()
    if not raw:
        return fallback
    normalized = _SAFE_COMPONENT_RE.sub("_", raw).strip("._-")
    return normalized or fallback


def _workflow_root(workflow_type: str):
    root = get_runtime_dir(_STATE_ROOT_SUBDIR)
    workflow_key = _normalize_component(workflow_type, fallback="default")
    path = root / workflow_key
    path.mkdir(parents=True, exist_ok=True)
    return path


def _workflow_id_digest(workflow_id: str) -> str:
    return hashlib.sha256(str(workflow_id or "").encode("utf-8")).hexdigest()


def _record_path(workflow_type: str, workflow_id: str):
    return _workflow_root(workflow_type) / f"{_workflow_id_digest(workflow_id)}.json"


def _history_path(workflow_type: str, workflow_id: str):
    return _workflow_root(workflow_type) / f"{_workflow_id_digest(workflow_id)}.history.jsonl"


def _normalize_workflow_id(workflow_id: str) -> str:
    return str(workflow_id or "").strip()


def _write_json_atomic(path, payload: dict[str, Any]) -> None:
    temp_path = path.with_suffix(path.suffix + ".tmp")
    temp_path.write_text(
        json.dumps(payload, ensure_ascii=False, separators=(",", ":"), sort_keys=True),
        encoding="utf-8",
    )
    temp_path.replace(path)


def _build_record(
    workflow_type: str,
    workflow_id: str,
    state: dict[str, Any],
    *,
    metadata: dict[str, Any] | None = None,
) -> dict[str, Any]:
    return {
        "schema_version": _SCHEMA_VERSION,
        "workflow_type": str(workflow_type or "").strip(),
        "workflow_id": str(workflow_id or "").strip(),
        "updated_at": _now_iso(),
        "metadata": dict(metadata or {}),
        "state": dict(state),
    }


def _append_history(record: dict[str, Any]) -> None:
    workflow_type = str(record.get("workflow_type", "")).strip()
    workflow_id = str(record.get("workflow_id", "")).strip()
    if not workflow_type or not workflow_id:
        return
    line = json.dumps(record, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    history_file = _history_path(workflow_type, workflow_id)
    with history_file.open("a", encoding="utf-8") as handle:
        handle.write(line)
        handle.write("\n")


def save_workflow_state(
    workflow_type: str,
    workflow_id: str,
    state: dict[str, Any],
    *,
    metadata: dict[str, Any] | None = None,
) -> bool:
    """Persist workflow state for one workflow_id."""
    workflow_id = _normalize_workflow_id(workflow_id)
    if not workflow_id or not isinstance(state, dict):
        return False
    record = _build_record(
        workflow_type=workflow_type,
        workflow_id=workflow_id,
        state=state,
        metadata=metadata,
    )
    path = _record_path(workflow_type, workflow_id)
    _write_json_atomic(path, record)
    _append_history(record)
    return True


def load_workflow_state(workflow_type: str, workflow_id: str) -> dict[str, Any] | None:
    """Load persisted state for one workflow_id."""
    workflow_id = _normalize_workflow_id(workflow_id)
    if not workflow_id:
        return None
    path = _record_path(workflow_type, workflow_id)
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    state = payload.get("state")
    return state if isinstance(state, dict) else None


def delete_workflow_state(workflow_type: str, workflow_id: str) -> bool:
    """Delete persisted state for one workflow_id."""
    workflow_id = _normalize_workflow_id(workflow_id)
    if not workflow_id:
        return False
    removed = False
    for path in (
        _record_path(workflow_type, workflow_id),
        _history_path(workflow_type, workflow_id),
    ):
        if path.exists():
            path.unlink()
            removed = True
    return removed


def get_workflow_history(
    workflow_type: str,
    workflow_id: str,
    *,
    limit: int = 20,
) -> list[dict[str, Any]]:
    """Return workflow history entries (most recent first)."""
    workflow_id = _normalize_workflow_id(workflow_id)
    if not workflow_id:
        return []
    history_file = _history_path(workflow_type, workflow_id)
    if not history_file.exists():
        return []
    try:
        lines = history_file.read_text(encoding="utf-8").splitlines()
    except OSError:
        return []
    records: list[dict[str, Any]] = []
    for line in reversed(lines):
        if len(records) >= max(0, int(limit)):
            break
        try:
            parsed = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(parsed, dict):
            records.append(parsed)
    return records


def save_workflow_state_sqlite(
    workflow_type: str,
    workflow_id: str,
    state: dict[str, Any],
    *,
    metadata: dict[str, Any] | None = None,
) -> bool:
    """Compatibility alias to current workflow-state backend."""
    return save_workflow_state(
        workflow_type=workflow_type,
        workflow_id=workflow_id,
        state=state,
        metadata=metadata,
    )


def load_workflow_state_sqlite(workflow_type: str, workflow_id: str) -> dict[str, Any] | None:
    """Compatibility alias to current workflow-state backend."""
    return load_workflow_state(workflow_type=workflow_type, workflow_id=workflow_id)


@dataclass(slots=True, frozen=True)
class WorkflowStateHandle:
    """Lightweight handle for workflow state operations."""

    workflow_type: str

    def save_checkpoint(
        self,
        table_name: str,
        thread_id: str,
        checkpoint_json: str,
        *,
        metadata: dict[str, Any] | None = None,
    ) -> bool:
        try:
            state = json.loads(checkpoint_json)
        except json.JSONDecodeError:
            return False
        if not isinstance(state, dict):
            return False
        workflow_type = str(table_name or self.workflow_type).strip() or self.workflow_type
        return save_workflow_state(workflow_type, thread_id, state, metadata=metadata)

    def get_latest(self, table_name: str, thread_id: str) -> str | None:
        workflow_type = str(table_name or self.workflow_type).strip() or self.workflow_type
        state = load_workflow_state(workflow_type, thread_id)
        if not isinstance(state, dict):
            return None
        return json.dumps(state, ensure_ascii=False, separators=(",", ":"), sort_keys=True)

    def get_history(self, table_name: str, thread_id: str, limit: int = 20) -> list[str]:
        workflow_type = str(table_name or self.workflow_type).strip() or self.workflow_type
        history = get_workflow_history(workflow_type, thread_id, limit=limit)
        return [
            json.dumps(entry, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
            for entry in history
        ]

    def delete_thread(self, table_name: str, thread_id: str) -> bool:
        workflow_type = str(table_name or self.workflow_type).strip() or self.workflow_type
        return delete_workflow_state(workflow_type, thread_id)


def get_checkpointer(workflow_type: str) -> WorkflowStateHandle:
    """Return a workflow-state handle for one workflow type."""
    return WorkflowStateHandle(workflow_type=str(workflow_type or "").strip() or "default")


__all__ = [
    "WorkflowStateHandle",
    "delete_workflow_state",
    "get_checkpointer",
    "get_workflow_history",
    "load_workflow_state",
    "load_workflow_state_sqlite",
    "save_workflow_state",
    "save_workflow_state_sqlite",
]
