#!/usr/bin/env python3
"""Dependency loader for command events entry script."""

from __future__ import annotations

import importlib
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class CommandEventsModuleBindings:
    """Resolved sibling modules and helper callables for command event probes."""

    group_profile_chat_ids: Any
    group_profile_int: Any
    normalize_telegram_session_partition_mode: Any
    session_partition_mode_from_runtime_log: Any
    telegram_session_partition_mode: Any
    telegram_webhook_secret_token: Any
    shared_read_log_tail_lines: Any
    report_module: Any
    admin_isolation_module: Any
    config_module: Any
    case_catalog_module: Any
    probe_runtime_module: Any
    orchestrator_module: Any
    runtime_context_module: Any
    models_module: Any
    runtime_bindings_module: Any
    entry_bindings_module: Any


def load_module_bindings(caller_file: str) -> CommandEventsModuleBindings:
    """Load all sibling modules required by `test_omni_agent_command_events.py`."""
    load_sibling_module = importlib.import_module("module_loader").load_sibling_module

    def _load_sibling(module_name: str, file_name: str, error_context: str) -> Any:
        return load_sibling_module(
            module_name=module_name,
            file_name=file_name,
            caller_file=caller_file,
            error_context=error_context,
        )

    resolver_module = _load_sibling("config_resolver", "config_resolver.py", "resolver module")
    log_io_module = _load_sibling("log_io", "log_io.py", "shared log I/O helpers")
    report_module = _load_sibling(
        "command_events_report",
        "command_events_report.py",
        "command events report helpers",
    )
    admin_isolation_module = _load_sibling(
        "command_events_admin_isolation",
        "command_events_admin_isolation.py",
        "command events admin-isolation helpers",
    )
    config_module = _load_sibling(
        "command_events_config",
        "command_events_config.py",
        "command events config helpers",
    )
    case_catalog_module = _load_sibling(
        "command_events_case_catalog",
        "command_events_case_catalog.py",
        "command events case catalog helpers",
    )
    probe_runtime_module = _load_sibling(
        "command_events_probe_runtime",
        "command_events_probe_runtime.py",
        "command events probe runtime helpers",
    )
    orchestrator_module = _load_sibling(
        "command_events_orchestrator",
        "command_events_orchestrator.py",
        "command events orchestration helpers",
    )
    runtime_context_module = _load_sibling(
        "command_events_runtime_context",
        "command_events_runtime_context.py",
        "command events runtime context helpers",
    )
    models_module = _load_sibling(
        "command_events_models",
        "command_events_models.py",
        "command events datamodels",
    )
    runtime_bindings_module = _load_sibling(
        "command_events_runtime_bindings",
        "command_events_runtime_bindings.py",
        "command events runtime bindings",
    )
    entry_bindings_module = _load_sibling(
        "command_events_entry_bindings",
        "command_events_entry_bindings.py",
        "command events entry bindings",
    )

    return CommandEventsModuleBindings(
        group_profile_chat_ids=resolver_module.group_profile_chat_ids,
        group_profile_int=resolver_module.group_profile_int,
        normalize_telegram_session_partition_mode=(
            resolver_module.normalize_telegram_session_partition_mode
        ),
        session_partition_mode_from_runtime_log=resolver_module.session_partition_mode_from_runtime_log,
        telegram_session_partition_mode=resolver_module.telegram_session_partition_mode,
        telegram_webhook_secret_token=resolver_module.telegram_webhook_secret_token,
        shared_read_log_tail_lines=log_io_module.read_log_tail_lines,
        report_module=report_module,
        admin_isolation_module=admin_isolation_module,
        config_module=config_module,
        case_catalog_module=case_catalog_module,
        probe_runtime_module=probe_runtime_module,
        orchestrator_module=orchestrator_module,
        runtime_context_module=runtime_context_module,
        models_module=models_module,
        runtime_bindings_module=runtime_bindings_module,
        entry_bindings_module=entry_bindings_module,
    )
