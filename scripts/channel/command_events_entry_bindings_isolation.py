#!/usr/bin/env python3
"""Isolation-assertion bindings for command-events probe entrypoint."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def run_admin_isolation_assertions(
    *,
    blackbox_script: Path,
    matrix_chat_ids: tuple[int, ...],
    admin_user_id: int | None,
    group_thread_id: int | None,
    username: str,
    allow_chat_ids: tuple[str, ...],
    max_wait: int,
    max_idle_secs: int,
    secret_token: str,
    retries: int,
    backoff_secs: float,
    attempt_records: list[Any],
    runtime_partition_mode: str | None,
    runtime_bindings_module: Any,
    admin_isolation_module: Any,
    build_cases_fn: Any,
    run_case_with_retry_fn: Any,
    probe_case_cls: Any,
) -> int:
    """Run admin-isolation assertions across matrix chat ids."""
    return runtime_bindings_module.run_admin_isolation_assertions(
        blackbox_script=blackbox_script,
        matrix_chat_ids=matrix_chat_ids,
        admin_user_id=admin_user_id,
        group_thread_id=group_thread_id,
        username=username,
        allow_chat_ids=allow_chat_ids,
        max_wait=max_wait,
        max_idle_secs=max_idle_secs,
        secret_token=secret_token,
        retries=retries,
        backoff_secs=backoff_secs,
        attempt_records=attempt_records,
        runtime_partition_mode=runtime_partition_mode,
        admin_isolation_module=admin_isolation_module,
        build_cases_fn=build_cases_fn,
        run_case_with_retry_fn=run_case_with_retry_fn,
        probe_case_cls=probe_case_cls,
    )


def run_admin_topic_isolation_assertions(
    *,
    blackbox_script: Path,
    group_chat_id: int,
    admin_user_id: int | None,
    thread_a: int,
    thread_b: int,
    username: str,
    allow_chat_ids: tuple[str, ...],
    max_wait: int,
    max_idle_secs: int,
    secret_token: str,
    retries: int,
    backoff_secs: float,
    attempt_records: list[Any],
    runtime_partition_mode: str | None,
    runtime_bindings_module: Any,
    admin_isolation_module: Any,
    build_cases_fn: Any,
    run_case_with_retry_fn: Any,
    probe_case_cls: Any,
) -> int:
    """Run admin-topic-isolation assertions for a chat/thread pair."""
    return runtime_bindings_module.run_admin_topic_isolation_assertions(
        blackbox_script=blackbox_script,
        group_chat_id=group_chat_id,
        admin_user_id=admin_user_id,
        thread_a=thread_a,
        thread_b=thread_b,
        username=username,
        allow_chat_ids=allow_chat_ids,
        max_wait=max_wait,
        max_idle_secs=max_idle_secs,
        secret_token=secret_token,
        retries=retries,
        backoff_secs=backoff_secs,
        attempt_records=attempt_records,
        runtime_partition_mode=runtime_partition_mode,
        admin_isolation_module=admin_isolation_module,
        build_cases_fn=build_cases_fn,
        run_case_with_retry_fn=run_case_with_retry_fn,
        probe_case_cls=probe_case_cls,
    )
