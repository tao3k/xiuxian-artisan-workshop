#!/usr/bin/env python3
"""Retry/backoff probe helpers for command-events runtime."""

from __future__ import annotations

import time
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def run_case_with_retry(
    *,
    blackbox_script: Path,
    case: Any,
    username: str,
    allow_chat_ids: tuple[str, ...],
    max_wait: int,
    max_idle_secs: int,
    secret_token: str,
    retries: int,
    backoff_secs: float,
    attempt_records: list[Any] | None = None,
    mode_label: str = "default",
    runtime_partition_mode: str | None = None,
    resolve_runtime_partition_mode_fn: Any,
    apply_runtime_partition_defaults_fn: Any,
    run_case_fn: Any,
    is_transient_matrix_failure_fn: Any,
    transient_exit_codes: set[int] | frozenset[int],
    probe_attempt_record_cls: Any,
    monotonic_fn: Any = time.monotonic,
    sleep_fn: Any = time.sleep,
) -> int:
    """Run one probe case with retry/backoff for transient matrix failures."""
    effective_partition_mode = runtime_partition_mode or resolve_runtime_partition_mode_fn()
    attempts = retries + 1
    for attempt in range(attempts):
        effective_case = apply_runtime_partition_defaults_fn(case, effective_partition_mode)
        started = monotonic_fn()
        status = run_case_fn(
            blackbox_script=blackbox_script,
            case=effective_case,
            username=username,
            allow_chat_ids=allow_chat_ids,
            max_wait=max_wait,
            max_idle_secs=max_idle_secs,
            secret_token=secret_token,
            runtime_partition_mode=effective_partition_mode,
        )
        duration_ms = int((monotonic_fn() - started) * 1000)
        retry_scheduled = (
            status != 0
            and attempt < retries
            and is_transient_matrix_failure_fn(status, transient_exit_codes)
        )
        if attempt_records is not None:
            attempt_records.append(
                probe_attempt_record_cls(
                    mode=mode_label,
                    case_id=effective_case.case_id,
                    prompt=effective_case.prompt,
                    event_name=effective_case.event_name,
                    suites=effective_case.suites,
                    chat_id=effective_case.chat_id,
                    user_id=effective_case.user_id,
                    thread_id=effective_case.thread_id,
                    attempt=attempt + 1,
                    max_attempts=attempts,
                    returncode=status,
                    passed=status == 0,
                    duration_ms=duration_ms,
                    retry_scheduled=retry_scheduled,
                )
            )
        if status == 0:
            return 0
        if attempt >= retries or not is_transient_matrix_failure_fn(status, transient_exit_codes):
            return status
        wait_secs = max(0.0, backoff_secs) * (2**attempt)
        print(
            "Transient admin-matrix probe failure: "
            f"case={case.case_id} exit={status} "
            f"retry={attempt + 2}/{attempts} backoff={wait_secs:.1f}s"
        )
        if wait_secs > 0:
            sleep_fn(wait_secs)
    return 1
