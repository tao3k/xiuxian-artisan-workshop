mod agent_ops;
mod storage;
mod types;

pub(crate) use types::SessionMemoryRecallSnapshotInput;
pub use types::{SessionMemoryRecallDecision, SessionMemoryRecallSnapshot};

#[cfg(test)]
use super::Agent;
#[cfg(test)]
pub(crate) use storage::{MEMORY_RECALL_SNAPSHOT_MESSAGE_NAME, snapshot_session_id};
#[cfg(test)]
pub(crate) use types::{
    EMBEDDING_SOURCE_EMBEDDING, EMBEDDING_SOURCE_EMBEDDING_REPAIRED, EMBEDDING_SOURCE_UNKNOWN,
};
