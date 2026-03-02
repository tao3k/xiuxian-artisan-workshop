//! Unit tests for Qianji safety guards.

use std::sync::Arc;
use xiuxian_qianji::executors::MockMechanism;
use xiuxian_qianji::{QianjiEngine, QianjiSafetyGuard, QianjiScheduler};

#[tokio::test]
async fn test_qianji_safety_static_cycle_detection() {
    let mut engine = QianjiEngine::new();
    let mech = Arc::new(MockMechanism {
        name: "A".to_string(),
        weight: 1.0,
    });

    let a = engine.add_mechanism("A", mech.clone());
    let b = engine.add_mechanism("B", mech.clone());

    engine.add_link(a, b, None, 1.0);
    engine.add_link(b, a, None, 1.0);

    let guard = QianjiSafetyGuard::new(10);
    let result = guard.audit_topology(&engine);

    let Err(error) = result else {
        panic!("cycle topology should fail safety audit");
    };
    assert!(
        error.to_string().contains("Infinite cycle detected"),
        "unexpected error message: {error}"
    );
}

#[tokio::test]
async fn test_qianji_runtime_loop_guard() {
    let mut engine = QianjiEngine::new();
    let mech = Arc::new(MockMechanism {
        name: "A".to_string(),
        weight: 1.0,
    });
    let _a = engine.add_mechanism("A", mech);

    let _scheduler = QianjiScheduler::new(engine);
}
