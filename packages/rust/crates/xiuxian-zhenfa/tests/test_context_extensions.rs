//! Context extension semantics coverage for native zhenfa dispatch.

use std::sync::Arc;

use tokio::sync::mpsc::unbounded_channel;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaSignal};

#[test]
fn context_extensions_roundtrip_by_type() {
    let mut ctx = ZhenfaContext::default();
    assert!(!ctx.has_extension::<String>());
    assert_eq!(ctx.extension_count(), 0);

    assert!(
        ctx.insert_extension::<String>("alpha".to_string())
            .is_none()
    );
    assert!(ctx.has_extension::<String>());
    assert_eq!(ctx.extension_count(), 1);

    let stored = ctx
        .get_extension::<String>()
        .unwrap_or_else(|| panic!("string extension should exist"));
    assert_eq!(stored.as_str(), "alpha");

    let previous = ctx
        .insert_extension::<String>("beta".to_string())
        .unwrap_or_else(|| panic!("previous string extension should be returned"));
    assert_eq!(previous.as_str(), "alpha");
    let replaced = ctx
        .get_extension::<String>()
        .unwrap_or_else(|| panic!("replacement extension should exist"));
    assert_eq!(replaced.as_str(), "beta");
}

#[test]
fn context_extensions_clone_uses_copy_on_write_registry() {
    let mut ctx = ZhenfaContext::default();
    assert!(
        ctx.insert_shared_extension::<String>(Arc::new("shared".to_string()))
            .is_none()
    );

    let mut cloned = ctx.clone();
    let value_from_clone = cloned
        .get_extension::<String>()
        .unwrap_or_else(|| panic!("cloned context should read extension"));
    assert_eq!(value_from_clone.as_str(), "shared");

    let _ = cloned.insert_extension::<usize>(7);
    assert!(!ctx.has_extension::<usize>());
    let cloned_seen = cloned
        .get_extension::<usize>()
        .unwrap_or_else(|| panic!("extension inserted into clone should be visible in clone"));
    assert_eq!(*cloned_seen, 7);
}

#[test]
fn context_emit_signal_sends_payload_to_attached_channel() {
    let (signal_tx, mut signal_rx) = unbounded_channel::<ZhenfaSignal>();
    let mut ctx = ZhenfaContext::default();
    ctx.attach_signal_sender(signal_tx);
    ctx.set_correlation_id_if_absent("corr:ctx-test".to_string());

    ctx.emit_signal(ZhenfaSignal::Trace {
        node_id: "Agenda_Steward_Proposer".to_string(),
        event: "generated_draft".to_string(),
    });

    let payload = signal_rx
        .try_recv()
        .unwrap_or_else(|error| panic!("signal payload should be emitted: {error}"));
    let ZhenfaSignal::Trace { node_id, event } = payload else {
        panic!("expected trace signal");
    };
    assert_eq!(node_id, "Agenda_Steward_Proposer");
    assert_eq!(event, "generated_draft");
    assert_eq!(ctx.correlation_id.as_deref(), Some("corr:ctx-test"));
}
