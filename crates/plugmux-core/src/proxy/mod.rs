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
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
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
}
