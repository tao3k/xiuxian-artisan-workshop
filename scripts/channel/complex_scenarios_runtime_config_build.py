#!/usr/bin/env python3
"""Config assembly helpers for complex scenario runtime config."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any

from complex_scenarios_runtime_config_build_sessions import (
    build_sessions,
    ensure_distinct_session_identity,
)
from complex_scenarios_runtime_config_build_validation import (
    resolve_peer_user_ids,
    resolve_primary_identity,
    resolve_required_paths,
    validate_runtime_args,
)
from complex_scenarios_runtime_config_identity import (
    parse_numeric_user_ids,
    pick_default_peer_user_id,
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

    chat_b = int(args.chat_b if args.chat_b is not None else chat_a)
    chat_c = int(args.chat_c if args.chat_c is not None else chat_a)

    user_a_int = user_a
    user_b, user_c = resolve_peer_user_ids(
        args,
        user_a_int=user_a_int,
        parse_numeric_user_ids_fn=parse_numeric_user_ids,
        pick_default_peer_user_id_fn=pick_default_peer_user_id,
        allowed_users_from_settings_fn=allowed_users_from_settings_fn,
    )

    thread_b = args.thread_b
    thread_c = args.thread_c

    username = args.username.strip() if args.username else None
    if not username:
        username = username_from_settings_fn()
    if not username:
        username = username_from_runtime_log_fn(log_file)

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

    secret_token: str | None = args.secret_token.strip() if args.secret_token else None
    if not secret_token:
        secret_token = telegram_webhook_secret_token_fn()

    merged_forbidden = tuple(dict.fromkeys([*default_forbid_log_regexes, *args.forbid_log_regex]))

    return runner_config_cls(
        dataset_path=dataset_path,
        scenario_id=(args.scenario.strip() if args.scenario else None),
        blackbox_script=blackbox_script,
        webhook_url=args.webhook_url,
        log_file=log_file,
        username=username,
        secret_token=secret_token,
        max_wait=int(args.max_wait),
        max_idle_secs=int(args.max_idle_secs),
        max_parallel=int(args.max_parallel),
        execute_wave_parallel=bool(args.execute_wave_parallel),
        runtime_partition_mode=runtime_partition_mode,
        sessions=sessions,
        output_json=Path(args.output_json),
        output_markdown=Path(args.output_markdown),
        forbid_log_regexes=merged_forbidden,
        global_requirement=complexity_requirement_cls(
            steps=int(args.min_steps),
            dependency_edges=int(args.min_dependency_edges),
            critical_path_len=int(args.min_critical_path),
            parallel_waves=int(args.min_parallel_waves),
        ),
        global_quality_requirement=quality_requirement_cls(
            min_error_signals=int(args.min_error_signals),
            min_negative_feedback_events=int(args.min_negative_feedback_events),
            min_correction_checks=int(args.min_correction_checks),
            min_successful_corrections=int(args.min_successful_corrections),
            min_planned_hits=int(args.min_planned_hits),
            min_natural_language_steps=int(args.min_natural_language_steps),
            min_recall_credit_events=int(args.min_recall_credit_events),
            min_decay_events=int(args.min_decay_events),
        ),
    )
