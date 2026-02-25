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
    ManagedControlCommand, ManagedSlashCommand, OutputFormat, ResumeContextCommand,
    SessionAdminAction, SessionFeedbackDirection, SessionInjectionAction, SessionPartitionMode,
    detect_managed_control_command, detect_managed_slash_command, parse_help_command,
    parse_job_status_command, parse_resume_context_command, parse_session_admin_command,
    parse_session_feedback_command, parse_session_injection_command,
    parse_session_partition_command,
};

#[test]
fn test_support_parses_help_and_job_status_output_formats() {
    assert_eq!(parse_help_command("/help"), Some(OutputFormat::Dashboard));
    assert_eq!(parse_help_command("/help json"), Some(OutputFormat::Json));

    let job = parse_job_status_command("/job abc123 json").expect("expected /job json parse");
    assert_eq!(job.job_id, "abc123");
    assert_eq!(job.format, OutputFormat::Json);
}

#[test]
fn test_support_maps_resume_feedback_and_partition_modes() {
    assert_eq!(
        parse_resume_context_command("/resume drop"),
        Some(ResumeContextCommand::Drop)
    );

    let feedback =
        parse_session_feedback_command("/feedback up").expect("expected /feedback up parse");
    assert_eq!(feedback.direction, SessionFeedbackDirection::Up);
    assert_eq!(feedback.format, OutputFormat::Dashboard);

    let partition = parse_session_partition_command("/session partition chat_user json")
        .expect("expected /session partition chat_user json parse");
    assert_eq!(partition.mode, Some(SessionPartitionMode::ChatUser));
    assert_eq!(partition.format, OutputFormat::Json);
    assert_eq!(
        SessionPartitionMode::ChatThreadUser.as_str(),
        "chat_thread_user"
    );

    let injection = parse_session_injection_command("/session inject status json")
        .expect("expected /session inject status json parse");
    assert_eq!(injection.action, SessionInjectionAction::Status);
    assert_eq!(injection.format, OutputFormat::Json);

    let admin =
        parse_session_admin_command("/session admin add 1001,1002").expect("expected admin parse");
    assert_eq!(
        admin.action,
        SessionAdminAction::Add(vec!["1001".to_string(), "1002".to_string()])
    );
}

#[test]
fn test_support_managed_detectors_remain_stable() {
    assert_eq!(
        detect_managed_slash_command("/jobs"),
        Some(ManagedSlashCommand::JobsSummary)
    );
    assert_eq!(
        detect_managed_control_command("/reset"),
        Some(ManagedControlCommand::Reset)
    );
    assert_eq!(
        detect_managed_control_command("/session admin add 1001"),
        Some(ManagedControlCommand::SessionAdmin)
    );
    assert_eq!(
        detect_managed_control_command("/session inject status json"),
        Some(ManagedControlCommand::SessionInjection)
    );
}
