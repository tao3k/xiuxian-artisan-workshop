"""Contracts for removed checkpoint schema API."""

from __future__ import annotations

import importlib

import pytest


def test_checkpoint_schema_module_is_removed() -> None:
    """Checkpoint schema API should stay removed after workflow migration."""
    with pytest.raises(ModuleNotFoundError):
        importlib.import_module("omni.foundation.api.checkpoint_schema")
