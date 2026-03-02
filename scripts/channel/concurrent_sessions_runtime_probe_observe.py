#!/usr/bin/env python3
"""Observation loop helpers for concurrent-session runtime probe."""

from __future__ import annotations

from typing import Any


def observe_until_done(
    cfg: Any,
    *,
    cursor: int,
    update_a: int,
    update_b: int,
    key_a_candidates: tuple[str, ...],
    key_b_candidates: tuple[str, ...],
    read_new_lines_fn: Any,
    collect_observation_fn: Any,
    observation_cls: Any,
    sleep_fn: Any,
    monotonic_fn: Any,
) -> tuple[Any, int]:
    """Observe runtime log stream until done, forbidden hit, or timeout."""
    deadline = monotonic_fn() + cfg.max_wait
    obs = observation_cls(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, ())
    observed_lines: list[str] = []
    next_cursor = cursor

    while monotonic_fn() < deadline:
        next_cursor, chunk = read_new_lines_fn(cfg.log_file, next_cursor)
        if chunk:
            observed_lines.extend(chunk)
        obs = collect_observation_fn(
            observed_lines,
            update_a=update_a,
            update_b=update_b,
            key_a_candidates=key_a_candidates,
            key_b_candidates=key_b_candidates,
            forbid_log_regexes=cfg.forbid_log_regexes,
        )
        dedup_a_ready = obs.accepted_a >= 1 or obs.dedup_fail_open_a >= 1
        dedup_b_ready = obs.accepted_b >= 1 or obs.dedup_fail_open_b >= 1
        done = (
            dedup_a_ready
            and dedup_b_ready
            and obs.parsed_a >= 1
            and obs.parsed_b >= 1
            and (cfg.allow_send_failure or (obs.replied_a >= 1 and obs.replied_b >= 1))
        )
        if done or obs.forbidden_hits:
            break
        sleep_fn(0.5)

    return obs, next_cursor
