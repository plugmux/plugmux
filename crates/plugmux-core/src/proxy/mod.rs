pub mod http_sse;
pub mod stdio;

use async_trait::async_trait;
use rmcp::{
    RoleClient,
    model::{CallToolRequestParams, GetPromptRequestParams, ReadResourceRequestParams},
    service::RunningService,
};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::Mutex;

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
    ApprovalRequired { action_id: String, message: String },
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

// ---------------------------------------------------------------------------
// Shared rmcp service helpers
// ---------------------------------------------------------------------------

/// Helper to get the running service or return `NotInitialized`.
fn get_service(
    guard: &Option<RunningService<RoleClient, ()>>,
) -> Result<&RunningService<RoleClient, ()>, ProxyError> {
    guard.as_ref().ok_or(ProxyError::NotInitialized)
}

/// Shared implementation of `list_tools` for any rmcp-backed client.
pub(crate) async fn rmcp_list_tools(
    service: &Mutex<Option<RunningService<RoleClient, ()>>>,
) -> Result<Vec<ToolInfo>, ProxyError> {
    let guard = service.lock().await;
    let svc = get_service(&guard)?;

    let tools = svc
        .list_all_tools()
        .await
        .map_err(|e| ProxyError::Transport(format!("list_tools failed: {e}")))?;

    Ok(tools
        .into_iter()
        .map(|t| ToolInfo {
            name: t.name.to_string(),
            description: t.description.as_deref().unwrap_or("").to_string(),
            input_schema: serde_json::to_value(&*t.input_schema)
                .unwrap_or(Value::Object(Default::default())),
            output_schema: t
                .output_schema
                .as_ref()
                .and_then(|s| serde_json::to_value(&**s).ok()),
            annotations: t
                .annotations
                .as_ref()
                .and_then(|a| serde_json::to_value(a).ok()),
        })
        .collect())
}

/// Shared implementation of `call_tool` for any rmcp-backed client.
pub(crate) async fn rmcp_call_tool(
    service: &Mutex<Option<RunningService<RoleClient, ()>>>,
    name: &str,
    args: Value,
) -> Result<Value, ProxyError> {
    let guard = service.lock().await;
    let svc = get_service(&guard)?;

    let params = match args {
        Value::Object(map) => CallToolRequestParams::new(name.to_string()).with_arguments(map),
        Value::Null => CallToolRequestParams::new(name.to_string()),
        other => {
            let mut map = serde_json::Map::new();
            map.insert("input".to_string(), other);
            CallToolRequestParams::new(name.to_string()).with_arguments(map)
        }
    };

    let result = svc
        .call_tool(params)
        .await
        .map_err(|e| ProxyError::ToolCallFailed(format!("{e}")))?;

    if result.is_error == Some(true) {
        let msg = result
            .content
            .iter()
            .filter_map(|c| {
                let val = serde_json::to_value(c).ok()?;
                val.get("text").and_then(|t| t.as_str()).map(String::from)
            })
            .collect::<Vec<_>>()
            .join("\n");
        return Err(ProxyError::ToolCallFailed(msg));
    }

    if let Some(structured) = result.structured_content {
        Ok(structured)
    } else {
        serde_json::to_value(&result.content)
            .map_err(|e| ProxyError::Transport(format!("failed to serialize result: {e}")))
    }
}

/// Shared implementation of `list_resources` for any rmcp-backed client.
pub(crate) async fn rmcp_list_resources(
    service: &Mutex<Option<RunningService<RoleClient, ()>>>,
) -> Result<Vec<ResourceInfo>, ProxyError> {
    let guard = service.lock().await;
    let svc = get_service(&guard)?;

    let resources = svc
        .list_all_resources()
        .await
        .map_err(|e| ProxyError::Transport(format!("list_resources failed: {e}")))?;

    Ok(resources
        .into_iter()
        .map(|r| ResourceInfo {
            uri: r.uri.clone(),
            name: r.name.clone(),
            description: r.description.clone(),
            mime_type: r.mime_type.clone(),
        })
        .collect())
}

/// Shared implementation of `read_resource` for any rmcp-backed client.
pub(crate) async fn rmcp_read_resource(
    service: &Mutex<Option<RunningService<RoleClient, ()>>>,
    uri: &str,
) -> Result<Value, ProxyError> {
    let guard = service.lock().await;
    let svc = get_service(&guard)?;

    let params = ReadResourceRequestParams::new(uri);
    let result = svc
        .read_resource(params)
        .await
        .map_err(|e| ProxyError::Transport(format!("read_resource failed: {e}")))?;

    serde_json::to_value(&result)
        .map_err(|e| ProxyError::Transport(format!("failed to serialize resource: {e}")))
}

/// Shared implementation of `list_prompts` for any rmcp-backed client.
pub(crate) async fn rmcp_list_prompts(
    service: &Mutex<Option<RunningService<RoleClient, ()>>>,
) -> Result<Vec<PromptInfo>, ProxyError> {
    let guard = service.lock().await;
    let svc = get_service(&guard)?;

    let prompts = svc
        .list_all_prompts()
        .await
        .map_err(|e| ProxyError::Transport(format!("list_prompts failed: {e}")))?;

    Ok(prompts
        .into_iter()
        .map(|p| PromptInfo {
            name: p.name.clone(),
            description: p.description.clone(),
            arguments: p
                .arguments
                .as_ref()
                .map(|args| {
                    args.iter()
                        .map(|a| PromptArgument {
                            name: a.name.clone(),
                            description: a.description.clone(),
                            required: a.required.unwrap_or(false),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect())
}

/// Shared implementation of `get_prompt` for any rmcp-backed client.
pub(crate) async fn rmcp_get_prompt(
    service: &Mutex<Option<RunningService<RoleClient, ()>>>,
    name: &str,
    args: Value,
) -> Result<Value, ProxyError> {
    let guard = service.lock().await;
    let svc = get_service(&guard)?;

    let params = match args {
        Value::Object(map) => GetPromptRequestParams::new(name).with_arguments(map),
        _ => GetPromptRequestParams::new(name),
    };

    let result = svc
        .get_prompt(params)
        .await
        .map_err(|e| ProxyError::Transport(format!("get_prompt failed: {e}")))?;

    serde_json::to_value(&result)
        .map_err(|e| ProxyError::Transport(format!("failed to serialize prompt: {e}")))
}

/// Shared implementation of `health_check` for any rmcp-backed client.
pub(crate) async fn rmcp_health_check(
    service: &Mutex<Option<RunningService<RoleClient, ()>>>,
) -> bool {
    let guard = service.lock().await;
    match guard.as_ref() {
        Some(svc) => !svc.is_closed(),
        None => false,
    }
}

/// Shared implementation of `shutdown` for any rmcp-backed client.
pub(crate) async fn rmcp_shutdown(
    service: &Mutex<Option<RunningService<RoleClient, ()>>>,
) -> Result<(), ProxyError> {
    let mut guard = service.lock().await;
    if let Some(mut running) = guard.take() {
        running
            .close()
            .await
            .map_err(|e| ProxyError::Transport(format!("shutdown error: {e}")))?;
    }
    Ok(())
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
