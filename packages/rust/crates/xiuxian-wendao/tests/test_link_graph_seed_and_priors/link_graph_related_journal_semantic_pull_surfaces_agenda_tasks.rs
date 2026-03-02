use super::*;

#[test]
fn test_link_graph_related_journal_semantic_pull_surfaces_agenda_tasks()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;

    write_file(
        &tmp.path().join("docs/journal/journal-entry-2026-02-26.md"),
        r"# Journal 2026-02-26

## Reflection
Need to complete the release checklist and close carryover debt before any risky operation.

## Planning
Reference agenda: [[agenda-tasks-2026-02-26]]
",
    )?;
    write_file(
        &tmp.path().join("docs/agenda/agenda-tasks-2026-02-26.md"),
        r"# Agenda 2026-02-26

## Tasks
- [ ] Harden checkpoint lock path <!-- id: t-release, journal:carryover: 4 -->
- [ ] Rotate deploy token safely <!-- id: t-token, timer:scheduled: 2026-02-26T09:00:00Z -->
",
    )?;
    write_file(
        &tmp.path().join("docs/agenda/2026-02-27.md"),
        r"# Agenda 2026-02-27

## Tasks
- [ ] Buy groceries
",
    )?;

    let index = LinkGraphIndex::build(tmp.path()).map_err(|e| e.clone())?;
    let options = LinkGraphSearchOptions {
        filters: LinkGraphSearchFilters {
            related: Some(LinkGraphRelatedFilter {
                seeds: vec!["docs/journal/journal-entry-2026-02-26.md".to_string()],
                max_distance: Some(3),
                ppr: Some(LinkGraphRelatedPprOptions {
                    alpha: Some(0.9),
                    max_iter: Some(64),
                    tol: Some(1e-6),
                    subgraph_mode: Some(LinkGraphPprSubgraphMode::Force),
                }),
            }),
            ..LinkGraphSearchFilters::default()
        },
        ..LinkGraphSearchOptions::default()
    };

    let hits = index
        .search_planned("checkpoint token carryover", 16, options)
        .1;
    assert!(
        !hits.is_empty(),
        "expected related semantic hits from journal seed"
    );

    let agenda_hit = hits
        .iter()
        .find(|row| row.path == "docs/agenda/agenda-tasks-2026-02-26.md")
        .ok_or_else(|| {
            std::io::Error::other("expected seeded related search to surface linked agenda doc")
        })?;
    assert!(
        agenda_hit
            .best_section
            .as_deref()
            .is_some_and(|section| section.contains("Tasks")),
        "expected agenda task section to be surfaced, got {best_section:?}",
        best_section = agenda_hit.best_section
    );

    let stems: HashSet<String> = hits.iter().map(|row| row.stem.clone()).collect();
    assert!(
        !stems.contains("2026-02-27"),
        "unrelated agenda note should not leak into seeded retrieval: {stems:?}"
    );
    Ok(())
}
