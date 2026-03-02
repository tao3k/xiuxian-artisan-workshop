"""
Command exports for search subpackage.

This file is loaded by the skill framework to discover @skill_command decorated functions.
"""

import time
from collections.abc import Awaitable, Callable

from omni.foundation.api.decorators import skill_command

_CODE_SEARCH_RESULT_CACHE_TTL_SECONDS = 5.0
_CODE_SEARCH_RESULT_CACHE: dict[str, tuple[str, float]] = {}
_CODE_SEARCH_EXECUTOR: Callable[[str, str], Awaitable[dict]] | None = None


__all__ = ["code_search"]


def _code_search_cache_key(query: str, session_id: str) -> str:
    return f"{session_id}|{query}"


def _code_search_cache_get(key: str) -> str | None:
    cached = _CODE_SEARCH_RESULT_CACHE.get(key)
    if cached is None:
        return None
    value, expires_at = cached
    if time.monotonic() >= expires_at:
        _CODE_SEARCH_RESULT_CACHE.pop(key, None)
        return None
    return value


def _code_search_cache_put(key: str, value: str) -> None:
    _CODE_SEARCH_RESULT_CACHE[key] = (
        value,
        time.monotonic() + _CODE_SEARCH_RESULT_CACHE_TTL_SECONDS,
    )


def clear_code_search_cache() -> None:
    """Clear process-local code search result cache."""
    _CODE_SEARCH_RESULT_CACHE.clear()


def _get_code_search_executor() -> Callable[[str, str], Awaitable[dict]]:
    global _CODE_SEARCH_EXECUTOR
    if _CODE_SEARCH_EXECUTOR is None:
        from .graph import execute_search

        _CODE_SEARCH_EXECUTOR = execute_search
    return _CODE_SEARCH_EXECUTOR


@skill_command(
    name="code_search",
    category="search",
    description="""
    Interactive Code Search - The primary search tool.

    Automatically routes queries to the best engine:
    - AST (Structural): 'class User', 'def authenticate'
    - Vector (Semantic): 'how does auth work?', 'user validation logic'
    - Grep (Exact): 'TODO', 'FIXME', '"error message"'

    Returns structured XML optimized for LLM consumption.

    Args:
        - query: Search query (required)
        - session_id: Optional session ID for tracking

    Returns:
        XML-formatted search results with interactive guidance.
    """,
)
async def code_search(query: str, session_id: str = "default") -> str:
    """Execute interactive code search.

    Uses native workflow runtime to orchestrate search execution
    and returns XML-formatted results.
    """
    cache_key = _code_search_cache_key(query, session_id)
    cached = _code_search_cache_get(cache_key)
    if cached is not None:
        return cached

    try:
        search_executor = _get_code_search_executor()
        result = await search_executor(query, session_id)
        output = str(result.get("final_output", ""))
        _code_search_cache_put(cache_key, output)
        return output
    except Exception as e:
        return f"<error>{e!s}</error>"
