#!/usr/bin/env python3
"""Dependency loader for complex scenarios entry script."""

from __future__ import annotations

import importlib
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class ComplexScenariosModuleBindings:
    """Resolved sibling modules and resolver callables for complex scenarios."""

    allowed_users_from_settings: Any
    default_telegram_webhook_url: Any
    normalize_telegram_session_partition_mode: Any
    session_ids_from_runtime_log: Any
    session_partition_mode_from_runtime_log: Any
    telegram_session_partition_mode: Any
    telegram_webhook_secret_token: Any
    username_from_runtime_log: Any
    username_from_settings: Any
    report_module: Any
    evaluation_module: Any
    execution_module: Any
    config_module: Any
    runtime_config_module: Any
    session_keys_module: Any
    models_module: Any
    signal_bindings_module: Any
    runner_module: Any
    entry_bindings_module: Any
    runtime_bindings_module: Any


def load_module_bindings(caller_file: str) -> ComplexScenariosModuleBindings:
    """Load all sibling modules required by `test_omni_agent_complex_scenarios.py`."""
    load_sibling_module = importlib.import_module("module_loader").load_sibling_module

    def _load_sibling(module_name: str, file_name: str, error_context: str) -> Any:
        return load_sibling_module(
            module_name=module_name,
            file_name=file_name,
            caller_file=caller_file,
            error_context=error_context,
        )

    resolver_module = _load_sibling("config_resolver", "config_resolver.py", "resolver module")
    report_module = _load_sibling(
        "complex_scenarios_report",
        "complex_scenarios_report.py",
        "complex scenarios report helpers",
    )
    evaluation_module = _load_sibling(
        "complex_scenarios_evaluation",
        "complex_scenarios_evaluation.py",
        "complex scenarios evaluation helpers",
    )
    execution_module = _load_sibling(
        "complex_scenarios_execution",
        "complex_scenarios_execution.py",
        "complex scenarios execution helpers",
    )
    config_module = _load_sibling(
        "complex_scenarios_config",
        "complex_scenarios_config.py",
        "complex scenarios config helpers",
    )
    runtime_config_module = _load_sibling(
        "complex_scenarios_runtime_config",
        "complex_scenarios_runtime_config.py",
        "complex scenarios runtime config helpers",
    )
    session_keys_module = _load_sibling(
        "telegram_session_keys",
        "telegram_session_keys.py",
        "telegram session key helpers",
    )
    models_module = _load_sibling(
        "complex_scenarios_models",
        "complex_scenarios_models.py",
        "complex scenarios datamodels",
    )
    signal_bindings_module = _load_sibling(
        "complex_scenarios_signal_bindings",
        "complex_scenarios_signal_bindings.py",
        "complex scenarios signal bindings",
    )
    runner_module = _load_sibling(
        "complex_scenarios_runner",
        "complex_scenarios_runner.py",
        "complex scenarios main runner helpers",
    )
    entry_bindings_module = _load_sibling(
        "complex_scenarios_entry_bindings",
        "complex_scenarios_entry_bindings.py",
        "complex scenarios entry bindings",
    )
    runtime_bindings_module = _load_sibling(
        "complex_scenarios_runtime_bindings",
        "complex_scenarios_runtime_bindings.py",
        "complex scenarios runtime bindings",
    )

    return ComplexScenariosModuleBindings(
        allowed_users_from_settings=resolver_module.allowed_users_from_settings,
        default_telegram_webhook_url=resolver_module.default_telegram_webhook_url,
        normalize_telegram_session_partition_mode=(
            resolver_module.normalize_telegram_session_partition_mode
        ),
        session_ids_from_runtime_log=resolver_module.session_ids_from_runtime_log,
        session_partition_mode_from_runtime_log=resolver_module.session_partition_mode_from_runtime_log,
        telegram_session_partition_mode=resolver_module.telegram_session_partition_mode,
        telegram_webhook_secret_token=resolver_module.telegram_webhook_secret_token,
        username_from_runtime_log=resolver_module.username_from_runtime_log,
        username_from_settings=resolver_module.username_from_settings,
        report_module=report_module,
        evaluation_module=evaluation_module,
        execution_module=execution_module,
        config_module=config_module,
        runtime_config_module=runtime_config_module,
        session_keys_module=session_keys_module,
        models_module=models_module,
        signal_bindings_module=signal_bindings_module,
        runner_module=runner_module,
        entry_bindings_module=entry_bindings_module,
        runtime_bindings_module=runtime_bindings_module,
    )
