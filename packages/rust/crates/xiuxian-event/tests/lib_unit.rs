//! Integration tests for `xiuxian-event` core bus behavior.

use chrono::Utc;
use serde_json::json;
use tokio::sync::broadcast;
use xiuxian_event::{EventBus, OmniEvent};

async fn recv_or_panic(rx: &mut broadcast::Receiver<OmniEvent>) -> OmniEvent {
    match rx.recv().await {
        Ok(event) => event,
        Err(error) => panic!("expected event in broadcast receiver: {error}"),
    }
}

#[test]
fn test_event_creation() {
    let event = OmniEvent::new("test", "test/topic", json!({"key": "value"}));
    assert_eq!(event.source, "test");
    assert_eq!(event.topic, "test/topic");
    assert!(!event.id.is_empty());
    assert!(event.timestamp <= Utc::now());
}

#[test]
fn test_file_event() {
    let event = OmniEvent::file_event("watcher", "file/changed", "/path/to/file.py", false);
    assert_eq!(event.source, "watcher");
    assert_eq!(event.topic, "file/changed");
    assert_eq!(event.payload["path"], "/path/to/file.py");
    assert_eq!(event.payload["is_dir"], false);
}

#[tokio::test]
async fn test_event_bus_publish() {
    let bus = EventBus::new(10);
    let mut rx = bus.subscribe();

    let _ = bus.publish(OmniEvent::new("test", "topic", json!({"data": 42})));

    let received = recv_or_panic(&mut rx).await;
    assert_eq!(received.source, "test");
    assert_eq!(received.topic, "topic");
}

#[tokio::test]
async fn test_multiple_subscribers() {
    let bus = EventBus::new(10);
    let mut rx1 = bus.subscribe();
    let mut rx2 = bus.subscribe();

    let _ = bus.publish(OmniEvent::new("test", "topic", json!({"msg": "hello"})));

    let received1 = recv_or_panic(&mut rx1).await;
    let received2 = recv_or_panic(&mut rx2).await;

    assert_eq!(received1.payload, received2.payload);
}

#[tokio::test]
async fn test_subscriber_count() {
    let bus = EventBus::new(10);
    assert_eq!(bus.subscriber_count(), 0);

    let _rx = bus.subscribe();
    assert_eq!(bus.subscriber_count(), 1);

    let _rx2 = bus.subscribe();
    assert_eq!(bus.subscriber_count(), 2);
}
