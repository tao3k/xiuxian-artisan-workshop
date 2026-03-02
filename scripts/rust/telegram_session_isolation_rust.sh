#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cargo_bin="${CARGO_BIN:-${script_dir}/cargo_exec.sh}"
target_dir="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}"

CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --test channels_telegram telegram_parse_update_partition_chat_only_isolates_different_chats
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --test channels_telegram telegram_send_global_rate_limit_gate_delays_parallel_send
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --test channels_telegram telegram_send_global_rate_limit_gate_spreads_parallel_followup_requests
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --test channels_telegram_send_gate telegram_send_rate_limit_valkey_constructor_rejects_invalid_url
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --test channels_webhook webhook_partition_chat_only_isolates_same_user_across_chats
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --lib runtime_partition_chat_only_isolates_same_user_across_chats
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --lib runtime_partition_chat_thread_user_isolates_threads
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --test channels_telegram_group_policy telegram_group_policy_recipient_admin_users_runtime_mutation_topic_scope
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --test channels_telegram_group_policy telegram_group_policy_recipient_admin_users_runtime_mutation_group_topic_isolation
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --lib runtime_handle_inbound_session_budget_denies_when_scope_not_granted
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" test -p omni-agent --lib runtime_handle_inbound_plain_text_is_not_blocked_by_slash_acl
