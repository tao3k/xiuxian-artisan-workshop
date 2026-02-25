#!/usr/bin/env python3
"""Shared runtime invocation helpers for admin isolation probes."""

from __future__ import annotations

from typing import Any


def run_isolation_case(
    *,
    run_case_with_retry_fn: Any,
    blackbox_script: Any,
    case: Any,
    username: str,
    allow_chat_ids: tuple[str, ...],
    max_wait: int,
    max_idle_secs: int,
    secret_token: str,
    retries: int,
    backoff_secs: float,
    attempt_records: list[Any],
    mode_label: str,
    runtime_partition_mode: str | None,
) -> int:
    """Execute one admin isolation probe case with shared runtime parameters."""
    return run_case_with_retry_fn(
        blackbox_script=blackbox_script,
        case=case,
        username=username,
        allow_chat_ids=allow_chat_ids,
        max_wait=max_wait,
        max_idle_secs=max_idle_secs,
        secret_token=secret_token,
        retries=retries,
        backoff_secs=backoff_secs,
        attempt_records=attempt_records,
        mode_label=mode_label,
        runtime_partition_mode=runtime_partition_mode,
    )
