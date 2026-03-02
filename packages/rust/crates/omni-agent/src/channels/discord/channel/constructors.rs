use std::sync::RwLock;

use crate::channels::control_command_authorization::ControlCommandPolicy;

use super::super::client::build_discord_http_client;
use super::super::constants::DISCORD_DEFAULT_API_BASE;
use super::super::session_partition::DiscordSessionPartition;
use super::policy::{
    DiscordCommandAdminRule, DiscordControlCommandPolicy, DiscordSlashCommandPolicy,
};
use super::policy_builders::{
    build_slash_command_policy, normalize_allowed_guild_entries, normalize_allowed_user_entries,
    normalize_control_command_policy,
};
use super::state::DiscordChannel;

struct DiscordChannelCoreInit {
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_guilds: Vec<String>,
    api_base_url: String,
    control_command_policy: ControlCommandPolicy<DiscordCommandAdminRule>,
    slash_command_policy: DiscordSlashCommandPolicy,
    session_partition: DiscordSessionPartition,
    client: reqwest::Client,
}

impl DiscordChannel {
    /// Create a Discord channel skeleton with default admin policy (`admin_users=allowed_users`).
    #[must_use]
    pub fn new(bot_token: String, allowed_users: Vec<String>, allowed_guilds: Vec<String>) -> Self {
        Self::new_with_partition(
            bot_token,
            allowed_users,
            allowed_guilds,
            DiscordSessionPartition::from_env(),
        )
    }

    /// Create a Discord channel skeleton with explicit session partition.
    #[must_use]
    pub fn new_with_partition(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_guilds: Vec<String>,
        session_partition: DiscordSessionPartition,
    ) -> Self {
        let admin_users = allowed_users.clone();
        Self::new_with_partition_and_parsed_control_command_policy(
            bot_token,
            allowed_users,
            allowed_guilds,
            ControlCommandPolicy::new(admin_users, None, Vec::new()),
            DiscordSlashCommandPolicy::default(),
            session_partition,
        )
    }

    /// Create a Discord channel with custom API base URL (useful for tests/proxies).
    #[must_use]
    pub fn new_with_base_url(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_guilds: Vec<String>,
        api_base_url: String,
    ) -> Self {
        Self::new_with_base_url_and_partition(
            bot_token,
            allowed_users,
            allowed_guilds,
            api_base_url,
            DiscordSessionPartition::from_env(),
        )
    }

    /// Create a Discord channel with custom API base URL and explicit session partition.
    #[must_use]
    pub fn new_with_base_url_and_partition(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_guilds: Vec<String>,
        api_base_url: String,
        session_partition: DiscordSessionPartition,
    ) -> Self {
        let admin_users = allowed_users.clone();
        Self::new_with_base_url_and_partition_and_parsed_control_command_policy(
            DiscordChannelCoreInit {
                bot_token,
                allowed_users,
                allowed_guilds,
                api_base_url,
                control_command_policy: ControlCommandPolicy::new(admin_users, None, Vec::new()),
                slash_command_policy: DiscordSlashCommandPolicy::default(),
                session_partition,
                client: build_discord_http_client(),
            },
        )
    }

    /// Create a Discord channel skeleton with explicit control-command policy.
    #[must_use]
    pub fn new_with_control_command_policy(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_guilds: Vec<String>,
        control_command_policy: DiscordControlCommandPolicy,
    ) -> Self {
        Self::new_with_partition_and_control_command_policy(
            bot_token,
            allowed_users,
            allowed_guilds,
            control_command_policy,
            DiscordSessionPartition::from_env(),
        )
    }

    /// Create a Discord channel skeleton with explicit control-command policy and session
    /// partition.
    #[must_use]
    pub fn new_with_partition_and_control_command_policy(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_guilds: Vec<String>,
        control_command_policy: DiscordControlCommandPolicy,
        session_partition: DiscordSessionPartition,
    ) -> Self {
        let DiscordControlCommandPolicy {
            admin_users,
            control_command_allow_from,
            control_command_rules,
            slash_command_policy,
        } = control_command_policy;
        Self::new_with_partition_and_parsed_control_command_policy(
            bot_token,
            allowed_users,
            allowed_guilds,
            ControlCommandPolicy::new(
                admin_users,
                control_command_allow_from,
                control_command_rules,
            ),
            slash_command_policy,
            session_partition,
        )
    }

    fn new_with_partition_and_parsed_control_command_policy(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_guilds: Vec<String>,
        control_command_policy: ControlCommandPolicy<DiscordCommandAdminRule>,
        slash_command_policy: DiscordSlashCommandPolicy,
        session_partition: DiscordSessionPartition,
    ) -> Self {
        Self::new_with_base_url_and_partition_and_parsed_control_command_policy(
            DiscordChannelCoreInit {
                bot_token,
                allowed_users,
                allowed_guilds,
                api_base_url: DISCORD_DEFAULT_API_BASE.to_string(),
                control_command_policy,
                slash_command_policy,
                session_partition,
                client: build_discord_http_client(),
            },
        )
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_with_base_url_and_partition_and_client(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_guilds: Vec<String>,
        api_base_url: String,
        session_partition: DiscordSessionPartition,
        client: reqwest::Client,
    ) -> Self {
        let admin_users = allowed_users.clone();
        Self::new_with_base_url_and_partition_and_parsed_control_command_policy(
            DiscordChannelCoreInit {
                bot_token,
                allowed_users,
                allowed_guilds,
                api_base_url,
                control_command_policy: ControlCommandPolicy::new(admin_users, None, Vec::new()),
                slash_command_policy: DiscordSlashCommandPolicy::default(),
                session_partition,
                client,
            },
        )
    }

    fn new_with_base_url_and_partition_and_parsed_control_command_policy(
        init: DiscordChannelCoreInit,
    ) -> Self {
        let DiscordChannelCoreInit {
            bot_token,
            allowed_users,
            allowed_guilds,
            api_base_url,
            control_command_policy,
            slash_command_policy,
            session_partition,
            client,
        } = init;

        let control_command_policy = normalize_control_command_policy(control_command_policy);
        let slash_command_policy = build_slash_command_policy(
            control_command_policy.admin_users.clone(),
            slash_command_policy,
        );
        Self {
            bot_token,
            api_base_url,
            allowed_users: normalize_allowed_user_entries(allowed_users),
            allowed_guilds: normalize_allowed_guild_entries(allowed_guilds),
            control_command_policy,
            slash_command_policy,
            session_partition: RwLock::new(session_partition),
            recipient_admin_users: RwLock::new(std::collections::HashMap::new()),
            sender_acl_identities: RwLock::new(std::collections::HashMap::new()),
            client,
        }
    }
}
