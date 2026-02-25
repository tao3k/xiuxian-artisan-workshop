#!/usr/bin/env python3
"""
Run Discord ACL black-box probes against local omni-agent Discord ingress runtime.

The probe posts synthetic Discord ingress events to `/discord/ingress`, then validates
managed-command observability events from runtime logs with strict target-scope checks.
"""

from __future__ import annotations

import json
import sys
import time
from pathlib import Path
from typing import TYPE_CHECKING

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

try:
    from scripts.channel.discord_acl_events_module_bindings import load_module_bindings
except ModuleNotFoundError:
    from discord_acl_events_module_bindings import load_module_bindings

if TYPE_CHECKING:
    import argparse

DISCORD_INGRESS_SECRET_HEADER = "x-omni-discord-ingress-token"
SUITES = ("core", "all")
ERROR_PATTERNS = (
    "discord failed to send command reply",
    "Foreground message handling failed",
    "tools/call: Mcp error",
)
FORBIDDEN_LOG_PATTERN = "tools/call: Mcp error"
DISCORD_SESSION_SCOPE_PREFIX = "discord:"

_MODULES = load_module_bindings(__file__)
BLACKBOX = _MODULES.blackbox_module
TARGET_SESSION_SCOPE_PLACEHOLDER = _MODULES.target_session_scope_placeholder
ProbeCase = _MODULES.models_module.ProbeCase
ProbeConfig = _MODULES.models_module.ProbeConfig


def _normalize_ingress_bind_for_local_url(bind_addr: str) -> str:
    return _MODULES.config_module.normalize_ingress_bind_for_local_url(bind_addr)


def default_ingress_url() -> str:
    return _MODULES.config_module.default_ingress_url()


def parse_args() -> argparse.Namespace:
    return _MODULES.config_module.parse_args(
        suites=SUITES,
        default_ingress_url_value=default_ingress_url(),
    )


def normalize_partition_mode(value: str) -> str:
    return _MODULES.config_module.normalize_partition_mode(value)


def dedup(values: list[str]) -> tuple[str, ...]:
    return _MODULES.config_module.dedup(values)


def build_config(args: argparse.Namespace) -> ProbeConfig:
    return _MODULES.config_module.build_config(args, config_cls=ProbeConfig)


def now_event_id() -> str:
    return _MODULES.runtime_module.now_event_id()


def expected_session_keys(
    partition_mode: str,
    guild_id: str | None,
    channel_id: str,
    user_id: str,
) -> tuple[str, ...]:
    return _MODULES.runtime_module.expected_session_keys(
        partition_mode, guild_id, channel_id, user_id
    )


def expected_session_scopes(
    partition_mode: str,
    guild_id: str | None,
    channel_id: str,
    user_id: str,
) -> tuple[str, ...]:
    return _MODULES.runtime_module.expected_session_scopes(
        partition_mode,
        guild_id,
        channel_id,
        user_id,
        session_scope_prefix=DISCORD_SESSION_SCOPE_PREFIX,
        expected_session_keys_fn=expected_session_keys,
    )


def build_ingress_payload(config: ProbeConfig, event_id: str, prompt: str) -> str:
    return _MODULES.runtime_module.build_ingress_payload(config, event_id, prompt)


def post_ingress_event(url: str, payload: str, secret_token: str | None) -> tuple[int, str]:
    return _MODULES.runtime_module.post_ingress_event(
        url,
        payload,
        secret_token,
        secret_header_name=DISCORD_INGRESS_SECRET_HEADER,
    )


def parse_expected_field(value: str) -> tuple[str, str]:
    return BLACKBOX.parse_expected_field(value)


def compile_patterns(patterns: tuple[str, ...]):
    return _MODULES.runtime_module.compile_patterns(patterns)


def reply_json_field_matches(
    *,
    key: str,
    expected: str,
    observation: dict[str, str],
    expected_session_scopes_values: tuple[str, ...],
) -> bool:
    return _MODULES.runtime_module.reply_json_field_matches(
        key=key,
        expected=expected,
        observation=observation,
        expected_session_scopes_values=expected_session_scopes_values,
        target_session_scope_placeholder=TARGET_SESSION_SCOPE_PLACEHOLDER,
    )


def run_case(config: ProbeConfig, case: ProbeCase) -> int:
    return _MODULES.runtime_module.run_case(
        config,
        case,
        blackbox=BLACKBOX,
        parse_expected_field_fn=parse_expected_field,
        expected_session_keys_fn=expected_session_keys,
        expected_session_scopes_fn=_MODULES.runtime_module.expected_session_scopes,
        now_event_id_fn=now_event_id,
        build_ingress_payload_fn=build_ingress_payload,
        post_ingress_event_fn=post_ingress_event,
        compile_patterns_fn=compile_patterns,
        forbidden_log_pattern=FORBIDDEN_LOG_PATTERN,
        error_patterns=ERROR_PATTERNS,
        target_session_scope_placeholder=TARGET_SESSION_SCOPE_PLACEHOLDER,
        session_scope_prefix=DISCORD_SESSION_SCOPE_PREFIX,
    )


def selected_suites(args: argparse.Namespace) -> tuple[str, ...]:
    return _MODULES.config_module.selected_suites(args)


def build_cases(user_id: str) -> list[ProbeCase]:
    return _MODULES.config_module.build_cases(user_id, case_cls=ProbeCase)


def filter_cases(
    cases: list[ProbeCase],
    suites: tuple[str, ...],
    requested_case_ids: tuple[str, ...],
) -> list[ProbeCase]:
    return _MODULES.config_module.filter_cases(cases, suites, requested_case_ids)


def list_cases(cases: list[ProbeCase]) -> int:
    return _MODULES.config_module.list_cases(cases)


def main() -> int:
    args = parse_args()
    suites = selected_suites(args)
    requested_case_ids = dedup(args.case)

    if args.list_cases:
        preview_cases = build_cases(user_id=args.user_id.strip() or "{user_id}")
        selected_preview = filter_cases(preview_cases, suites, requested_case_ids)
        return list_cases(selected_preview if selected_preview else preview_cases)

    try:
        config = build_config(args)
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 2

    cases = build_cases(user_id=config.user_id)
    selected = filter_cases(cases, suites, requested_case_ids)
    if not selected:
        print("No Discord ACL cases selected.", file=sys.stderr)
        return 2

    start = time.time()
    records: list[dict[str, object]] = []
    failures = 0
    for case in selected:
        started = time.time()
        rc = run_case(config, case)
        duration_ms = int((time.time() - started) * 1000)
        records.append(
            {
                "case_id": case.case_id,
                "prompt": case.prompt,
                "event_name": case.event_name,
                "returncode": rc,
                "passed": rc == 0,
                "duration_ms": duration_ms,
            }
        )
        if rc != 0:
            failures += 1

    elapsed_ms = int((time.time() - start) * 1000)
    report = {
        "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "ingress_url": config.ingress_url,
        "log_file": str(config.log_file),
        "session_partition": config.session_partition,
        "channel_id": config.channel_id,
        "user_id": config.user_id,
        "guild_id": config.guild_id,
        "total": len(records),
        "passed": len(records) - failures,
        "failed": failures,
        "duration_ms": elapsed_ms,
        "records": records,
    }
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0 if failures == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
