use super::*;

#[test]
fn test_saliency_touch_updates_inbound_edge_zset() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let inbound_key = format!("{prefix}:kg:edge:in:note-b");
    let out_key = format!("{prefix}:kg:edge:out:note-a");
    let mut conn = valkey_connection()?;
    redis::cmd("SADD")
        .arg(&inbound_key)
        .arg("note-a")
        .query::<i64>(&mut conn)?;

    let state = valkey_saliency_touch_with_valkey(
        LinkGraphSaliencyTouchRequest {
            node_id: "note-b".to_string(),
            activation_delta: 2,
            saliency_base: Some(7.0),
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

    let zscore: Option<f64> = redis::cmd("ZSCORE")
        .arg(&out_key)
        .arg("note-b")
        .query(&mut conn)?;
    assert!(zscore.is_some());
    let score = zscore.ok_or("missing zscore for updated edge")?;
    assert!((score - state.current_saliency).abs() < 1e-9);

    clear_prefix(&prefix)?;
    Ok(())
}
