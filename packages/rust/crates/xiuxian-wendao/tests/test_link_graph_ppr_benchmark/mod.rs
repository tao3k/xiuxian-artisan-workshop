//! Benchmark test for `LinkGraph` related-PPR latency on large fixtures.
//!
//! This benchmark is intentionally `ignored` by default because it materializes
//! a 10k+ markdown fixture to validate long-horizon PPR runtime behavior.

use std::cmp;
use std::fs;
use std::path::Path;
use std::time::Instant;

use tempfile::tempdir;
use xiuxian_wendao::{LinkGraphIndex, LinkGraphPprSubgraphMode, LinkGraphRelatedPprOptions};

const NODE_COUNT: usize = 10_240;
const HUB_COUNT: usize = 32;
const QUERY_COUNT: usize = 48;
const RELATED_MAX_DISTANCE: usize = 4;
const RELATED_LIMIT: usize = 24;
const HARD_SANITY_P95_MS: f64 = 1_000.0;
const DEFAULT_TARGET_P95_MS: f64 = 50.0;
const DEFAULT_PPR_ALPHA: f64 = 0.9;
const DEFAULT_PPR_MAX_ITER: usize = 30;
const DEFAULT_PPR_TOL: f64 = 1e-6;

fn write_note(path: &Path, body: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, body)?;
    Ok(())
}

fn note_id(i: usize) -> String {
    format!("note-{i:05}")
}

fn hub_id(i: usize) -> String {
    format!("hub-{i:02}")
}

fn build_large_fixture(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..NODE_COUNT {
        let current = note_id(i);
        let next = note_id((i + 1) % NODE_COUNT);
        let jump = note_id((i + 97) % NODE_COUNT);
        let hub = hub_id(i % HUB_COUNT);
        let body = format!(
            "# {current}\n\n\
             Synthetic benchmark note {i}.\n\n\
             Links: [[{next}]] [[{jump}]] [[{hub}]]\n\n\
             This is deterministic fixture content to stress related-PPR traversal.\n"
        );
        write_note(&root.join(format!("{current}.md")), &body)?;
    }

    for h in 0..HUB_COUNT {
        let hub = hub_id(h);
        let mut links = String::new();
        let stride = HUB_COUNT * 2;
        let mut idx = h;
        let mut emitted = 0_usize;
        while idx < NODE_COUNT && emitted < 220 {
            if !links.is_empty() {
                links.push(' ');
            }
            links.push_str("[[");
            links.push_str(&note_id(idx));
            links.push_str("]]");
            emitted += 1;
            idx += stride;
        }
        let body = format!(
            "# {hub}\n\n\
             Synthetic hub node {h}.\n\n\
             Outbound links: {links}\n"
        );
        write_note(&root.join(format!("{hub}.md")), &body)?;
    }

    Ok(())
}

fn percentile(values: &[f64], percentile: u32) -> f64 {
    assert!(!values.is_empty(), "percentile requires at least one value");
    assert!(
        percentile <= 100,
        "percentile must be between 0 and 100 inclusive"
    );
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let len = sorted.len();
    let percentile_usize = usize::try_from(percentile).unwrap_or(100);
    let rank = len
        .saturating_mul(percentile_usize)
        .div_ceil(100)
        .saturating_sub(1);
    sorted[cmp::min(rank, sorted.len() - 1)]
}

fn env_flag(name: &str) -> bool {
    matches!(
        std::env::var(name).ok().as_deref(),
        Some("1" | "true" | "TRUE" | "yes" | "YES")
    )
}

fn env_f64(name: &str, default_value: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|raw| raw.trim().parse::<f64>().ok())
        .filter(|value| *value > 0.0)
        .unwrap_or(default_value)
}

fn env_usize(name: &str, default_value: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_value)
}

fn env_subgraph_mode(
    name: &str,
    default_value: LinkGraphPprSubgraphMode,
) -> LinkGraphPprSubgraphMode {
    match std::env::var(name)
        .ok()
        .map(|raw| raw.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("force") => LinkGraphPprSubgraphMode::Force,
        Some("disabled") => LinkGraphPprSubgraphMode::Disabled,
        Some("auto") => LinkGraphPprSubgraphMode::Auto,
        _ => default_value,
    }
}

mod link_graph_related_ppr_latency_on_10k_fixture;
