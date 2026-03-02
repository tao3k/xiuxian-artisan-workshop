//! Integration tests for embedded markdown resource registry.

use include_dir::{Dir, include_dir};
use xiuxian_wendao::{WendaoResourceRegistry, WendaoResourceRegistryError};

static VALID_EMBEDDED_RESOURCES: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/tests/fixtures/embedded-registry/valid");
static MISSING_EMBEDDED_RESOURCES: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/tests/fixtures/embedded-registry/missing");
static WENDAO_URI_EMBEDDED_RESOURCES: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/tests/fixtures/embedded-registry/wendao-uri");

#[test]
fn build_from_embedded_indexes_blocks_and_links() {
    let Ok(registry) = WendaoResourceRegistry::build_from_embedded(&VALID_EMBEDDED_RESOURCES)
    else {
        panic!("expected valid embedded resources to pass");
    };

    assert_eq!(registry.files_len(), 1);
    assert!(registry.get("agenda_steward").is_some());
    assert!(registry.get("draft_agenda").is_some());

    let Some(file) = registry.file("zhixing/skill.md") else {
        panic!("expected zhixing/skill.md registry entry");
    };
    assert_eq!(
        file.links_for_id("agenda_steward"),
        Some(&["zhixing/personas/agenda_steward.toml".to_string()][..])
    );
    assert_eq!(
        file.links_for_id("draft_agenda"),
        Some(&["zhixing/templates/draft_agenda.j2".to_string()][..])
    );
}

#[test]
fn build_from_embedded_rejects_missing_linked_targets() {
    let Err(err) = WendaoResourceRegistry::build_from_embedded(&MISSING_EMBEDDED_RESOURCES) else {
        panic!("expected missing-link fixture to fail validation");
    };
    let WendaoResourceRegistryError::MissingLinkedResources { count, missing } = err else {
        panic!("unexpected error: {err:?}");
    };
    assert_eq!(count, 1);
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].source_path, "zhixing/skill.md");
    assert_eq!(missing[0].id, "missing_template");
    assert_eq!(missing[0].target_path, "zhixing/templates/not_found.j2");
}

#[test]
fn build_from_embedded_semantically_lifts_relative_wikilinks_to_wendao_uris() {
    let Ok(registry) = WendaoResourceRegistry::build_from_embedded(&WENDAO_URI_EMBEDDED_RESOURCES)
    else {
        panic!("expected relative wikilinks to resolve and semantically lift");
    };

    let Some(file) = registry.file("zhixing/skills/agenda-management/SKILL.md") else {
        panic!("expected zhixing/skills/agenda-management/SKILL.md registry entry");
    };
    assert_eq!(
        file.links_for_id("draft_agenda.j2"),
        Some(&["wendao://skills/agenda-management/references/draft_agenda.j2".to_string()][..])
    );
    assert_eq!(
        file.links_for_id("logo.png"),
        Some(&["wendao://skills/agenda-management/references/logo.png".to_string()][..])
    );
    assert_eq!(
        file.links_for_reference_type("TEMPLATE"),
        vec!["wendao://skills/agenda-management/references/draft_agenda.j2".to_string()]
    );
    assert_eq!(
        file.links_for_reference_type("attachment"),
        vec!["wendao://skills/agenda-management/references/logo.png".to_string()]
    );
}
