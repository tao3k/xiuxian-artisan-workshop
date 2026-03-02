#!/usr/bin/env python3
"""Runtime probe loop for deterministic dedup events."""

from __future__ import annotations

import random
import sys
import time
from typing import Any


def run_probe(
    cfg: Any,
    *,
    count_lines_fn: Any,
    build_payload_fn: Any,
    post_webhook_update_fn: Any,
    read_new_lines_fn: Any,
    collect_stats_fn: Any,
    print_relevant_tail_fn: Any,
) -> int:
    """Run the deterministic duplicate-post probe."""
    cfg.log_file.parent.mkdir(parents=True, exist_ok=True)
    if not cfg.log_file.exists():
        cfg.log_file.touch()

    cursor = count_lines_fn(cfg.log_file)
    update_id = (time.time_ns() // 1_000) + random.randint(0, 999)
    payload = build_payload_fn(cfg, update_id)

    status_first, body_first = post_webhook_update_fn(cfg.webhook_url, payload, cfg.secret_token)
    status_second, body_second = post_webhook_update_fn(cfg.webhook_url, payload, cfg.secret_token)

    print("Dedup probe posted.")
    print(f"  update_id={update_id}")
    print(f"  webhook_url={cfg.webhook_url}")
    print(f"  log_file={cfg.log_file}")
    print(f"  first_status={status_first} second_status={status_second}")

    if status_first != 200 or status_second != 200:
        print("Error: webhook POST failed.", file=sys.stderr)
        print(f"  first_status={status_first} body={body_first}", file=sys.stderr)
        print(f"  second_status={status_second} body={body_second}", file=sys.stderr)
        return 1

    stats = {
        "accepted_count": 0,
        "duplicate_count": 0,
        "accepted_line": 0,
        "duplicate_line": 0,
        "evaluated_total": 0,
        "evaluated_true": 0,
        "evaluated_false": 0,
    }
    deadline = time.monotonic() + cfg.max_wait
    observed_lines: list[str] = []
    while time.monotonic() < deadline:
        cursor, chunk = read_new_lines_fn(cfg.log_file, cursor)
        if chunk:
            observed_lines.extend(chunk)
        stats = collect_stats_fn(observed_lines, update_id)
        if stats["accepted_count"] >= 1 and stats["duplicate_count"] >= 1:
            break
        time.sleep(1)

    if stats["accepted_count"] < 1 or stats["duplicate_count"] < 1:
        print(
            f"Error: expected dedup events were not observed within {cfg.max_wait}s.",
            file=sys.stderr,
        )
        print(
            "  update_accepted="
            f"{stats['accepted_count']} duplicate_detected={stats['duplicate_count']}",
            file=sys.stderr,
        )
        print(f"  update_id={update_id}", file=sys.stderr)
        print_relevant_tail_fn(observed_lines, update_id)
        return 1

    if stats["accepted_line"] >= stats["duplicate_line"]:
        print(f"Error: unexpected dedup event order for update_id={update_id}.", file=sys.stderr)
        print(f"  line_update_accepted={stats['accepted_line']}", file=sys.stderr)
        print(f"  line_duplicate_detected={stats['duplicate_line']}", file=sys.stderr)
        return 1

    if stats["evaluated_total"] > 0 and (
        stats["evaluated_true"] < 1 or stats["evaluated_false"] < 1
    ):
        print(
            "Warning: dedup evaluated events were observed but did not include both duplicate states.",
            file=sys.stderr,
        )
        print(
            "  evaluated_total="
            f"{stats['evaluated_total']} duplicate_true={stats['evaluated_true']} "
            f"duplicate_false={stats['evaluated_false']}",
            file=sys.stderr,
        )

    print("Dedup probe passed.")
    print(f"  update_accepted={stats['accepted_count']}")
    print(f"  duplicate_detected={stats['duplicate_count']}")
    print(f"  evaluated_total={stats['evaluated_total']}")
    print(f"  evaluated_duplicate_false={stats['evaluated_false']}")
    print(f"  evaluated_duplicate_true={stats['evaluated_true']}")
    print("  order_ok=true (accepted before duplicate)")
    return 0
