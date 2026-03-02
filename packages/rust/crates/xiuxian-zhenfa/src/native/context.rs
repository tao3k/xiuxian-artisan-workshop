use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

use super::ZhenfaSignal;
use crate::JsonRpcMeta;

type ExtensionValue = Arc<dyn Any + Send + Sync>;
type ExtensionMap = HashMap<TypeId, ExtensionValue>;

/// Runtime context propagated to native zhenfa tools.
#[derive(Clone, Default)]
pub struct ZhenfaContext {
    /// Optional session identifier propagated from caller runtime.
    pub session_id: Option<String>,
    /// Optional trace identifier for correlation.
    pub trace_id: Option<String>,
    /// Optional correlation identifier used to link signals and outcomes.
    pub correlation_id: Option<String>,
    /// Additional metadata fields propagated by the caller.
    pub extra: HashMap<String, Value>,
    extensions: Arc<ExtensionMap>,
    signal_tx: Option<UnboundedSender<ZhenfaSignal>>,
}

impl ZhenfaContext {
    /// Build a context from explicit metadata fields.
    #[must_use]
    pub fn new(
        session_id: Option<String>,
        trace_id: Option<String>,
        extra: HashMap<String, Value>,
    ) -> Self {
        Self {
            session_id,
            trace_id,
            correlation_id: None,
            extra,
            extensions: Arc::default(),
            signal_tx: None,
        }
    }

    /// Build a context from optional JSON-RPC metadata.
    #[must_use]
    pub fn from_meta(meta: Option<JsonRpcMeta>) -> Self {
        meta.map_or_else(Self::default, Self::from)
    }

    /// Insert one owned extension value.
    ///
    /// Returns the previous extension for the same type when present.
    pub fn insert_extension<T>(&mut self, value: T) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.insert_shared_extension(Arc::new(value))
    }

    /// Insert one shared extension value.
    ///
    /// Returns the previous extension for the same type when present.
    pub fn insert_shared_extension<T>(&mut self, value: Arc<T>) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let previous = Arc::make_mut(&mut self.extensions).insert(TypeId::of::<T>(), value);
        previous.and_then(|erased| Arc::downcast::<T>(erased).ok())
    }

    /// Fetch one typed extension value.
    #[must_use]
    pub fn get_extension<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let value = self.extensions.get(&TypeId::of::<T>())?.clone();
        Arc::downcast::<T>(value).ok()
    }

    /// Returns true when one typed extension is registered.
    #[must_use]
    pub fn has_extension<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.extensions.contains_key(&TypeId::of::<T>())
    }

    /// Returns the number of registered extension types.
    #[must_use]
    pub fn extension_count(&self) -> usize {
        self.extensions.len()
    }

    /// Attach one asynchronous signal sender used by native tools.
    pub fn attach_signal_sender(&mut self, signal_tx: UnboundedSender<ZhenfaSignal>) {
        self.signal_tx = Some(signal_tx);
    }

    /// Set correlation identifier used for cross-node signal association.
    pub fn set_correlation_id(&mut self, correlation_id: Option<String>) {
        self.correlation_id = correlation_id;
    }

    /// Set correlation identifier only when no existing value is present.
    pub fn set_correlation_id_if_absent(&mut self, correlation_id: String) {
        if self.correlation_id.is_none() {
            self.correlation_id = Some(correlation_id);
        }
    }

    /// Emit one fire-and-forget runtime signal.
    ///
    /// Signal delivery errors are intentionally non-fatal and only logged.
    pub fn emit_signal(&self, signal: ZhenfaSignal) {
        let Some(signal_tx) = self.signal_tx.as_ref() else {
            return;
        };
        if let Err(error) = signal_tx.send(signal) {
            tracing::warn!(
                event = "zhenfa.signal.emit_failed",
                error = %error,
                "zhenfa signal emission failed"
            );
        }
    }
}

impl From<JsonRpcMeta> for ZhenfaContext {
    fn from(meta: JsonRpcMeta) -> Self {
        Self::new(meta.session_id, meta.trace_id, meta.extra)
    }
}

impl std::fmt::Debug for ZhenfaContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ZhenfaContext")
            .field("session_id", &self.session_id)
            .field("trace_id", &self.trace_id)
            .field("correlation_id", &self.correlation_id)
            .field("extra", &self.extra)
            .field("extensions", &self.extension_count())
            .field("has_signal_tx", &self.signal_tx.is_some())
            .finish()
    }
}

impl PartialEq for ZhenfaContext {
    fn eq(&self, other: &Self) -> bool {
        self.session_id == other.session_id
            && self.trace_id == other.trace_id
            && self.correlation_id == other.correlation_id
            && self.extra == other.extra
    }
}
