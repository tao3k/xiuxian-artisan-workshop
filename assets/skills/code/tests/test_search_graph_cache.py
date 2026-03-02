"""Tests for cached native workflow compilation in code search graph."""

from __future__ import annotations

import pytest


@pytest.fixture
def code_skill_loaded() -> None:
    """Load code skill modules so dynamic package imports are available."""
    from omni.core.skills.tools_loader import create_tools_loader
    from omni.foundation.config.skills import SKILLS_DIR

    scripts_path = SKILLS_DIR() / "code" / "scripts"
    loader = create_tools_loader(scripts_path, "code")
    loader.load_all()


def test_get_compiled_search_graph_reuses_instance(code_skill_loaded: None) -> None:
    """The compiled graph should be created once per process."""
    from code.scripts.search import graph as search_graph

    original_graph = search_graph._search_graph
    original_compiled = search_graph._compiled_search_graph
    try:
        search_graph._search_graph = None
        search_graph._compiled_search_graph = None

        compiled_first = search_graph.get_compiled_search_graph()
        compiled_second = search_graph.get_compiled_search_graph()

        assert compiled_first is compiled_second
    finally:
        search_graph._search_graph = original_graph
        search_graph._compiled_search_graph = original_compiled
