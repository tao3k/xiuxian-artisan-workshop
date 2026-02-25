#!/usr/bin/env python3
"""Log and runtime helper bindings for memory CI gate."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def ensure_parent_dirs(*paths: Path) -> None:
    """Ensure parent directories exist for all provided paths."""
    for path in paths:
        path.parent.mkdir(parents=True, exist_ok=True)


def read_tail_text(path: Path, *, tail_bytes: int, read_log_tail_text_fn: Any) -> str:
    """Read a bounded text tail from log file."""
    return read_log_tail_text_fn(path, tail_bytes=tail_bytes)


def read_tail(
    path: Path,
    *,
    max_lines: int = 80,
    runtime_module: Any,
    read_log_tail_text_fn: Any,
    tail_bytes: int,
) -> str:
    """Read last max_lines from log via runtime helper."""
    return runtime_module.read_tail(
        path,
        max_lines=max_lines,
        read_log_tail_text_fn=lambda tail_path: read_tail_text(
            tail_path,
            tail_bytes=tail_bytes,
            read_log_tail_text_fn=read_log_tail_text_fn,
        ),
    )


def count_log_event(
    path: Path,
    event_name: str,
    *,
    runtime_module: Any,
    iter_log_lines_fn: Any,
) -> int:
    """Count matching log events via runtime helper."""
    return runtime_module.count_log_event(
        path,
        event_name,
        iter_log_lines_fn=iter_log_lines_fn,
    )


def wait_for_log_regex(
    path: Path,
    pattern: str,
    *,
    timeout_secs: int,
    process: Any = None,
    runtime_module: Any,
    read_tail_fn: Any,
    init_log_cursor_fn: Any,
    read_new_log_lines_with_cursor_fn: Any,
) -> None:
    """Wait until regex appears in log stream."""
    runtime_module.wait_for_log_regex(
        path,
        pattern,
        timeout_secs=timeout_secs,
        process=process,
        read_tail_fn=read_tail_fn,
        init_log_cursor_fn=init_log_cursor_fn,
        read_new_log_lines_with_cursor_fn=read_new_log_lines_with_cursor_fn,
    )


def start_background_process(
    cmd: list[str],
    *,
    cwd: Path,
    env: dict[str, str],
    log_file: Path,
    title: str,
    runtime_module: Any,
    ensure_parent_dirs_fn: Any,
) -> tuple[Any, Any]:
    """Start background process through runtime helper."""
    return runtime_module.start_background_process(
        cmd,
        cwd=cwd,
        env=env,
        log_file=log_file,
        title=title,
        ensure_parent_dirs_fn=ensure_parent_dirs_fn,
    )
