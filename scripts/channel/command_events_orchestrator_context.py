#!/usr/bin/env python3
"""Context setup helpers for command event probe orchestration."""

from __future__ import annotations

import sys
import time
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from command_events_orchestrator_inputs import (
    resolve_admin_user_id,
    resolve_topic_thread_inputs,
    validate_basic_args,
)


@dataclass(frozen=True)
class OrchestratorContext:
    """Resolved inputs and runtime state required by orchestration flow."""

    output_json: Path
    output_markdown: Path
    started_dt: datetime
    started_mono: float
    attempts: list[Any]
    suites: tuple[str, ...]
    secret_token: str
    username: str
    admin_user_id: int | None
    allow_chat_ids: tuple[str, ...]
    group_chat_id: int
    group_thread_id: int | None
    topic_thread_pair: tuple[int, int] | None
    runtime_partition_mode: str | None
    blackbox_script: Path


def prepare_orchestrator_context(
    args: Any,
    *,
    script_file: str,
    parse_optional_int_env_fn: Any,
    group_profile_int_fn: Any,
    resolve_allow_chat_ids_fn: Any,
    resolve_group_chat_id_fn: Any,
    resolve_topic_thread_pair_fn: Any,
    resolve_runtime_partition_mode_fn: Any,
    infer_group_thread_id_from_runtime_log_fn: Any,
    telegram_webhook_secret_token_fn: Any,
) -> tuple[OrchestratorContext | None, int | None]:
    """Resolve all shared orchestration inputs before executing probe cases."""
    validation_exit = validate_basic_args(args)
    if validation_exit is not None:
        return None, validation_exit

    output_json = Path(args.output_json).expanduser().resolve()
    output_markdown = Path(args.output_markdown).expanduser().resolve()
    started_dt = datetime.now(UTC)
    started_mono = time.monotonic()
    attempts: list[Any] = []
    suites = tuple(args.suite) if args.suite else ("all",)
    secret_token = (args.secret_token or "").strip() or (telegram_webhook_secret_token_fn() or "")
    username = args.username.strip()

    admin_user_id, admin_exit = resolve_admin_user_id(
        args,
        parse_optional_int_env_fn=parse_optional_int_env_fn,
        group_profile_int_fn=group_profile_int_fn,
    )
    if admin_exit is not None:
        return None, admin_exit

    cli_allow_chat_ids = tuple(
        token.strip() for token in args.allow_chat_id if token and token.strip()
    )
    allow_chat_ids = resolve_allow_chat_ids_fn(cli_allow_chat_ids)

    try:
        group_chat_id = resolve_group_chat_id_fn(
            explicit_group_chat_id=args.group_chat_id,
            allow_chat_ids=allow_chat_ids,
        )
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return None, 2

    group_thread_id, _group_thread_id_b, topic_thread_pair, thread_exit = (
        resolve_topic_thread_inputs(
            args,
            parse_optional_int_env_fn=parse_optional_int_env_fn,
            resolve_topic_thread_pair_fn=resolve_topic_thread_pair_fn,
        )
    )
    if thread_exit is not None:
        return None, thread_exit

    script_dir = Path(script_file).resolve().parent
    blackbox_script = script_dir / "agent_channel_blackbox.py"
    if not blackbox_script.exists():
        print(f"Error: black-box script not found: {blackbox_script}", file=sys.stderr)
        return None, 2

    runtime_partition_mode = resolve_runtime_partition_mode_fn()
    if group_thread_id is None and runtime_partition_mode == "chat_thread_user":
        inferred_thread_id = infer_group_thread_id_from_runtime_log_fn(group_chat_id)
        if inferred_thread_id is not None:
            group_thread_id = inferred_thread_id
            args.group_thread_id = inferred_thread_id

    context = OrchestratorContext(
        output_json=output_json,
        output_markdown=output_markdown,
        started_dt=started_dt,
        started_mono=started_mono,
        attempts=attempts,
        suites=suites,
        secret_token=secret_token,
        username=username,
        admin_user_id=admin_user_id,
        allow_chat_ids=allow_chat_ids,
        group_chat_id=group_chat_id,
        group_thread_id=group_thread_id,
        topic_thread_pair=topic_thread_pair,
        runtime_partition_mode=runtime_partition_mode,
        blackbox_script=blackbox_script,
    )
    return context, None
