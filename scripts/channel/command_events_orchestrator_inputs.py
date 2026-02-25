#!/usr/bin/env python3
"""Input validation and value resolution for command-events orchestration."""

from __future__ import annotations

import sys
from typing import Any


def validate_basic_args(args: Any) -> int | None:
    """Validate basic numeric CLI arguments; return exit code on failure."""
    if args.max_wait <= 0:
        print("Error: --max-wait must be a positive integer.", file=sys.stderr)
        return 2
    if args.max_idle_secs <= 0:
        print("Error: --max-idle-secs must be a positive integer.", file=sys.stderr)
        return 2
    if args.matrix_retries < 0:
        print("Error: --matrix-retries must be a non-negative integer.", file=sys.stderr)
        return 2
    if args.matrix_backoff_secs < 0:
        print("Error: --matrix-backoff-secs must be >= 0.", file=sys.stderr)
        return 2
    return None


def resolve_admin_user_id(
    args: Any,
    *,
    parse_optional_int_env_fn: Any,
    group_profile_int_fn: Any,
) -> tuple[int | None, int | None]:
    """Resolve admin user id from args, env fallback, and group profile."""
    admin_user_id = args.admin_user_id
    if admin_user_id is None:
        try:
            admin_user_id = parse_optional_int_env_fn("OMNI_TEST_ADMIN_USER_ID")
        except ValueError as error:
            print(f"Error: {error}", file=sys.stderr)
            return None, 2
    if admin_user_id is None:
        try:
            admin_user_id = group_profile_int_fn("OMNI_TEST_USER_ID")
        except ValueError as error:
            print(f"Error: {error}", file=sys.stderr)
            return None, 2
    return admin_user_id, None


def resolve_topic_thread_inputs(
    args: Any,
    *,
    parse_optional_int_env_fn: Any,
    resolve_topic_thread_pair_fn: Any,
) -> tuple[int | None, int | None, tuple[int, int] | None, int | None]:
    """Resolve group thread inputs and normalized topic pair."""
    group_thread_id = args.group_thread_id
    if group_thread_id is None:
        try:
            group_thread_id = parse_optional_int_env_fn("OMNI_TEST_GROUP_THREAD_ID")
        except ValueError as error:
            print(f"Error: {error}", file=sys.stderr)
            return None, None, None, 2

    group_thread_id_b = args.group_thread_id_b
    if group_thread_id_b is None:
        try:
            group_thread_id_b = parse_optional_int_env_fn("OMNI_TEST_GROUP_THREAD_B")
        except ValueError as error:
            print(f"Error: {error}", file=sys.stderr)
            return None, None, None, 2

    try:
        topic_thread_pair = resolve_topic_thread_pair_fn(
            primary_thread_id=group_thread_id,
            secondary_thread_id=group_thread_id_b,
        )
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return None, None, None, 2

    if topic_thread_pair is not None:
        group_thread_id, group_thread_id_b = topic_thread_pair

    args.group_thread_id = group_thread_id
    args.group_thread_id_b = group_thread_id_b
    return group_thread_id, group_thread_id_b, topic_thread_pair, None
