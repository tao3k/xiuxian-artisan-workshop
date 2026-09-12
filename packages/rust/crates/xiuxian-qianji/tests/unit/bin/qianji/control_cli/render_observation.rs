use super::render_hot_state_snapshot_text;
use xiuxian_qianji_control::{HotStateObservation, HotStateSnapshot};

#[test]
fn observation_receipt_and_legacy_uncertainty_are_visible() {
    let mut snapshot = HotStateSnapshot::new(42);
    assert!(render_hot_state_snapshot_text(&snapshot).contains("not a completeness guarantee"));
    let mut observation = HotStateObservation {
        started_at_ms: 100,
        finished_at_ms: 120,
        ..Default::default()
    };
    observation.missing.heartbeats = 1;
    snapshot.observation = Some(observation);
    let text = render_hot_state_snapshot_text(&snapshot);
    assert!(text.contains("100..120"));
    assert!(text.contains("heartbeats: 1"));
    assert!(text.contains("Atomic observation: `false`"));
}
