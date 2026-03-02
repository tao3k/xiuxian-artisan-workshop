use super::*;

#[test]
fn test_saliency_touch_and_get_with_valkey() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let first = valkey_saliency_touch_with_valkey(
        LinkGraphSaliencyTouchRequest {
            node_id: "note-a".to_string(),
            activation_delta: 2,
            saliency_base: Some(5.0),
            decay_rate: Some(0.05),
            alpha: Some(0.5),
            minimum_saliency: Some(1.0),
            maximum_saliency: Some(10.0),
            now_unix: Some(1_700_000_000),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
    )
    .map_err(|err| err.clone())?;
    assert_eq!(first.activation_count, 2);
    assert!(first.current_saliency >= 1.0);
    assert!((first.saliency_base - first.current_saliency).abs() < 1e-9);

    let second = valkey_saliency_touch_with_valkey(
        LinkGraphSaliencyTouchRequest {
            node_id: "note-a".to_string(),
            activation_delta: 3,
            saliency_base: None,
            decay_rate: None,
            alpha: Some(0.5),
            minimum_saliency: Some(1.0),
            maximum_saliency: Some(10.0),
            now_unix: Some(1_700_086_400),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
    )
    .map_err(|err| err.clone())?;
    assert_eq!(second.activation_count, 5);
    assert!((second.saliency_base - second.current_saliency).abs() < 1e-9);

    let fetched = valkey_saliency_get_with_valkey("note-a", TEST_VALKEY_URL, Some(&prefix))
        .map_err(|err| err.clone())?;
    assert!(fetched.is_some());
    let state = fetched.ok_or("missing saliency state after touch")?;
    assert_eq!(state.activation_count, 5);
    assert_eq!(state.last_accessed_unix, 1_700_086_400);

    clear_prefix(&prefix)?;
    Ok(())
}
