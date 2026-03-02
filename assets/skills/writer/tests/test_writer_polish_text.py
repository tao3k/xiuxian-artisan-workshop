from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

import pytest


def _skill_root() -> Path:
    return Path(__file__).resolve().parents[1]


def _load_writer_text_module():
    """Load writer text module directly by path to avoid cross-skill `scripts` collisions."""
    module_name = "writer_text_test_module"
    module_path = _skill_root() / "scripts" / "text.py"
    spec = importlib.util.spec_from_file_location(module_name, module_path)
    if spec is None or spec.loader is None:
        raise ImportError(f"Failed to create module spec for {module_path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def _unwrap_skill_output(payload: object) -> dict[str, object]:
    if isinstance(payload, dict):
        content = payload.get("content")
        if isinstance(content, list) and content:
            first = content[0]
            if isinstance(first, dict):
                text = first.get("text")
                if isinstance(text, str):
                    return json.loads(text)
        return payload
    if isinstance(payload, str):
        return json.loads(payload)
    raise TypeError(f"Unexpected payload type: {type(payload)!r}")


@pytest.mark.asyncio
async def test_polish_text_accepts_wrapped_internal_results(monkeypatch) -> None:
    writer_text = _load_writer_text_module()

    out = await writer_text.polish_text("# Title\n\nThis is very basic.")
    payload = _unwrap_skill_output(out)

    assert payload.get("status") in {"clean", "needs_polish"}
    assert isinstance(payload.get("violations"), list)
