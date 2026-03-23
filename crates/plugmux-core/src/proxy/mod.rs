pub mod http_sse;
pub mod stdio;

use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("failed to start server process: {0}")]
    SpawnFailed(String),
    #[error("server not initialized")]
    NotInitialized,
    #[error("tool call failed: {0}")]
    ToolCallFailed(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("timeout")]
    Timeout,
    #[error("approval required: {message}")]
    ApprovalRequired {
        action_id: String,
        message: String,
    },
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Option<Value>,
    pub annotations: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PromptInfo {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

#[async_trait]
pub trait McpClient: Send + Sync {
    /// Initialize the MCP connection (handshake with the upstream server).
    async fn initialize(&mut self) -> Result<(), ProxyError>;

    /// List all tools exposed by the upstream MCP server.
    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ProxyError>;

    /// Call a tool on the upstream MCP server.
    async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ProxyError>;

    /// Check whether the upstream MCP server is still reachable.
    async fn health_check(&self) -> bool;

    /// Gracefully shut down the connection to the upstream server.
    async fn shutdown(&mut self) -> Result<(), ProxyError>;

    /// List all resources exposed by the upstream MCP server.
    async fn list_resources(&self) -> Result<Vec<ResourceInfo>, ProxyError> {
        Ok(Vec::new())
    }

    /// Read a resource by URI from the upstream MCP server.
    async fn read_resource(&self, _uri: &str) -> Result<Value, ProxyError> {
        Err(ProxyError::Transport("resources not supported".into()))
    }

    /// List all prompts exposed by the upstream MCP server.
    async fn list_prompts(&self) -> Result<Vec<PromptInfo>, ProxyError> {
        Ok(Vec::new())
    }

    /// Get a prompt by name from the upstream MCP server.
    async fn get_prompt(&self, _name: &str, _args: Value) -> Result<Value, ProxyError> {
        Err(ProxyError::Transport("prompts not supported".into()))
    }

    /// Send roots to the upstream MCP server.
    async fn send_roots(&self, _roots: Value) -> Result<(), ProxyError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_info_with_optional_fields() {
        let tool = ToolInfo {
            name: "test".to_string(),
            description: "desc".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: Some(serde_json::json!({"type": "string"})),
            annotations: Some(serde_json::json!({"readOnlyHint": true})),
        };
        assert_eq!(tool.name, "test");
        assert!(tool.output_schema.is_some());
        assert!(tool.annotations.is_some());
    }

    #[test]
    fn test_resource_info_construction() {
        let res = ResourceInfo {
            uri: "file:///test".to_string(),
            name: "test".to_string(),
            description: Some("a test resource".to_string()),
            mime_type: Some("text/plain".to_string()),
        };
        assert_eq!(res.uri, "file:///test");
    }

    #[test]
    fn test_prompt_info_construction() {
        let prompt = PromptInfo {
            name: "code-review".to_string(),
            description: Some("Review code".to_string()),
            arguments: vec![PromptArgument {
                name: "language".to_string(),
                description: Some("Programming language".to_string()),
                required: true,
            }],
        };
        assert_eq!(prompt.arguments.len(), 1);
        assert!(prompt.arguments[0].required);
    }
}
