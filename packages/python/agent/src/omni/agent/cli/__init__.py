"""
agent/cli - Atomic CLI Module for Omni DevEnv

Modular CLI structure:
- app.py: Typer application and configuration
- console.py: Console and output formatting
- runner.py: Skill execution logic
- omni_loop.py: Legacy decommissioned compatibility shim
- commands/: Command submodules

Usage:
    python -m agent.cli                    # Run CLI
    python -m agent.cli skill list         # List skills
    omni mcp                               # Start MCP server
"""

from __future__ import annotations

from importlib import import_module
from typing import Any

from .app import app, main

__all__ = ["app", "err_console", "main", "run_skills"]


def __getattr__(name: str) -> Any:
    """Lazy attribute loading to keep CLI package import overhead low."""
    if name == "err_console":
        mod = import_module(".console", __name__)
        return getattr(mod, name)
    if name == "run_skills":
        mod = import_module(".runner", __name__)
        return getattr(mod, name)
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
