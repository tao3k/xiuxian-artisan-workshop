#!/usr/bin/env python3
"""Config and scenario-step builders for session matrix runner."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from session_matrix_config_args import parse_args as _parse_args_impl
from session_matrix_config_runtime import (
    build_config as _build_config_impl,
)
from session_matrix_config_runtime import (
    resolve_runtime_partition_mode as _resolve_runtime_partition_mode_impl,
)
from session_matrix_config_runtime import (
    session_context_result_fields as _session_context_result_fields_impl,
)
from session_matrix_config_runtime import (
    session_memory_result_fields as _session_memory_result_fields_impl,
)

if TYPE_CHECKING:
    import argparse
    from pathlib import Path


def parse_args(*, webhook_url_default: str) -> argparse.Namespace:
    """Parse session matrix CLI arguments."""
    return _parse_args_impl(webhook_url_default=webhook_url_default)


def resolve_runtime_partition_mode(
    log_file: Path,
    *,
    normalize_telegram_session_partition_mode_fn: Any,
    session_partition_mode_from_runtime_log_fn: Any,
    telegram_session_partition_mode_fn: Any,
) -> str | None:
    """Resolve runtime partition mode from override/log/settings chain."""
    return _resolve_runtime_partition_mode_impl(
        log_file,
        normalize_telegram_session_partition_mode_fn=normalize_telegram_session_partition_mode_fn,
        session_partition_mode_from_runtime_log_fn=session_partition_mode_from_runtime_log_fn,
        telegram_session_partition_mode_fn=telegram_session_partition_mode_fn,
    )


def session_context_result_fields(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None,
    *,
    expected_session_key_fn: Any,
) -> tuple[str, ...]:
    """Build expected JSON fields for `/session json` result."""
    return _session_context_result_fields_impl(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        expected_session_key_fn=expected_session_key_fn,
    )


def session_memory_result_fields(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None,
    *,
    expected_session_key_fn: Any,
) -> tuple[str, ...]:
    """Build expected JSON fields for `/session memory json` result."""
    return _session_memory_result_fields_impl(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        expected_session_key_fn=expected_session_key_fn,
    )


def build_config(
    args: argparse.Namespace,
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
    return _build_config_impl(
        args,
        config_cls=config_cls,
        resolve_runtime_partition_mode_fn=resolve_runtime_partition_mode_fn,
        group_profile_int_fn=group_profile_int_fn,
        session_ids_from_runtime_log_fn=session_ids_from_runtime_log_fn,
        username_from_settings_fn=username_from_settings_fn,
        username_from_runtime_log_fn=username_from_runtime_log_fn,
        expected_session_key_fn=expected_session_key_fn,
    )
