#!/usr/bin/env python3
"""Log utility tests split from memory CI gate triage suite."""

from __future__ import annotations

import threading
import time

from test_omni_agent_memory_ci_gate import count_log_event, read_tail, wait_for_log_regex


def test_read_tail_reads_last_lines_from_large_log(tmp_path) -> None:
    runtime_log = tmp_path / "runtime.log"
    with runtime_log.open("wb") as handle:
        handle.write(b"A" * 350_000)
        handle.write(b"\n")
        handle.write(b"line-1\nline-2\nline-3\n")

    tail = read_tail(runtime_log, max_lines=2)
    assert tail == "line-2\nline-3"


def test_count_log_event_handles_large_log_streaming(tmp_path) -> None:
    runtime_log = tmp_path / "runtime.log"
    with runtime_log.open("wb") as handle:
        handle.write(b"B" * 320_000)
        handle.write(b"\n")
        handle.write(b'2026-02-20T00:00:00Z WARN event="mcp.pool.call.waiting"\n')
        handle.write(b'2026-02-20T00:00:01Z WARN event="mcp.pool.call.waiting"\n')
        handle.write(b'2026-02-20T00:00:02Z WARN event="mcp.pool.connect.waiting"\n')

    assert count_log_event(runtime_log, "mcp.pool.call.waiting") == 2
    assert count_log_event(runtime_log, "mcp.pool.connect.waiting") == 1


def test_wait_for_log_regex_matches_existing_tail(tmp_path) -> None:
    runtime_log = tmp_path / "runtime.log"
    runtime_log.write_text(
        '2026-02-22T00:00:00Z INFO event="gateway.ready"\n',
        encoding="utf-8",
    )
    wait_for_log_regex(runtime_log, r'event="gateway\.ready"', timeout_secs=1)


def test_wait_for_log_regex_matches_appended_line(tmp_path) -> None:
    runtime_log = tmp_path / "runtime.log"
    runtime_log.write_text("", encoding="utf-8")

    def _append_ready_line() -> None:
        time.sleep(0.2)
        with runtime_log.open("a", encoding="utf-8") as handle:
            handle.write('2026-02-22T00:00:01Z INFO event="gateway.ready"\n')

    worker = threading.Thread(target=_append_ready_line, daemon=True)
    worker.start()
    wait_for_log_regex(runtime_log, r'event="gateway\.ready"', timeout_secs=3)
    worker.join(timeout=1)
