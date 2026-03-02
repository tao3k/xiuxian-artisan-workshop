//! Test coverage for omni-agent behavior.

mod managed_runtime {
    pub mod parsing {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/channels/managed_runtime/parsing/types.rs"
        ));
    }
    pub mod session_partition {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/channels/managed_runtime/session_partition.rs"
        ));
    }
}

use managed_runtime::parsing::{
    FeedbackDirection, JobStatusCommand, OutputFormat, ResumeCommand, SessionFeedbackCommand,
    SessionPartitionCommand, SessionPartitionModeToken, parse_session_partition_mode_token,
    session_partition_mode_name,
};
use managed_runtime::session_partition::{
    SessionPartitionProfile, quick_toggle_usage, set_mode_usage, supported_modes,
    supported_modes_csv,
};

#[test]
fn supported_modes_are_always_parseable() {
    for mode in supported_modes(SessionPartitionProfile::Telegram) {
        assert!(
            parse_session_partition_mode_token(mode).is_some(),
            "telegram mode should be parseable: {mode}",
        );
    }
    for mode in supported_modes(SessionPartitionProfile::Discord) {
        assert!(
            parse_session_partition_mode_token(mode).is_some(),
            "discord mode should be parseable: {mode}",
        );
    }
}

#[test]
fn partition_mode_name_and_parser_roundtrip() {
    let cases = [
        SessionPartitionModeToken::Chat,
        SessionPartitionModeToken::ChatUser,
        SessionPartitionModeToken::User,
        SessionPartitionModeToken::ChatThreadUser,
        SessionPartitionModeToken::GuildChannelUser,
        SessionPartitionModeToken::Channel,
        SessionPartitionModeToken::GuildUser,
    ];
    for case in cases {
        let name = session_partition_mode_name(case);
        assert_eq!(parse_session_partition_mode_token(name), Some(case));
    }
}

#[test]
fn managed_runtime_aux_types_and_usage_strings_are_exercised() {
    let dashboard = OutputFormat::Dashboard;
    let json = OutputFormat::Json;
    assert!(!dashboard.is_json());
    assert!(json.is_json());

    assert!(matches!(ResumeCommand::Restore, ResumeCommand::Restore));
    assert!(matches!(ResumeCommand::Drop, ResumeCommand::Drop));
    assert!(matches!(ResumeCommand::Status, ResumeCommand::Status));

    let feedback_up = SessionFeedbackCommand {
        direction: FeedbackDirection::Up,
        format: dashboard,
    };
    let feedback_down = SessionFeedbackCommand {
        direction: FeedbackDirection::Down,
        format: json,
    };
    assert!(matches!(feedback_up.direction, FeedbackDirection::Up));
    assert!(matches!(feedback_down.direction, FeedbackDirection::Down));
    assert!(feedback_down.format.is_json());

    let job = JobStatusCommand {
        job_id: "job-1".to_string(),
        format: OutputFormat::Dashboard,
    };
    assert_eq!(job.job_id, "job-1");
    assert!(!job.format.is_json());

    let partition = SessionPartitionCommand::<SessionPartitionModeToken> {
        mode: Some(SessionPartitionModeToken::Chat),
        format: OutputFormat::Json,
    };
    assert_eq!(partition.mode, Some(SessionPartitionModeToken::Chat));
    assert!(partition.format.is_json());

    let telegram_csv = supported_modes_csv(SessionPartitionProfile::Telegram);
    assert!(telegram_csv.contains("chat"));
    let discord_usage = set_mode_usage(SessionPartitionProfile::Discord);
    assert!(discord_usage.starts_with("/session partition "));
    assert_eq!(quick_toggle_usage(), "/session partition on|off");
}
