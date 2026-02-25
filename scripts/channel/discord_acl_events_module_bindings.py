#!/usr/bin/env python3
"""Dependency loader for Discord ACL probe entry script."""

from __future__ import annotations

import importlib
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class DiscordAclEventsModuleBindings:
    """Resolved sibling modules and runtime objects for Discord ACL probes."""

    blackbox_module: Any
    target_session_scope_placeholder: str
    models_module: Any
    config_module: Any
    runtime_module: Any


def load_module_bindings(caller_file: str) -> DiscordAclEventsModuleBindings:
    """Load all sibling modules required by `test_omni_agent_discord_acl_events.py`."""
    load_sibling_module = importlib.import_module("module_loader").load_sibling_module

    blackbox_module = load_sibling_module(
        module_name="agent_channel_blackbox",
        file_name="agent_channel_blackbox.py",
        caller_file=caller_file,
        error_context="blackbox module",
    )
    target_session_scope_placeholder = getattr(
        blackbox_module,
        "TARGET_SESSION_SCOPE_PLACEHOLDER",
        "__target_session_scope__",
    )

    models_module = load_sibling_module(
        module_name="discord_acl_events_models",
        file_name="discord_acl_events_models.py",
        caller_file=caller_file,
        error_context="discord acl datamodels",
    )
    config_module = load_sibling_module(
        module_name="discord_acl_events_config",
        file_name="discord_acl_events_config.py",
        caller_file=caller_file,
        error_context="discord acl config helpers",
    )
    runtime_module = load_sibling_module(
        module_name="discord_acl_events_runtime",
        file_name="discord_acl_events_runtime.py",
        caller_file=caller_file,
        error_context="discord acl runtime helpers",
    )

    return DiscordAclEventsModuleBindings(
        blackbox_module=blackbox_module,
        target_session_scope_placeholder=target_session_scope_placeholder,
        models_module=models_module,
        config_module=config_module,
        runtime_module=runtime_module,
    )
