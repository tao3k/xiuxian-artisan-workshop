#!/usr/bin/env python3
"""Probe execution loop for concurrent Telegram session runtime checks."""

from __future__ import annotations

import random
import time
from typing import Any

from concurrent_sessions_runtime_probe_context import (
    initialize_probe_context,
    post_concurrent_updates,
    print_probe_intro,
)
from concurrent_sessions_runtime_probe_observe import observe_until_done
from concurrent_sessions_runtime_probe_outcome import finalize_probe, validate_webhook_post


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
    ctx = initialize_probe_context(
        cfg,
        count_lines_fn=count_lines_fn,
        expected_session_keys_fn=expected_session_keys_fn,
        build_payload_fn=build_payload_fn,
        random_int_fn=random_int_fn,
        now_ns_fn=now_ns_fn,
    )

    status_a, body_a, status_b, body_b = post_concurrent_updates(
        cfg,
        payload_a=ctx["payload_a"],
        payload_b=ctx["payload_b"],
        post_webhook_fn=post_webhook_fn,
    )
    print_probe_intro(
        cfg,
        key_a=ctx["key_a"],
        key_b=ctx["key_b"],
        update_a=ctx["update_a"],
        update_b=ctx["update_b"],
        status_a=status_a,
        status_b=status_b,
    )

    post_error = validate_webhook_post(
        status_a=status_a,
        body_a=body_a,
        status_b=status_b,
        body_b=body_b,
    )
    if post_error is not None:
        return post_error

    obs, _ = observe_until_done(
        cfg,
        cursor=ctx["cursor"],
        update_a=ctx["update_a"],
        update_b=ctx["update_b"],
        key_a_candidates=ctx["key_a_candidates"],
        key_b_candidates=ctx["key_b_candidates"],
        read_new_lines_fn=read_new_lines_fn,
        collect_observation_fn=collect_observation_fn,
        observation_cls=observation_cls,
        sleep_fn=sleep_fn,
        monotonic_fn=monotonic_fn,
    )

    return finalize_probe(
        cfg,
        obs=obs,
        key_a=ctx["key_a"],
        key_b=ctx["key_b"],
    )
