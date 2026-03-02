//! Integration tests for agenda entry domain defaults.

use xiuxian_zhixing::AgendaEntry;

#[test]
fn test_new_entry_is_fresh() {
    let entry = AgendaEntry::new("Master the Wind Sword".to_string());
    assert!((entry.heat - 0.5).abs() < f32::EPSILON);
    assert_eq!(entry.carryover_count, 0);
    assert!(!entry.reminded);
}
