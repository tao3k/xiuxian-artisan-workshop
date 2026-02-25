#!/usr/bin/env python3
"""Dependency loader for memory benchmark entry script."""

from __future__ import annotations

import importlib
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class MemoryBenchmarkModuleBindings:
    """Resolved sibling modules and helper callables for memory benchmark runner."""

    normalize_telegram_session_partition_mode: Any
    session_ids_from_runtime_log: Any
    session_partition_mode_from_runtime_log: Any
    telegram_session_partition_mode: Any
    shared_log_cursor_cls: Any
    shared_init_log_cursor: Any
    shared_read_new_log_lines_with_cursor: Any
    entry_bindings_module: Any
    signals_module: Any
    analysis_module: Any
    report_module: Any
    execution_module: Any
    config_module: Any
    models_module: Any
    runtime_bindings_module: Any
    output_module: Any
    runner_module: Any


def load_module_bindings(caller_file: str) -> MemoryBenchmarkModuleBindings:
    """Load all sibling modules required by `test_omni_agent_memory_benchmark.py`."""
    load_sibling_module = importlib.import_module("module_loader").load_sibling_module

    def _load_sibling(module_name: str, file_name: str, error_context: str) -> Any:
        return load_sibling_module(
            module_name=module_name,
            file_name=file_name,
            caller_file=caller_file,
            error_context=error_context,
        )

    resolver_module = _load_sibling("config_resolver", "config_resolver.py", "resolver module")
    entry_bindings_module = _load_sibling(
        "memory_benchmark_entry_bindings",
        "memory_benchmark_entry_bindings.py",
        "memory benchmark entry bindings",
    )
    log_io_module = _load_sibling("log_io", "log_io.py", "shared log I/O helpers")
    signals_module = _load_sibling(
        "memory_benchmark_signals",
        "memory_benchmark_signals.py",
        "memory benchmark signal parser",
    )
    analysis_module = _load_sibling(
        "memory_benchmark_analysis",
        "memory_benchmark_analysis.py",
        "memory benchmark analysis helpers",
    )
    report_module = _load_sibling(
        "memory_benchmark_report",
        "memory_benchmark_report.py",
        "memory benchmark report renderer",
    )
    execution_module = _load_sibling(
        "memory_benchmark_execution",
        "memory_benchmark_execution.py",
        "memory benchmark execution helpers",
    )
    config_module = _load_sibling(
        "memory_benchmark_config",
        "memory_benchmark_config.py",
        "memory benchmark config helpers",
    )
    models_module = _load_sibling(
        "memory_benchmark_models",
        "memory_benchmark_models.py",
        "memory benchmark datamodels",
    )
    runtime_bindings_module = _load_sibling(
        "memory_benchmark_runtime_bindings",
        "memory_benchmark_runtime_bindings.py",
        "memory benchmark runtime bindings",
    )
    output_module = _load_sibling(
        "memory_benchmark_output",
        "memory_benchmark_output.py",
        "memory benchmark output helpers",
    )
    runner_module = _load_sibling(
        "memory_benchmark_runner",
        "memory_benchmark_runner.py",
        "memory benchmark runner helpers",
    )

    return MemoryBenchmarkModuleBindings(
        normalize_telegram_session_partition_mode=(
            resolver_module.normalize_telegram_session_partition_mode
        ),
        session_ids_from_runtime_log=resolver_module.session_ids_from_runtime_log,
        session_partition_mode_from_runtime_log=resolver_module.session_partition_mode_from_runtime_log,
        telegram_session_partition_mode=resolver_module.telegram_session_partition_mode,
        shared_log_cursor_cls=log_io_module.LogCursor,
        shared_init_log_cursor=log_io_module.init_log_cursor,
        shared_read_new_log_lines_with_cursor=log_io_module.read_new_log_lines_with_cursor,
        entry_bindings_module=entry_bindings_module,
        signals_module=signals_module,
        analysis_module=analysis_module,
        report_module=report_module,
        execution_module=execution_module,
        config_module=config_module,
        models_module=models_module,
        runtime_bindings_module=runtime_bindings_module,
        output_module=output_module,
        runner_module=runner_module,
    )
