#!/usr/bin/env python3
"""Gate config parsing/building for omni-agent memory CI."""

from __future__ import annotations

import secrets
from pathlib import Path
from typing import Any

from memory_ci_gate_config_build_helpers import (
    ARTIFACT_SPECS,
    resolve_artifact_relpaths,
    resolve_numeric_identities,
)
from memory_ci_gate_config_build_values import build_gate_config
from memory_ci_gate_config_parser import build_parser
from memory_ci_gate_config_ports import (
    default_run_suffix,
    default_valkey_prefix,
    resolve_runtime_ports,
)
from memory_ci_gate_config_validation import validate_args

# Backward-compatible aliases for existing test hooks/imports.
_ARTIFACT_SPECS = ARTIFACT_SPECS
_resolve_artifact_relpaths = resolve_artifact_relpaths
_resolve_numeric_identities = resolve_numeric_identities


def parse_args(
    project_root: Path,
    *,
    gate_config_cls: Any,
    default_artifact_relpath_fn: Any,
    resolve_runtime_ports_fn: Any = resolve_runtime_ports,
    default_run_suffix_fn: Any = default_run_suffix,
    default_valkey_prefix_fn: Any = default_valkey_prefix,
) -> Any:
    """Parse CLI args and return GateConfig instance."""
    parser = build_parser(project_root)
    args = parser.parse_args()
    validate_args(args)

    agent_bin = Path(args.agent_bin).expanduser().resolve() if args.agent_bin.strip() else None
    if agent_bin is not None:
        if not agent_bin.exists():
            raise ValueError(f"--agent-bin not found: {agent_bin}")
        if not agent_bin.is_file():
            raise ValueError(f"--agent-bin must point to a file: {agent_bin}")

    resolved_webhook_port, resolved_telegram_api_port = resolve_runtime_ports_fn(
        int(args.webhook_port),
        int(args.telegram_api_port),
    )
    run_suffix = default_run_suffix_fn()
    artifact_relpaths = resolve_artifact_relpaths(
        args,
        run_suffix=run_suffix,
        default_artifact_relpath_fn=default_artifact_relpath_fn,
    )

    valkey_url = args.valkey_url.strip() or f"redis://127.0.0.1:{args.valkey_port}/0"
    valkey_prefix = args.valkey_prefix.strip() or default_valkey_prefix_fn(args.profile)
    webhook_secret = args.webhook_secret.strip() or secrets.token_urlsafe(24)
    ids = resolve_numeric_identities(args)

    return build_gate_config(
        args,
        gate_config_cls=gate_config_cls,
        project_root=project_root,
        agent_bin=agent_bin,
        webhook_port=resolved_webhook_port,
        telegram_api_port=resolved_telegram_api_port,
        valkey_url=valkey_url,
        valkey_prefix=valkey_prefix,
        webhook_secret=webhook_secret,
        ids=ids,
        artifact_relpaths=artifact_relpaths,
    )
