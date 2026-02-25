#!/usr/bin/env python3
"""Config and argument bindings for complex scenarios runner."""

from __future__ import annotations

from typing import Any


def parse_args(
    *,
    config_module: Any,
    script_dir: Any,
    webhook_url_default: str | None,
    default_log_file: str,
    default_max_wait: int,
    default_max_idle_secs: int,
) -> Any:
    """Parse CLI args for complex scenarios runner."""
    return config_module.parse_args(
        script_dir=script_dir,
        webhook_url_default=webhook_url_default,
        default_log_file=default_log_file,
        default_max_wait=default_max_wait,
        default_max_idle_secs=default_max_idle_secs,
    )


def build_config(
    args: Any,
    *,
    runtime_bindings_module: Any,
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
    """Build normalized config through runtime bindings."""
    return runtime_bindings_module.build_config(
        args,
        runtime_config_module=runtime_config_module,
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
