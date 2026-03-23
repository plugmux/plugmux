//! HTTP + Streamable-HTTP MCP client — connects to an upstream MCP server over
//! HTTP using the Streamable HTTP transport from the `rmcp` crate with `reqwest`.

use async_trait::async_trait;
use rmcp::{
    RoleClient, ServiceExt, service::RunningService, transport::StreamableHttpClientTransport,
};
use serde_json::Value;
use tokio::sync::Mutex;

use super::{
    McpClient, PromptInfo, ProxyError, ResourceInfo, ToolInfo, rmcp_call_tool, rmcp_get_prompt,
    rmcp_health_check, rmcp_list_prompts, rmcp_list_resources, rmcp_list_tools, rmcp_read_resource,
    rmcp_shutdown,
};

/// An MCP client that communicates with an upstream server over HTTP
/// (MCP Streamable HTTP transport — single POST endpoint with optional SSE streaming).
pub struct HttpSseMcpClient {
    url: String,
    /// The running rmcp service, populated after `initialize()`.
    service: Mutex<Option<RunningService<RoleClient, ()>>>,
}

impl HttpSseMcpClient {
    /// Create a new HTTP+SSE MCP client configuration.
    ///
    /// The connection is not established until [`McpClient::initialize`] is called.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            service: Mutex::new(None),
        }
    }
}

#[async_trait]
impl McpClient for HttpSseMcpClient {
    async fn initialize(&mut self) -> Result<(), ProxyError> {
        let transport = StreamableHttpClientTransport::from_uri(self.url.as_str());

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
