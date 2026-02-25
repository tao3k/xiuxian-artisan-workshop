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
use xiuxian_qianhuan::{PersonaRegistry, ThousandFacesOrchestrator};
#[cfg(feature = "llm")]
use xiuxian_wendao::LinkGraphIndex;

#[pyclass(name = "QianjiEngine")]
pub struct PyQianjiEngine {
    pub inner: QianjiEngine,
}

#[pymethods]
impl PyQianjiEngine {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: QianjiEngine::new(),
        }
    }

    /// Adds a mock node for testing from Python.
    /// In production, we'd add real mechanisms like Seeker/Annotator.
    pub fn add_mock_node(&mut self, id: String, weight: f32) -> usize {
        let mech = Arc::new(MockMechanism {
            name: id.clone(),
            weight,
        });
        self.inner.add_mechanism(&id, mech).index()
    }

    pub fn add_link(&mut self, from: usize, to: usize, label: Option<String>, weight: f32) {
        use petgraph::stable_graph::NodeIndex;
        self.inner.add_link(
            NodeIndex::new(from),
            NodeIndex::new(to),
            label.as_deref(),
            weight,
        );
    }
}

#[pyclass(name = "QianjiScheduler")]
pub struct PyQianjiScheduler {
    pub inner: QianjiScheduler,
}

#[pymethods]
impl PyQianjiScheduler {
    #[new]
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
    pub fn run(&self, py: Python<'_>, context_json: String) -> PyResult<String> {
        let context: serde_json::Value = serde_json::from_str(&context_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        py.allow_threads(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt
                .block_on(self.inner.run(context))
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            serde_json::to_string(&result)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
        })
    }
}

#[cfg(feature = "llm")]
#[pyfunction]
pub fn run_master_research_array(
    py: Python<'_>,
    repo_path: String,
    query: String,
    api_key: String,
    base_url: String,
) -> PyResult<String> {
    py.allow_threads(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let index = Arc::new(
                LinkGraphIndex::build(Path::new(&repo_path))
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?,
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
fn _xiuxian_qianji(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyQianjiEngine>()?;
    m.add_class::<PyQianjiScheduler>()?;
    #[cfg(feature = "llm")]
    m.add_function(wrap_pyfunction!(run_master_research_array, m)?)?;
    Ok(())
}
