use super::control_error;
use crate::ControlError;
use xiuxian_db_store::ValkeyStoreError;

#[test]
fn unknown_mutation_remains_typed_at_control_boundary() {
    assert_eq!(
        control_error(ValkeyStoreError::OutcomeUnknown {
            operation: "claim",
            message: "response lost".into(),
        }),
        ControlError::OutcomeUnknown {
            operation: "claim",
            message: "response lost".into(),
        }
    );
}

#[test]
fn scan_exhaustion_is_not_an_ordinary_storage_failure() {
    assert_eq!(
        control_error(ValkeyStoreError::ScanBudgetExceeded { resource: "keys" }),
        ControlError::ObservationBudgetExceeded { resource: "keys" }
    );
}
