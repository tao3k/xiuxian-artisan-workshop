#!/usr/bin/env python3
"""Compatibility log IO exports for agent_channel_blackbox module."""

from __future__ import annotations

from typing import Any


def apply_log_exports(namespace: dict[str, Any], *, log_bindings_module: Any) -> None:
    """Populate `count_lines/read_new_lines/tail_lines` exports with patch-friendly lookups."""

    def count_lines(path: Any) -> int:
        return log_bindings_module.count_lines(
            path, init_log_cursor_fn=namespace["_shared_init_log_cursor"]
        )

    def read_new_lines(path: Any, cursor: int) -> tuple[int, list[str]]:
        return log_bindings_module.read_new_lines(
            path,
            cursor,
            read_new_log_lines_with_cursor_fn=namespace["_shared_read_new_log_lines_with_cursor"],
            shared_log_cursor_cls=namespace["_SharedLogCursor"],
        )

    def tail_lines(path: Any, n: int) -> list[str]:
        return log_bindings_module.tail_lines(
            path, n, tail_log_lines_fn=namespace["_MODULES"].shared_tail_log_lines
        )

    namespace["count_lines"] = count_lines
    namespace["read_new_lines"] = read_new_lines
    namespace["tail_lines"] = tail_lines
