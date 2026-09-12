use super::{is_scalar, is_vector};

#[test]
fn coverage_preserves_index_kind_and_field_boundaries() {
    assert!(is_vector("custom", "IVF_PQ"));
    assert!(is_vector("custom", "flat"));
    assert!(is_vector("vector_idx", "unknown"));
    assert!(!is_vector("category", "BTree"));
    assert!(is_scalar("skill_name_idx", "BTree"));
    assert!(is_scalar("category_idx", "Bitmap"));
    assert!(!is_scalar("unrelated", "BTree"));
    assert!(!is_scalar("category_idx", "IVF_FLAT"));
}
