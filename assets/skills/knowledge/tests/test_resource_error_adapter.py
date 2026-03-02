"""Tests for knowledge skill resource error payload adapters."""

from __future__ import annotations

import sys
import types

import pytest
from _module_loader import load_script_module

link_graph_search = load_script_module(
    "link_graph_search", alias="knowledge_link_graph_search_test"
)
graph_stats_resource = load_script_module(
    "graph", alias="knowledge_graph_resource_test"
).graph_stats_resource


@pytest.mark.asyncio
async def test_link_graph_stats_resource_error_payload(monkeypatch: pytest.MonkeyPatch) -> None:
    def _raise_backend():
        raise RuntimeError("link graph backend boom")

    monkeypatch.setattr(link_graph_search, "_get_link_graph_backend", _raise_backend)

    out = await link_graph_search.link_graph_stats_resource()

    assert out == {"error": "link graph backend boom"}


def test_graph_stats_resource_error_payload(monkeypatch: pytest.MonkeyPatch) -> None:
    def _raise_store():
        raise RuntimeError("graph backend boom")

    fake_graph = types.SimpleNamespace(get_graph_store=_raise_store)
    monkeypatch.setitem(sys.modules, "omni.rag.graph", fake_graph)

    out = graph_stats_resource()

    assert out == {"error": "graph backend boom"}
