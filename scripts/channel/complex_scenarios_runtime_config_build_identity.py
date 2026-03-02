#!/usr/bin/env python3
"""Identity and small value resolvers for complex scenarios runtime config."""

from __future__ import annotations

from typing import Any

from complex_scenarios_runtime_config_build_validation import resolve_peer_user_ids
from complex_scenarios_runtime_config_identity import (
    parse_numeric_user_ids,
    pick_default_peer_user_id,
)


def resolve_peer_identity(
    args: Any,
    *,
    chat_a: int,
    user_a_int: int,
    allowed_users_from_settings_fn: Any,
) -> tuple[int, int, int, int, int | None, int | None]:
    """Resolve peer chat/user/thread identity values for sessions B/C."""
    chat_b = int(args.chat_b if args.chat_b is not None else chat_a)
    chat_c = int(args.chat_c if args.chat_c is not None else chat_a)
    user_b, user_c = resolve_peer_user_ids(
        args,
        user_a_int=user_a_int,
        parse_numeric_user_ids_fn=parse_numeric_user_ids,
        pick_default_peer_user_id_fn=pick_default_peer_user_id,
        allowed_users_from_settings_fn=allowed_users_from_settings_fn,
    )
    return chat_b, chat_c, user_b, user_c, args.thread_b, args.thread_c


def resolve_username(
    args: Any,
    *,
    log_file: Any,
    username_from_settings_fn: Any,
    username_from_runtime_log_fn: Any,
) -> str | None:
    """Resolve username from CLI, settings, then runtime log."""
    username = args.username.strip() if args.username else None
    if not username:
        username = username_from_settings_fn()
    if not username:
        username = username_from_runtime_log_fn(log_file)
    return username


def resolve_secret_token(args: Any, *, telegram_webhook_secret_token_fn: Any) -> str | None:
    """Resolve webhook secret from CLI first, then settings/env resolver."""
    secret_token: str | None = args.secret_token.strip() if args.secret_token else None
    if not secret_token:
        secret_token = telegram_webhook_secret_token_fn()
    return secret_token


def merge_forbidden_regexes(
    default_forbid_log_regexes: tuple[str, ...],
    extra_forbid_log_regexes: tuple[str, ...],
) -> tuple[str, ...]:
    """Merge and deduplicate default + CLI forbidden regexes."""
    return tuple(dict.fromkeys([*default_forbid_log_regexes, *extra_forbid_log_regexes]))
