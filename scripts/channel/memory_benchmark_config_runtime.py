#!/usr/bin/env python3
"""Runtime/config helpers for memory benchmark runner."""

from __future__ import annotations

import os
from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from memory_benchmark_models import BenchmarkConfig


def resolve_runtime_partition_mode(
    log_file: Path,
    *,
    normalize_telegram_session_partition_mode_fn: Any,
    session_partition_mode_from_runtime_log_fn: Any,
    telegram_session_partition_mode_fn: Any,
) -> str | None:
    """Resolve runtime session partition mode from override/log/config order."""
    override = os.environ.get("OMNI_BLACKBOX_SESSION_PARTITION_MODE", "").strip()
    normalized_override = normalize_telegram_session_partition_mode_fn(override)
    if normalized_override:
        return normalized_override

    mode_from_log = session_partition_mode_from_runtime_log_fn(log_file)
    if mode_from_log:
        return mode_from_log

    return telegram_session_partition_mode_fn()


def build_config(
    args: Any,
    *,
    config_cls: type[BenchmarkConfig],
    infer_session_ids_fn: Any,
    resolve_runtime_partition_mode_fn: Any,
) -> BenchmarkConfig:
    """Validate CLI args and build typed benchmark config."""
    modes = tuple(args.mode) if args.mode else ("baseline", "adaptive")
    if args.max_wait <= 0:
        raise ValueError("--max-wait must be a positive integer.")
    if args.max_idle_secs <= 0:
        raise ValueError("--max-idle-secs must be a positive integer.")
    if args.iterations <= 0:
        raise ValueError("--iterations must be a positive integer.")
    if not (0.0 <= args.feedback_down_threshold <= 1.0):
        raise ValueError("--feedback-down-threshold must be between 0.0 and 1.0.")

    env_chat = os.environ.get("OMNI_TEST_CHAT_ID", "").strip()
    env_user = os.environ.get("OMNI_TEST_USER_ID", "").strip()
    env_thread = os.environ.get("OMNI_TEST_THREAD_ID", "").strip()
    chat_id = args.chat_id if args.chat_id is not None else (int(env_chat) if env_chat else None)
    user_id = args.user_id if args.user_id is not None else (int(env_user) if env_user else None)
    thread_id = (
        args.thread_id if args.thread_id is not None else (int(env_thread) if env_thread else None)
    )

    config = config_cls(
        dataset_path=Path(args.dataset).expanduser().resolve(),
        log_file=Path(args.log_file).expanduser().resolve(),
        blackbox_script=Path(args.blackbox_script).expanduser().resolve(),
        chat_id=0,
        user_id=0,
        thread_id=thread_id,
        runtime_partition_mode=None,
        username=args.username.strip(),
        max_wait=args.max_wait,
        max_idle_secs=args.max_idle_secs,
        modes=modes,
        iterations=args.iterations,
        skip_reset=bool(args.skip_reset),
        output_json=Path(args.output_json).expanduser().resolve(),
        output_markdown=Path(args.output_markdown).expanduser().resolve(),
        fail_on_mcp_error=bool(args.fail_on_mcp_error),
        feedback_policy=args.feedback_policy,
        feedback_down_threshold=float(args.feedback_down_threshold),
    )

    if not config.dataset_path.exists():
        raise ValueError(f"dataset not found: {config.dataset_path}")
    if not config.blackbox_script.exists():
        raise ValueError(f"black-box script not found: {config.blackbox_script}")

    if chat_id is None or user_id is None:
        inferred_chat, inferred_user, inferred_thread = infer_session_ids_fn(config.log_file)
        if chat_id is None:
            chat_id = inferred_chat
        if user_id is None:
            user_id = inferred_user
        if config.thread_id is None:
            config.thread_id = inferred_thread
    if chat_id is None or user_id is None:
        raise ValueError(
            "chat/user id are required. Set --chat-id/--user-id (or OMNI_TEST_CHAT_ID/OMNI_TEST_USER_ID), "
            "or ensure runtime log has a recent session marker for inference."
        )

    config.chat_id = int(chat_id)
    config.user_id = int(user_id)
    config.runtime_partition_mode = resolve_runtime_partition_mode_fn(config.log_file)

    config.log_file.parent.mkdir(parents=True, exist_ok=True)
    config.output_json.parent.mkdir(parents=True, exist_ok=True)
    config.output_markdown.parent.mkdir(parents=True, exist_ok=True)
    return config
