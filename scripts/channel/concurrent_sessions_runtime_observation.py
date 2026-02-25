#!/usr/bin/env python3
"""Observation aggregation for concurrent session probes."""

from __future__ import annotations

import re
from typing import Any


def collect_observation(
    lines: list[str],
    *,
    update_a: int,
    update_b: int,
    key_a_candidates: tuple[str, ...],
    key_b_candidates: tuple[str, ...],
    forbid_log_regexes: tuple[str, ...],
    strip_ansi_fn: Any,
    session_key_re: re.Pattern[str],
    observation_cls: Any,
) -> Any:
    """Aggregate dedup/parse/reply counters from observed logs."""
    normalized = [strip_ansi_fn(line) for line in lines]
    key_a_set = set(key_a_candidates)
    key_b_set = set(key_b_candidates)
    forbidden_hits: list[str] = []
    for pattern in forbid_log_regexes:
        regex = re.compile(pattern)
        for line in normalized:
            if regex.search(line):
                forbidden_hits.append(line)

    def _count(predicate: Any) -> int:
        return sum(1 for line in normalized if predicate(line))

    def _session_key_in(line: str, candidates: set[str]) -> bool:
        match = session_key_re.search(line)
        return bool(match and match.group(1) in candidates)

    return observation_cls(
        accepted_a=_count(
            lambda line: f"update_id={update_a}" in line
            and 'event="telegram.dedup.update_accepted"' in line
        ),
        accepted_b=_count(
            lambda line: f"update_id={update_b}" in line
            and 'event="telegram.dedup.update_accepted"' in line
        ),
        dedup_fail_open_a=_count(
            lambda line: f"update_id={update_a}" in line and "Webhook dedup check failed" in line
        ),
        dedup_fail_open_b=_count(
            lambda line: f"update_id={update_b}" in line and "Webhook dedup check failed" in line
        ),
        duplicate_a=_count(
            lambda line: f"update_id={update_a}" in line
            and 'event="telegram.dedup.duplicate_detected"' in line
        ),
        duplicate_b=_count(
            lambda line: f"update_id={update_b}" in line
            and 'event="telegram.dedup.duplicate_detected"' in line
        ),
        parsed_a=_count(
            lambda line: "Parsed message, forwarding to agent" in line
            and _session_key_in(line, key_a_set)
        ),
        parsed_b=_count(
            lambda line: "Parsed message, forwarding to agent" in line
            and _session_key_in(line, key_b_set)
        ),
        replied_a=_count(
            lambda line: "telegram command reply sent" in line
            and 'event="telegram.command.session_status_json.replied"' in line
            and _session_key_in(line, key_a_set)
        ),
        replied_b=_count(
            lambda line: "telegram command reply sent" in line
            and 'event="telegram.command.session_status_json.replied"' in line
            and _session_key_in(line, key_b_set)
        ),
        forbidden_hits=tuple(forbidden_hits),
    )
