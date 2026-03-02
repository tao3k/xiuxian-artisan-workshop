#!/usr/bin/env python3
"""Compatibility facade for agent channel blackbox parsing helpers."""

from __future__ import annotations

import importlib

_TOKENS = importlib.import_module("agent_channel_blackbox_parsing_tokens")
_EXPECTATIONS = importlib.import_module("agent_channel_blackbox_parsing_expectations")
_COMMAND_REPLY = importlib.import_module("agent_channel_blackbox_parsing_command_reply")

strip_ansi = _TOKENS.strip_ansi
extract_event_token = _TOKENS.extract_event_token
extract_session_key_token = _TOKENS.extract_session_key_token
parse_log_tokens = _TOKENS.parse_log_tokens
parse_expected_field = _EXPECTATIONS.parse_expected_field
parse_allow_chat_ids = _EXPECTATIONS.parse_allow_chat_ids
parse_command_reply_event_line = _COMMAND_REPLY.parse_command_reply_event_line
parse_command_reply_json_summary_line = _COMMAND_REPLY.parse_command_reply_json_summary_line
telegram_send_retry_grace_seconds = _COMMAND_REPLY.telegram_send_retry_grace_seconds
