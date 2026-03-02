//! Integration tests for embedded dynamic semantic URI discovery.

use xiuxian_wendao::embedded_discover_canonical_uris;

#[test]
fn discovery_by_reference_type_returns_template_uris() -> Result<(), Box<dyn std::error::Error>> {
    let uris = embedded_discover_canonical_uris("reference_type:template")?;
    assert!(
        uris.contains(&"wendao://skills/agenda-management/references/draft_agenda.j2".to_string()),
        "template discovery should return canonical template URIs"
    );
    Ok(())
}

#[test]
fn discovery_by_config_id_returns_exact_uri() -> Result<(), Box<dyn std::error::Error>> {
    let uris = embedded_discover_canonical_uris("id:agenda_flow")?;
    assert_eq!(
        uris,
        vec!["wendao://skills/agenda-management/references/agenda_flow.toml".to_string()]
    );
    Ok(())
}

#[test]
fn discovery_by_config_id_returns_forge_flow_uri() -> Result<(), Box<dyn std::error::Error>> {
    let uris = embedded_discover_canonical_uris("id:soul_forge_flow")?;
    assert_eq!(
        uris,
        vec!["wendao://skills/forge-evolution/references/soul_forge_flow.toml".to_string()]
    );
    Ok(())
}

#[test]
fn discovery_by_semantic_query_matches_carryover_resources()
-> Result<(), Box<dyn std::error::Error>> {
    let uris = embedded_discover_canonical_uris("carryover:>=1")?;
    assert!(
        uris.contains(&"wendao://skills/agenda-management/references/rules.md".to_string()),
        "semantic query should discover carryover-related canonical URIs"
    );
    Ok(())
}
