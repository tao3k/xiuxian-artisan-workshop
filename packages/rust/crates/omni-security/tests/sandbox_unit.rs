//! Integration tests for `omni-security` sandbox models.

use omni_security::{SandboxConfig, SandboxMode, SandboxResult, SandboxRunner};

#[test]
fn test_sandbox_config_defaults() {
    let config = SandboxConfig::default();

    if cfg!(target_os = "linux") {
        assert_eq!(config.mode, SandboxMode::NsJail);
    } else {
        assert_eq!(config.mode, SandboxMode::Docker);
    }

    assert_eq!(config.memory_mb, 512);
    assert!((config.max_cpus - 1.0).abs() < f64::EPSILON);
    assert!(config.network_isolation);
}

#[test]
fn test_sandbox_runner_creation() {
    let runner = SandboxRunner::new();
    assert!(matches!(
        runner.mode(),
        SandboxMode::Docker | SandboxMode::NsJail
    ));
}

#[test]
fn test_sandbox_result_structure() {
    let result = SandboxResult {
        success: true,
        exit_code: 0,
        stdout: "test output".to_string(),
        stderr: String::new(),
        duration_ms: 100,
    };

    assert!(result.success);
    assert_eq!(result.exit_code, 0);
    assert!(result.duration_ms > 0);
}
