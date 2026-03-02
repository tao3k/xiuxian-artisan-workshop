//! Sandbox executor implementations
//!
//! Executes pre-generated sandbox configurations.
//! This module does NOT parse NCL - it reads exported JSON.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use std::path::Path;
use std::time::Instant;
use tokio::process::Command;

/// Unified sandbox configuration (from NCL-exported JSON)
#[pyclass]
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Unique skill identifier
    #[pyo3(get)]
    pub skill_id: String,

    /// Execution mode: "EXEC" or "ONCE"
    #[pyo3(get)]
    pub mode: String,

    /// Container hostname
    #[pyo3(get)]
    pub hostname: String,

    /// Command to execute
    #[pyo3(get)]
    pub cmd: Vec<String>,

    /// Environment variables
    #[pyo3(get)]
    pub env: Vec<String>,

    /// Mount configurations
    #[pyo3(get)]
    pub mounts: Vec<MountConfig>,

    /// Max memory in bytes
    #[pyo3(get)]
    pub rlimit_as: u64,

    /// Max CPU seconds
    #[pyo3(get)]
    pub rlimit_cpu: u64,

    /// Max file size in bytes
    #[pyo3(get)]
    pub rlimit_fsize: u64,

    /// Seccomp mode (0=disabled, 2=enabled)
    #[pyo3(get)]
    pub seccomp_mode: u32,

    /// Log level
    #[pyo3(get)]
    pub log_level: String,
}

#[pymethods]
impl SandboxConfig {
    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    fn new(args: &Bound<'_, PyTuple>, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        if !args.is_empty() {
            if let Some(dict) = kwargs
                && !dict.is_empty()
            {
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "SandboxConfig accepts either positional arguments or keyword arguments, not both.",
                ));
            }
            return Self::from_positional_args(args);
        }

        if let Some(dict) = kwargs
            && !dict.is_empty()
        {
            return Self::from_keyword_args(dict);
        }

        Err(pyo3::exceptions::PyTypeError::new_err(
            "SandboxConfig requires 11 positional arguments or named keyword arguments.",
        ))
    }
}

impl SandboxConfig {
    fn from_positional_args(args: &Bound<'_, PyTuple>) -> PyResult<Self> {
        if args.len() != 11 {
            return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                "SandboxConfig expected 11 positional arguments, got {}.",
                args.len()
            )));
        }

        Ok(Self {
            skill_id: args.get_item(0)?.extract()?,
            mode: args.get_item(1)?.extract()?,
            hostname: args.get_item(2)?.extract()?,
            cmd: args.get_item(3)?.extract()?,
            env: args.get_item(4)?.extract()?,
            mounts: args.get_item(5)?.extract()?,
            rlimit_as: args.get_item(6)?.extract()?,
            rlimit_cpu: args.get_item(7)?.extract()?,
            rlimit_fsize: args.get_item(8)?.extract()?,
            seccomp_mode: args.get_item(9)?.extract()?,
            log_level: args.get_item(10)?.extract()?,
        })
    }

    fn from_keyword_args(kwargs: &Bound<'_, PyDict>) -> PyResult<Self> {
        fn required<'py>(kwargs: &Bound<'py, PyDict>, key: &str) -> PyResult<Bound<'py, PyAny>> {
            kwargs.get_item(key)?.ok_or_else(|| {
                pyo3::exceptions::PyTypeError::new_err(format!(
                    "SandboxConfig missing required argument: {key}."
                ))
            })
        }

        Ok(Self {
            skill_id: required(kwargs, "skill_id")?.extract()?,
            mode: required(kwargs, "mode")?.extract()?,
            hostname: required(kwargs, "hostname")?.extract()?,
            cmd: required(kwargs, "cmd")?.extract()?,
            env: required(kwargs, "env")?.extract()?,
            mounts: required(kwargs, "mounts")?.extract()?,
            rlimit_as: required(kwargs, "rlimit_as")?.extract()?,
            rlimit_cpu: required(kwargs, "rlimit_cpu")?.extract()?,
            rlimit_fsize: required(kwargs, "rlimit_fsize")?.extract()?,
            seccomp_mode: required(kwargs, "seccomp_mode")?.extract()?,
            log_level: required(kwargs, "log_level")?.extract()?,
        })
    }
}

/// Mount configuration
#[pyclass]
#[derive(Debug, Clone)]
pub struct MountConfig {
    /// Source path
    #[pyo3(get)]
    pub src: String,

    /// Destination path
    #[pyo3(get)]
    pub dst: String,

    /// Filesystem type
    #[pyo3(get)]
    pub fstype: String,

    /// Read-write access
    #[pyo3(get)]
    pub rw: bool,
}

#[pymethods]
impl MountConfig {
    #[new]
    fn new(src: String, dst: String, fstype: String, rw: bool) -> Self {
        MountConfig {
            src,
            dst,
            fstype,
            rw,
        }
    }
}

/// Execution result returned to Python
#[pyclass]
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    #[pyo3(get)]
    pub success: bool,

    /// Process exit code
    #[pyo3(get)]
    pub exit_code: Option<i32>,

    /// Standard output
    #[pyo3(get)]
    pub stdout: String,

    /// Standard error
    #[pyo3(get)]
    pub stderr: String,

    /// Execution time in milliseconds
    #[pyo3(get)]
    pub execution_time_ms: u64,

    /// Memory used in bytes
    #[pyo3(get)]
    pub memory_used_bytes: Option<u64>,

    /// Error message if failed
    #[pyo3(get)]
    pub error: Option<String>,
}

#[pymethods]
impl ExecutionResult {
    #[new]
    fn new(
        success: bool,
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
        execution_time_ms: u64,
        memory_used_bytes: Option<u64>,
        error: Option<String>,
    ) -> Self {
        ExecutionResult {
            success,
            exit_code,
            stdout,
            stderr,
            execution_time_ms,
            memory_used_bytes,
            error,
        }
    }
}

/// Sandbox executor trait - unified interface for all sandbox backends
#[async_trait::async_trait]
pub trait SandboxExecutor: Send + Sync {
    /// Execute a skill in the sandbox
    async fn execute(&self, config_path: &Path, input: &str) -> Result<ExecutionResult, String>;

    /// Get the executor name (e.g., "nsjail", "seatbelt")
    fn name(&self) -> &'static str;
}

fn millis_to_u64(ms: u128) -> u64 {
    u64::try_from(ms).unwrap_or(u64::MAX)
}

/// Execute a command with resource limits
async fn execute_with_limits(
    mut cmd: Command,
    timeout_secs: u64,
    _max_memory_bytes: u64,
) -> Result<ExecutionResult, String> {
    use tokio::time::timeout;

    let start_time = Instant::now();

    // Execute with timeout
    match timeout(std::time::Duration::from_secs(timeout_secs), cmd.output()).await {
        Ok(output) => match output {
            Ok(o) => {
                let elapsed = start_time.elapsed();
                Ok(ExecutionResult {
                    success: o.status.success(),
                    exit_code: o.status.code(),
                    stdout: String::from_utf8_lossy(&o.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&o.stderr).to_string(),
                    execution_time_ms: millis_to_u64(elapsed.as_millis()),
                    memory_used_bytes: None,
                    error: None,
                })
            }
            Err(e) => Ok(ExecutionResult {
                success: false,
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                execution_time_ms: millis_to_u64(start_time.elapsed().as_millis()),
                memory_used_bytes: None,
                error: Some(format!("Failed to execute: {e}")),
            }),
        },
        Err(_) => Ok(ExecutionResult {
            success: false,
            exit_code: Some(-1),
            stdout: String::new(),
            stderr: String::from("Timeout: execution exceeded limit"),
            execution_time_ms: millis_to_u64(start_time.elapsed().as_millis()),
            memory_used_bytes: None,
            error: Some(String::from("Timeout")),
        }),
    }
}

mod nsjail;
mod seatbelt;

pub use nsjail::NsJailExecutor;
pub use seatbelt::SeatbeltExecutor;
