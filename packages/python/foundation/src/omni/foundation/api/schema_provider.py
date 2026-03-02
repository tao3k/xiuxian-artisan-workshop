"""
Centralized schema provider that loads JSON schemas from Rust binary bindings.
Follows CyberXiuXian Artisan Studio 2026 standards for self-contained resources.
"""

from __future__ import annotations

import json
from functools import cache
from typing import Any

_BUILTIN_SCHEMAS: dict[str, dict[str, Any]] = {
    "omni.mcp.tool_result.v1": {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "omni.mcp.tool_result.v1.schema.json",
        "type": "object",
        "additionalProperties": False,
        "required": ["content", "isError"],
        "properties": {
            "content": {
                "type": "array",
                "minItems": 1,
                "items": {
                    "type": "object",
                    "additionalProperties": False,
                    "required": ["type", "text"],
                    "properties": {
                        "type": {"const": "text"},
                        "text": {"type": "string"},
                    },
                },
            },
            "isError": {"type": "boolean"},
        },
    },
}

_OMNI_CORE_RS_SCHEMA_MAP: dict[str, str] = {
    # Rust omni_core_rs exposes type-based schema retrieval.
    "omni.vector.hybrid.v1": "HybridSearchResult",
    "omni.vector.search.v1": "VectorSearchResult",
    "omni.vector.tool_search.v1": "ToolSearchResult",
}


@cache
def get_schema(name: str) -> dict[str, Any]:
    """
    Load a schema by name from the Rust backend.

    Args:
        name: The canonical schema identifier (e.g., 'omni.link_graph.record.v1')

    Returns:
        The parsed JSON schema as a dictionary.

    Raises:
        ImportError: If the Rust backend is not available.
        ValueError: If the schema name is unknown.
    """
    builtin = _BUILTIN_SCHEMAS.get(name)
    if builtin is not None:
        return builtin

    # Canonical backend: xiuxian_wendao schema registry (canonical id -> JSON schema)
    last_error: Exception | None = None

    try:
        from _xiuxian_wendao import get_schema as rust_get_schema
    except ImportError:
        rust_get_schema = None
    if rust_get_schema is not None:
        try:
            return json.loads(rust_get_schema(name))
        except ValueError:
            last_error = ValueError(f"Unknown schema identifier: {name}")
        except Exception as e:
            last_error = RuntimeError(f"Failed to load schema '{name}' from Rust binding: {e}")

    # Fallback backend: omni_core_rs named schema registry.
    try:
        import omni_core_rs

        if hasattr(omni_core_rs, "py_get_named_schema_json"):
            return json.loads(omni_core_rs.py_get_named_schema_json(name))
    except Exception as e:
        last_error = e

    # Secondary backend: omni_core_rs type registry (subset mapping only)
    mapped_type = _OMNI_CORE_RS_SCHEMA_MAP.get(name)
    if mapped_type is not None:
        try:
            import omni_core_rs

            return json.loads(omni_core_rs.py_get_schema_json(mapped_type))
        except Exception as e:
            last_error = RuntimeError(
                f"Failed to load schema '{name}' from omni_core_rs type '{mapped_type}': {e}"
            )

    if last_error is not None:
        raise last_error
    raise ImportError(
        f"No Rust schema binding available for '{name}'. "
        "Install `_xiuxian_wendao` or expose named-schema APIs via `omni_core_rs`."
    )


def get_schema_id(name: str) -> str:
    """Return the $id field from a schema."""
    schema = get_schema(name)
    return str(schema.get("$id", "")).strip()
