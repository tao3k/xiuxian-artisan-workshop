#!/usr/bin/env python3
"""Context/setup helpers for concurrent-session runtime probe."""

from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
from typing import Any


def initialize_probe_context(
    cfg: Any,
    *,
    count_lines_fn: Any,
    expected_session_keys_fn: Any,
    build_payload_fn: Any,
    random_int_fn: Any,
    now_ns_fn: Any,
) -> dict[str, Any]:
    """Prepare IDs, session keys, payloads, and initial cursor."""
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

    return {
        "cursor": cursor,
        "update_a": update_a,
        "update_b": update_b,
        "key_a_candidates": key_a_candidates,
        "key_b_candidates": key_b_candidates,
        "key_a": key_a,
        "key_b": key_b,
        "payload_a": payload_a,
        "payload_b": payload_b,
    }


def post_concurrent_updates(
    cfg: Any,
    *,
    payload_a: bytes,
    payload_b: bytes,
    post_webhook_fn: Any,
) -> tuple[int, str, int, str]:
    """Post both payloads concurrently and return statuses/bodies."""
    with ThreadPoolExecutor(max_workers=2) as pool:
        fut_a = pool.submit(post_webhook_fn, cfg.webhook_url, payload_a, cfg.secret_token)
        fut_b = pool.submit(post_webhook_fn, cfg.webhook_url, payload_b, cfg.secret_token)
        status_a, body_a = fut_a.result()
        status_b, body_b = fut_b.result()
    return status_a, body_a, status_b, body_b


def print_probe_intro(
    cfg: Any,
    *,
    key_a: str,
    key_b: str,
    update_a: int,
    update_b: int,
    status_a: int,
    status_b: int,
) -> None:
    """Print probe request summary before runtime observation."""
    print("Concurrent probe posted.")
    print(f"  webhook_url={cfg.webhook_url}")
    print(f"  log_file={cfg.log_file}")
    print(f"  session_partition={cfg.session_partition or 'unknown'}")
    print(f"  session_a={key_a} chat={cfg.chat_id} update_id={update_a} status={status_a}")
    print(f"  session_b={key_b} chat={cfg.chat_b} update_id={update_b} status={status_b}")
    if cfg.allow_send_failure:
        print("  allow_send_failure=true")
