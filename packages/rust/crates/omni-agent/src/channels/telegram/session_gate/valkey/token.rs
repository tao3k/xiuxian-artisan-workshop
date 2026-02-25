use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use super::NEXT_LEASE_OWNER_ID;

pub(super) fn next_lease_owner_token(session_id: &str) -> String {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let seq = NEXT_LEASE_OWNER_ID.fetch_add(1, Ordering::Relaxed);
    format!("{session_id}:{}:{now_ms}:{seq}", std::process::id())
}
