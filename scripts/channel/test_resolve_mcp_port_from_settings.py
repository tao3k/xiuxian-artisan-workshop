from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


def _load_module():
    script_path = Path(__file__).resolve().with_name("resolve_mcp_port_from_settings.py")
    module_name = "test_resolve_mcp_port_from_settings_module"
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def test_resolve_mcp_port_prefers_mcp_port_setting(monkeypatch) -> None:
    module = _load_module()
    monkeypatch.setattr(module, "resolve_mcp_endpoint", lambda: {"port": "18501"})
    assert module.resolve_mcp_port() == 18501


def test_resolve_mcp_port_falls_back_to_embedding_client_url(monkeypatch) -> None:
    module = _load_module()
    monkeypatch.setattr(module, "resolve_mcp_endpoint", lambda: {"port": "18601"})
    assert module.resolve_mcp_port() == 18601


def test_resolve_mcp_port_returns_none_for_invalid_settings(monkeypatch) -> None:
    module = _load_module()
    monkeypatch.setattr(module, "resolve_mcp_endpoint", lambda: {"port": "invalid"})
    assert module.resolve_mcp_port() is None
