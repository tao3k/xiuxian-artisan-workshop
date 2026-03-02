#!/usr/bin/env python3
"""Dependency loader for agent channel blackbox entry script."""

from __future__ import annotations

from agent_channel_blackbox_module_bindings_build import build_bindings
from agent_channel_blackbox_module_bindings_models import BlackboxModuleBindings
from agent_channel_blackbox_module_bindings_resolve import resolve_modules


def load_module_bindings(caller_file: str) -> BlackboxModuleBindings:
    """Load all sibling modules required by `agent_channel_blackbox.py`."""
    modules = resolve_modules(caller_file)
    return build_bindings(modules, bindings_cls=BlackboxModuleBindings)


__all__ = ["BlackboxModuleBindings", "load_module_bindings"]
