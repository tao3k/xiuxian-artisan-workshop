#[test]
fn test_compute_hash() {
    use xiuxian_wendao::SyncEngine;

    let hash1 = SyncEngine::compute_hash("hello world");
    let hash2 = SyncEngine::compute_hash("hello world");
    let hash3 = SyncEngine::compute_hash("different");

    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash3);
    // xxhash produces 16 character hex
    assert_eq!(hash1.len(), 16);
}
