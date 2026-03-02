#!/usr/bin/env python3
"""State observation processing for blackbox probe log chunks."""

from __future__ import annotations

import sys
from typing import Any


def process_state_lines(
    cfg: Any,
    runtime_state: Any,
    loop_state: Any,
    *,
    update_id: int,
    trace_mode: bool,
    trace_id: str,
    normalized_chunk: list[str],
    extract_session_key_token_fn: Any,
    error_patterns: tuple[str, ...],
    helpers_module: Any,
) -> int | None:
    """Process chunk state transitions and fail-fast checks."""
    for line in normalized_chunk:
        if f"update_id=Some({update_id})" in line:
            loop_state.webhook_seen = True
        if str(update_id) in line and "duplicate update" in line.lower():
            loop_state.dedup_duplicate_line = line
        if trace_mode and trace_id in line:
            observed_session_key = extract_session_key_token_fn(line)
            if observed_session_key and observed_session_key not in runtime_state.expected_sessions:
                loop_state.dispatch_session_mismatch_line = line
                break

    if (
        trace_mode
        and not loop_state.seen_trace
        and any(trace_id in line for line in normalized_chunk)
    ):
        loop_state.seen_trace = True

    dispatch_start_index: int | None = None
    if not loop_state.seen_user_dispatch:
        if trace_mode:
            for index, line in enumerate(normalized_chunk):
                if "← User:" in line and trace_id in line:
                    dispatch_start_index = index
                    break
        else:
            for index, line in enumerate(normalized_chunk):
                if (
                    "← User:" in line or "Parsed message, forwarding to agent" in line
                ) and cfg.prompt in line:
                    dispatch_start_index = index
                    break
        loop_state.seen_user_dispatch = dispatch_start_index is not None

    if loop_state.seen_user_dispatch:
        relevant_lines = (
            normalized_chunk[dispatch_start_index:]
            if dispatch_start_index is not None
            else normalized_chunk
        )

        for line in relevant_lines:
            if any(pattern in line for pattern in error_patterns):
                loop_state.error_line = line
                if cfg.fail_fast_error_logs:
                    print("", file=sys.stderr)
                    print("Probe failed: fail-fast error log detected.", file=sys.stderr)
                    print(f"  line={line}", file=sys.stderr)
                    return 6

        for line in relevant_lines:
            if "→ Bot:" in line:
                loop_state.seen_bot = True
                loop_state.bot_line = line
                helpers_module.mark_expect_bot_patterns(runtime_state, line)
    return None
