#!/usr/bin/env python3
"""
Black-box Telegram webhook probe for local omni-agent channel runtime.

This probe posts one synthetic Telegram update to local webhook endpoint, then waits for:
  - inbound log marker:  ← User: "[bbx-...] ..."
  - outbound log marker: → Bot: "..."
"""

from __future__ import annotations

import importlib
import os
import secrets
import sys
import time
from pathlib import Path
from typing import Any

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

try:
    from scripts.channel.agent_channel_blackbox_module_bindings import load_module_bindings
except ModuleNotFoundError:
    from agent_channel_blackbox_module_bindings import load_module_bindings

_entry_context = importlib.import_module("agent_channel_blackbox_entry_context")
_context_builder = importlib.import_module("agent_channel_blackbox_context_builder")
_exports = importlib.import_module("agent_channel_blackbox_exports")
_log_exports = importlib.import_module("agent_channel_blackbox_log_exports")
_update_id = importlib.import_module("agent_channel_blackbox_update_id")

_MODULES = load_module_bindings(__file__)
_PARSING = _MODULES.parsing_module
_SESSION_BINDINGS = _MODULES.session_bindings_module
_ENTRY_BINDINGS = _MODULES.entry_bindings_module
_ENTRY_FLOW = _MODULES.entry_flow_module
_LOG_BINDINGS = _MODULES.log_bindings_module
_SharedLogCursor = _MODULES.shared_log_cursor_cls
_shared_init_log_cursor = _MODULES.shared_init_log_cursor
_shared_read_new_log_lines_with_cursor = _MODULES.shared_read_new_log_lines_with_cursor
ProbeConfig = _MODULES.probe_config_cls

DISCORD_SESSION_SCOPE_PREFIX = _MODULES.discord_session_scope_prefix
TELEGRAM_SESSION_SCOPE_PREFIX = _MODULES.telegram_session_scope_prefix
ERROR_PATTERNS = _MODULES.error_patterns
MCP_OBSERVABILITY_EVENTS = _MODULES.mcp_observability_events
MCP_WAITING_EVENTS = _MODULES.mcp_waiting_events
TARGET_SESSION_SCOPE_PLACEHOLDER = _MODULES.target_session_scope_placeholder

_exports.apply_compat_exports(
    globals(),
    modules=_MODULES,
    parsing_module=_PARSING,
    session_bindings_module=_SESSION_BINDINGS,
    session_exports_module=_MODULES.session_exports_module,
    telegram_prefix=TELEGRAM_SESSION_SCOPE_PREFIX,
    discord_prefix=DISCORD_SESSION_SCOPE_PREFIX,
)
_log_exports.apply_log_exports(globals(), log_bindings_module=_LOG_BINDINGS)


def _build_entry_context() -> Any:
    return _context_builder.build_entry_context(
        namespace=globals(),
        modules=_MODULES,
        entry_flow_module=_ENTRY_FLOW,
        entry_bindings_module=_ENTRY_BINDINGS,
        probe_config_cls=ProbeConfig,
        target_session_scope_placeholder=TARGET_SESSION_SCOPE_PLACEHOLDER,
    )


def parse_args() -> Any:
    return _entry_context.parse_args(_build_entry_context())


def build_config(args: Any) -> ProbeConfig:
    return _entry_context.build_config(args, _build_entry_context())


_LAST_STRONG_UPDATE_ID = 0


def next_update_id(strong_update_id: bool) -> int:
    global _LAST_STRONG_UPDATE_ID
    update_id, _LAST_STRONG_UPDATE_ID = _update_id.next_update_id_with_state(
        strong_update_id=strong_update_id,
        last_strong_update_id=_LAST_STRONG_UPDATE_ID,
        time_module=time,
        os_module=os,
        secrets_module=secrets,
    )
    return update_id


def run_probe(cfg: ProbeConfig) -> int:
    return _entry_context.run_probe(cfg, _build_entry_context())


def main() -> int:
    return _ENTRY_FLOW.run_main(
        entry_bindings_module=_ENTRY_BINDINGS,
        parse_args_fn=parse_args,
        build_config_fn=build_config,
        run_probe_fn=run_probe,
    )


if __name__ == "__main__":
    raise SystemExit(main())
