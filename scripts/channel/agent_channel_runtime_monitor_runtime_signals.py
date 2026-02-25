#!/usr/bin/env python3
"""Signal handling helpers for runtime monitor."""

from __future__ import annotations

import signal
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import subprocess


def install_termination_handlers(
    proc: subprocess.Popen[str],
    termination: Any,
) -> dict[int, signal.Handlers]:
    """Install SIGTERM/SIGHUP forwarding handlers for the monitored process."""
    previous_signal_handlers: dict[int, signal.Handlers] = {}

    def _terminate_handler(signum: int, _frame: object) -> None:
        if termination.requested_signal is None:
            termination.requested_signal = signum
        try:
            if proc.poll() is None:
                proc.send_signal(signum)
        except ProcessLookupError:
            pass

    for signum in (signal.SIGTERM, signal.SIGHUP):
        previous_signal_handlers[signum] = signal.getsignal(signum)
        signal.signal(signum, _terminate_handler)
    return previous_signal_handlers


def restore_signal_handlers(previous_signal_handlers: dict[int, signal.Handlers]) -> None:
    """Restore previous SIGTERM/SIGHUP handlers."""
    for signum, handler in previous_signal_handlers.items():
        signal.signal(signum, handler)
