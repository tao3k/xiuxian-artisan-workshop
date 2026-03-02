//! Global schema registry for Omni.
//! Schemas are physically stored in their owner crates but exported here for Python accessibility.

// --- xiuxian-wendao schemas ---
const LINK_GRAPH_RECORD_V1: &str =
    include_str!("../resources/omni.link_graph.record.v1.schema.json");
const LINK_GRAPH_RETRIEVAL_PLAN_V1: &str =
    include_str!("../resources/omni.link_graph.retrieval_plan.v1.schema.json");
const LINK_GRAPH_SEARCH_OPTIONS_V1: &str =
    include_str!("../resources/omni.link_graph.search_options.v1.schema.json");
const LINK_GRAPH_SEARCH_OPTIONS_V2: &str =
    include_str!("../resources/omni.link_graph.search_options.v2.schema.json");
const LINK_GRAPH_STATS_CACHE_V1: &str =
    include_str!("../resources/omni.link_graph.stats.cache.v1.schema.json");
const LINK_GRAPH_VALKEY_CACHE_SNAPSHOT_V1: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.valkey_cache_snapshot.v1.schema.json");
const LINK_GRAPH_VALKEY_CACHE_SNAPSHOT_OMNI_V1: &str =
    include_str!("../resources/omni.link_graph.valkey_cache_snapshot.v1.schema.json");
const LINK_GRAPH_SALIENCY_V1: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.saliency.v1.schema.json");
const LINK_GRAPH_STATS_CACHE_WENDAO_V1: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.stats.cache.v1.schema.json");
const LINK_GRAPH_SUGGESTED_LINK_DECISION_V1: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.suggested_link_decision.v1.schema.json");
const LINK_GRAPH_SUGGESTED_LINK_V1: &str =
    include_str!("../resources/xiuxian_wendao.link_graph.suggested_link.v1.schema.json");
const HMAS_CONCLUSION_V1: &str =
    include_str!("../resources/xiuxian_wendao.hmas.conclusion.v1.schema.json");
const HMAS_DIGITAL_THREAD_V1: &str =
    include_str!("../resources/xiuxian_wendao.hmas.digital_thread.v1.schema.json");
const HMAS_EVIDENCE_V1: &str =
    include_str!("../resources/xiuxian_wendao.hmas.evidence.v1.schema.json");
const HMAS_TASK_V1: &str = include_str!("../resources/xiuxian_wendao.hmas.task.v1.schema.json");

// --- omni-agent schemas ---
const AGENT_ROUTE_TRACE_V1: &str =
    include_str!("../../omni-agent/resources/omni.agent.route_trace.v1.schema.json");
const AGENT_SERVER_INFO_V1: &str =
    include_str!("../../omni-agent/resources/omni.agent.server_info.v1.schema.json");
const AGENT_SESSION_CLOSED_V1: &str =
    include_str!("../../omni-agent/resources/omni.agent.session_closed.v1.schema.json");
const ROUTER_ROUTE_TEST_V1: &str =
    include_str!("../../omni-agent/resources/omni.router.route_test.v1.schema.json");
const ROUTER_ROUTING_SEARCH_V1: &str =
    include_str!("../../omni-agent/resources/omni.router.routing_search.v1.schema.json");
const ROUTER_SEARCH_CONFIG_V1: &str =
    include_str!("../../omni-agent/resources/omni.router.search_config.v1.schema.json");
const DISCOVER_MATCH_V1: &str =
    include_str!("../../omni-agent/resources/omni.discover.match.v1.schema.json");
const SKILLS_MONITOR_SIGNALS_V1: &str =
    include_str!("../../omni-agent/resources/omni.skills_monitor.signals.v1.schema.json");

// --- omni-memory schemas ---
const MEMORY_GATE_EVENT_V1: &str =
    include_str!("../../omni-memory/resources/omni.memory.gate_event.v1.schema.json");

// --- xiuxian-skills schemas ---
const SKILL_METADATA_V1: &str =
    include_str!("../../xiuxian-skills/resources/skill_metadata.schema.json");
const SKILL_COMMAND_INDEX_V1: &str =
    include_str!("../../xiuxian-skills/resources/omni.skill.command_index.v1.schema.json");

// --- omni-vector schemas ---
const VECTOR_HYBRID_V1: &str =
    include_str!("../../omni-vector/resources/omni.vector.hybrid.v1.schema.json");
const VECTOR_SEARCH_V1: &str =
    include_str!("../../omni-vector/resources/omni.vector.search.v1.schema.json");
const VECTOR_TOOL_SEARCH_V1: &str =
    include_str!("../../omni-vector/resources/omni.vector.tool_search.v1.schema.json");

// --- xiuxian-mcp schemas ---
const MCP_TOOL_RESULT_V1: &str =
    include_str!("../../xiuxian-mcp/resources/omni.mcp.tool_result.v1.schema.json");

/// Returns the JSON schema content for a canonical schema name.
#[must_use]
pub fn get_schema(name: &str) -> Option<&'static str> {
    match name {
        "omni.link_graph.record.v1" => Some(LINK_GRAPH_RECORD_V1),
        "omni.link_graph.retrieval_plan.v1" => Some(LINK_GRAPH_RETRIEVAL_PLAN_V1),
        "omni.link_graph.search_options.v1" => Some(LINK_GRAPH_SEARCH_OPTIONS_V1),
        "omni.link_graph.search_options.v2" => Some(LINK_GRAPH_SEARCH_OPTIONS_V2),
        "omni.link_graph.stats.cache.v1" => Some(LINK_GRAPH_STATS_CACHE_V1),
        "omni.link_graph.valkey_cache_snapshot.v1" => {
            Some(LINK_GRAPH_VALKEY_CACHE_SNAPSHOT_OMNI_V1)
        }
        "xiuxian_wendao.link_graph.valkey_cache_snapshot.v1" => {
            Some(LINK_GRAPH_VALKEY_CACHE_SNAPSHOT_V1)
        }
        "xiuxian_wendao.link_graph.saliency.v1" => Some(LINK_GRAPH_SALIENCY_V1),
        "xiuxian_wendao.link_graph.stats.cache.v1" => Some(LINK_GRAPH_STATS_CACHE_WENDAO_V1),
        "xiuxian_wendao.link_graph.suggested_link_decision.v1" => {
            Some(LINK_GRAPH_SUGGESTED_LINK_DECISION_V1)
        }
        "xiuxian_wendao.link_graph.suggested_link.v1" => Some(LINK_GRAPH_SUGGESTED_LINK_V1),
        "xiuxian_wendao.hmas.conclusion.v1" => Some(HMAS_CONCLUSION_V1),
        "xiuxian_wendao.hmas.digital_thread.v1" => Some(HMAS_DIGITAL_THREAD_V1),
        "xiuxian_wendao.hmas.evidence.v1" => Some(HMAS_EVIDENCE_V1),
        "xiuxian_wendao.hmas.task.v1" => Some(HMAS_TASK_V1),

        "omni.agent.route_trace.v1" => Some(AGENT_ROUTE_TRACE_V1),
        "omni.agent.server_info.v1" => Some(AGENT_SERVER_INFO_V1),
        "omni.agent.session_closed.v1" => Some(AGENT_SESSION_CLOSED_V1),
        "omni.router.route_test.v1" => Some(ROUTER_ROUTE_TEST_V1),
        "omni.router.routing_search.v1" => Some(ROUTER_ROUTING_SEARCH_V1),
        "omni.router.search_config.v1" => Some(ROUTER_SEARCH_CONFIG_V1),
        "omni.discover.match.v1" => Some(DISCOVER_MATCH_V1),
        "omni.skills_monitor.signals.v1" => Some(SKILLS_MONITOR_SIGNALS_V1),

        "omni.memory.gate_event.v1" => Some(MEMORY_GATE_EVENT_V1),

        "skill_metadata.schema" => Some(SKILL_METADATA_V1),
        "omni.skill.command_index.v1" => Some(SKILL_COMMAND_INDEX_V1),

        "omni.vector.hybrid.v1" => Some(VECTOR_HYBRID_V1),
        "omni.vector.search.v1" => Some(VECTOR_SEARCH_V1),
        "omni.vector.tool_search.v1" => Some(VECTOR_TOOL_SEARCH_V1),

        "omni.mcp.tool_result.v1" => Some(MCP_TOOL_RESULT_V1),
        _ => None,
    }
}
