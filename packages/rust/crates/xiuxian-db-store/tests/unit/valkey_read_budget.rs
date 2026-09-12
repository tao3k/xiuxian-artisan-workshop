use super::{ReadBudget, ValkeyReadPolicy};
use crate::valkey::ValkeyStoreError;

#[test]
fn admission_is_shared_and_failure_is_terminal() -> Result<(), ValkeyStoreError> {
    let budget = ReadBudget::new(ValkeyReadPolicy {
        max_commands: 2,
        max_items: 2,
        max_bytes: 4,
        ..Default::default()
    });
    budget.admit(1, 1, 2)?;
    budget.admit(1, 1, 2)?;
    assert!(matches!(
        budget.admit(1, 0, 0),
        Err(ValkeyStoreError::ReadBudgetExceeded {
            resource: "commands"
        })
    ));
    assert!(matches!(
        budget.admit(0, 0, 0),
        Err(ValkeyStoreError::ReadBudgetExceeded {
            resource: "commands"
        })
    ));
    Ok(())
}

#[test]
fn aggregate_items_and_bytes_and_overflow_are_rejected() -> Result<(), ValkeyStoreError> {
    for (commands, items, bytes, resource) in [(0, 3, 0, "items"), (0, 0, 5, "bytes")] {
        let budget = ReadBudget::new(ValkeyReadPolicy {
            max_items: 2,
            max_bytes: 4,
            ..Default::default()
        });
        assert!(
            matches!(budget.admit(commands, items, bytes), Err(ValkeyStoreError::ReadBudgetExceeded { resource: r }) if r == resource)
        );
    }
    let budget = ReadBudget::new(ValkeyReadPolicy {
        max_bytes: usize::MAX,
        ..Default::default()
    });
    budget.admit(0, 0, usize::MAX)?;
    assert!(matches!(
        budget.admit(0, 0, 1),
        Err(ValkeyStoreError::ReadBudgetExceeded { resource: "bytes" })
    ));
    Ok(())
}

#[tokio::test]
async fn deadline_covers_pending_io_and_stays_exhausted() {
    let budget = ReadBudget::new(ValkeyReadPolicy {
        timeout: std::time::Duration::from_millis(1),
        ..Default::default()
    });
    let result = budget
        .within(std::future::pending::<Result<(), ValkeyStoreError>>())
        .await;
    assert!(matches!(
        result,
        Err(ValkeyStoreError::ReadBudgetExceeded {
            resource: "deadline"
        })
    ));
    assert!(budget.admit(0, 0, 0).is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_components_cannot_multiply_allowance() -> Result<(), Box<dyn std::error::Error>>
{
    let budget = std::sync::Arc::new(ReadBudget::new(ValkeyReadPolicy {
        max_commands: 8,
        ..Default::default()
    }));
    let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(32));
    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..32 {
        let budget = std::sync::Arc::clone(&budget);
        let barrier = std::sync::Arc::clone(&barrier);
        tasks.spawn(async move {
            barrier.wait().await;
            budget.admit(1, 0, 0).is_ok()
        });
    }
    let mut admitted = 0;
    while let Some(result) = tasks.join_next().await {
        admitted += usize::from(result?);
    }
    assert_eq!(admitted, 8);
    assert!(budget.admit(0, 0, 0).is_err());
    Ok(())
}
