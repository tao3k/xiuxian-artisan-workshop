#!/usr/bin/env python3
"""Session helper export bindings for channel blackbox probe."""

from __future__ import annotations

from functools import partial
from typing import Any


def normalize_session_partition(
    value: str | None,
    *,
    session_bindings_module: Any,
    normalize_partition_fn: Any,
) -> str | None:
    """Normalize optional session partition value."""
    return session_bindings_module.normalize_session_partition(
        value,
        normalize_partition_fn=normalize_partition_fn,
    )


def build_session_helpers(
    *,
    session_bindings_module: Any,
    session_keys_module: Any,
    normalize_partition_fn: Any,
    telegram_prefix: str,
    discord_prefix: str,
) -> dict[str, Any]:
    """Build session helper callables used by blackbox entry/runtime."""
    normalize_partition_helper = partial(
        normalize_session_partition,
        session_bindings_module=session_bindings_module,
        normalize_partition_fn=normalize_partition_fn,
    )
    expected_session_keys = partial(
        session_bindings_module.expected_session_keys,
        session_keys_module=session_keys_module,
        normalize_partition_fn=normalize_partition_helper,
    )
    expected_session_key = partial(
        session_bindings_module.expected_session_key,
        session_keys_module=session_keys_module,
        normalize_partition_fn=normalize_partition_helper,
    )
    expected_session_scope_values = partial(
        session_bindings_module.expected_session_scope_values,
        session_keys_module=session_keys_module,
        normalize_partition_fn=normalize_partition_helper,
    )
    expected_session_scope_prefixes = partial(
        session_bindings_module.expected_session_scope_prefixes,
        session_keys_module=session_keys_module,
        telegram_prefix=telegram_prefix,
        discord_prefix=discord_prefix,
    )
    return {
        "normalize_session_partition": normalize_partition_helper,
        "expected_session_keys": expected_session_keys,
        "expected_session_key": expected_session_key,
        "expected_session_scope_values": expected_session_scope_values,
        "expected_session_scope_prefixes": expected_session_scope_prefixes,
        "expected_recipient_key": session_bindings_module.expected_recipient_key,
    }
