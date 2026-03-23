//! Stdio MCP client — spawns a child process and communicates over stdin/stdout
//! using the MCP JSON-RPC 2.0 protocol, backed by the `rmcp` crate.

use async_trait::async_trait;
use rmcp::{RoleClient, ServiceExt, service::RunningService, transport::TokioChildProcess};
use serde_json::Value;
use tokio::sync::Mutex;

use super::{
    McpClient, PromptInfo, ProxyError, ResourceInfo, ToolInfo, rmcp_call_tool, rmcp_get_prompt,
    rmcp_health_check, rmcp_list_prompts, rmcp_list_resources, rmcp_list_tools, rmcp_read_resource,
    rmcp_shutdown,
};

/// An MCP client that communicates with an upstream server over stdio (child process).
pub struct StdioMcpClient {
    command: String,
    args: Vec<String>,
    /// The running rmcp service, populated after `initialize()`.
    service: Mutex<Option<RunningService<RoleClient, ()>>>,
}

impl StdioMcpClient {
    /// Create a new stdio MCP client configuration.
    ///
    /// The child process is not spawned until [`McpClient::initialize`] is called.
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            command: command.into(),
            args,
            service: Mutex::new(None),
        }
    }
}

#[async_trait]
impl McpClient for StdioMcpClient {
    async fn initialize(&mut self) -> Result<(), ProxyError> {
        let mut cmd = tokio::process::Command::new(&self.command);
        cmd.args(&self.args);

        let transport =
            TokioChildProcess::new(cmd).map_err(|e| ProxyError::SpawnFailed(e.to_string()))?;

        let running = ()
            .serve(transport)
            .await
            .map_err(|e| ProxyError::Transport(format!("MCP initialization failed: {e}")))?;

        *self.service.lock().await = Some(running);
        Ok(())
    }

    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ProxyError> {
        rmcp_list_tools(&self.service).await
    }

    async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ProxyError> {
        rmcp_call_tool(&self.service, name, args).await
    }

    async fn list_resources(&self) -> Result<Vec<ResourceInfo>, ProxyError> {
        rmcp_list_resources(&self.service).await
    }

    async fn read_resource(&self, uri: &str) -> Result<Value, ProxyError> {
        rmcp_read_resource(&self.service, uri).await
    }

    async fn list_prompts(&self) -> Result<Vec<PromptInfo>, ProxyError> {
        rmcp_list_prompts(&self.service).await
    }

    async fn get_prompt(&self, name: &str, args: Value) -> Result<Value, ProxyError> {
        rmcp_get_prompt(&self.service, name, args).await
    }

    // TODO: implement send_roots when rmcp supports arbitrary notifications

    async fn health_check(&self) -> bool {
        rmcp_health_check(&self.service).await
    }

    async fn shutdown(&mut self) -> Result<(), ProxyError> {
        rmcp_shutdown(&self.service).await
    }
}
