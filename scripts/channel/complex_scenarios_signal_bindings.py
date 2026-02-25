#!/usr/bin/env python3
"""Signal pattern and extractor bindings for complex scenarios entrypoint."""

from __future__ import annotations

from functools import partial
from typing import Any


def build_signal_extractors(
    *,
    execution_module: Any,
    regex_module: Any,
) -> tuple[Any, Any]:
    """Build memory + MCP extractor callables with compiled regex patterns."""
    ansi_escape_re = regex_module.compile(r"\x1b\[[0-9;]*m")
    memory_planned_bias_re = regex_module.compile(
        r'event\s*=\s*"agent\.memory\.recall\.planned".*?\brecall_feedback_bias\b\s*=\s*([\-0-9.eE]+)'
    )
    memory_decision_re = regex_module.compile(
        r'event\s*=\s*"agent\.memory\.recall\.(injected|skipped)"'
    )
    memory_feedback_re = regex_module.compile(
        r'event\s*=\s*"agent\.memory\.recall\.feedback_updated".*?'
        r'feedback_source\s*=\s*"([^"]+)".*?'
        r"recall_feedback_bias_before\s*=\s*([\-0-9.eE]+).*?"
        r"recall_feedback_bias_after\s*=\s*([\-0-9.eE]+)"
    )
    memory_recall_credit_re = regex_module.compile(
        r'event\s*=\s*"agent\.memory\.recall\.credit_applied"'
    )
    memory_decay_re = regex_module.compile(r'event\s*=\s*"agent\.memory\.decay\.applied"')
    mcp_last_event_re = regex_module.compile(r"^\s*mcp_last_event=(.*)$")
    mcp_waiting_seen_re = regex_module.compile(r"^\s*mcp_waiting_seen=(true|false)$")
    mcp_event_counts_re = regex_module.compile(r"^\s*mcp_event_counts=(\{.*\})$")

    extract_memory_metrics = partial(
        execution_module.extract_memory_metrics,
        ansi_escape_re=ansi_escape_re,
        memory_planned_bias_re=memory_planned_bias_re,
        memory_decision_re=memory_decision_re,
        memory_feedback_re=memory_feedback_re,
        memory_recall_credit_re=memory_recall_credit_re,
        memory_decay_re=memory_decay_re,
    )

    extract_mcp_metrics = partial(
        execution_module.extract_mcp_metrics,
        ansi_escape_re=ansi_escape_re,
        mcp_last_event_re=mcp_last_event_re,
        mcp_waiting_seen_re=mcp_waiting_seen_re,
        mcp_event_counts_re=mcp_event_counts_re,
    )
    return extract_memory_metrics, extract_mcp_metrics
