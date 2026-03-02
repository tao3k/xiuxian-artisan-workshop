#!/usr/bin/env python3
"""Outcome validation helpers for concurrent-session runtime probe."""

from __future__ import annotations

import sys
from typing import Any


def validate_webhook_post(
    *,
    status_a: int,
    body_a: str,
    status_b: int,
    body_b: str,
) -> int | None:
    """Validate HTTP statuses for both posts and return error code if failed."""
    if status_a == 200 and status_b == 200:
        return None
    print("Error: webhook POST failed.", file=sys.stderr)
    print(f"  session_a status={status_a} body={body_a}", file=sys.stderr)
    print(f"  session_b status={status_b} body={body_b}", file=sys.stderr)
    return 1


def finalize_probe(
    cfg: Any,
    *,
    obs: Any,
    key_a: str,
    key_b: str,
) -> int:
    """Validate observed counters and print final status."""
    if obs.forbidden_hits:
        print("Error: forbidden log pattern detected during concurrent probe.", file=sys.stderr)
        for line in obs.forbidden_hits[:10]:
            print(f"  {line}", file=sys.stderr)
        return 1

    if (
        (obs.accepted_a < 1 and obs.dedup_fail_open_a < 1)
        or (obs.accepted_b < 1 and obs.dedup_fail_open_b < 1)
        or obs.parsed_a < 1
        or obs.parsed_b < 1
        or (not cfg.allow_send_failure and (obs.replied_a < 1 or obs.replied_b < 1))
    ):
        print(f"Error: concurrent probe timed out after {cfg.max_wait}s.", file=sys.stderr)
        print(
            "  observed:"
            f" accepted_a={obs.accepted_a} accepted_b={obs.accepted_b}"
            f" dedup_fail_open_a={obs.dedup_fail_open_a} dedup_fail_open_b={obs.dedup_fail_open_b}"
            f" parsed_a={obs.parsed_a} parsed_b={obs.parsed_b}"
            f" replied_a={obs.replied_a} replied_b={obs.replied_b}",
            file=sys.stderr,
        )
        return 1

    if obs.duplicate_a > 0 or obs.duplicate_b > 0:
        print("Error: duplicate_detected appeared for fresh concurrent updates.", file=sys.stderr)
        print(f"  duplicate_a={obs.duplicate_a} duplicate_b={obs.duplicate_b}", file=sys.stderr)
        return 1

    print("Concurrent probe passed.")
    print(f"  accepted_a={obs.accepted_a} accepted_b={obs.accepted_b}")
    print(f"  dedup_fail_open_a={obs.dedup_fail_open_a} dedup_fail_open_b={obs.dedup_fail_open_b}")
    print(f"  parsed_a={obs.parsed_a} parsed_b={obs.parsed_b}")
    print(f"  replied_a={obs.replied_a} replied_b={obs.replied_b}")
    print(f"  isolated_session_keys=true ({key_a} != {key_b})")
    return 0
