//! PyO3 module surface for `xiuxian-qianji`.

use crate::engine::QianjiEngine;
use crate::executors::MockMechanism;
use crate::scheduler::QianjiScheduler;
use pyo3::prelude::*;
use std::sync::Arc;

#[cfg(feature = "llm")]
use std::path::Path;
#[cfg(feature = "llm")]
use xiuxian_llm::llm::OpenAIClient;
#[cfg(feature = "llm")]
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
#[cfg(feature = "llm")]
use xiuxian_wendao::LinkGraphIndex;

/// Python wrapper exposing `QianjiEngine`.
#[pyclass(name = "QianjiEngine")]
pub struct PyQianjiEngine {
    /// Inner Rust engine instance.
    pub inner: QianjiEngine,
}

#[pymethods]
impl PyQianjiEngine {
    /// Creates an empty `QianjiEngine`.
    #[new]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: QianjiEngine::new(),
        }
    }

    /// Adds a mock node for testing from Python.
    /// In production, we'd add real mechanisms like Seeker/Annotator.
    pub fn add_mock_node(&mut self, id: &str, weight: f32) -> usize {
        let id_owned = id.to_string();
        let mech = Arc::new(MockMechanism {
            name: id_owned.clone(),
            weight,
        });
        self.inner.add_mechanism(&id_owned, mech).index()
    }

    /// Adds a directed edge between two node indices.
    pub fn add_link(&mut self, from: usize, to: usize, label: Option<&str>, weight: f32) {
        use petgraph::stable_graph::NodeIndex;
        self.inner
            .add_link(NodeIndex::new(from), NodeIndex::new(to), label, weight);
    }
}

impl Default for PyQianjiEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Python wrapper exposing `QianjiScheduler`.
#[pyclass(name = "QianjiScheduler")]
pub struct PyQianjiScheduler {
    /// Inner Rust scheduler instance.
    pub inner: QianjiScheduler,
}

#[pymethods]
impl PyQianjiScheduler {
    /// Creates a scheduler from an existing engine.
    #[new]
    #[must_use]
    pub fn new(engine: &PyQianjiEngine) -> Self {
        // Cloning the engine into the scheduler
        // In a real scenario, we might want to share ownership better
        Self {
            inner: QianjiScheduler::new(QianjiEngine {
                graph: engine.inner.graph.clone(),
            }),
        }
    }

    /// Runs the scheduler asynchronously from Python.
    ///
    /// # Errors
    ///
    /// Returns `PyValueError` for invalid JSON payload and `PyRuntimeError`
    /// for runtime creation or scheduler execution failures.
    pub fn run(&self, py: Python<'_>, context_json: &str) -> PyResult<String> {
        let context: serde_json::Value = serde_json::from_str(context_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        py.detach(|| {
            let rt = tokio::runtime::Runtime::new().map_err(|error| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to create Tokio runtime: {error}"
                ))
            })?;
            let result = rt
                .block_on(self.inner.run(context))
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            serde_json::to_string(&result)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
        })
    }
}

impl Default for PyQianjiScheduler {
    fn default() -> Self {
        Self {
            inner: QianjiScheduler::new(QianjiEngine::new()),
        }
    }
}

#[cfg(feature = "llm")]
#[pyfunction]
/// Runs the built-in master-research workflow and returns the final context as JSON.
///
/// # Errors
///
/// Returns `PyRuntimeError` for runtime, indexing, compilation, or scheduler
/// failures and `PyValueError` when result serialization fails.
pub fn run_master_research_array(
    py: Python<'_>,
    repo_path: &str,
    query: &str,
    api_key: &str,
    base_url: &str,
) -> PyResult<String> {
    let repo_path = repo_path.to_string();
    let query = query.to_string();
    let api_key = api_key.to_string();
    let base_url = base_url.to_string();
    py.detach(move || {
        let rt = tokio::runtime::Runtime::new().map_err(|error| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to create Tokio runtime: {error}"
            ))
        })?;
        rt.block_on(async move {
            let index = Arc::new(
                LinkGraphIndex::build(Path::new(&repo_path))
                    .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.clone()))?,
            );
            let orchestrator = Arc::new(ThousandFacesOrchestrator::new("Rules".to_string(), None));
            let registry = Arc::new(PersonaRegistry::with_builtins());
            let llm_client = Arc::new(OpenAIClient {
                api_key,
                base_url,
                http: reqwest::Client::new(),
            });

            let compiler = crate::engine::compiler::QianjiCompiler::new(
                index,
                orchestrator,
                registry,
                Some(llm_client),
            );

            let master_toml = include_str!("../resources/research_master.toml");
            let engine = compiler
                .compile(master_toml)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            let scheduler = QianjiScheduler::new(engine);

            let result = scheduler
                .run(serde_json::json!({
                    "query": query,
                }))
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            serde_json::to_string(&result)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
        })
    })
}

#[pymodule]
fn _xiuxian_qianji(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyQianjiEngine>()?;
    m.add_class::<PyQianjiScheduler>()?;
    #[cfg(feature = "llm")]
    m.add_function(wrap_pyfunction!(run_master_research_array, m)?)?;
    Ok(())
}
