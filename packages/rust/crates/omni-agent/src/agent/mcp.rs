use std::collections::HashSet;

use anyhow::Result;

use super::Agent;

const MEMORY_SEARCH_TOOL_NAME: &str = "memory.search_memory";
const MEMORY_SAVE_TOOL_NAME: &str = "memory.save_memory";

pub(super) struct ToolCallOutput {
    pub(super) text: String,
    pub(super) is_error: bool,
}

impl Agent {
    pub(super) fn soft_fail_mcp_tool_error_output(
        name: &str,
        error: &anyhow::Error,
    ) -> Option<ToolCallOutput> {
        soft_fail_mcp_tool_error_output(name, error)
    }

    /// List all tools (Native + Zhenfa + MCP) for the LLM.
    pub(super) async fn mcp_tools_for_llm(&self) -> Result<Option<Vec<serde_json::Value>>> {
        let mut tools = self.native_tools.list_for_llm();
        let mut seen_tool_names = collect_seen_tool_names(&tools);

        if let Some(ref zhenfa_tools) = self.zhenfa_tools {
            for tool in zhenfa_tools.list_for_llm() {
                push_unique_tool(&mut tools, &mut seen_tool_names, tool);
            }
        }

        if let Some(ref mcp) = self.mcp {
            let list = mcp.list_tools(None).await?;
            for t in list.tools {
                let mut obj = serde_json::Map::new();
                obj.insert(
                    "name".to_string(),
                    serde_json::Value::String(t.name.to_string()),
                );
                if let Some(ref d) = t.description {
                    obj.insert(
                        "description".to_string(),
                        serde_json::Value::String(d.to_string()),
                    );
                }
                obj.insert(
                    "parameters".to_string(),
                    serde_json::Value::Object(t.input_schema.as_ref().clone()),
                );
                push_unique_tool(
                    &mut tools,
                    &mut seen_tool_names,
                    serde_json::Value::Object(obj),
                );
            }
        }

        if tools.is_empty() {
            return Ok(None);
        }
        Ok(Some(tools))
    }

    /// Primary tool dispatcher: Native first, then Zhenfa bridge, then MCP.
    pub(super) async fn call_mcp_tool_with_diagnostics(
        &self,
        session_id: Option<&str>,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<ToolCallOutput> {
        // 1. Check Native Tools (internal Rust functions)
        if let Some(native_tool) = self.native_tools.get(name) {
            let context = super::native_tools::registry::NativeToolCallContext {
                session_id: session_id.map(ToString::to_string),
            };
            match native_tool.call(arguments, &context).await {
                Ok(text) => {
                    return Ok(ToolCallOutput {
                        text,
                        is_error: false,
                    });
                }
                Err(error) => {
                    return Ok(ToolCallOutput {
                        text: format!("Native tool error: {error}"),
                        is_error: true,
                    });
                }
            }
        }

        // 2. Optional zhenfa tool bridge (Rust matrix gateway)
        if let Some(ref zhenfa_tools) = self.zhenfa_tools
            && zhenfa_tools.handles_tool(name)
        {
            return match zhenfa_tools.call_tool(session_id, name, arguments).await {
                Ok(text) => Ok(ToolCallOutput {
                    text,
                    is_error: false,
                }),
                Err(error) => Ok(ToolCallOutput {
                    text: format!("Zhenfa tool error: {error}"),
                    is_error: true,
                }),
            };
        }

        // 3. Fallback to MCP tools (external servers)
        let Some(ref mcp) = self.mcp else {
            return Err(anyhow::anyhow!(
                "no native tool, zhenfa tool, or MCP client found for `{name}`"
            ));
        };
        let result = mcp.call_tool(name.to_string(), arguments).await?;
        let text: String = result
            .content
            .iter()
            .filter_map(|c| {
                if let rmcp::model::RawContent::Text(t) = &c.raw {
                    Some(t.text.as_str())
                } else {
                    None
                }
            })
            .collect();
        Ok(ToolCallOutput {
            text,
            is_error: result.is_error.unwrap_or(false),
        })
    }
}

fn collect_seen_tool_names(tools: &[serde_json::Value]) -> HashSet<String> {
    tools
        .iter()
        .filter_map(extract_tool_name)
        .map(ToString::to_string)
        .collect()
}

fn push_unique_tool(
    tools: &mut Vec<serde_json::Value>,
    seen_tool_names: &mut HashSet<String>,
    tool: serde_json::Value,
) {
    if let Some(name) = extract_tool_name(&tool)
        && !seen_tool_names.insert(name.to_string())
    {
        return;
    }
    tools.push(tool);
}

fn extract_tool_name(tool: &serde_json::Value) -> Option<&str> {
    tool.get("name")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

fn soft_fail_mcp_tool_error_output(name: &str, error: &anyhow::Error) -> Option<ToolCallOutput> {
    let message = format!("{error:#}");
    let lower = message.to_ascii_lowercase();
    if name == MEMORY_SAVE_TOOL_NAME {
        let error_kind = if is_timeout_error_message(&lower) {
            "timeout"
        } else {
            "save_failed"
        };
        tracing::warn!(
            event = "agent.mcp.tool.soft_fail",
            tool = name,
            error_kind,
            error = %message,
            "mcp tool failed while saving memory; degrading to soft tool error output"
        );
        return Some(ToolCallOutput {
            text: serde_json::json!({
                "ok": false,
                "degraded": true,
                "tool": name,
                "error_kind": error_kind,
                "message": "Memory save failed; continuing without blocking this turn.",
            })
            .to_string(),
            is_error: true,
        });
    }

    let is_embedding_timeout = lower.contains("embedding timed out")
        || (lower.contains("embedding")
            && lower.contains("timed out")
            && lower.contains("mcp error: -32603"));
    if name != MEMORY_SEARCH_TOOL_NAME || !is_embedding_timeout {
        return None;
    }
    tracing::warn!(
        event = "agent.mcp.tool.soft_fail",
        tool = name,
        error = %message,
        "mcp tool failed with embedding timeout; degrading to soft tool error output"
    );
    Some(ToolCallOutput {
        text: serde_json::json!({
            "ok": false,
            "degraded": true,
            "tool": name,
            "error_kind": "embedding_timeout",
            "message": "Embedding lookup timed out; continuing without tool result.",
        })
        .to_string(),
        is_error: true,
    })
}

fn is_timeout_error_message(lowercase_error: &str) -> bool {
    lowercase_error.contains("timed out")
        || lowercase_error.contains("timeout")
        || lowercase_error.contains("mcp.pool.call.waiting")
}
