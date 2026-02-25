use crate::channels::managed_runtime::parsing::{
    SessionPartitionModeToken, parse_session_partition_command as parse_session_partition_shared,
    parse_session_partition_mode_token as parse_partition_mode_token,
};

use super::{SessionPartitionCommand, SessionPartitionMode};

pub(super) fn parse_session_partition_command(input: &str) -> Option<SessionPartitionCommand> {
    parse_session_partition_shared(input, parse_session_partition_mode)
}

fn parse_session_partition_mode(raw: &str) -> Option<SessionPartitionMode> {
    let token = parse_partition_mode_token(raw)?;
    match token {
        SessionPartitionModeToken::Chat | SessionPartitionModeToken::Channel => {
            Some(SessionPartitionMode::ChannelOnly)
        }
        SessionPartitionModeToken::ChatUser
        | SessionPartitionModeToken::ChatThreadUser
        | SessionPartitionModeToken::GuildChannelUser => {
            Some(SessionPartitionMode::GuildChannelUser)
        }
        SessionPartitionModeToken::User => Some(SessionPartitionMode::UserOnly),
        SessionPartitionModeToken::GuildUser => Some(SessionPartitionMode::GuildUser),
    }
}
