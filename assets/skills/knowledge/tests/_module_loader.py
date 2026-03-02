"""Skill-local module loader for knowledge tests.

Ensures imports resolve to `assets/skills/knowledge/scripts/*.py` during
mixed-skill test runs (avoids collisions with other skills that expose
same bare module names like `graph`).
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

_SCRIPTS_DIR = Path(__file__).resolve().parent.parent / "scripts"


def load_script_module(
    stem: str,
    *,
    alias: str | None = None,
    expose_stem: bool = False,
):
    """Load one knowledge skill script module by stem name."""
    module_name = alias or stem
    module_path = _SCRIPTS_DIR / f"{stem}.py"
    if not module_path.exists():
        raise FileNotFoundError(f"Knowledge script module not found: {module_path}")

    spec = importlib.util.spec_from_file_location(module_name, module_path)
    if spec is None or spec.loader is None:
        raise ImportError(f"Failed to create module spec for: {module_path}")

    module = importlib.util.module_from_spec(spec)
    # Register under explicit alias by default to avoid cross-skill module collisions.
    sys.modules[module_name] = module
    if expose_stem:
        sys.modules[stem] = module
    spec.loader.exec_module(module)
    return module


__all__ = ["load_script_module"]
