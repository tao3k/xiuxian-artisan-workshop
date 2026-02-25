#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::option_as_ref_deref,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::manual_assert,
    clippy::manual_string_new,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::similar_names,
    clippy::format_collect,
    clippy::async_yields_async,
    clippy::assigning_clones
)]

use omni_agent::test_support::{
    ManagedControlCommand, ManagedSlashCommand, detect_managed_control_command,
    detect_managed_slash_command,
};

#[test]
fn detect_managed_slash_commands_supports_session_job_and_background_shapes() {
    assert_eq!(
        detect_managed_slash_command("/session"),
        Some(ManagedSlashCommand::SessionStatus)
    );
    assert_eq!(
        detect_managed_slash_command("/window status json"),
        Some(ManagedSlashCommand::SessionStatus)
    );
    assert_eq!(
        detect_managed_slash_command("/session budget json"),
        Some(ManagedSlashCommand::SessionBudget)
    );
    assert_eq!(
        detect_managed_slash_command("[bbx-1] /session memory json"),
        Some(ManagedSlashCommand::SessionMemory)
    );
    assert_eq!(
        detect_managed_slash_command("/feedback down json"),
        Some(ManagedSlashCommand::SessionFeedback)
    );
    assert_eq!(
        detect_managed_slash_command("/job abc123"),
        Some(ManagedSlashCommand::JobStatus)
    );
    assert_eq!(
        detect_managed_slash_command("/jobs json"),
        Some(ManagedSlashCommand::JobsSummary)
    );
    assert_eq!(
        detect_managed_slash_command("/bg collect logs"),
        Some(ManagedSlashCommand::BackgroundSubmit)
    );
    assert_eq!(
        detect_managed_slash_command("/research compare two approaches"),
        Some(ManagedSlashCommand::BackgroundSubmit)
    );
}

#[test]
fn detect_managed_slash_commands_rejects_invalid_shapes() {
    assert_eq!(detect_managed_slash_command("/feedback"), None);
    assert_eq!(detect_managed_slash_command("/session feedback"), None);
    assert_eq!(detect_managed_slash_command("/session budget pretty"), None);
    assert_eq!(detect_managed_slash_command("/jobs pretty"), None);
    assert_eq!(detect_managed_slash_command("/bg"), None);
}

#[test]
fn detect_managed_control_commands_supports_reset_resume_and_partition() {
    assert_eq!(
        detect_managed_control_command("/reset"),
        Some(ManagedControlCommand::Reset)
    );
    assert_eq!(
        detect_managed_control_command("/clear"),
        Some(ManagedControlCommand::Reset)
    );
    assert_eq!(
        detect_managed_control_command("/resume"),
        Some(ManagedControlCommand::ResumeRestore)
    );
    assert_eq!(
        detect_managed_control_command("/resume status"),
        Some(ManagedControlCommand::ResumeStatus)
    );
    assert_eq!(
        detect_managed_control_command("/resume drop"),
        Some(ManagedControlCommand::ResumeDrop)
    );
    assert_eq!(
        detect_managed_control_command("/session partition"),
        Some(ManagedControlCommand::SessionPartition)
    );
    assert_eq!(
        detect_managed_control_command("/session partition json"),
        Some(ManagedControlCommand::SessionPartition)
    );
    assert_eq!(
        detect_managed_control_command("/session partition chat_user json"),
        Some(ManagedControlCommand::SessionPartition)
    );
    assert_eq!(
        detect_managed_control_command("/session partition guild_channel_user"),
        Some(ManagedControlCommand::SessionPartition)
    );
    assert_eq!(
        detect_managed_control_command("/session partition channel"),
        Some(ManagedControlCommand::SessionPartition)
    );
    assert_eq!(
        detect_managed_control_command("/session partition guild_user json"),
        Some(ManagedControlCommand::SessionPartition)
    );
    assert_eq!(
        detect_managed_control_command("/session partition channel-user"),
        Some(ManagedControlCommand::SessionPartition)
    );
    assert_eq!(
        detect_managed_control_command("/session partition topic-user json"),
        Some(ManagedControlCommand::SessionPartition)
    );
    assert_eq!(
        detect_managed_control_command("/session admin"),
        Some(ManagedControlCommand::SessionAdmin)
    );
    assert_eq!(
        detect_managed_control_command("/session admin json"),
        Some(ManagedControlCommand::SessionAdmin)
    );
    assert_eq!(
        detect_managed_control_command("/session admin add 1001"),
        Some(ManagedControlCommand::SessionAdmin)
    );
    assert_eq!(
        detect_managed_control_command("/window admin set 1001,1002 json"),
        Some(ManagedControlCommand::SessionAdmin)
    );
    assert_eq!(
        detect_managed_control_command("/session inject"),
        Some(ManagedControlCommand::SessionInjection)
    );
    assert_eq!(
        detect_managed_control_command("/session inject status json"),
        Some(ManagedControlCommand::SessionInjection)
    );
    assert_eq!(
        detect_managed_control_command("/context injection clear"),
        Some(ManagedControlCommand::SessionInjection)
    );
}

#[test]
fn detect_managed_control_commands_rejects_invalid_shapes() {
    assert_eq!(detect_managed_control_command("/resume now"), None);
    assert_eq!(
        detect_managed_control_command("/session partition maybe"),
        None
    );
    assert_eq!(
        detect_managed_control_command("/session partition on pretty"),
        None
    );
    assert_eq!(
        detect_managed_control_command("/session injection"),
        Some(ManagedControlCommand::SessionInjection)
    );
}
