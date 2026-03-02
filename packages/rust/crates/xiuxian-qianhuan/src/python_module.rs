use crate::orchestrator::ThousandFacesOrchestrator;
use crate::persona::{PersonaProfile, PersonaRegistry};
use crate::transmuter::MockTransmuter;
use pyo3::prelude::*;
use std::sync::Arc;

/// Python wrapper for [`PersonaProfile`].
#[pyclass(name = "PersonaProfile")]
#[derive(Clone)]
pub struct PyPersonaProfile {
    /// Underlying Rust persona profile.
    pub inner: PersonaProfile,
}

/// Python wrapper for [`PersonaRegistry`].
#[pyclass(name = "PersonaRegistry")]
pub struct PyPersonaRegistry {
    /// Underlying Rust persona registry.
    pub inner: PersonaRegistry,
}

#[pymethods]
impl PyPersonaRegistry {
    /// Builds a registry preloaded with built-in personas.
    #[staticmethod]
    #[must_use]
    pub fn with_builtins() -> Self {
        Self {
            inner: PersonaRegistry::with_builtins(),
        }
    }

    /// Returns a persona by identifier if it exists.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<PyPersonaProfile> {
        self.inner
            .get(id)
            .map(|profile| PyPersonaProfile { inner: profile })
    }
}

/// Python wrapper for [`ThousandFacesOrchestrator`].
#[pyclass(name = "Orchestrator")]
pub struct PyOrchestrator {
    /// Underlying Rust orchestrator.
    pub inner: ThousandFacesOrchestrator,
}

#[pymethods]
impl PyOrchestrator {
    /// Creates an orchestrator with mock tone transmutation.
    #[new]
    #[must_use]
    pub fn new(genesis_rules: String) -> Self {
        // Defaulting to MockTransmuter for now in the thin Python slice
        Self {
            inner: ThousandFacesOrchestrator::new(genesis_rules, Some(Arc::new(MockTransmuter))),
        }
    }

    /// Assembles the snapshot. Python only sees the final string.
    ///
    /// # Errors
    ///
    /// Returns `PyRuntimeError` when prompt assembly or transmutation fails.
    pub fn assemble_snapshot(
        &self,
        py: Python<'_>,
        persona: PyPersonaProfile,
        narrative_blocks: Vec<String>,
        history: String,
    ) -> PyResult<String> {
        let persona = persona.inner;
        // We detach to release the GIL during potential computation.
        py.detach(move || {
            // Note: Since assemble_snapshot is async in Rust, we'd typically need a runtime here.
            // For the thin-slice API, we'll provide a synchronous wrapper or use a blocking bridge.
            futures::executor::block_on(self.inner.assemble_snapshot(
                &persona,
                narrative_blocks,
                &history,
            ))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }
}

#[pymodule]
fn _xiuxian_qianhuan(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPersonaProfile>()?;
    m.add_class::<PyPersonaRegistry>()?;
    m.add_class::<PyOrchestrator>()?;
    Ok(())
}
