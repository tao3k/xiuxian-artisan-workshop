mod support;

use std::{error::Error, sync::atomic::Ordering, time::Duration};

use support::{array, bulk, index_reply, server};
use xiuxian_qianji_control::{
    ControlError, HotStateStore, ValkeyHotStateConfig, ValkeyHotStateStore, ValkeyReadPolicy,
};

fn store(url: &str, policy: ValkeyReadPolicy) -> Result<ValkeyHotStateStore, Box<dyn Error>> {
    Ok(ValkeyHotStateStore::new(
        ValkeyHotStateConfig::new(url)?
            .with_namespace("test")?
            .with_read_policy(policy),
    ))
}

#[tokio::test]
async fn vanished_payload_and_heartbeat_are_explicit_not_complete_empty()
-> Result<(), Box<dyn Error>> {
    let backend = server(|command| match command[0].as_str() {
        "SCAN" => Some(format!(
            "*2\r\n{}{}",
            bulk("0"),
            array(&["test:heartbeat:worker"])
        )),
        "HGET" | "GET" => Some("$-1\r\n".into()),
        _ => index_reply(command),
    })
    .await?;
    let snapshot = store(&backend.url, ValkeyReadPolicy::default())?
        .load_snapshot(42)
        .await?;
    assert!(!snapshot.atomic);
    assert_eq!(snapshot.observed_at_ms, 42);
    assert!(snapshot.pending_steps.is_empty());
    let observation = snapshot.observation.ok_or("missing observation receipt")?;
    assert!(observation.started_at_ms > 42);
    assert!(observation.finished_at_ms > 42);
    assert_eq!(observation.missing.pending_step_payloads, 1);
    assert_eq!(observation.missing.heartbeats, 1);
    assert_eq!(observation.usage.items, 2);
    assert_eq!(observation.usage.commands, 7);
    assert_eq!(backend.commands.load(Ordering::SeqCst), 7);
    Ok(())
}

#[tokio::test]
async fn components_share_item_budget_and_overflow_is_not_truncation() -> Result<(), Box<dyn Error>>
{
    let backend = server(|command| {
        if command[0] == "ZRANGE" {
            assert_ne!(command[3], "-1");
            Some(array(&["one", "two"]))
        } else {
            index_reply(command)
        }
    })
    .await?;
    let result = store(
        &backend.url,
        ValkeyReadPolicy {
            max_items: 3,
            ..Default::default()
        },
    )?
    .load_snapshot(0)
    .await;
    assert!(matches!(
        result,
        Err(ControlError::ObservationBudgetExceeded { resource: "items" })
    ));
    Ok(())
}

#[tokio::test]
async fn command_budget_includes_payload_phase() -> Result<(), Box<dyn Error>> {
    let backend = server(index_reply).await?;
    let result = store(
        &backend.url,
        ValkeyReadPolicy {
            max_commands: 5,
            ..Default::default()
        },
    )?
    .load_snapshot(0)
    .await;
    assert!(matches!(
        result,
        Err(ControlError::ObservationBudgetExceeded {
            resource: "commands"
        })
    ));
    assert_eq!(backend.commands.load(Ordering::SeqCst), 5);
    Ok(())
}

#[tokio::test]
async fn deadline_includes_payload_phase_not_just_scan() -> Result<(), Box<dyn Error>> {
    let backend = server(index_reply).await?;
    let result = store(
        &backend.url,
        ValkeyReadPolicy {
            timeout: Duration::from_millis(200),
            ..Default::default()
        },
    )?
    .load_snapshot(0)
    .await;
    assert!(matches!(
        result,
        Err(ControlError::ObservationBudgetExceeded {
            resource: "deadline"
        })
    ));
    assert!(backend.commands.load(Ordering::SeqCst) >= 5);
    Ok(())
}

#[tokio::test]
async fn raw_payload_byte_budget_precedes_json_decoding() -> Result<(), Box<dyn Error>> {
    let backend = server(|command| {
        if command[0] == "HGET" {
            Some(bulk(&"x".repeat(256)))
        } else {
            index_reply(command)
        }
    })
    .await?;
    let result = store(
        &backend.url,
        ValkeyReadPolicy {
            max_bytes: 128,
            ..Default::default()
        },
    )?
    .load_snapshot(0)
    .await;
    assert!(matches!(
        result,
        Err(ControlError::ObservationBudgetExceeded { resource: "bytes" })
    ));
    Ok(())
}

#[tokio::test]
async fn zero_deadline_does_not_connect() -> Result<(), Box<dyn Error>> {
    let result = store(
        "redis://127.0.0.1:1",
        ValkeyReadPolicy {
            timeout: Duration::ZERO,
            ..Default::default()
        },
    )?
    .load_snapshot(0)
    .await;
    assert!(matches!(
        result,
        Err(ControlError::ObservationBudgetExceeded {
            resource: "deadline"
        })
    ));
    Ok(())
}

#[tokio::test]
async fn lease_removed_after_index_read_is_reported() -> Result<(), Box<dyn Error>> {
    let backend = server(|command| match command[0].as_str() {
        "ZRANGE" => Some(array(if command[1] == "test:lease_deadlines" {
            &["run|step"]
        } else {
            &[]
        })),
        "HGET" => Some(bulk(
            r#"{"run_id":"run","step_id":"step","priority":0,"not_before_ms":0,"metadata":null}"#,
        )),
        "HGETALL" => Some(array(&[])),
        _ => index_reply(command),
    })
    .await?;
    let snapshot = store(&backend.url, ValkeyReadPolicy::default())?
        .load_snapshot(42)
        .await?;
    assert!(!snapshot.atomic);
    assert!(snapshot.leased_steps.is_empty());
    let observation = snapshot.observation.ok_or("missing observation receipt")?;
    assert_eq!(observation.missing.step_leases, 1);
    assert_eq!(observation.usage.items, 1);
    Ok(())
}
