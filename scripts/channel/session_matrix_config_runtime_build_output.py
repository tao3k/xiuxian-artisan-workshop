#!/usr/bin/env python3
"""Output model construction helpers for session matrix runtime config build."""

from __future__ import annotations

from pathlib import Path
from typing import Any


def build_config_output(
    args: Any,
    *,
    config_cls: Any,
    runtime_partition_mode: str | None,
    chat_id: int,
    chat_b: int,
    chat_c: int,
    user_a: int,
    user_b: int,
    user_c: int,
    username: str | None,
    thread_a: int | None,
    thread_b: int | None,
    thread_c: int | None,
) -> Any:
    """Create final typed config object."""
    return config_cls(
        max_wait=int(args.max_wait),
        max_idle_secs=int(args.max_idle_secs),
        webhook_url=args.webhook_url,
        log_file=Path(args.log_file),
        chat_id=int(chat_id),
        chat_b=int(chat_b),
        chat_c=int(chat_c),
        user_a=int(user_a),
        user_b=int(user_b),
        user_c=int(user_c),
        username=username,
        thread_a=thread_a,
        thread_b=thread_b,
        thread_c=thread_c,
        mixed_plain_prompt=args.mixed_plain_prompt.strip(),
        secret_token=(args.secret_token.strip() if args.secret_token else None),
        output_json=Path(args.output_json),
        output_markdown=Path(args.output_markdown),
        forbid_log_regexes=tuple(args.forbid_log_regex),
        session_partition=runtime_partition_mode,
    )
