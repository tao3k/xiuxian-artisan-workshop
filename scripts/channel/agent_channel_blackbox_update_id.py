#!/usr/bin/env python3
"""Update-id generation helpers for channel blackbox probes."""

from __future__ import annotations

from typing import Any


def next_update_id_with_state(
    *,
    strong_update_id: bool,
    last_strong_update_id: int,
    time_module: Any,
    os_module: Any,
    secrets_module: Any,
) -> tuple[int, int]:
    """Return (update_id, new_last_strong_update_id) for this probe execution."""
    base_ms = int(time_module.time() * 1000)
    if not strong_update_id:
        return base_ms, last_strong_update_id

    # Use composed time + pid + random components so concurrent probe subprocesses
    # do not collide on update_id and get dropped by webhook dedup.
    pid_component = os_module.getpid() % 10_000
    rand_component = secrets_module.randbelow(100)
    candidate = (base_ms * 1_000_000) + (pid_component * 100) + rand_component
    if candidate <= last_strong_update_id:
        candidate = last_strong_update_id + 1
    return candidate, candidate
