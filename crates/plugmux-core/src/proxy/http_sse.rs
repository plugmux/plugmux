//! HTTP + Streamable-HTTP MCP client — connects to an upstream MCP server over
//! HTTP using the Streamable HTTP transport from the `rmcp` crate with `reqwest`.

use async_trait::async_trait;
use rmcp::{
    RoleClient, ServiceExt,
    model::CallToolRequestParams,
    service::RunningService,
    transport::StreamableHttpClientTransport,
};
use serde_json::Value;
use tokio::sync::Mutex;

use super::{McpClient, ProxyError, ToolInfo};

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

        // `serve` (via ServiceExt on `()` which implements ClientHandler) performs
        // the MCP initialize handshake automatically:
        //   1. Sends `initialize` request with client info
        //   2. Receives server capabilities
        //   3. Sends `notifications/initialized`
        let running = ().serve(transport)
            .await
            .map_err(|e| ProxyError::Transport(format!("MCP initialization failed: {e}")))?;

        *self.service.lock().await = Some(running);
        Ok(())
    }

    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ProxyError> {
        let guard = self.service.lock().await;
        let svc = guard.as_ref().ok_or(ProxyError::NotInitialized)?;

        let tools = svc
            .list_all_tools()
            .await
            .map_err(|e| ProxyError::Transport(format!("list_tools failed: {e}")))?;

        Ok(tools
            .into_iter()
            .map(|t| ToolInfo {
                name: t.name.to_string(),
                description: t
                    .description
                    .as_deref()
                    .unwrap_or("")
                    .to_string(),
                input_schema: serde_json::to_value(&*t.input_schema)
                    .unwrap_or(Value::Object(Default::default())),
            })
            .collect())
    }

    async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ProxyError> {
        let guard = self.service.lock().await;
        let svc = guard.as_ref().ok_or(ProxyError::NotInitialized)?;

        let params = match args {
            Value::Object(map) => {
                CallToolRequestParams::new(name.to_string()).with_arguments(map)
            }
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

        // If the server reported an error, surface it.
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

        // Return structured content if present, otherwise serialize the content array.
        if let Some(structured) = result.structured_content {
            Ok(structured)
        } else {
            serde_json::to_value(&result.content)
                .map_err(|e| ProxyError::Transport(format!("failed to serialize result: {e}")))
        }
    }

    async fn health_check(&self) -> bool {
        let guard = self.service.lock().await;
        match guard.as_ref() {
            Some(svc) => !svc.is_closed(),
            None => false,
        }
    }

    async fn shutdown(&mut self) -> Result<(), ProxyError> {
        let mut guard = self.service.lock().await;
        if let Some(mut running) = guard.take() {
            running
                .close()
                .await
                .map_err(|e| ProxyError::Transport(format!("shutdown error: {e}")))?;
        }
        Ok(())
    }
}
