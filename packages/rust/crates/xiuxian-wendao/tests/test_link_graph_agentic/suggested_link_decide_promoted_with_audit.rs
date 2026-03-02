use super::*;

#[test]
fn test_suggested_link_decide_promoted_with_audit() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let entry = valkey_suggested_link_log_with_valkey(
        LinkGraphSuggestedLinkRequest {
            source_id: "docs/a.md".to_string(),
            target_id: "docs/b.md".to_string(),
            relation: "implements".to_string(),
            confidence: 0.9,
            evidence: "cross-reference".to_string(),
            agent_id: "qianhuan-architect".to_string(),
            created_at_unix: Some(1_700_000_100.0),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
        Some(20),
        None,
    )
    .map_err(|err| err.clone())?;

    let result = valkey_suggested_link_decide_with_valkey(
        LinkGraphSuggestedLinkDecisionRequest {
            suggestion_id: entry.suggestion_id.clone(),
            target_state: LinkGraphSuggestedLinkState::Promoted,
            decided_by: "omega-gate".to_string(),
            reason: "passed gate checks".to_string(),
            decided_at_unix: Some(1_700_000_120.0),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
        Some(20),
        None,
    )
    .map_err(|err| err.clone())?;

    assert_eq!(
        result.suggestion.promotion_state,
        LinkGraphSuggestedLinkState::Promoted
    );
    assert_eq!(result.suggestion.decision_by.as_deref(), Some("omega-gate"));
    assert_eq!(
        result.suggestion.decision_reason.as_deref(),
        Some("passed gate checks")
    );
    assert!((result.suggestion.updated_at_unix - 1_700_000_120.0).abs() < 1e-9);
    assert_eq!(
        result.decision.previous_state,
        LinkGraphSuggestedLinkState::Provisional
    );
    assert_eq!(
        result.decision.target_state,
        LinkGraphSuggestedLinkState::Promoted
    );

    let latest = valkey_suggested_link_recent_latest_with_valkey(
        10,
        TEST_VALKEY_URL,
        Some(&prefix),
        Some(LinkGraphSuggestedLinkState::Promoted),
        Some(50),
    )
    .map_err(|err| err.clone())?;
    assert_eq!(latest.len(), 1);
    assert_eq!(latest[0].suggestion_id, entry.suggestion_id);
    assert_eq!(
        latest[0].promotion_state,
        LinkGraphSuggestedLinkState::Promoted
    );

    let decisions =
        valkey_suggested_link_decisions_recent_with_valkey(10, TEST_VALKEY_URL, Some(&prefix))
            .map_err(|err| err.clone())?;
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].suggestion_id, entry.suggestion_id);
    assert_eq!(
        decisions[0].target_state,
        LinkGraphSuggestedLinkState::Promoted
    );

    clear_prefix(&prefix)?;
    Ok(())
}
