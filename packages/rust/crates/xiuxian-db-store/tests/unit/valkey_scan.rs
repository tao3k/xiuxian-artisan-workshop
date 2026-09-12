use super::{RetainedKeys, ValkeyScanPolicy, ValkeyStoreError};

#[tokio::test]
#[ignore = "explicit synthetic scan cost probe"]
async fn scan_cost_probe() -> Result<(), Box<dyn std::error::Error>> {
    for count in [1_000, 10_000, 100_000] {
        let started = std::time::Instant::now();
        let mut calls = 0;
        let result = super::collect_pages(ValkeyScanPolicy::default(), |cursor| {
            calls += 1;
            let start = usize::try_from(cursor).unwrap_or(usize::MAX);
            let end = (start + 256).min(count);
            let next = if end == count {
                0
            } else {
                u64::try_from(end).unwrap_or(u64::MAX)
            };
            std::future::ready(Ok((
                next,
                (start..end).map(|i| format!("key:{i:08}")).collect(),
            )))
        })
        .await?;
        assert_eq!(result.len(), count);
        let retained_bytes: usize = result.iter().map(String::len).sum();
        eprintln!(
            "scan_cost_probe keys={count} calls={calls} retained_bytes={retained_bytes} elapsed_us={}",
            started.elapsed().as_micros()
        );
    }
    Ok(())
}
use std::{collections::HashSet, num::NonZeroUsize};

#[tokio::test]
async fn empty_nonterminal_and_oversized_pages_complete() {
    let mut calls = 0;
    let result = super::collect_pages(ValkeyScanPolicy::default(), |cursor| {
        calls += 1;
        let page = if calls == 1 {
            assert_eq!(cursor, 0);
            (7, Vec::new())
        } else {
            assert_eq!(cursor, 7);
            (0, (0..300).map(|i| i.to_string()).collect())
        };
        std::future::ready(Ok(page))
    })
    .await;
    assert_eq!(result.unwrap_or_default().len(), 300);
    assert_eq!(calls, 2);
}

#[tokio::test]
async fn nonterminating_cursor_exhausts_calls_without_partial_success() {
    let mut calls = 0;
    let policy = ValkeyScanPolicy {
        max_calls: NonZeroUsize::MIN,
        ..Default::default()
    };
    let result = super::collect_pages(policy, |_| {
        calls += 1;
        std::future::ready(Ok((9, vec!["partial".into()])))
    })
    .await;
    assert!(matches!(
        result,
        Err(ValkeyStoreError::ScanBudgetExceeded { resource: "calls" })
    ));
    assert_eq!(calls, 1);
}

#[tokio::test]
async fn pending_fetch_expires() {
    let policy = ValkeyScanPolicy {
        timeout: std::time::Duration::from_millis(1),
        ..Default::default()
    };
    let result = super::collect_pages(policy, |_| std::future::pending()).await;
    assert!(matches!(
        result,
        Err(ValkeyStoreError::ScanBudgetExceeded {
            resource: "deadline"
        })
    ));
}

#[test]
fn duplicate_does_not_consume_retention_budget() {
    let policy = ValkeyScanPolicy {
        max_keys: NonZeroUsize::MIN,
        max_key_bytes: NonZeroUsize::MIN,
        ..Default::default()
    };
    let mut retained = RetainedKeys {
        keys: HashSet::new(),
        bytes: 0,
    };
    assert!(retained.admit("a".into(), policy).is_ok());
    assert!(retained.admit("a".into(), policy).is_ok());
    assert!(matches!(
        retained.admit("b".into(), policy),
        Err(ValkeyStoreError::ScanBudgetExceeded { resource: "keys" })
    ));
    assert_eq!(retained.bytes, 1);
    assert_eq!(retained.keys.len(), 1);
}

#[test]
fn oversized_key_is_rejected_before_retention() {
    let policy = ValkeyScanPolicy {
        max_key_bytes: NonZeroUsize::MIN,
        ..Default::default()
    };
    let mut retained = RetainedKeys {
        keys: HashSet::new(),
        bytes: 0,
    };
    assert!(matches!(
        retained.admit("ab".into(), policy),
        Err(ValkeyStoreError::ScanBudgetExceeded {
            resource: "key_bytes"
        })
    ));
    assert!(retained.keys.is_empty());
    assert_eq!(retained.bytes, 0);
}
