//! Shared classification of already loaded index descriptions.

pub(super) fn is_vector(name: &str, index_type: &str) -> bool {
    name == "vector"
        || name == "vector_idx"
        || index_type.contains("Vector")
        || index_type.contains("IVF")
        || index_type.contains("FLAT")
        || index_type.eq_ignore_ascii_case("flat")
}

pub(super) fn is_scalar(name: &str, index_type: &str) -> bool {
    (index_type == "BTree" || index_type == "Bitmap")
        && (name.contains("skill_name") || name.contains("category"))
}

#[cfg(test)]
#[path = "../../tests/unit/index_classification.rs"]
mod tests;
