#!/usr/bin/env python3
"""Log-stat extraction for Discord ingress stress rounds."""

from __future__ import annotations


def collect_log_stats(lines: list[str]) -> dict[str, int]:
    """Extract queue-pressure and parse counters from log lines."""

    def _count(token: str) -> int:
        return sum(1 for line in lines if token in line)

    return {
        "parsed_messages": _count("discord ingress parsed message"),
        "queue_wait_events": _count('event="discord.ingress.inbound_queue_wait"'),
        "foreground_gate_wait_events": _count('event="discord.foreground.gate_wait"'),
        "inbound_queue_unavailable_events": _count("discord inbound queue unavailable"),
    }
