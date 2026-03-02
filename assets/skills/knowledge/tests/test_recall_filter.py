"""Unit tests for recall TOC filtering and min_score (no vector store)."""

import json
import sys
import types
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from _module_loader import load_script_module

import omni.rag.retrieval.executor as retrieval_executor
from omni.foundation.runtime.skill_optimization import is_markdown_index_chunk
from omni.rag.retrieval import filter_recall_rows

# Force skill-local recall module to avoid cross-skill `recall` collisions.
recall = load_script_module("recall", alias="knowledge_recall_test")


@pytest.fixture(autouse=True)
def _clear_recall_single_call_cache() -> None:
    """Keep recall single-call cache isolated across tests."""
    recall.clear_recall_single_call_cache()
    yield
    recall.clear_recall_single_call_cache()


def test_is_toc_or_index_chunk_empty_or_short():
    """Short or empty content is not TOC."""
    assert is_markdown_index_chunk("") is False
    assert is_markdown_index_chunk("short") is False
    assert is_markdown_index_chunk("x" * 50) is False


def test_is_toc_or_index_chunk_doc_description_table():
    """Table with | Document | and | Description | and 3+ rows is TOC."""
    toc = """
| Document | Description |
| -------- | ----------- |
| [A](./a.md) | First doc |
| [B](./b.md) | Second doc |
"""
    assert is_markdown_index_chunk(toc) is True


def test_is_toc_or_index_chunk_many_table_rows_with_links():
    """Many table rows (>=8) with markdown links is TOC-like."""
    lines = ["| [Link](./x.md) |"] * 10
    content = "\n".join(lines)
    assert len(content) >= 80
    assert is_markdown_index_chunk(content) is True


def test_is_toc_or_index_chunk_substantive_section():
    """Substantive section content is not TOC."""
    section = """
## Git commit format

Use Conventional Commits. Scope examples: feat(router), fix(omni-vector).
Run `lefthook run pre-commit` before committing.
"""
    assert is_markdown_index_chunk(section) is False


def test_filter_and_rank_recall_respects_min_score():
    """Results below min_score are dropped."""
    results = [
        {"content": "high", "score": 0.9, "source": "a"},
        {"content": "low", "score": 0.2, "source": "b"},
        {"content": "mid", "score": 0.6, "source": "c"},
    ]
    out = filter_recall_rows(
        results,
        limit=5,
        min_score=0.5,
        index_detector=is_markdown_index_chunk,
    )
    assert len(out) == 2
    assert out[0]["score"] == 0.9
    assert out[1]["score"] == 0.6


def test_filter_and_rank_recall_demotes_toc_then_fills():
    """TOC-like chunks are demoted; substantive chunks fill limit first."""
    toc_chunk = (
        "| Document | Description |\n| -------- | ----------- |\n| [A](./a.md) | Desc |\n" * 2
    )
    results = [
        {"content": "real section about git commits", "score": 0.7, "source": "doc"},
        {"content": toc_chunk, "score": 0.8, "source": "index"},
        {"content": "another real section", "score": 0.65, "source": "ref"},
    ]
    out = filter_recall_rows(
        results,
        limit=3,
        min_score=0.0,
        index_detector=is_markdown_index_chunk,
    )
    assert len(out) == 3
    assert out[0]["content"].startswith("real section")
    assert out[1]["content"].startswith("another real")
    assert out[2]["source"] == "index"


def test_filter_and_rank_recall_limit():
    """Return at most `limit` results."""
    results = [
        {"content": f"chunk {i}", "score": 0.9 - i * 0.1, "source": f"s{i}"} for i in range(5)
    ]
    out = filter_recall_rows(
        results,
        limit=2,
        min_score=0.0,
        index_detector=is_markdown_index_chunk,
    )
    assert len(out) == 2


# -----------------------------------------------------------------------------
# Action-based recall (preview / fetch / batch) — lightweight, no vector store
# -----------------------------------------------------------------------------


def _parse_recall_out(out):
    """Handle both raw JSON string and skill_command wrapper {content: [{text: json_str}]}."""
    if isinstance(out, str):
        return json.loads(out)
    if isinstance(out, dict) and "content" in out:
        parts = out.get("content") or []
        if parts and isinstance(parts[0], dict) and "text" in parts[0]:
            return json.loads(parts[0]["text"])
    return out if isinstance(out, dict) else json.loads(out)


@pytest.mark.asyncio
async def test_recall_action_batch_missing_session_id_returns_error():
    """action=batch without session_id returns error JSON; no vector store or embedding."""
    mock_store = MagicMock()
    mock_store.store = True  # pass "vector store initialized" check
    with patch.object(recall, "get_vector_store", return_value=mock_store):
        out = await recall.recall(
            query="",  # not used for batch
            chunked=True,
            action="batch",
            session_id="",
            batch_index=0,
        )
    data = _parse_recall_out(out)
    assert data.get("action") == "batch"
    assert data.get("status") == "error"
    assert "session_id" in (data.get("message") or "").lower()


@pytest.mark.asyncio
async def test_recall_action_batch_unknown_session_id_returns_error():
    """action=batch with unknown session_id returns session_id not found."""
    mock_store = MagicMock()
    mock_store.store = True
    with (
        patch.object(recall, "get_vector_store", return_value=mock_store),
        patch.object(recall._RECALL_CHUNKED_STORE, "load", return_value=None),
    ):
        out = await recall.recall(
            query="",
            chunked=True,
            action="batch",
            session_id="unknown-uuid",
            batch_index=0,
        )
    data = _parse_recall_out(out)
    assert data.get("action") == "batch"
    assert data.get("status") == "error"
    assert "not found" in (data.get("message") or "").lower()


@pytest.mark.asyncio
async def test_recall_action_batch_reuses_cached_results_across_calls():
    """Second batch call should reuse cached rows from checkpoint and skip re-query."""
    mock_vector = MagicMock()
    mock_vector.get_store_for_collection.return_value = object()
    mock_vector.search = AsyncMock(
        return_value=[
            types.SimpleNamespace(
                id=f"id-{i}",
                content=f"chunk-{i}",
                distance=0.1,
                metadata={"source": f"doc-{i}.md"},
            )
            for i in range(4)
        ]
    )

    _state_store: dict[str, dict] = {}
    _session_store: dict[str, object] = {}

    def _save(session: object, metadata: dict | None = None) -> None:
        session_id = getattr(session, "session_id", "")
        assert isinstance(session_id, str) and session_id
        _session_store[session_id] = session
        _state_store[session_id] = json.loads(json.dumps(metadata or {}))

    def _load(workflow_id: str) -> tuple[object, dict] | None:
        state = _state_store.get(workflow_id)
        session = _session_store.get(workflow_id)
        if state is None or session is None:
            return None
        return session, json.loads(json.dumps(state))

    with (
        patch.object(recall, "get_vector_store", return_value=mock_vector),
        patch.object(recall._RECALL_CHUNKED_STORE, "save", side_effect=_save),
        patch.object(recall._RECALL_CHUNKED_STORE, "load", side_effect=_load),
    ):
        start_out = await recall.recall(
            query="cache test",
            chunked=True,
            action="start",
            limit=4,
            preview_limit=2,
            batch_size=2,
            max_chunks=4,
        )
        start_data = _parse_recall_out(start_out)
        sid = start_data["session_id"]

        batch0 = _parse_recall_out(
            await recall.recall(
                query="",
                chunked=True,
                action="batch",
                session_id=sid,
                batch_index=0,
            )
        )
        batch1 = _parse_recall_out(
            await recall.recall(
                query="",
                chunked=True,
                action="batch",
                session_id=sid,
                batch_index=1,
            )
        )

    assert batch0["status"] == "success"
    assert batch1["status"] == "success"
    assert [r["content"] for r in batch0["batch"]] == ["chunk-0", "chunk-1"]
    assert [r["content"] for r in batch1["batch"]] == ["chunk-2", "chunk-3"]
    # query once for start preview(limit=2) + once for first batch fetch(limit=4), no extra query on batch1.
    assert mock_vector.search.await_count == 2
    assert mock_vector.search.await_args_list[0].args[1] == 2
    assert mock_vector.search.await_args_list[1].args[1] == 4


@pytest.mark.asyncio
async def test_recall_action_start_batch_emits_retrieval_row_budget_phases() -> None:
    """start/batch flow should emit retrieval row-budget phases for preview+first fetch only."""
    mock_vector = MagicMock()
    mock_vector.get_store_for_collection.return_value = object()
    mock_vector.search = AsyncMock(
        return_value=[
            types.SimpleNamespace(
                id=f"id-{i}",
                content=f"chunk-{i}",
                distance=0.1,
                metadata={"source": f"doc-{i}.md"},
            )
            for i in range(4)
        ]
    )

    _state_store: dict[str, dict] = {}
    _session_store: dict[str, object] = {}

    def _save(session: object, metadata: dict | None = None) -> None:
        session_id = getattr(session, "session_id", "")
        assert isinstance(session_id, str) and session_id
        _session_store[session_id] = session
        _state_store[session_id] = json.loads(json.dumps(metadata or {}))

    def _load(workflow_id: str) -> tuple[object, dict] | None:
        state = _state_store.get(workflow_id)
        session = _session_store.get(workflow_id)
        if state is None or session is None:
            return None
        return session, json.loads(json.dumps(state))

    phases: list[tuple[str, dict[str, object]]] = []

    def _capture_phase(
        phase: str,
        _started_at: float,
        _rss_before: float | None,
        _rss_peak_before: float | None,
        **extra: object,
    ) -> None:
        phases.append((phase, dict(extra)))

    with (
        patch.object(recall, "get_vector_store", return_value=mock_vector),
        patch.object(recall._RECALL_CHUNKED_STORE, "save", side_effect=_save),
        patch.object(recall._RECALL_CHUNKED_STORE, "load", side_effect=_load),
        patch.object(retrieval_executor, "record_phase_with_memory", side_effect=_capture_phase),
    ):
        start_out = await recall.recall(
            query="cache test",
            chunked=True,
            action="start",
            limit=4,
            preview_limit=2,
            batch_size=2,
            max_chunks=4,
        )
        start_data = _parse_recall_out(start_out)
        sid = start_data["session_id"]

        batch0 = _parse_recall_out(
            await recall.recall(
                query="",
                chunked=True,
                action="batch",
                session_id=sid,
                batch_index=0,
            )
        )
        batch1 = _parse_recall_out(
            await recall.recall(
                query="",
                chunked=True,
                action="batch",
                session_id=sid,
                batch_index=1,
            )
        )

    assert batch0["status"] == "success"
    assert batch1["status"] == "success"
    phase_names = [name for name, _ in phases]
    assert phase_names.count("retrieval.rows.semantic") == 2
    assert phase_names.count("retrieval.rows.query") == 2

    semantic_limits = [
        int(extra.get("fetch_limit", -1))
        for name, extra in phases
        if name == "retrieval.rows.semantic"
    ]
    assert sorted(semantic_limits) == [2, 4]


@pytest.mark.asyncio
async def test_recall_action_preview_fetch_requires_query():
    """action=preview or fetch with empty query returns error; no vector search."""
    mock_store = MagicMock()
    mock_store.store = True
    with patch.object(recall, "get_vector_store", return_value=mock_store):
        for act in ("preview", "fetch"):
            out = await recall.recall(
                query="",
                chunked=True,
                action=act,
            )
            data = _parse_recall_out(out)
            assert data.get("action") == act
            assert data.get("status") == "error"
            assert "query required" in (data.get("message") or "").lower()


@pytest.mark.asyncio
async def test_recall_one_shot_chunked_caps_by_limit():
    """Chunked one-shot should use `limit` as hard cap for preview/max chunks."""
    mock_vector = MagicMock()
    mock_vector.get_store_for_collection.return_value = object()
    captured: dict[str, object] = {}

    async def _fake_run_chunked_recall(**kwargs):
        captured.update(kwargs)
        return {
            "query": kwargs.get("query", ""),
            "status": "success",
            "error": None,
            "preview_results": [],
            "batches": [],
            "all_chunks_count": 0,
            "results": [],
        }

    with (
        patch.object(recall, "get_vector_store", return_value=mock_vector),
        patch.object(recall, "_get_run_chunked_recall", return_value=_fake_run_chunked_recall),
    ):
        out = await recall.recall(
            query="x",
            chunked=True,
            limit=3,
            preview_limit=10,
            batch_size=5,
            max_chunks=15,
        )

    data = _parse_recall_out(out)
    assert data.get("status") == "success"
    assert captured["preview_limit"] == 3
    assert captured["max_chunks"] == 3
    assert captured["batch_size"] == 3


@pytest.mark.asyncio
async def test_recall_single_call_db_query_uses_limit_exactly():
    """Single-call semantic recall should pass exact `limit` to vector search."""
    mock_vector = MagicMock()
    mock_vector.get_store_for_collection.return_value = object()
    mock_vector.search = AsyncMock(return_value=[])

    with patch.object(recall, "get_vector_store", return_value=mock_vector):
        out = await recall.recall(
            query="search algorithm",
            chunked=False,
            limit=3,
            preview=False,
        )

    data = _parse_recall_out(out)
    assert data.get("status") == "success"
    mock_vector.search.assert_awaited_once()
    args = mock_vector.search.await_args.args
    assert args[0] == "search algorithm"
    assert args[1] == 3


@pytest.mark.asyncio
async def test_recall_single_call_reuses_cached_payload_for_identical_requests() -> None:
    """Identical non-chunked recall calls should hit run_recall_single_call once within TTL."""
    mock_vector = MagicMock()
    mock_vector.get_store_for_collection.return_value = object()

    call_counter = {"count": 0}

    async def _fake_single_call(**kwargs):
        call_counter["count"] += 1
        return {
            "success": True,
            "query": kwargs.get("query", ""),
            "found": 1,
            "status": "success",
            "results": [
                {
                    "content": "cached-result",
                    "source": "docs/cache.md",
                    "score": 0.9,
                    "title": "",
                    "section": "",
                }
            ],
        }

    with (
        patch.object(recall, "get_vector_store", return_value=mock_vector),
        patch.object(recall, "run_recall_single_call", side_effect=_fake_single_call),
    ):
        out1 = await recall.recall(
            query="cache hit",
            chunked=False,
            limit=3,
            preview=False,
            retrieval_mode="vector_only",
        )
        out2 = await recall.recall(
            query="cache hit",
            chunked=False,
            limit=3,
            preview=False,
            retrieval_mode="vector_only",
        )

    assert call_counter["count"] == 1
    data1 = _parse_recall_out(out1)
    data2 = _parse_recall_out(out2)
    assert data1.get("status") == "success"
    assert data2 == data1


@pytest.mark.asyncio
async def test_recall_hybrid_mode_uses_graph_path_when_policy_selects_graph_only(
    monkeypatch: pytest.MonkeyPatch,
):
    """When policy chooses graph_only, recall should return graph rows without vector search."""
    mock_vector = MagicMock()
    mock_vector.get_store_for_collection.return_value = object()
    mock_vector.search = AsyncMock(return_value=[])

    async def _fake_evaluate(*_args, **_kwargs):
        return types.SimpleNamespace(
            retrieval_path="graph_only",
            retrieval_reason="graph_sufficient",
            graph_backend="fake",
            graph_hit_count=1,
            graph_confidence_score=0.88,
            graph_confidence_level="high",
            retrieval_plan_schema_id=(
                "https://schemas.omni.dev/omni.link_graph.retrieval_plan.v1.schema.json"
            ),
            retrieval_plan={
                "schema": "omni.link_graph.retrieval_plan.v1",
                "requested_mode": "hybrid",
                "selected_mode": "graph_only",
                "reason": "graph_sufficient",
                "backend_name": "fake",
                "graph_hit_count": 1,
                "source_hint_count": 0,
                "graph_confidence_score": 0.88,
                "graph_confidence_level": "high",
                "budget": {"candidate_limit": 12, "max_sources": 4, "rows_per_source": 4},
            },
            graph_rows=(
                {
                    "content": "graph result",
                    "source": "docs/a.md",
                    "score": 0.99,
                    "title": "A",
                    "section": "",
                },
            ),
            graph_only_empty=False,
        )

    fake_link_graph = types.SimpleNamespace(
        evaluate_link_graph_recall_policy=_fake_evaluate,
    )
    monkeypatch.setitem(sys.modules, "omni.rag.link_graph", fake_link_graph)

    with patch.object(recall, "get_vector_store", return_value=mock_vector):
        out = await recall.recall(
            query="architecture",
            chunked=False,
            limit=3,
            preview=True,
            retrieval_mode="hybrid",
        )

    data = _parse_recall_out(out)
    assert data.get("status") == "success"
    assert data.get("retrieval_path") == "graph_only"
    assert data.get("graph_confidence_level") == "high"
    assert data.get("retrieval_plan", {}).get("schema") == "omni.link_graph.retrieval_plan.v1"
    assert str(data.get("retrieval_plan_schema_id")).endswith(
        "/omni.link_graph.retrieval_plan.v1.schema.json"
    )
    assert data.get("found") == 1
    mock_vector.search.assert_not_called()


@pytest.mark.asyncio
async def test_recall_graph_only_path_caps_results_by_limit(
    monkeypatch: pytest.MonkeyPatch,
):
    """Graph-only retrieval should still cap returned rows to user limit."""
    mock_vector = MagicMock()
    mock_vector.get_store_for_collection.return_value = object()
    mock_vector.search = AsyncMock(return_value=[])

    async def _fake_evaluate(*_args, **_kwargs):
        return types.SimpleNamespace(
            retrieval_path="graph_only",
            retrieval_reason="graph_sufficient",
            graph_backend="fake",
            graph_hit_count=4,
            graph_confidence_score=0.95,
            graph_confidence_level="high",
            retrieval_plan_schema_id=None,
            retrieval_plan=None,
            graph_rows=(
                {"content": "g0", "source": "docs/0.md", "score": 0.99, "title": "", "section": ""},
                {"content": "g1", "source": "docs/1.md", "score": 0.98, "title": "", "section": ""},
                {"content": "g2", "source": "docs/2.md", "score": 0.97, "title": "", "section": ""},
                {"content": "g3", "source": "docs/3.md", "score": 0.96, "title": "", "section": ""},
            ),
            graph_only_empty=False,
        )

    fake_link_graph = types.SimpleNamespace(
        evaluate_link_graph_recall_policy=_fake_evaluate,
    )
    monkeypatch.setitem(sys.modules, "omni.rag.link_graph", fake_link_graph)

    with patch.object(recall, "get_vector_store", return_value=mock_vector):
        out = await recall.recall(
            query="architecture",
            chunked=False,
            limit=2,
            preview=False,
            retrieval_mode="hybrid",
        )

    data = _parse_recall_out(out)
    assert data.get("status") == "success"
    assert data.get("retrieval_path") == "graph_only"
    assert data.get("found") == 2
    assert len(data.get("results", [])) == 2
    assert [row["content"] for row in data.get("results", [])] == ["g0", "g1"]
    mock_vector.search.assert_not_called()


@pytest.mark.asyncio
async def test_recall_hybrid_mode_falls_back_to_vector_when_graph_insufficient(
    monkeypatch: pytest.MonkeyPatch,
):
    """When policy returns vector_only, semantic vector search remains active."""
    mock_vector = MagicMock()
    mock_vector.get_store_for_collection.return_value = object()
    mock_vector.search = AsyncMock(
        return_value=[
            types.SimpleNamespace(
                id="id-1",
                content="vector result",
                distance=0.2,
                metadata={"source": "docs/v.md"},
            )
        ]
    )

    async def _fake_evaluate(*_args, **_kwargs):
        return types.SimpleNamespace(
            retrieval_path="vector_only",
            retrieval_reason="graph_insufficient",
            graph_backend="fake",
            graph_hit_count=0,
            graph_confidence_score=0.12,
            graph_confidence_level="low",
            retrieval_plan_schema_id=(
                "https://schemas.omni.dev/omni.link_graph.retrieval_plan.v1.schema.json"
            ),
            retrieval_plan={
                "schema": "omni.link_graph.retrieval_plan.v1",
                "requested_mode": "hybrid",
                "selected_mode": "vector_only",
                "reason": "graph_insufficient",
                "backend_name": "fake",
                "graph_hit_count": 0,
                "source_hint_count": 0,
                "graph_confidence_score": 0.12,
                "graph_confidence_level": "low",
                "budget": {"candidate_limit": 10, "max_sources": 4, "rows_per_source": 4},
            },
            graph_rows=(),
            graph_only_empty=False,
        )

    fake_link_graph = types.SimpleNamespace(
        evaluate_link_graph_recall_policy=_fake_evaluate,
    )
    monkeypatch.setitem(sys.modules, "omni.rag.link_graph", fake_link_graph)

    with patch.object(recall, "get_vector_store", return_value=mock_vector):
        out = await recall.recall(
            query="architecture",
            chunked=False,
            limit=2,
            preview=True,
            retrieval_mode="hybrid",
        )

    data = _parse_recall_out(out)
    assert data.get("status") == "success"
    assert data.get("retrieval_path") == "vector_only"
    assert data.get("graph_confidence_level") == "low"
    assert data.get("retrieval_plan", {}).get("selected_mode") == "vector_only"
    mock_vector.search.assert_awaited_once()


@pytest.mark.asyncio
async def test_recall_vector_failure_uses_graph_only_fallback_when_available(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Vector retrieval error should downgrade to graph-only rows when available."""
    mock_vector = MagicMock()
    mock_vector.get_store_for_collection.return_value = object()

    async def _fake_evaluate(*_args, **_kwargs):
        mode = str(_kwargs.get("retrieval_mode") or "")
        if mode == "hybrid":
            return types.SimpleNamespace(
                retrieval_path="vector_only",
                retrieval_reason="graph_insufficient",
                graph_backend="fake",
                graph_hit_count=0,
                graph_confidence_score=0.1,
                graph_confidence_level="low",
                retrieval_plan_schema_id=(
                    "https://schemas.omni.dev/omni.link_graph.retrieval_plan.v1.schema.json"
                ),
                retrieval_plan={
                    "schema": "omni.link_graph.retrieval_plan.v1",
                    "requested_mode": "hybrid",
                    "selected_mode": "vector_only",
                    "reason": "graph_insufficient",
                    "backend_name": "fake",
                    "graph_hit_count": 0,
                    "source_hint_count": 0,
                    "graph_confidence_score": 0.1,
                    "graph_confidence_level": "low",
                    "budget": {"candidate_limit": 8, "max_sources": 4, "rows_per_source": 4},
                },
                graph_rows=(),
                graph_only_empty=False,
            )
        assert mode == "graph_only"
        return types.SimpleNamespace(
            retrieval_path="graph_only",
            retrieval_reason="graph_only_requested",
            graph_backend="fake",
            graph_hit_count=1,
            graph_confidence_score=0.9,
            graph_confidence_level="high",
            retrieval_plan_schema_id=(
                "https://schemas.omni.dev/omni.link_graph.retrieval_plan.v1.schema.json"
            ),
            retrieval_plan={
                "schema": "omni.link_graph.retrieval_plan.v1",
                "requested_mode": "graph_only",
                "selected_mode": "graph_only",
                "reason": "graph_only_requested",
                "backend_name": "fake",
                "graph_hit_count": 1,
                "source_hint_count": 1,
                "graph_confidence_score": 0.9,
                "graph_confidence_level": "high",
                "budget": {"candidate_limit": 8, "max_sources": 4, "rows_per_source": 4},
            },
            graph_rows=(
                {
                    "content": "graph fallback result",
                    "source": "docs/fallback.md",
                    "score": 0.91,
                    "title": "Fallback",
                    "section": "",
                },
            ),
            graph_only_empty=False,
        )

    fake_link_graph = types.SimpleNamespace(
        evaluate_link_graph_recall_policy=_fake_evaluate,
    )
    monkeypatch.setitem(sys.modules, "omni.rag.link_graph", fake_link_graph)

    with (
        patch.object(recall, "get_vector_store", return_value=mock_vector),
        patch.object(
            recall,
            "run_recall_query_rows",
            AsyncMock(side_effect=RuntimeError("Embedding timed out after 5s")),
        ),
    ):
        out = await recall.recall(
            query="architecture",
            chunked=False,
            limit=2,
            preview=False,
            retrieval_mode="hybrid",
        )

    data = _parse_recall_out(out)
    assert data.get("status") == "success"
    assert data.get("retrieval_path") == "graph_only"
    assert data.get("retrieval_reason") == "graph_only_requested"
    assert data.get("found") == 1
    assert data.get("results", [])[0]["source"] == "docs/fallback.md"


@pytest.mark.asyncio
async def test_recall_vector_failure_returns_empty_success_when_graph_fallback_unavailable(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Vector retrieval error should return safe empty success when graph fallback is unavailable."""
    mock_vector = MagicMock()
    mock_vector.get_store_for_collection.return_value = object()

    async def _raise_policy(*_args, **_kwargs):
        raise RuntimeError("link_graph backend unavailable")

    fake_link_graph = types.SimpleNamespace(
        evaluate_link_graph_recall_policy=_raise_policy,
    )
    monkeypatch.setitem(sys.modules, "omni.rag.link_graph", fake_link_graph)

    with (
        patch.object(recall, "get_vector_store", return_value=mock_vector),
        patch.object(
            recall,
            "run_recall_query_rows",
            AsyncMock(side_effect=RuntimeError("Embedding timed out after 5s")),
        ),
    ):
        out = await recall.recall(
            query="architecture",
            chunked=False,
            limit=2,
            preview=False,
            retrieval_mode="hybrid",
        )

    data = _parse_recall_out(out)
    assert data.get("status") == "success"
    assert data.get("retrieval_path") == "vector_only"
    assert data.get("retrieval_reason") == "vector_error_fallback_empty"
    assert data.get("found") == 0
    assert data.get("results") == []


@pytest.mark.asyncio
async def test_recall_vector_only_mode_skips_fusion_boost() -> None:
    """vector_only should avoid fusion post-processing for lower latency."""
    mock_vector = MagicMock()
    mock_vector.get_store_for_collection.return_value = object()
    mock_vector.search = AsyncMock(
        return_value=[
            types.SimpleNamespace(
                id="id-1",
                content="vector result",
                distance=0.1,
                metadata={"source": "docs/v.md"},
            )
        ]
    )

    with (
        patch.object(recall, "get_vector_store", return_value=mock_vector),
        patch.object(
            recall,
            "_apply_fusion_recall_boost",
            AsyncMock(side_effect=AssertionError("fusion should not run in vector_only")),
        ),
    ):
        out = await recall.recall(
            query="architecture",
            chunked=False,
            limit=1,
            preview=False,
            retrieval_mode="vector_only",
        )

    data = _parse_recall_out(out)
    assert data.get("status") == "success"
    assert data.get("retrieval_mode") == "vector_only"


@pytest.mark.asyncio
async def test_fusion_boost_skips_low_signal_query(monkeypatch: pytest.MonkeyPatch):
    """Single-char query should skip fusion import/execution entirely."""
    called = {"compute": False, "graph": False, "kg": False}

    async def _fake_graph(rows, q, **kwargs):
        called["graph"] = True
        return rows

    def _fake_compute(_q):
        called["compute"] = True
        return types.SimpleNamespace(
            link_graph_proximity_scale=1.0,
            link_graph_entity_scale=1.0,
        )

    def _fake_kg(rows, _q, **kwargs):
        called["kg"] = True
        return rows

    fake_fusion = types.SimpleNamespace(
        compute_fusion_weights=_fake_compute,
        link_graph_proximity_boost=_fake_graph,
        apply_kg_recall_boost=_fake_kg,
    )
    monkeypatch.setitem(sys.modules, "omni.rag.fusion", fake_fusion)

    rows = [{"source": "a", "score": 0.9, "content": "hello"}]
    out = await recall._apply_fusion_recall_boost(rows, "x")
    assert out == rows
    assert called == {"compute": False, "graph": False, "kg": False}
