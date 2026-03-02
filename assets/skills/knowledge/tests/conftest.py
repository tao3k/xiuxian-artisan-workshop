"""Pytest configuration for knowledge skill tests.

Uses SKILLS_DIR for skill scripts path (SSOT). Imports fixtures from
omni.test_kit.fixtures.rag for pytest discovery.

Session-scoped mocks prevent loading vector store and embedding service
(sentence-transformers, LanceDB) to avoid memory leaks and high RSS.
Patch target: omni.foundation.services.vector.get_vector_store (not store)
so recall's import gets the mock when foundation forwards to services.vector.

tracemalloc: logs RSS before/after each test to stderr; dumps top allocations
when growth exceeds threshold. Run with `-s` to see output. Lower threshold
for more dumps: KNOWLEDGE_TEST_MEMORY_THRESHOLD_MB=10 pytest ... -s

Memory protection (abort on overflow) is project-wide; see root conftest.py.
Note: test_py_chunk_text_returns_correct_structure shows ~23 MiB growth on first
run due to omni_core_rs (Rust + tiktoken) load. This is expected one-time cost.
"""

import os
import sys
import tracemalloc
from pathlib import Path
from unittest.mock import MagicMock

import pytest
from omni.test_kit.fixtures import rag as _rag_fixtures

from omni.foundation.config.skills import SKILLS_DIR

# Add knowledge scripts to path for imports (recall, graph, etc.)
skill_scripts = SKILLS_DIR(skill="knowledge", path="scripts")
if str(skill_scripts) in sys.path:
    sys.path.remove(str(skill_scripts))
sys.path.insert(0, str(skill_scripts))

# Add tests directory path to support local helpers (e.g. _module_loader) in full-suite runs.
tests_dir = Path(__file__).resolve().parent
if str(tests_dir) in sys.path:
    sys.path.remove(str(tests_dir))
sys.path.insert(0, str(tests_dir))

# Prevent cross-skill pollution for bare module imports used by legacy tests
# (e.g., `import graph`, `import recall`) when running full `omni skill test --all`.
_LOCAL_MODULE_NAMES = {
    "graph",
    "ingest",
    "link_graph_search",
    "paper_workflow",
    "recall",
}
for _name in _LOCAL_MODULE_NAMES:
    sys.modules.pop(_name, None)


@pytest.fixture(autouse=True)
def _ensure_local_module_resolution() -> None:
    """Keep local tests/scripts first in sys.path and clear bare-name collisions."""
    for local_path in (str(tests_dir), str(skill_scripts)):
        if local_path in sys.path:
            sys.path.remove(local_path)
        sys.path.insert(0, local_path)
    for name in _LOCAL_MODULE_NAMES:
        sys.modules.pop(name, None)
    yield


def _get_rss_mb() -> float | None:
    """Current process RSS in MiB. Uses resource.getrusage (Unix) or psutil."""
    try:
        import resource

        r = resource.getrusage(resource.RUSAGE_SELF)
        rss = getattr(r, "ru_maxrss", 0) or 0
        if sys.platform == "darwin":
            return round(rss / (1024 * 1024), 2)
        return round(rss / 1024, 2)  # Linux: KB
    except Exception:
        try:
            import psutil

            return round(psutil.Process().memory_info().rss / (1024 * 1024), 2)
        except Exception:
            return None


def _log_tracemalloc_top(count: int = 10) -> None:
    """Log top memory allocations from tracemalloc."""
    if not tracemalloc.is_tracing():
        return
    try:
        snapshot = tracemalloc.take_snapshot()
        top = snapshot.statistics("lineno")[:count]
        import sys

        sys.stderr.write("\n[MEMORY] tracemalloc top allocations (possible leak sources):\n")
        for i, stat in enumerate(top[:5], 1):
            tb_str = "".join(stat.traceback.format())
            sys.stderr.write(f"  #{i} {stat.size / 1024 / 1024:.2f} MiB\n{tb_str}\n")
        sys.stderr.flush()
    except Exception:
        pass


@pytest.fixture(autouse=True)
def _tracemalloc_per_test(request):
    """Log RSS before/after each test; dump tracemalloc top when growth > threshold."""
    if not tracemalloc.is_tracing():
        tracemalloc.start(10)
    threshold_mb = float(os.environ.get("KNOWLEDGE_TEST_MEMORY_THRESHOLD_MB", "50"))
    before_mb = _get_rss_mb() or 0.0
    test_id = request.node.nodeid
    sys.stderr.write(f"\n[MEMORY] before {test_id}: RSS={before_mb:.1f} MiB\n")
    sys.stderr.flush()
    yield
    after_mb = _get_rss_mb()
    if after_mb is not None:
        delta = after_mb - before_mb
        sys.stderr.write(
            f"[MEMORY] after {test_id}: RSS={after_mb:.1f} MiB (delta={delta:+.1f} MiB)\n"
        )
        if delta > threshold_mb:
            sys.stderr.write(
                f"[MEMORY] {test_id} grew RSS by {delta:.1f} MiB (threshold={threshold_mb}); dumping allocations\n"
            )
            _log_tracemalloc_top()
        sys.stderr.flush()


@pytest.fixture(autouse=True, scope="session")
def _tracemalloc_cleanup():
    """Stop tracemalloc at session end to release tracking memory."""
    yield
    if tracemalloc.is_tracing():
        tracemalloc.stop()


@pytest.fixture(autouse=True, scope="session")
def _mock_heavy_services():
    """Prevent loading vector store and embedding service in knowledge tests.

    Avoids sentence-transformers (~1-2GB), LanceDB connections, and related
    native allocations that cause memory leaks when tests run in parallel.
    """
    from unittest.mock import patch

    mock_client = MagicMock()
    mock_client.store = True  # Pass "vector store initialized" check in recall
    mock_client.get_store_for_collection = MagicMock(return_value=None)

    mock_embed = MagicMock()
    mock_embed.dimension = 256
    mock_embed.embed = MagicMock(return_value=[[0.0] * 256])
    mock_embed.embed_batch = MagicMock(return_value=[[0.0] * 256] * 10)

    # Patch vector.get_vector_store (not store) so recall's "from omni.foundation import
    # get_vector_store" gets the mock when foundation forwards to services.vector
    with (
        patch("omni.foundation.services.vector.get_vector_store", return_value=mock_client),
        patch("omni.foundation.services.embedding.get_embedding_service", return_value=mock_embed),
    ):
        yield


def pytest_collection_modifyitems(config, items):
    """Add flaky reruns to knowledge tests (--reruns 2 when failures occur)."""
    for item in items:
        path = getattr(item, "path", None) or getattr(item, "fspath", None)
        if path and "knowledge" in str(path):
            item.add_marker(pytest.mark.flaky(reruns=2, reruns_delay=1))


# Re-export shared fixtures for pytest discovery in this skill test suite.
mock_knowledge_graph_store = _rag_fixtures.mock_knowledge_graph_store
mock_llm_for_extraction = _rag_fixtures.mock_llm_for_extraction
mock_llm_empty_response = _rag_fixtures.mock_llm_empty_response
mock_llm_invalid_json = _rag_fixtures.mock_llm_invalid_json
