//! Agent and Router schemas bundled as static strings.

/// JSON schema: `omni.agent.route_trace.v1`.
pub const AGENT_ROUTE_TRACE_V1: &str =
    include_str!("../resources/omni.agent.route_trace.v1.schema.json");
/// JSON schema: `omni.agent.server_info.v1`.
pub const AGENT_SERVER_INFO_V1: &str =
    include_str!("../resources/omni.agent.server_info.v1.schema.json");
/// JSON schema: `omni.agent.session_closed.v1`.
pub const AGENT_SESSION_CLOSED_V1: &str =
    include_str!("../resources/omni.agent.session_closed.v1.schema.json");
/// JSON schema: `omni.router.route_test.v1`.
pub const ROUTER_ROUTE_TEST_V1: &str =
    include_str!("../resources/omni.router.route_test.v1.schema.json");
/// JSON schema: `omni.router.routing_search.v1`.
pub const ROUTER_ROUTING_SEARCH_V1: &str =
    include_str!("../resources/omni.router.routing_search.v1.schema.json");
/// JSON schema: `omni.router.search_config.v1`.
pub const ROUTER_SEARCH_CONFIG_V1: &str =
    include_str!("../resources/omni.router.search_config.v1.schema.json");
/// JSON schema: `omni.discover.match.v1`.
pub const DISCOVER_MATCH_V1: &str = include_str!("../resources/omni.discover.match.v1.schema.json");
/// JSON schema: `omni.skills_monitor.signals.v1`.
pub const SKILLS_MONITOR_SIGNALS_V1: &str =
    include_str!("../resources/omni.skills_monitor.signals.v1.schema.json");

/// Lookup a bundled schema by canonical schema name.
pub fn get_schema(name: &str) -> Option<&'static str> {
    match name {
        "omni.agent.route_trace.v1" => Some(AGENT_ROUTE_TRACE_V1),
        "omni.agent.server_info.v1" => Some(AGENT_SERVER_INFO_V1),
        "omni.agent.session_closed.v1" => Some(AGENT_SESSION_CLOSED_V1),
        "omni.router.route_test.v1" => Some(ROUTER_ROUTE_TEST_V1),
        "omni.router.routing_search.v1" => Some(ROUTER_ROUTING_SEARCH_V1),
        "omni.router.search_config.v1" => Some(ROUTER_SEARCH_CONFIG_V1),
        "omni.discover.match.v1" => Some(DISCOVER_MATCH_V1),
        "omni.skills_monitor.signals.v1" => Some(SKILLS_MONITOR_SIGNALS_V1),
        _ => None,
    }
}
