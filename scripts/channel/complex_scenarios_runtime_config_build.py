#!/usr/bin/env python3
"""Config assembly helpers for complex scenario runtime config."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from complex_scenarios_runtime_config_build_identity import (
    merge_forbidden_regexes,
    resolve_peer_identity,
    resolve_secret_token,
    resolve_username,
)
from complex_scenarios_runtime_config_build_runner import build_runner_config
from complex_scenarios_runtime_config_build_sessions import (
    build_sessions,
    ensure_distinct_session_identity,
)
from complex_scenarios_runtime_config_build_validation import (
    resolve_primary_identity,
    resolve_required_paths,
    validate_runtime_args,
)
from complex_scenarios_runtime_config_partition import apply_runtime_partition_defaults

if TYPE_CHECKING:
    import argparse


def build_config(
    args: argparse.Namespace,
    *,
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
    """Build fully validated runner config from CLI args."""
    validate_runtime_args(args)
    log_file, dataset_path, blackbox_script = resolve_required_paths(args)
    chat_a, user_a, thread_a = resolve_primary_identity(
        args,
        log_file=log_file,
        session_ids_from_runtime_log_fn=session_ids_from_runtime_log_fn,
    )

    user_a_int = user_a
    chat_b, chat_c, user_b, user_c, thread_b, thread_c = resolve_peer_identity(
        args,
        chat_a=chat_a,
        user_a_int=user_a_int,
        allowed_users_from_settings_fn=allowed_users_from_settings_fn,
    )
    username = resolve_username(
        args,
        log_file=log_file,
        username_from_settings_fn=username_from_settings_fn,
        username_from_runtime_log_fn=username_from_runtime_log_fn,
    )

    sessions = build_sessions(
        args,
        session_identity_cls=session_identity_cls,
        chat_a=chat_a,
        user_a=user_a_int,
        thread_a=thread_a,
        chat_b=chat_b,
        user_b=user_b,
        thread_b=thread_b,
        chat_c=chat_c,
        user_c=user_c,
        thread_c=thread_c,
    )
    runtime_partition_mode = resolve_runtime_partition_mode_fn(log_file)
    sessions = apply_runtime_partition_defaults(sessions, runtime_partition_mode)
    ensure_distinct_session_identity(
        sessions,
        runtime_partition_mode=runtime_partition_mode,
        expected_session_keys_fn=expected_session_keys_fn,
        expected_session_key_fn=expected_session_key_fn,
    )
    secret_token = resolve_secret_token(
        args,
        telegram_webhook_secret_token_fn=telegram_webhook_secret_token_fn,
    )
    merged_forbidden = merge_forbidden_regexes(
        default_forbid_log_regexes,
        tuple(args.forbid_log_regex),
    )

    return build_runner_config(
        args,
        runner_config_cls=runner_config_cls,
        complexity_requirement_cls=complexity_requirement_cls,
        quality_requirement_cls=quality_requirement_cls,
        dataset_path=dataset_path,
        blackbox_script=blackbox_script,
        log_file=log_file,
        username=username,
        secret_token=secret_token,
        runtime_partition_mode=runtime_partition_mode,
        sessions=sessions,
        forbid_log_regexes=merged_forbidden,
    )
