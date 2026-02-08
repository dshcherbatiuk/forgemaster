//! Converts rmcp `Tool` definitions to Claude API `ToolDefinition` format.

use rmcp::model::Tool;

use crate::claude_api::tool_definition::ToolDefinition;

/// Converts MCP tool definitions to Claude `tools` parameter format.
///
/// Maps rmcp `Tool` fields to `ToolDefinition`:
/// - `name: Cow<str>` → `name: String`
/// - `description: Option<Cow<str>>` → `description: Option<String>`
/// - `input_schema: Arc<JsonObject>` → `input_schema: serde_json::Value`
pub fn to_claude_tools(mcp_tools: &[Tool]) -> Vec<ToolDefinition> {
    mcp_tools
        .iter()
        .map(|mcp_tool| {
            let input_schema =
                serde_json::Value::Object((*mcp_tool.input_schema).clone());

            ToolDefinition {
                name: mcp_tool.name.to_string(),
                description: mcp_tool.description.as_ref().map(|d| d.to_string()),
                input_schema,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn make_tool(name: &str, description: Option<&str>, schema: serde_json::Value) -> Tool {
        let input_schema = match schema {
            serde_json::Value::Object(map) => Arc::new(map),
            _ => Arc::new(serde_json::Map::new()),
        };

        Tool {
            name: name.to_string().into(),
            title: None,
            description: description.map(|d| d.to_string().into()),
            input_schema,
            output_schema: None,
            annotations: None,
            icons: None,
            meta: None,
        }
    }

    #[test]
    fn converts_empty_list() {
        let result = to_claude_tools(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn converts_with_description() {
        let tools = vec![make_tool(
            "create_agent",
            Some("Create a child agent"),
            serde_json::json!({"type": "object"}),
        )];

        let claude_tools = to_claude_tools(&tools);
        assert_eq!(claude_tools.len(), 1);
        assert_eq!(claude_tools[0].name, "create_agent");
        assert_eq!(
            claude_tools[0].description.as_deref(),
            Some("Create a child agent")
        );
        assert_eq!(claude_tools[0].input_schema["type"], "object");
    }

    #[test]
    fn converts_without_description() {
        let tools = vec![make_tool(
            "list_agents",
            None,
            serde_json::json!({"type": "object"}),
        )];

        let claude_tools = to_claude_tools(&tools);
        assert_eq!(claude_tools[0].name, "list_agents");
        assert!(claude_tools[0].description.is_none());
    }

    #[test]
    fn converts_multiple_tools() {
        let tools = vec![
            make_tool(
                "create_agent",
                Some("Create agent"),
                serde_json::json!({"type": "object"}),
            ),
            make_tool(
                "get_status",
                Some("Get status"),
                serde_json::json!({"type": "object"}),
            ),
        ];

        let claude_tools = to_claude_tools(&tools);
        assert_eq!(claude_tools.len(), 2);
        assert_eq!(claude_tools[0].name, "create_agent");
        assert_eq!(claude_tools[1].name, "get_status");
    }

    #[test]
    fn preserves_input_schema() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "agent_type": {"type": "string"},
                "task_prompt": {"type": "string"}
            },
            "required": ["agent_type", "task_prompt"]
        });

        let tools = vec![make_tool("create_agent", None, schema.clone())];

        let claude_tools = to_claude_tools(&tools);
        assert_eq!(claude_tools[0].input_schema, schema);
    }
}
