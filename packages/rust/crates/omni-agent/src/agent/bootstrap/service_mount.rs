use serde::{Deserialize, Serialize};

/// Service mount lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceMountStatus {
    Mounted,
    Skipped,
    Failed,
}

impl ServiceMountStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Mounted => "mounted",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
        }
    }
}

/// Standardized mount metadata for service wiring.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceMountMeta {
    pub endpoint: Option<String>,
    pub storage: Option<String>,
    pub detail: Option<String>,
}

impl ServiceMountMeta {
    /// Add endpoint metadata.
    #[must_use]
    pub fn endpoint(mut self, value: impl Into<String>) -> Self {
        self.endpoint = Some(value.into());
        self
    }

    /// Add storage metadata.
    #[must_use]
    pub fn storage(mut self, value: impl Into<String>) -> Self {
        self.storage = Some(value.into());
        self
    }

    /// Add details metadata.
    #[must_use]
    pub fn detail(mut self, value: impl Into<String>) -> Self {
        self.detail = Some(value.into());
        self
    }
}

/// Durable mount record exposed for runtime diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceMountRecord {
    pub service: String,
    pub category: String,
    pub status: ServiceMountStatus,
    pub endpoint: Option<String>,
    pub storage: Option<String>,
    pub detail: Option<String>,
}

/// In-memory catalog used during bootstrap and emitted as standardized logs.
#[derive(Debug, Default)]
pub(super) struct ServiceMountCatalog {
    records: Vec<ServiceMountRecord>,
}

impl ServiceMountCatalog {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn mounted(
        &mut self,
        service: impl Into<String>,
        category: impl Into<String>,
        meta: ServiceMountMeta,
    ) {
        self.record(service, category, ServiceMountStatus::Mounted, meta);
    }

    pub(super) fn skipped(
        &mut self,
        service: impl Into<String>,
        category: impl Into<String>,
        meta: ServiceMountMeta,
    ) {
        self.record(service, category, ServiceMountStatus::Skipped, meta);
    }

    pub(super) fn failed(
        &mut self,
        service: impl Into<String>,
        category: impl Into<String>,
        meta: ServiceMountMeta,
    ) {
        self.record(service, category, ServiceMountStatus::Failed, meta);
    }

    fn record(
        &mut self,
        service: impl Into<String>,
        category: impl Into<String>,
        status: ServiceMountStatus,
        meta: ServiceMountMeta,
    ) {
        let record = ServiceMountRecord {
            service: service.into(),
            category: category.into(),
            status,
            endpoint: meta.endpoint,
            storage: meta.storage,
            detail: meta.detail,
        };
        tracing::info!(
            event = "agent.service.mount",
            service = %record.service,
            category = %record.category,
            status = record.status.as_str(),
            endpoint = %record.endpoint.as_deref().unwrap_or(""),
            storage = %record.storage.as_deref().unwrap_or(""),
            detail = %record.detail.as_deref().unwrap_or(""),
            "service mount recorded"
        );
        self.records.push(record);
    }

    pub(super) fn finish(self) -> Vec<ServiceMountRecord> {
        let total = self.records.len();
        let mounted = self
            .records
            .iter()
            .filter(|record| record.status == ServiceMountStatus::Mounted)
            .count();
        let skipped = self
            .records
            .iter()
            .filter(|record| record.status == ServiceMountStatus::Skipped)
            .count();
        let failed = self
            .records
            .iter()
            .filter(|record| record.status == ServiceMountStatus::Failed)
            .count();
        tracing::info!(
            event = "agent.service.mount.summary",
            total,
            mounted,
            skipped,
            failed,
            "service mount summary"
        );
        self.records
    }
}
