//! Swarm orchestration for multi-agent concurrent execution.

mod discovery;
mod engine;
mod possession;

pub use discovery::{ClusterNodeIdentity, ClusterNodeRecord, GlobalSwarmRegistry};
pub use engine::{
    SwarmAgentConfig, SwarmAgentReport, SwarmEngine, SwarmExecutionOptions, SwarmExecutionReport,
};
pub use possession::{
    RemoteNodeRequest, RemoteNodeResponse, RemotePossessionBus, map_execution_error_to_response,
};
