#!/usr/bin/env python3
"""Compatibility facade for complex-scenarios module dependency bindings."""

from __future__ import annotations

from complex_scenarios_module_bindings_loader import load_module_bindings as _load_bindings_impl
from complex_scenarios_module_bindings_models import (
    ComplexScenariosModuleBindings as _ComplexScenariosModuleBindings,
)

ComplexScenariosModuleBindings = _ComplexScenariosModuleBindings


def load_module_bindings(caller_file: str) -> ComplexScenariosModuleBindings:
    """Load all sibling modules required by `test_omni_agent_complex_scenarios.py`."""
    return _load_bindings_impl(caller_file)
