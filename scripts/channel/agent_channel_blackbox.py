#!/usr/bin/env python3
"""
Black-box Telegram webhook probe for local omni-agent channel runtime.

This probe posts one synthetic Telegram update to local webhook endpoint, then waits for:
  - inbound log marker:  ← User: "[bbx-...] ..."
  - outbound log marker: → Bot: "..."
"""

from __future__ import annotations

import os
import secrets
import time
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path

try:
    from scripts.channel.agent_channel_blackbox_module_bindings import load_module_bindings
except ModuleNotFoundError:
    from agent_channel_blackbox_module_bindings import load_module_bindings

_MODULES = load_module_bindings(__file__)
_PARSING = _MODULES.parsing_module
_SESSION_BINDINGS = _MODULES.session_bindings_module
_ENTRY_BINDINGS = _MODULES.entry_bindings_module
_ENTRY_FLOW = _MODULES.entry_flow_module
_LOG_BINDINGS = _MODULES.log_bindings_module
_SESSION_EXPORTS = _MODULES.session_exports_module
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

strip_ansi = _PARSING.strip_ansi
extract_event_token = _PARSING.extract_event_token
extract_session_key_token = _PARSING.extract_session_key_token
parse_log_tokens = _PARSING.parse_log_tokens
parse_expected_field = _PARSING.parse_expected_field
parse_allow_chat_ids = _PARSING.parse_allow_chat_ids
parse_command_reply_event_line = _PARSING.parse_command_reply_event_line
parse_command_reply_json_summary_line = _PARSING.parse_command_reply_json_summary_line
telegram_send_retry_grace_seconds = _PARSING.telegram_send_retry_grace_seconds


def parse_args() -> Any:
    return _ENTRY_FLOW.parse_args(
        entry_bindings_module=_ENTRY_BINDINGS,
        config_module=_MODULES.config_module,
        default_telegram_webhook_url_fn=_MODULES.default_telegram_webhook_url,
        target_session_scope_placeholder=TARGET_SESSION_SCOPE_PLACEHOLDER,
    )


def count_lines(path: Path) -> int:
    return _LOG_BINDINGS.count_lines(path, init_log_cursor_fn=_shared_init_log_cursor)


def read_new_lines(path: Path, cursor: int) -> tuple[int, list[str]]:
    return _LOG_BINDINGS.read_new_lines(
        path,
        cursor,
        read_new_log_lines_with_cursor_fn=_shared_read_new_log_lines_with_cursor,
        shared_log_cursor_cls=_SharedLogCursor,
    )


def tail_lines(path: Path, n: int) -> list[str]:
    return _LOG_BINDINGS.tail_lines(path, n, tail_log_lines_fn=_MODULES.shared_tail_log_lines)


# Backward-compatible aliases for existing test-kit imports.
infer_ids_from_log = _MODULES.session_ids_from_runtime_log
infer_username_from_log = _MODULES.username_from_runtime_log
username_from_settings = _MODULES.username_from_settings
build_update_payload = _MODULES.config_module.build_update_payload
build_probe_message = _MODULES.config_module.build_probe_message

_SESSION_HELPERS = _SESSION_EXPORTS.build_session_helpers(
    session_bindings_module=_SESSION_BINDINGS,
    session_keys_module=_MODULES.session_keys_module,
    normalize_partition_fn=_MODULES.normalize_telegram_session_partition_mode,
    telegram_prefix=TELEGRAM_SESSION_SCOPE_PREFIX,
    discord_prefix=DISCORD_SESSION_SCOPE_PREFIX,
)
normalize_session_partition = _SESSION_HELPERS["normalize_session_partition"]
expected_session_keys = _SESSION_HELPERS["expected_session_keys"]
expected_session_key = _SESSION_HELPERS["expected_session_key"]
expected_session_scope_values = _SESSION_HELPERS["expected_session_scope_values"]
expected_session_scope_prefixes = _SESSION_HELPERS["expected_session_scope_prefixes"]
expected_recipient_key = _SESSION_HELPERS["expected_recipient_key"]
post_webhook_update = _MODULES.http_module.post_webhook_update


def build_config(args: Any) -> ProbeConfig:
    return _ENTRY_FLOW.build_config(
        args,
        entry_bindings_module=_ENTRY_BINDINGS,
        config_module=_MODULES.config_module,
        probe_config_cls=ProbeConfig,
        session_ids_from_runtime_log_fn=_MODULES.session_ids_from_runtime_log,
        username_from_settings_fn=username_from_settings,
        username_from_runtime_log_fn=_MODULES.username_from_runtime_log,
        parse_expected_field_fn=parse_expected_field,
        parse_allow_chat_ids_fn=parse_allow_chat_ids,
        normalize_session_partition_fn=normalize_session_partition,
        telegram_webhook_secret_token_fn=_MODULES.telegram_webhook_secret_token,
    )


_LAST_STRONG_UPDATE_ID = 0


def next_update_id(strong_update_id: bool) -> int:
    base_ms = int(time.time() * 1000)
    if not strong_update_id:
        return base_ms

    # Use composed time + pid + random components so concurrent probe subprocesses
    # do not collide on update_id and get dropped by webhook dedup.
    pid_component = os.getpid() % 10_000
    rand_component = secrets.randbelow(100)
    candidate = (base_ms * 1_000_000) + (pid_component * 100) + rand_component
    global _LAST_STRONG_UPDATE_ID
    if candidate <= _LAST_STRONG_UPDATE_ID:
        candidate = _LAST_STRONG_UPDATE_ID + 1
    _LAST_STRONG_UPDATE_ID = candidate
    return candidate


def run_probe(cfg: ProbeConfig) -> int:
    return _ENTRY_FLOW.run_probe(
        cfg,
        entry_bindings_module=_ENTRY_BINDINGS,
        runtime_module=_MODULES.runtime_module,
        count_lines_fn=count_lines,
        next_update_id_fn=next_update_id,
        build_probe_message_fn=build_probe_message,
        build_update_payload_fn=build_update_payload,
        post_webhook_update_fn=post_webhook_update,
        expected_session_keys_fn=expected_session_keys,
        expected_session_scope_values_fn=expected_session_scope_values,
        expected_session_scope_prefixes_fn=expected_session_scope_prefixes,
        expected_session_key_fn=expected_session_key,
        expected_recipient_key_fn=expected_recipient_key,
        read_new_lines_fn=read_new_lines,
        tail_lines_fn=tail_lines,
        strip_ansi_fn=strip_ansi,
        extract_event_token_fn=extract_event_token,
        extract_session_key_token_fn=extract_session_key_token,
        parse_command_reply_event_line_fn=parse_command_reply_event_line,
        parse_command_reply_json_summary_line_fn=parse_command_reply_json_summary_line,
        telegram_send_retry_grace_seconds_fn=telegram_send_retry_grace_seconds,
        parse_log_tokens_fn=parse_log_tokens,
        error_patterns=ERROR_PATTERNS,
        mcp_observability_events=MCP_OBSERVABILITY_EVENTS,
        mcp_waiting_events=MCP_WAITING_EVENTS,
        target_session_scope_placeholder=TARGET_SESSION_SCOPE_PLACEHOLDER,
    )


def main() -> int:
    return _ENTRY_FLOW.run_main(
        entry_bindings_module=_ENTRY_BINDINGS,
        parse_args_fn=parse_args,
        build_config_fn=build_config,
        run_probe_fn=run_probe,
    )


if __name__ == "__main__":
    raise SystemExit(main())
