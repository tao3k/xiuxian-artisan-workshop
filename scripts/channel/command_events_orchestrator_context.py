#!/usr/bin/env python3
"""Compatibility facade for command event orchestration context helpers."""

from __future__ import annotations

from typing import Any

from command_events_orchestrator_context_models import (
    OrchestratorContext as _OrchestratorContext,
)
from command_events_orchestrator_context_prepare import (
    prepare_orchestrator_context as _prepare_orchestrator_context_impl,
)

OrchestratorContext = _OrchestratorContext


def prepare_orchestrator_context(
    args: Any, **kwargs: Any
) -> tuple[OrchestratorContext | None, int | None]:
    """Resolve all shared orchestration inputs before executing probe cases."""
    return _prepare_orchestrator_context_impl(args, **kwargs)
