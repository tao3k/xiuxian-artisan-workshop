use std::sync::PoisonError;

use crate::channels::control_command_authorization::{
    ControlCommandAuthRule, ControlCommandAuthSource, ControlCommandAuthorization,
    ControlCommandPolicy, resolve_control_command_authorization_with_policy,
};

use super::state::DiscordChannel;

const MAX_CACHED_AUTH_CONTEXTS: usize = 4_096;

pub(super) fn normalize_discord_identity(identity: &str) -> String {
    let trimmed = identity.trim();
    if trimmed == "*" {
        return "*".to_string();
    }
    trimmed.trim_start_matches('@').to_ascii_lowercase()
}

fn build_sender_acl_cache_key(sender: &str, recipient: &str) -> Option<String> {
    let normalized_sender = normalize_discord_identity(sender);
    let normalized_recipient = recipient.trim();
    if normalized_sender.is_empty() || normalized_recipient.is_empty() {
        return None;
    }
    Some(format!("{normalized_recipient}:{normalized_sender}"))
}

fn list_allows_identity(entries: &[String], identity: &str) -> bool {
    entries
        .iter()
        .any(|entry| entry == "*" || entry == identity)
}

fn normalize_acl_identities(entries: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for entry in entries {
        let identity = normalize_discord_identity(&entry);
        if identity.is_empty() {
            continue;
        }
        if !normalized.iter().any(|existing| existing == &identity) {
            normalized.push(identity);
        }
    }
    normalized
}

fn resolve_authorization_for_identities<R: ControlCommandAuthRule>(
    identities: &[String],
    command_text: &str,
    policy: &ControlCommandPolicy<R>,
) -> ControlCommandAuthorization {
    if let Some(entries) = policy.control_command_allow_from.as_deref() {
        return ControlCommandAuthorization {
            allowed: identities
                .iter()
                .any(|identity| list_allows_identity(entries, identity)),
            source: ControlCommandAuthSource::ControlCommandAllowFrom,
        };
    }

    let mut saw_rule = false;
    for identity in identities {
        let authorization =
            resolve_control_command_authorization_with_policy(identity, command_text, policy);
        if authorization.allowed {
            return authorization;
        }
        if authorization.source == ControlCommandAuthSource::Rule {
            saw_rule = true;
        }
    }

    if saw_rule {
        return ControlCommandAuthorization {
            allowed: false,
            source: ControlCommandAuthSource::Rule,
        };
    }

    ControlCommandAuthorization {
        allowed: false,
        source: ControlCommandAuthSource::AdminUsers,
    }
}

impl DiscordChannel {
    pub(in super::super) fn normalize_identity(identity: &str) -> String {
        normalize_discord_identity(identity)
    }

    pub(in super::super) fn cache_sender_acl_identities(
        &self,
        identity: &str,
        recipient: &str,
        acl_identities: Vec<String>,
    ) {
        let Some(cache_key) = build_sender_acl_cache_key(identity, recipient) else {
            return;
        };
        let normalized_sender = normalize_discord_identity(identity);
        let mut normalized_identities = normalize_acl_identities(acl_identities);
        if !normalized_identities
            .iter()
            .any(|entry| entry == &normalized_sender)
        {
            normalized_identities.insert(0, normalized_sender);
        }

        let mut cache = self
            .sender_acl_identities
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        if cache.len() >= MAX_CACHED_AUTH_CONTEXTS && !cache.contains_key(&cache_key) {
            cache.clear();
        }
        cache.insert(cache_key, normalized_identities);
    }

    fn resolve_acl_identities_for_recipient(&self, identity: &str, recipient: &str) -> Vec<String> {
        let normalized_identity = normalize_discord_identity(identity);
        if normalized_identity.is_empty() {
            return Vec::new();
        }
        let Some(cache_key) = build_sender_acl_cache_key(&normalized_identity, recipient) else {
            return vec![normalized_identity];
        };
        self.sender_acl_identities
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&cache_key)
            .cloned()
            .unwrap_or_else(|| vec![normalized_identity])
    }

    pub(super) fn authorize_control_command(&self, identity: &str, command_text: &str) -> bool {
        resolve_authorization_for_identities(
            &[normalize_discord_identity(identity)],
            command_text,
            &self.control_command_policy,
        )
        .allowed
    }

    pub(super) fn authorize_control_command_for_recipient(
        &self,
        identity: &str,
        command_text: &str,
        recipient: &str,
    ) -> bool {
        let identities = self.resolve_acl_identities_for_recipient(identity, recipient);
        let authorization = resolve_authorization_for_identities(
            &identities,
            command_text,
            &self.control_command_policy,
        );
        if authorization.allowed {
            return true;
        }
        if authorization.source != ControlCommandAuthSource::AdminUsers {
            return false;
        }
        let Some(recipient_admin_users) = self.resolve_recipient_command_admin_users(recipient)
        else {
            return false;
        };
        identities.iter().any(|identity| {
            recipient_admin_users
                .iter()
                .any(|entry| entry == "*" || entry == identity)
        })
    }

    pub(super) fn authorize_slash_command(&self, identity: &str, command_scope: &str) -> bool {
        resolve_authorization_for_identities(
            &[normalize_discord_identity(identity)],
            command_scope,
            &self.slash_command_policy,
        )
        .allowed
    }

    pub(super) fn authorize_slash_command_for_recipient(
        &self,
        identity: &str,
        command_scope: &str,
        recipient: &str,
    ) -> bool {
        let identities = self.resolve_acl_identities_for_recipient(identity, recipient);
        let authorization = resolve_authorization_for_identities(
            &identities,
            command_scope,
            &self.slash_command_policy,
        );
        if authorization.allowed {
            return true;
        }
        if authorization.source != ControlCommandAuthSource::AdminUsers {
            return false;
        }
        let Some(recipient_admin_users) = self.resolve_recipient_command_admin_users(recipient)
        else {
            return false;
        };
        identities.iter().any(|identity| {
            recipient_admin_users
                .iter()
                .any(|entry| entry == "*" || entry == identity)
        })
    }

    pub(in super::super) fn api_url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.api_base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}
