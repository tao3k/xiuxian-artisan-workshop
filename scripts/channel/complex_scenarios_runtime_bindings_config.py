#!/usr/bin/env python3
"""Config bindings for complex-scenarios runtime entrypoint."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def resolve_runtime_partition_mode(
    log_file: Path,
    *,
    runtime_config_module: Any,
    env_get_fn: Any,
    normalize_partition_fn: Any,
    partition_mode_from_runtime_log_fn: Any,
    partition_mode_from_settings_fn: Any,
) -> str | None:
    """Resolve active partition mode from runtime signals and settings."""
    return runtime_config_module.resolve_runtime_partition_mode(
        log_file,
        env_get_fn=env_get_fn,
        normalize_partition_fn=normalize_partition_fn,
        partition_mode_from_runtime_log_fn=partition_mode_from_runtime_log_fn,
        partition_mode_from_settings_fn=partition_mode_from_settings_fn,
    )


def build_config(
    args: Any,
    *,
    runtime_config_module: Any,
    expected_session_keys_fn: Any,
    expected_session_key_fn: Any,
    session_ids_from_runtime_log_fn: Any,
    allowed_users_from_settings_fn: Any,
    username_from_settings_fn: Any,
    username_from_runtime_log_fn: Any,
    telegram_webhook_secret_token_fn: Any,
    resolve_runtime_partition_mode_fn: Any,
    session_identity_cls: Any,
    runner_config_cls: Any,
    complexity_requirement_cls: Any,
    quality_requirement_cls: Any,
    default_forbid_log_regexes: tuple[str, ...],
) -> Any:
    """Build normalized runner configuration."""
    return runtime_config_module.build_config(
        args,
        expected_session_keys_fn=expected_session_keys_fn,
        expected_session_key_fn=expected_session_key_fn,
        session_ids_from_runtime_log_fn=session_ids_from_runtime_log_fn,
        allowed_users_from_settings_fn=allowed_users_from_settings_fn,
        username_from_settings_fn=username_from_settings_fn,
        username_from_runtime_log_fn=username_from_runtime_log_fn,
        telegram_webhook_secret_token_fn=telegram_webhook_secret_token_fn,
        resolve_runtime_partition_mode_fn=resolve_runtime_partition_mode_fn,
        session_identity_cls=session_identity_cls,
        runner_config_cls=runner_config_cls,
        complexity_requirement_cls=complexity_requirement_cls,
        quality_requirement_cls=quality_requirement_cls,
        default_forbid_log_regexes=default_forbid_log_regexes,
    )
