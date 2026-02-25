#!/usr/bin/env python3
"""Valkey retry checks."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from event_sequence_checks_core import Reporter


def check_valkey_retry(
    reporter: Reporter,
    *,
    count_retry_failed: int,
    count_retry_succeeded: int,
) -> None:
    """Validate retry failure/recovery observability."""
    if count_retry_failed > 0:
        if count_retry_succeeded > 0:
            reporter.emit_pass(
                "valkey retry recovery observed "
                f"(failed={count_retry_failed}, succeeded={count_retry_succeeded})"
            )
        else:
            reporter.emit_warn(
                "valkey retries failed without observed recovery "
                f"(retry_failed={count_retry_failed})"
            )
    else:
        reporter.emit_pass("no valkey retry failures observed")
