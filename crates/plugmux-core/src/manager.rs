//! Server Manager — owns all running MCP client connections.
//!
//! Provides lifecycle control (start, stop, shutdown) and delegates
//! tool listing / tool calling to the underlying [`McpClient`] instances.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::proxy::http_sse::HttpSseMcpClient;
use crate::proxy::stdio::StdioMcpClient;
use crate::proxy::{McpClient, PromptInfo, ProxyError, ResourceInfo, ToolInfo};
use crate::server::{HealthStatus, ServerConfig, Transport};

/// A running MCP server together with its configuration and health status.
pub struct ManagedServer {
    pub config: ServerConfig,
    pub client: Box<dyn McpClient>,
    pub health: HealthStatus,
}

/// Owns all running MCP client connections.
///
/// All methods that touch the internal map acquire the `RwLock` as needed,
/// so `ServerManager` is safe to share behind an `Arc`.
pub struct ServerManager {
    servers: Arc<RwLock<HashMap<String, ManagedServer>>>,
}

impl ServerManager {
    /// Create a new, empty server manager.
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a server: create the right client type based on transport, initialize it,
    /// and add it to the managed set.
    pub async fn start_server(&self, config: ServerConfig) -> Result<(), ProxyError> {
        let id = config.id.clone();

        let mut client: Box<dyn McpClient> = match config.transport {
            Transport::Stdio => {
                let command = config
                    .command
                    .as_deref()
                    .ok_or_else(|| {
                        ProxyError::SpawnFailed("stdio transport requires a command".into())
                    })?
                    .to_string();
                let args = config.args.clone().unwrap_or_default();
                Box::new(StdioMcpClient::new(command, args))
            }
            Transport::Http => {
                let url = config
                    .url
                    .as_deref()
                    .ok_or_else(|| ProxyError::Transport("http transport requires a url".into()))?
                    .to_string();
                Box::new(HttpSseMcpClient::new(url))
            }
        };

        info!(server_id = %id, "initializing MCP client");
        client.initialize().await?;

        let managed = ManagedServer {
            config,
            client,
            health: HealthStatus::Healthy,
        };

        self.servers.write().await.insert(id.clone(), managed);
        info!(server_id = %id, "server started successfully");
        Ok(())
    }

    /// Stop a server: shut down the client and remove it from the managed set.
    pub async fn stop_server(&self, id: &str) -> Result<(), ProxyError> {
        let mut map = self.servers.write().await;
        let mut managed = map
            .remove(id)
            .ok_or_else(|| ProxyError::Transport(format!("server not found: {id}")))?;

        info!(server_id = %id, "stopping server");
        managed.client.shutdown().await?;
        info!(server_id = %id, "server stopped");
        Ok(())
    }

    /// List all tools exposed by a specific server.
    pub async fn list_tools(&self, server_id: &str) -> Result<Vec<ToolInfo>, ProxyError> {
        let map = self.servers.read().await;
        let managed = map
            .get(server_id)
            .ok_or_else(|| ProxyError::Transport(format!("server not found: {server_id}")))?;
        managed.client.list_tools().await
    }

    /// Call a tool on a specific server.
    pub async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        args: Value,
    ) -> Result<Value, ProxyError> {
        let map = self.servers.read().await;
        let managed = map
            .get(server_id)
            .ok_or_else(|| ProxyError::Transport(format!("server not found: {server_id}")))?;
        managed.client.call_tool(tool_name, args).await
    }

    /// List all resources exposed by a specific server.
    pub async fn list_resources(&self, server_id: &str) -> Result<Vec<ResourceInfo>, ProxyError> {
        let map = self.servers.read().await;
        let managed = map
            .get(server_id)
            .ok_or_else(|| ProxyError::Transport(format!("server not found: {server_id}")))?;
        managed.client.list_resources().await
    }

    /// Read a resource from a specific server.
    pub async fn read_resource(&self, server_id: &str, uri: &str) -> Result<Value, ProxyError> {
        let map = self.servers.read().await;
        let managed = map
            .get(server_id)
            .ok_or_else(|| ProxyError::Transport(format!("server not found: {server_id}")))?;
        managed.client.read_resource(uri).await
    }

    /// List all prompts exposed by a specific server.
    pub async fn list_prompts(&self, server_id: &str) -> Result<Vec<PromptInfo>, ProxyError> {
        let map = self.servers.read().await;
        let managed = map
            .get(server_id)
            .ok_or_else(|| ProxyError::Transport(format!("server not found: {server_id}")))?;
        managed.client.list_prompts().await
    }

    /// Get a prompt from a specific server.
    pub async fn get_prompt(
        &self,
        server_id: &str,
        name: &str,
        args: Value,
    ) -> Result<Value, ProxyError> {
        let map = self.servers.read().await;
        let managed = map
            .get(server_id)
            .ok_or_else(|| ProxyError::Transport(format!("server not found: {server_id}")))?;
        managed.client.get_prompt(name, args).await
    }

    /// Send roots to specific servers (scoped to an environment's server list).
    pub async fn broadcast_roots(&self, server_ids: &[String], roots: Value) {
        let map = self.servers.read().await;
        for id in server_ids {
            if let Some(managed) = map.get(id) {
                if let Err(e) = managed.client.send_roots(roots.clone()).await {
                    tracing::warn!(server_id = %id, error = %e, "failed to send roots");
                }
            }
        }
    }

    /// List all managed servers with their health status.
    pub async fn list_servers(&self) -> Vec<(String, HealthStatus)> {
        let map = self.servers.read().await;
        map.iter()
            .map(|(id, ms)| (id.clone(), ms.health.clone()))
            .collect()
    }

    /// Check whether a specific server is healthy.
    pub async fn is_healthy(&self, server_id: &str) -> bool {
        let map = self.servers.read().await;
        map.get(server_id)
            .map(|ms| matches!(ms.health, HealthStatus::Healthy))
            .unwrap_or(false)
    }

    /// Get the health status of a specific server.
    pub async fn get_health(&self, server_id: &str) -> Option<HealthStatus> {
        let map = self.servers.read().await;
        map.get(server_id).map(|ms| ms.health.clone())
    }

    /// Update the health status of a specific server.
    pub async fn set_health(&self, id: &str, health: HealthStatus) {
        let mut map = self.servers.write().await;
        if let Some(ms) = map.get_mut(id) {
            ms.health = health;
        }
    }

    /// Delegate to the underlying client's `health_check()` method.
    ///
    /// Returns `false` if the server is not found or the check fails.
    pub async fn check_health(&self, server_id: &str) -> bool {
        let map = self.servers.read().await;
        match map.get(server_id) {
            Some(ms) => ms.client.health_check().await,
            None => false,
        }
    }

    /// Gracefully shut down all managed servers.
    pub async fn shutdown_all(&self) {
        let mut map = self.servers.write().await;
        let ids: Vec<String> = map.keys().cloned().collect();
        for id in ids {
            if let Some(mut managed) = map.remove(&id) {
                info!(server_id = %id, "shutting down server");
                if let Err(e) = managed.client.shutdown().await {
                    warn!(server_id = %id, error = %e, "error during shutdown");
                }
            }
        }
    }
}

impl Default for ServerManager {
    fn default() -> Self {
        Self::new()
    }
}
