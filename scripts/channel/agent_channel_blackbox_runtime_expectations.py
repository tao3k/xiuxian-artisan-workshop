#!/usr/bin/env python3
"""Expectation and scope matching helpers for blackbox runtime probes."""

from __future__ import annotations

from agent_channel_blackbox_runtime_expectations_core import (
    all_expectations_satisfied,
    event_matches_expectations,
    latest_json_summary_for_event,
    missing_expectations,
)
from agent_channel_blackbox_runtime_expectations_marking import (
    mark_event_expectation,
    mark_expect_bot_patterns,
    mark_expect_log_patterns,
    mark_reply_json_expectation,
)
from agent_channel_blackbox_runtime_expectations_selection import (
    pick_target_command_reply_observation,
    pick_target_json_summary_observation,
    validate_target_session_scope,
)
from agent_channel_blackbox_runtime_expectations_targeting import (
    event_line_matches_target_recipient,
    observation_matches_target_recipient,
    observation_matches_target_scope,
    recipient_matches_target,
    reply_json_field_matches,
)

__all__ = [
    "all_expectations_satisfied",
    "event_line_matches_target_recipient",
    "event_matches_expectations",
    "latest_json_summary_for_event",
    "mark_event_expectation",
    "mark_expect_bot_patterns",
    "mark_expect_log_patterns",
    "mark_reply_json_expectation",
    "missing_expectations",
    "observation_matches_target_recipient",
    "observation_matches_target_scope",
    "pick_target_command_reply_observation",
    "pick_target_json_summary_observation",
    "recipient_matches_target",
    "reply_json_field_matches",
    "validate_target_session_scope",
]
