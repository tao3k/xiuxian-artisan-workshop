#!/usr/bin/env python3
"""Argument/command bindings for memory CI gate runner."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def parse_args(
    project_root: Path,
    *,
    config_module: Any,
    gate_config_cls: Any,
    default_artifact_relpath_fn: Any,
    resolve_runtime_ports_fn: Any,
    default_run_suffix_fn: Any,
    default_valkey_prefix_fn: Any,
) -> Any:
    """Parse CLI args into gate config."""
    return config_module.parse_args(
        project_root,
        gate_config_cls=gate_config_cls,
        default_artifact_relpath_fn=default_artifact_relpath_fn,
        resolve_runtime_ports_fn=resolve_runtime_ports_fn,
        default_run_suffix_fn=default_run_suffix_fn,
        default_valkey_prefix_fn=default_valkey_prefix_fn,
    )


def run_command(
    cmd: list[str],
    *,
    title: str,
    cwd: Path,
    env: dict[str, str] | None,
    runtime_module: Any,
    gate_step_error_cls: Any,
) -> None:
    """Run one gate stage command."""
    runtime_module.run_command(
        cmd,
        title=title,
        cwd=cwd,
        env=env,
        gate_step_error_cls=gate_step_error_cls,
    )
