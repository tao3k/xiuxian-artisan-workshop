//! `LinkGraph` entity-reference `PyO3` bindings.

mod py_functions;
mod py_types;

pub use py_functions::{
    link_graph_count_refs, link_graph_extract_entity_refs, link_graph_find_referencing_notes,
    link_graph_get_ref_stats, link_graph_is_valid_ref, link_graph_parse_entity_ref,
};
pub use py_types::{PyLinkGraphEntityRef, PyLinkGraphRefStats};
