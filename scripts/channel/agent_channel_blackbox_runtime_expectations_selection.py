#!/usr/bin/env python3
"""Target observation selection + scope validation for blackbox probes."""

from __future__ import annotations

from typing import Any

from agent_channel_blackbox_runtime_expectations_core import event_matches_expectations
from agent_channel_blackbox_runtime_expectations_targeting import (
    observation_matches_target_recipient,
    observation_matches_target_scope,
)


def pick_target_command_reply_observation(
    cfg: Any,
    state: Any,
) -> dict[str, object] | None:
    """Pick best command-reply observation for target recipient/scope."""
    fallback_recipient_only: dict[str, object] | None = None
    for obs in state.command_reply_observations:
        event = str(obs.get("event") or "")
        if not event_matches_expectations(cfg, event):
            continue
        if observation_matches_target_scope(state, obs):
            return obs
        if fallback_recipient_only is None and observation_matches_target_recipient(state, obs):
            fallback_recipient_only = obs
    return fallback_recipient_only


def pick_target_json_summary_observation(
    cfg: Any,
    state: Any,
) -> dict[str, str] | None:
    """Pick best reply-json summary observation for target recipient/scope."""
    fallback_recipient_only: dict[str, str] | None = None
    for obs in state.json_reply_summary_observations:
        event = str(obs.get("event") or "")
        if not event_matches_expectations(cfg, event):
            continue
        if observation_matches_target_scope(state, obs):
            return obs
        if fallback_recipient_only is None and observation_matches_target_recipient(state, obs):
            fallback_recipient_only = obs
    return fallback_recipient_only


def validate_target_session_scope(cfg: Any, state: Any) -> tuple[bool, str]:
    """Validate target-scoped command reply/json observations."""
    if not cfg.expect_events and not cfg.expect_reply_json_fields:
        return True, ""
    target_reply = pick_target_command_reply_observation(cfg, state)
    if target_reply:
        observed_session = str(target_reply.get("session_key") or "")
        if observed_session and observed_session not in state.expected_sessions:
            return (
                False,
                "command_reply "
                f"event={target_reply.get('event')} recipient={target_reply.get('recipient')} "
                f"observed_session_key={observed_session}",
            )
        return True, ""
    target_summary = pick_target_json_summary_observation(cfg, state)
    if target_summary:
        observed_session_key = str(target_summary.get("session_key") or "")
        if observed_session_key and observed_session_key not in state.expected_sessions:
            return (
                False,
                "command_reply_json_summary "
                f"event={target_summary.get('event')} recipient={target_summary.get('recipient')} "
                f"observed_session_key={observed_session_key}",
            )
        observed_session_scope = str(target_summary.get("json_session_scope") or "")
        if observed_session_scope and observed_session_scope not in state.expected_session_scopes:
            return (
                False,
                "command_reply_json_summary "
                f"event={target_summary.get('event')} recipient={target_summary.get('recipient')} "
                f"observed_json_session_scope={observed_session_scope}",
            )
        return True, ""

    requires_target_observation = bool(cfg.expect_reply_json_fields) or any(
        ".command." in event and event.endswith(".replied") for event in cfg.expect_events
    )
    if requires_target_observation:
        return (
            False,
            "missing target-scoped command reply/json observation "
            f"for chat_id={cfg.chat_id} session_key={state.expected_session}",
        )
    return True, ""
