#!/usr/bin/env python3
"""Config construction helpers for session matrix runtime config."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from session_matrix_config_runtime_build_identity import (
    resolve_peer_chats,
    resolve_peer_users,
    resolve_primary_identity,
    resolve_threads,
    resolve_username,
)
from session_matrix_config_runtime_build_output import build_config_output
from session_matrix_config_runtime_build_validation import (
    ensure_distinct_session_keys,
    validate_runtime_args,
)


def build_config(
    args: Any,
    *,
    config_cls: Any,
    resolve_runtime_partition_mode_fn: Any,
    group_profile_int_fn: Any,
    session_ids_from_runtime_log_fn: Any,
    username_from_settings_fn: Any,
    username_from_runtime_log_fn: Any,
    expected_session_key_fn: Any,
) -> Any:
    """Validate and construct session matrix config."""
    validate_runtime_args(
        max_wait=int(args.max_wait),
        max_idle_secs=int(args.max_idle_secs),
    )

    log_file = Path(args.log_file)
    chat_id, user_a, thread_a = resolve_primary_identity(
        args,
        log_file=log_file,
        group_profile_int_fn=group_profile_int_fn,
        session_ids_from_runtime_log_fn=session_ids_from_runtime_log_fn,
    )
    chat_b, chat_c = resolve_peer_chats(
        args,
        chat_id=chat_id,
        group_profile_int_fn=group_profile_int_fn,
    )
    runtime_partition_mode = resolve_runtime_partition_mode_fn(log_file)
    thread_a, thread_b, thread_c = resolve_threads(
        args,
        thread_a=thread_a,
        runtime_partition_mode=runtime_partition_mode,
    )
    user_b, user_c = resolve_peer_users(
        args,
        chat_id=chat_id,
        chat_b=chat_b,
        user_a=user_a,
        thread_a=thread_a,
        thread_b=thread_b,
        group_profile_int_fn=group_profile_int_fn,
    )

    key_a = expected_session_key_fn(int(chat_id), int(user_a), thread_a, runtime_partition_mode)
    key_b = expected_session_key_fn(int(chat_b), int(user_b), thread_b, runtime_partition_mode)
    key_c = expected_session_key_fn(int(chat_c), int(user_c), thread_c, runtime_partition_mode)
    ensure_distinct_session_keys(key_a, key_b, key_c)

    username = resolve_username(
        args,
        log_file=log_file,
        username_from_settings_fn=username_from_settings_fn,
        username_from_runtime_log_fn=username_from_runtime_log_fn,
    )

    return build_config_output(
        args,
        config_cls=config_cls,
        runtime_partition_mode=runtime_partition_mode,
        chat_id=chat_id,
        chat_b=chat_b,
        chat_c=chat_c,
        user_a=user_a,
        user_b=user_b,
        user_c=user_c,
        username=username,
        thread_a=thread_a,
        thread_b=thread_b,
        thread_c=thread_c,
    )
