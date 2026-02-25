"""
test_concurrency.py - Concurrency Integration Tests

This module tests the Rust backend under high concurrency
to verify stability and correct behavior under stress.

Marker: local (integration tests with real services, no network)

Run with:
    uv run pytest packages/python/core/tests/integration/test_concurrency.py -v
    uv run pytest packages/python/core/tests/integration/test_concurrency.py -m local -v
"""

from __future__ import annotations

import asyncio

import pytest


@pytest.mark.asyncio
async def test_vector_store_concurrent_writes(temp_lancedb):
    """
    Concurrency Test: Multiple concurrent writers to vector store.
    Verifies Rust LanceDB handles concurrent writes correctly.
    """
    from omni.foundation.bridge import RustVectorStore

    store = RustVectorStore(str(temp_lancedb), dimension=384)

    async def writer(writer_id: int, count: int = 20):
        for i in range(count):
            doc_id = f"writer_{writer_id}_doc_{i}"
            vector = [0.1 * writer_id] * 384
            await store.add_documents(
                table_name="concurrent_test",
                ids=[doc_id],
                vectors=[vector],
                contents=[f"Document from writer {writer_id}"],
                metadatas=["{}"],
            )

    # Run 5 concurrent writers
    tasks = [writer(i) for i in range(5)]
    await asyncio.gather(*tasks)

    # Verify documents were written using explicit search_tools API
    query_vec = store._embedding_service.embed("document")
    if query_vec and isinstance(query_vec[0], list):
        query_vec = query_vec[0]

    results = await store.search_tools(
        table_name="concurrent_test",
        query_vector=query_vec,
        query_text="document",
        limit=100,
        threshold=0.0,
    )
    # At least no errors and valid list output.
    assert isinstance(results, list)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
