#!/usr/bin/env python3
"""Datamodels for complex-scenarios module dependency bindings."""

from __future__ import annotations

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
