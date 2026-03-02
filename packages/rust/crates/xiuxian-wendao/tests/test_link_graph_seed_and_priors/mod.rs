//! Integration tests for structural priors and seed-grounded related retrieval.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use xiuxian_wendao::LinkGraphIndex;
use xiuxian_wendao::link_graph::{
    LinkGraphEdgeType, LinkGraphPprSubgraphMode, LinkGraphRelatedFilter,
    LinkGraphRelatedPprOptions, LinkGraphSearchFilters, LinkGraphSearchOptions,
};

fn write_file(path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

mod link_graph_related_filter_seed_accuracy_is_cluster_grounded;
mod link_graph_related_journal_semantic_pull_surfaces_agenda_tasks;
mod link_graph_structural_priors_promote_architecture_hub_top3;
