"""pipeline_checkpoint.py - Checkpointer utilities for native workflow runtime."""

from __future__ import annotations

from typing import Any

from omni.foundation.config.logging import get_logger
from omni.foundation.workflow_state import get_checkpointer

logger = get_logger("omni.tracer.pipeline")


def create_in_memory_checkpointer() -> Any | None:
    """Create a native workflow-state checkpointer handle."""
    try:
        return get_checkpointer("omni_tracer_pipeline")
    except Exception as exc:  # pragma: no cover - environment dependent
        logger.warning("workflow_checkpointer_unavailable", error=str(exc))
        return None


def compile_workflow(
    workflow: Any,
    *,
    checkpointer: Any | None = None,
    use_memory_saver: bool = False,
) -> Any:
    """Compile a workflow with optional checkpointer injection."""
    active_checkpointer = checkpointer
    if active_checkpointer is None and use_memory_saver:
        active_checkpointer = create_in_memory_checkpointer()

    if active_checkpointer is not None:
        return workflow.compile(checkpointer=active_checkpointer)
    return workflow.compile()


__all__ = [
    "compile_workflow",
    "create_in_memory_checkpointer",
]
