//! Integration tests for `LinkGraph` saliency persistence and update behavior.

use redis::Connection;
use std::time::{SystemTime, UNIX_EPOCH};
use xiuxian_wendao::{
    LinkGraphSaliencyPolicy, LinkGraphSaliencyTouchRequest, compute_link_graph_saliency,
    valkey_saliency_get_with_valkey, valkey_saliency_touch_with_valkey,
};

const TEST_VALKEY_URL: &str = "redis://127.0.0.1:6379/0";

fn unique_prefix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    format!("omni:test:saliency:{nanos}")
}

fn valkey_connection() -> Result<Connection, Box<dyn std::error::Error>> {
    let client = redis::Client::open(TEST_VALKEY_URL)?;
    Ok(client.get_connection()?)
}

fn clear_prefix(prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = valkey_connection()?;
    let pattern = format!("{prefix}:*");
    let keys: Vec<String> = redis::cmd("KEYS").arg(&pattern).query(&mut conn)?;
    if !keys.is_empty() {
        redis::cmd("DEL").arg(keys).query::<()>(&mut conn)?;
    }
    Ok(())
}

mod compute_link_graph_saliency_activation_boosts_score;
mod compute_link_graph_saliency_clamps_bounds;
mod saliency_store_auto_repairs_invalid_payload;
mod saliency_touch_and_get_with_valkey;
mod saliency_touch_updates_inbound_edge_zset;
