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
            name: "plugmux__add_environment".to_string(),
            description: "Create a new environment. If name is not provided, ask the user what they want to name it. Optionally include server IDs to pre-populate the environment, or the user can add servers later.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Human-readable name for the environment (e.g. 'Work', 'Personal'). Ask the user if not known."
                    },
                    "servers": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional list of server IDs to add to the environment. Ask the user if they want to include servers now or add them later."
                    }
                },
                "required": ["name"]
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
    fn test_list_tools_returns_four_tools() {
        let tools = list_tools();
        assert_eq!(tools.len(), 4);
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
