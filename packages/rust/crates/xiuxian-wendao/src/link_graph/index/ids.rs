use super::{LinkGraphDocument, LinkGraphIndex};
use crate::link_graph::parser::normalize_alias;
use std::collections::HashSet;

impl LinkGraphIndex {
    pub(in crate::link_graph::index) fn resolve_doc_id(&self, stem_or_id: &str) -> Option<&str> {
        let key = normalize_alias(stem_or_id);
        self.alias_to_doc_id.get(&key).map(String::as_str)
    }

    pub(in crate::link_graph::index) fn resolve_doc_ids(
        &self,
        values: &[String],
    ) -> HashSet<String> {
        values
            .iter()
            .filter_map(|value| self.resolve_doc_id(value))
            .map(str::to_string)
            .collect()
    }

    pub(in crate::link_graph::index) fn resolve_doc(
        &self,
        stem_or_id: &str,
    ) -> Option<&LinkGraphDocument> {
        let doc_id = self.resolve_doc_id(stem_or_id)?;
        self.docs_by_id.get(doc_id)
    }

    pub(in crate::link_graph::index) fn resolve_weighted_doc_ids(
        &self,
        seeds: &std::collections::HashMap<String, f64>,
    ) -> std::collections::HashMap<String, f64> {
        seeds
            .iter()
            .filter_map(|(key, &weight)| {
                self.resolve_doc_id(key).map(|id| (id.to_string(), weight))
            })
            .collect()
    }

    pub(in crate::link_graph::index) fn all_doc_ids(&self) -> HashSet<String> {
        self.docs_by_id.keys().cloned().collect()
    }
}
