//! Tool definitions for the plugmux management layer.
//!
//! These tools are exposed on `/env/global` and allow LLMs to manage
//! plugmux itself: listing servers, enabling/disabling servers in
//! environments, and confirming pending approval actions.

use serde_json::json;

use crate::proxy::ToolInfo;

/// Return the full set of plugmux management tool schemas.
pub fn list_tools() -> Vec<ToolInfo> {
    vec![
        ToolInfo {
            name: "plugmux__list_servers".to_string(),
            description: "List all MCP servers with health status".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            output_schema: None,
            annotations: None,
        },
        ToolInfo {
            name: "plugmux__enable_server".to_string(),
            description: "Add a server to an environment".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "env_id": {
                        "type": "string",
                        "description": "The environment ID to add the server to"
                    },
                    "server_id": {
                        "type": "string",
                        "description": "The server ID to enable"
                    }
                },
                "required": ["env_id", "server_id"]
            }),
            output_schema: None,
            annotations: None,
        },
        ToolInfo {
            name: "plugmux__disable_server".to_string(),
            description: "Remove a server from an environment".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "env_id": {
                        "type": "string",
                        "description": "The environment ID to remove the server from"
                    },
                    "server_id": {
                        "type": "string",
                        "description": "The server ID to disable"
                    }
                },
                "required": ["env_id", "server_id"]
            }),
            output_schema: None,
            annotations: None,
        },
        ToolInfo {
            name: "plugmux__list_environments".to_string(),
            description: "List all environments".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            output_schema: None,
            annotations: None,
        },
        ToolInfo {
            name: "plugmux__server_status".to_string(),
            description: "Detailed status of a server".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "server_id": {
                        "type": "string",
                        "description": "The server ID to get status for"
                    }
                },
                "required": ["server_id"]
            }),
            output_schema: None,
            annotations: None,
        },
        ToolInfo {
            name: "plugmux__confirm_action".to_string(),
            description: "Confirm a pending approval action".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "action_id": {
                        "type": "string",
                        "description": "The action ID to confirm"
                    }
                },
                "required": ["action_id"]
            }),
            output_schema: None,
            annotations: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools_returns_six_tools() {
        let tools = list_tools();
        assert_eq!(tools.len(), 6);
    }

    #[test]
    fn test_tool_names_have_plugmux_prefix() {
        for tool in list_tools() {
            assert!(
                tool.name.starts_with("plugmux__"),
                "tool '{}' should start with 'plugmux__'",
                tool.name,
            );
        }
    }

    #[test]
    fn test_enable_server_has_required_fields() {
        let tools = list_tools();
        let enable = tools
            .iter()
            .find(|t| t.name == "plugmux__enable_server")
            .expect("enable_server tool should exist");
        let required = enable.input_schema["required"]
            .as_array()
            .expect("required should be an array");
        assert!(required.contains(&json!("env_id")));
        assert!(required.contains(&json!("server_id")));
    }
}
