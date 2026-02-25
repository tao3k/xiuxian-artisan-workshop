#!/usr/bin/env python3
"""Probe execution loop for concurrent Telegram session runtime checks."""

from __future__ import annotations

import random
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from typing import Any


def run_probe(
    cfg: Any,
    *,
    count_lines_fn: Any,
    read_new_lines_fn: Any,
    expected_session_keys_fn: Any,
    build_payload_fn: Any,
    post_webhook_fn: Any,
    collect_observation_fn: Any,
    observation_cls: Any,
    random_int_fn: Any = random.randint,
    now_ns_fn: Any = time.time_ns,
    sleep_fn: Any = time.sleep,
    monotonic_fn: Any = time.monotonic,
) -> int:
    """Execute full concurrent dual-session probe."""
    cfg.log_file.parent.mkdir(parents=True, exist_ok=True)
    if not cfg.log_file.exists():
        cfg.log_file.touch()

    cursor = count_lines_fn(cfg.log_file)
    update_a = (now_ns_fn() // 1_000) + random_int_fn(0, 999)
    update_b = update_a + random_int_fn(1_000, 9_999)
    key_a_candidates = expected_session_keys_fn(
        cfg.chat_id,
        cfg.user_a,
        cfg.thread_a,
        cfg.session_partition,
    )
    key_b_candidates = expected_session_keys_fn(
        cfg.chat_b,
        cfg.user_b,
        cfg.thread_b,
        cfg.session_partition,
    )
    key_a = key_a_candidates[0]
    key_b = key_b_candidates[0]

    payload_a = build_payload_fn(
        update_id=update_a,
        chat_id=cfg.chat_id,
        user_id=cfg.user_a,
        username=cfg.username,
        prompt=cfg.prompt,
        thread_id=cfg.thread_a,
    )
    payload_b = build_payload_fn(
        update_id=update_b,
        chat_id=cfg.chat_b,
        user_id=cfg.user_b,
        username=cfg.username,
        prompt=cfg.prompt,
        thread_id=cfg.thread_b,
    )

    with ThreadPoolExecutor(max_workers=2) as pool:
        fut_a = pool.submit(post_webhook_fn, cfg.webhook_url, payload_a, cfg.secret_token)
        fut_b = pool.submit(post_webhook_fn, cfg.webhook_url, payload_b, cfg.secret_token)
        status_a, body_a = fut_a.result()
        status_b, body_b = fut_b.result()

    print("Concurrent probe posted.")
    print(f"  webhook_url={cfg.webhook_url}")
    print(f"  log_file={cfg.log_file}")
    print(f"  session_partition={cfg.session_partition or 'unknown'}")
    print(f"  session_a={key_a} chat={cfg.chat_id} update_id={update_a} status={status_a}")
    print(f"  session_b={key_b} chat={cfg.chat_b} update_id={update_b} status={status_b}")
    if cfg.allow_send_failure:
        print("  allow_send_failure=true")

    if status_a != 200 or status_b != 200:
        print("Error: webhook POST failed.", file=sys.stderr)
        print(f"  session_a status={status_a} body={body_a}", file=sys.stderr)
        print(f"  session_b status={status_b} body={body_b}", file=sys.stderr)
        return 1

    deadline = monotonic_fn() + cfg.max_wait
    obs = observation_cls(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, ())
    observed_lines: list[str] = []
    while monotonic_fn() < deadline:
        cursor, chunk = read_new_lines_fn(cfg.log_file, cursor)
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
