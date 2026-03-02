"""Chunked workflow helpers shared by skill scripts.

These utilities provide a stable action=start/batch/full_document workflow surface
without depending on deprecated graph-runtime adapters.
"""

from __future__ import annotations

import asyncio
from math import ceil
from typing import TYPE_CHECKING, Any

from omni.foundation.context_delivery.sessions import (
    ChunkedSessionStore,
    normalize_chunked_action_name,
)
from omni.foundation.context_delivery.strategies import ChunkedSession

if TYPE_CHECKING:
    from collections.abc import Awaitable, Callable


def build_chunked_action_error_payload(
    *,
    action: str,
    message: str,
    **extra: Any,
) -> dict[str, Any]:
    """Build a normalized chunked action error payload."""
    payload: dict[str, Any] = {
        "status": "error",
        "action": str(action or "").strip().lower(),
        "message": str(message or "").strip() or "chunked action failed",
    }
    if extra:
        payload.update(extra)
    return payload


def build_chunked_unavailable_payload(
    *,
    query: str = "",
    action: str = "",
    message: str,
    **extra: Any,
) -> dict[str, Any]:
    """Build a normalized unavailable payload for chunked workflows."""
    payload: dict[str, Any] = {
        "status": "unavailable",
        "action": str(action or "").strip().lower(),
        "query": str(query or ""),
        "message": str(message or "").strip() or "chunked workflow unavailable",
    }
    if extra:
        payload.update(extra)
    return payload


def build_chunked_dispatch_error_payload(
    *,
    action: str,
    dispatch_result: Any,
    fallback_message: str,
) -> dict[str, Any]:
    """Normalize unexpected dispatcher output into a user-facing error payload."""
    message = str(fallback_message or "chunked action dispatch failed")
    if isinstance(dispatch_result, dict):
        maybe = dispatch_result.get("message") or dispatch_result.get("error")
        if isinstance(maybe, str) and maybe.strip():
            message = maybe.strip()
    return build_chunked_action_error_payload(action=action, message=message)


def build_chunked_session_store_adapters(
    store: ChunkedSessionStore,
) -> tuple[
    Callable[[str], dict[str, Any] | None],
    Callable[[str], tuple[Any, dict[str, Any]] | None],
    Callable[[Any, dict[str, Any]], None],
]:
    """Expose state/session adapter callables for a ChunkedSessionStore."""

    def _load_state(session_id: str) -> dict[str, Any] | None:
        loaded = store.load(session_id)
        if loaded is None:
            return None
        _session, metadata = loaded
        return metadata if isinstance(metadata, dict) else {}

    def _load_session_state(session_id: str) -> tuple[Any, dict[str, Any]] | None:
        loaded = store.load(session_id)
        if loaded is None:
            return None
        session, metadata = loaded
        return session, metadata if isinstance(metadata, dict) else {}

    def _save_session_state(session: Any, state: dict[str, Any]) -> None:
        metadata = state if isinstance(state, dict) else {}
        if isinstance(session, ChunkedSession):
            store.save(session, metadata=metadata)
            return
        session_id = str(getattr(session, "session_id", "") or "").strip()
        if not session_id:
            raise ValueError("session must expose a non-empty session_id")
        placeholder = ChunkedSession(
            session_id=session_id,
            batches=[""],
            batch_size=1,
            total_chars=0,
        )
        store.save(placeholder, metadata=metadata)

    return _load_state, _load_session_state, _save_session_state


def persist_chunked_lazy_start_state(
    *,
    store: ChunkedSessionStore,
    session_id: str,
    state: dict[str, Any],
) -> None:
    """Persist chunked lazy-start state under a placeholder session."""
    placeholder = ChunkedSession(
        session_id=str(session_id or "").strip(),
        batches=[""],
        batch_size=1,
        total_chars=0,
    )
    store.save(placeholder, metadata=state if isinstance(state, dict) else {})


def create_chunked_lazy_start_payload(
    *,
    query: str,
    batch_size: int,
    max_items: int,
    preview_results: list[Any],
    status: str,
    state: dict[str, Any],
    persist_state: Callable[[str, dict[str, Any]], None],
    session_id_factory: Callable[[], str],
    action: str = "start",
) -> dict[str, Any]:
    """Persist initial state and return normalized action=start payload."""
    session_id = str(session_id_factory() or "").strip()
    if not session_id:
        raise ValueError("session_id_factory returned empty session id")
    persist_state(session_id, state)
    return {
        "status": str(status or "success"),
        "action": str(action or "start").strip().lower(),
        "query": str(query or ""),
        "session_id": session_id,
        "batch_size": int(batch_size),
        "max_chunks": int(max_items),
        "preview_results": preview_results if isinstance(preview_results, list) else [],
        "message": (
            "Call action=batch with session_id and batch_index=0..N to read all chunks "
            "without re-running preview."
        ),
    }


async def run_chunked_preview_action(
    *,
    query: str,
    run_preview: Callable[[], Awaitable[Any]],
    parse_preview_payload: Callable[[Any], dict[str, Any]],
    timeout_seconds: float,
    action: str = "preview",
    success_message: str = "",
    timeout_message: str = "",
) -> dict[str, Any]:
    """Run preview with timeout and normalize the payload."""
    try:
        out = await asyncio.wait_for(run_preview(), timeout=timeout_seconds)
        parsed = parse_preview_payload(out)
    except TimeoutError:
        return build_chunked_action_error_payload(
            action=action,
            message=timeout_message or "preview timed out",
            preview_results=[],
            query=query,
        )
    except Exception as exc:
        return build_chunked_action_error_payload(
            action=action,
            message=str(exc),
            preview_results=[],
            query=query,
        )

    preview_results = parsed.get("results")
    return {
        "status": str(parsed.get("status", "success")),
        "action": str(action or "preview").strip().lower(),
        "query": str(query or ""),
        "preview_results": preview_results if isinstance(preview_results, list) else [],
        "message": success_message
        or "Preview ready. Call action=start for session-based batching.",
    }


def _as_row_dict(entry: Any) -> dict[str, Any] | None:
    if isinstance(entry, dict):
        return entry
    if hasattr(entry, "__dict__"):
        data = {
            key: value
            for key, value in vars(entry).items()
            if isinstance(key, str) and not key.startswith("_")
        }
        return data if data else None
    return None


def _row_source_and_index(row: dict[str, Any]) -> tuple[str, int | None]:
    metadata = row.get("metadata")
    source = ""
    chunk_index: int | None = None
    if isinstance(metadata, dict):
        source = str(metadata.get("source") or row.get("source") or "")
        raw_index = metadata.get("chunk_index")
    else:
        source = str(row.get("source") or "")
        raw_index = row.get("chunk_index")
    try:
        chunk_index = int(raw_index) if raw_index is not None else None
    except (TypeError, ValueError):
        chunk_index = None
    return source, chunk_index


async def run_chunked_full_document_action(
    *,
    source: str,
    list_all_entries: Callable[[str], Awaitable[list[Any]] | list[Any]],
    batch_size: int,
    batch_index: int,
    action: str,
    batch_index_param: str,
    extra_payload_factory: Callable[[str], dict[str, Any] | None] | None = None,
) -> dict[str, Any]:
    """Load, deduplicate, and paginate all rows for one source suffix."""
    source_suffix = str(source or "").strip()
    if not source_suffix:
        return build_chunked_action_error_payload(
            action=action,
            message="source is required",
        )

    loaded = list_all_entries(source_suffix)
    rows_raw = await loaded if asyncio.iscoroutine(loaded) else loaded
    if not isinstance(rows_raw, list):
        rows_raw = []

    filtered_rows: list[dict[str, Any]] = []
    for entry in rows_raw:
        row = _as_row_dict(entry)
        if row is None:
            continue
        row_source, _chunk_index = _row_source_and_index(row)
        if row_source and (row_source == source_suffix or row_source.endswith(source_suffix)):
            filtered_rows.append(row)

    dedup_by_index: dict[int, dict[str, Any]] = {}
    passthrough_rows: list[dict[str, Any]] = []
    for row in filtered_rows:
        row_source, chunk_index = _row_source_and_index(row)
        content = row.get("content")
        normalized = {
            "content": content if isinstance(content, str) else str(content or ""),
            "source": row_source,
            "chunk_index": chunk_index,
        }
        if chunk_index is None:
            passthrough_rows.append(normalized)
            continue
        if chunk_index not in dedup_by_index:
            dedup_by_index[chunk_index] = normalized

    ordered_rows = [dedup_by_index[idx] for idx in sorted(dedup_by_index.keys())]
    ordered_rows.extend(passthrough_rows)

    total_count = len(ordered_rows)
    if batch_size <= 0:
        selected_rows = ordered_rows
        resolved_batch_count = 1 if total_count > 0 else 0
        resolved_batch_index = 0
    else:
        resolved_batch_count = ceil(total_count / batch_size) if total_count > 0 else 0
        if resolved_batch_count == 0:
            selected_rows = []
            resolved_batch_index = 0
        else:
            try:
                resolved_batch_index = int(batch_index)
            except (TypeError, ValueError):
                resolved_batch_index = -1
            if resolved_batch_index < 0 or resolved_batch_index >= resolved_batch_count:
                return build_chunked_action_error_payload(
                    action=action,
                    message=f"{batch_index_param} must be 0..{resolved_batch_count - 1}",
                    source=source_suffix,
                    batch_count=resolved_batch_count,
                    batch_index=resolved_batch_index,
                )
            begin = resolved_batch_index * batch_size
            end = begin + batch_size
            selected_rows = ordered_rows[begin:end]

    payload: dict[str, Any] = {
        "status": "success",
        "action": str(action or "full_document").strip().lower(),
        "source": source_suffix,
        "count": total_count,
        "batch_count": resolved_batch_count,
        "batch_index": resolved_batch_index,
        "results": selected_rows,
    }
    if extra_payload_factory is not None:
        extra = extra_payload_factory(source_suffix)
        if isinstance(extra, dict):
            payload.update(extra)
    return payload


async def run_chunked_lazy_start_batch_dispatch(
    *,
    action: str,
    session_id: str,
    batch_index: int,
    workflow_type: str,
    load_state: Callable[[str], dict[str, Any] | None] | None,
    on_start: Callable[[], Awaitable[dict[str, Any]]],
    load_session_state: Callable[[str], tuple[Any, dict[str, Any]] | None],
    save_session_state: Callable[[Any, dict[str, Any]], None],
    fetch_rows: Callable[[dict[str, Any]], Awaitable[list[Any]]],
    batch_action: str,
    batch_size_key: str,
    max_items_key: str,
    cache_ready_key: str,
    cache_rows_key: str,
    default_batch_size: int,
    default_max_items: int,
    missing_session_template: str,
    invalid_batch_template: str,
    fetch_timeout_message: str,
    session_required_error: str,
    session_missing_error: str,
) -> dict[str, Any]:
    """Dispatch start/batch actions for lazy chunked workflows."""
    normalized_action = normalize_chunked_action_name(action, action_aliases={"fetch": "batch"})
    if normalized_action == "start":
        return await on_start()

    if normalized_action != "batch":
        return {
            "success": False,
            "error": f"unsupported action={normalized_action} for workflow={workflow_type}",
        }

    normalized_session_id = str(session_id or "").strip()
    if not normalized_session_id:
        return build_chunked_action_error_payload(
            action=batch_action,
            message=session_required_error,
        )

    loaded = load_session_state(normalized_session_id)
    if loaded is None:
        return build_chunked_action_error_payload(
            action=batch_action,
            message=session_missing_error.format(session_id=normalized_session_id),
            session_id=normalized_session_id,
        )
    session, state = loaded
    if not isinstance(state, dict):
        state = {}

    if not bool(state.get(cache_ready_key)):
        fetched_rows: list[Any]
        try:
            fetched_rows = await fetch_rows(state)
        except TimeoutError:
            return build_chunked_action_error_payload(
                action=batch_action,
                message=fetch_timeout_message,
                session_id=normalized_session_id,
            )
        except Exception as exc:
            return build_chunked_action_error_payload(
                action=batch_action,
                message=str(exc),
                session_id=normalized_session_id,
            )

        max_items = state.get(max_items_key, default_max_items)
        try:
            max_items_int = max(1, int(max_items))
        except (TypeError, ValueError):
            max_items_int = max(1, int(default_max_items))
        rows_list = list(fetched_rows)[:max_items_int]
        state[cache_rows_key] = rows_list
        state[cache_ready_key] = True
        save_session_state(session, state)

    rows = state.get(cache_rows_key)
    rows_list = list(rows) if isinstance(rows, list) else []

    batch_size_val = state.get(batch_size_key, default_batch_size)
    try:
        batch_size_int = max(1, int(batch_size_val))
    except (TypeError, ValueError):
        batch_size_int = max(1, int(default_batch_size))

    batch_count = ceil(len(rows_list) / batch_size_int) if rows_list else 1
    try:
        resolved_batch_index = int(batch_index)
    except (TypeError, ValueError):
        resolved_batch_index = -1

    if resolved_batch_index < 0 or resolved_batch_index >= batch_count:
        return build_chunked_action_error_payload(
            action=batch_action,
            message=invalid_batch_template.format(max_index=batch_count - 1),
            session_id=normalized_session_id,
            batch_count=batch_count,
            batch_index=resolved_batch_index,
        )

    begin = resolved_batch_index * batch_size_int
    end = begin + batch_size_int
    return {
        "status": "success",
        "action": batch_action,
        "session_id": normalized_session_id,
        "batch_index": resolved_batch_index,
        "batch_count": batch_count,
        "batch": rows_list[begin:end],
        "preview_results": state.get("preview_results", []),
    }


async def run_chunked_auto_complete(
    workflow_name: str,
    run_start: Callable[[], Any | Awaitable[Any]],
    run_step: Callable[[dict[str, Any]], dict[str, Any] | Awaitable[dict[str, Any]]],
    run_synthesize: Callable[[dict[str, Any]], dict[str, Any] | Awaitable[dict[str, Any]]],
    *,
    queue_key: str = "queue",
    max_steps: int = 512,
) -> dict[str, Any]:
    """Run start -> step loop -> synthesize for one-shot chunked workflows."""
    try:
        state = run_start()
        if asyncio.iscoroutine(state):
            state = await state
        if not isinstance(state, dict):
            return {"success": False, "error": f"{workflow_name}: run_start must return dict"}

        step_count = 0
        while True:
            queue = state.get(queue_key)
            if not isinstance(queue, list) or not queue:
                break
            if step_count >= max_steps:
                return {
                    "success": False,
                    "error": f"{workflow_name}: exceeded max_steps={max_steps}",
                }
            state = run_step(state)
            if asyncio.iscoroutine(state):
                state = await state
            if not isinstance(state, dict):
                return {"success": False, "error": f"{workflow_name}: run_step must return dict"}
            step_count += 1

        final_state = run_synthesize(state)
        if asyncio.iscoroutine(final_state):
            final_state = await final_state
        if not isinstance(final_state, dict):
            return {"success": False, "error": f"{workflow_name}: run_synthesize must return dict"}
        return {"success": True, "result": final_state.get("final_report", final_state)}
    except Exception as exc:
        return {"success": False, "error": str(exc)}


__all__ = [
    "build_chunked_action_error_payload",
    "build_chunked_dispatch_error_payload",
    "build_chunked_session_store_adapters",
    "build_chunked_unavailable_payload",
    "create_chunked_lazy_start_payload",
    "persist_chunked_lazy_start_state",
    "run_chunked_auto_complete",
    "run_chunked_full_document_action",
    "run_chunked_lazy_start_batch_dispatch",
    "run_chunked_preview_action",
]
