#!/usr/bin/env python3
"""CLI/config and dataset parsing helpers for memory benchmark runner."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from memory_benchmark_config_args import (
    default_report_path as _default_report_path_impl,
)
from memory_benchmark_config_args import (
    parse_args as _parse_args_impl,
)
from memory_benchmark_config_dataset import load_scenarios as _load_scenarios_impl
from memory_benchmark_config_runtime import (
    build_config as _build_config_impl,
)
from memory_benchmark_config_runtime import (
    resolve_runtime_partition_mode as _resolve_runtime_partition_mode_impl,
)

if TYPE_CHECKING:
    import argparse
    from pathlib import Path

    from memory_benchmark_models import BenchmarkConfig, QuerySpec, ScenarioSpec


def default_report_path(filename: str) -> Path:
    """Build a report path under PRJ runtime report directory."""
    return _default_report_path_impl(filename)


def parse_args(
    *,
    script_dir: Path,
    default_log_file: str,
    default_max_wait: int,
    default_max_idle_secs: int,
) -> argparse.Namespace:
    """Parse command-line arguments for memory benchmark script."""
    return _parse_args_impl(
        script_dir=script_dir,
        default_log_file=default_log_file,
        default_max_wait=default_max_wait,
        default_max_idle_secs=default_max_idle_secs,
    )


def resolve_runtime_partition_mode(
    log_file: Path,
    *,
    normalize_telegram_session_partition_mode_fn: Any,
    session_partition_mode_from_runtime_log_fn: Any,
    telegram_session_partition_mode_fn: Any,
) -> str | None:
    """Resolve runtime session partition mode from override/log/config order."""
    return _resolve_runtime_partition_mode_impl(
        log_file,
        normalize_telegram_session_partition_mode_fn=normalize_telegram_session_partition_mode_fn,
        session_partition_mode_from_runtime_log_fn=session_partition_mode_from_runtime_log_fn,
        telegram_session_partition_mode_fn=telegram_session_partition_mode_fn,
    )


def build_config(
    args: argparse.Namespace,
    *,
    config_cls: type[BenchmarkConfig],
    infer_session_ids_fn: Any,
    resolve_runtime_partition_mode_fn: Any,
) -> BenchmarkConfig:
    """Validate CLI args and build typed benchmark config."""
    return _build_config_impl(
        args,
        config_cls=config_cls,
        infer_session_ids_fn=infer_session_ids_fn,
        resolve_runtime_partition_mode_fn=resolve_runtime_partition_mode_fn,
    )


def load_scenarios(
    path: Path,
    *,
    query_spec_cls: type[QuerySpec],
    scenario_spec_cls: type[ScenarioSpec],
) -> tuple[ScenarioSpec, ...]:
    """Parse and validate benchmark scenario dataset JSON."""
    return _load_scenarios_impl(
        path,
        query_spec_cls=query_spec_cls,
        scenario_spec_cls=scenario_spec_cls,
    )
