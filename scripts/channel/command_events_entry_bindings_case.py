#!/usr/bin/env python3
"""Case-execution bindings for command-events probe entrypoint."""

from __future__ import annotations

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
    runtime_bindings_module: Any,
    probe_runtime_module: Any,
    resolve_runtime_partition_mode_fn: Any,
    apply_runtime_partition_defaults_fn: Any,
    run_case_fn: Any,
    transient_exit_codes: frozenset[int],
    probe_attempt_record_cls: Any,
    monotonic_fn: Any,
    sleep_fn: Any,
) -> int:
    """Run one case with retry-on-transient policy."""
    return runtime_bindings_module.run_case_with_retry(
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
        probe_runtime_module=probe_runtime_module,
        resolve_runtime_partition_mode_fn=resolve_runtime_partition_mode_fn,
        apply_runtime_partition_defaults_fn=apply_runtime_partition_defaults_fn,
        run_case_fn=run_case_fn,
        transient_exit_codes=transient_exit_codes,
        probe_attempt_record_cls=probe_attempt_record_cls,
        monotonic_fn=monotonic_fn,
        sleep_fn=sleep_fn,
    )
