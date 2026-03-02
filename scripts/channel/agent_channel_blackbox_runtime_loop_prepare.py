#!/usr/bin/env python3
"""Preparation helpers for blackbox runtime loop."""

from __future__ import annotations

import os
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class PreparedProbe:
    """Prepared runtime probe state prior to log polling."""

    cursor: int
    update_id: int
    trace_id: str
    trace_mode: bool
    state: Any


def prepare_probe(
    cfg: Any,
    *,
    count_lines_fn: Any,
    next_update_id_fn: Any,
    build_probe_message_fn: Any,
    build_update_payload_fn: Any,
    post_webhook_update_fn: Any,
    expected_session_keys_fn: Any,
    expected_session_scope_values_fn: Any,
    expected_session_scope_prefixes_fn: Any,
    expected_session_key_fn: Any,
    expected_recipient_key_fn: Any,
    helpers_module: Any,
    http_loop_module: Any,
) -> tuple[PreparedProbe | None, int | None]:
    """Prepare probe state and send webhook payload before polling logs."""
    cfg.log_file.parent.mkdir(parents=True, exist_ok=True)
    cursor = count_lines_fn(cfg.log_file)

    update_id = next_update_id_fn(cfg.strong_update_id)
    trace_id = f"bbx-{update_id}-{os.getpid()}"
    message_text = build_probe_message_fn(cfg.prompt, trace_id)

    post_error = http_loop_module.handle_webhook_post(
        cfg,
        update_id=update_id,
        message_text=message_text,
        build_update_payload_fn=build_update_payload_fn,
        post_webhook_update_fn=post_webhook_update_fn,
    )
    if post_error is not None:
        return None, post_error

    helpers_module.print_probe_intro(
        cfg,
        update_id=update_id,
        trace_id=trace_id,
        message_text=message_text,
    )
    state = helpers_module.build_probe_runtime_state(
        cfg,
        expected_session_keys_fn=expected_session_keys_fn,
        expected_session_scope_values_fn=expected_session_scope_values_fn,
        expected_session_scope_prefixes_fn=expected_session_scope_prefixes_fn,
        expected_session_key_fn=expected_session_key_fn,
        expected_recipient_key_fn=expected_recipient_key_fn,
    )
    prepared = PreparedProbe(
        cursor=cursor,
        update_id=update_id,
        trace_id=trace_id,
        trace_mode=(trace_id in message_text),
        state=state,
    )
    return prepared, None
