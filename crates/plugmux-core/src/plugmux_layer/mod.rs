//! Plugmux layer — plugmux's own MCP management interface.
//!
//! This module provides the tools and resources that plugmux exposes on
//! `/env/global`, allowing LLMs to introspect and manage the gateway:
//! list/enable/disable servers, inspect environments, and confirm
//! pending approval actions.

pub mod resources;
pub mod tools;

use std::sync::Arc;

use serde_json::{Value, json};
use tokio::sync::{Mutex, RwLock};

use crate::config::{Config, PermissionLevel};
use crate::environment;
use crate::manager::ServerManager;
use crate::pending_actions::PendingActions;
use crate::proxy::{ProxyError, ResourceInfo, ToolInfo};
use crate::server::HealthStatus;

/// The plugmux management layer — served only on `/env/global`.
pub struct PlugmuxLayer {
    pub config: Arc<RwLock<Config>>,
    pub manager: Arc<ServerManager>,
    pub pending: Mutex<PendingActions>,
}

impl PlugmuxLayer {
    /// Create a new `PlugmuxLayer`.
    pub fn new(config: Arc<RwLock<Config>>, manager: Arc<ServerManager>) -> Self {
        Self {
            config,
            manager,
            pending: Mutex::new(PendingActions::new()),
        }
    }

    /// List all plugmux management tools.
    pub fn list_tools(&self) -> Vec<ToolInfo> {
        tools::list_tools()
    }

    /// List all plugmux management resources.
    pub fn list_resources(&self) -> Vec<ResourceInfo> {
        resources::list_resources()
    }

    /// Dispatch a tool call by name.
    ///
    /// Strips the `plugmux__` prefix and routes to the appropriate handler.
    pub async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ProxyError> {
        let stripped = name.strip_prefix("plugmux__").unwrap_or(name);

        match stripped {
            "list_servers" => self.handle_list_servers().await,
            "enable_server" => {
                let env_id = require_str(&args, "env_id")?;
                let server_id = require_str(&args, "server_id")?;
                self.handle_enable_server(&env_id, &server_id).await
            }
            "disable_server" => {
                let env_id = require_str(&args, "env_id")?;
                let server_id = require_str(&args, "server_id")?;
                self.handle_disable_server(&env_id, &server_id).await
            }
            "list_environments" => self.handle_list_environments().await,
            "server_status" => {
                let server_id = require_str(&args, "server_id")?;
                self.handle_server_status(&server_id).await
            }
            "confirm_action" => {
                let action_id = require_str(&args, "action_id")?;
                self.handle_confirm_action(&action_id).await
            }
            _ => Err(ProxyError::ToolCallFailed(format!(
                "unknown plugmux tool: {name}"
            ))),
        }
    }

    /// Read a resource by URI.
    pub async fn read_resource(&self, uri: &str) -> Result<Value, ProxyError> {
        match uri {
            "plugmux://servers" => {
                let servers = self.build_servers_json().await;
                Ok(wrap_resource(uri, &servers.to_string()))
            }
            "plugmux://environments" => {
                let envs = self.build_environments_json().await;
                Ok(wrap_resource(uri, &envs.to_string()))
            }
            "plugmux://agents" => {
                // TODO: wire agent state
                Ok(wrap_resource(uri, "[]"))
            }
            "plugmux://logs/recent" => {
                // TODO: wire DB logs
                Ok(wrap_resource(uri, "[]"))
            }
            _ => Err(ProxyError::Transport(format!(
                "unknown plugmux resource: {uri}"
            ))),
        }
    }

    // -----------------------------------------------------------------------
    // Tool handlers
    // -----------------------------------------------------------------------

    async fn handle_list_servers(&self) -> Result<Value, ProxyError> {
        let servers = self.build_servers_json().await;
        Ok(wrap_content(&servers.to_string()))
    }

    async fn handle_enable_server(
        &self,
        env_id: &str,
        server_id: &str,
    ) -> Result<Value, ProxyError> {
        self.check_permission(env_id, server_id, "enable_server")
            .await?;

        let mut cfg = self.config.write().await;
        environment::add_server(&mut cfg, env_id, server_id)
            .map_err(|e| ProxyError::ToolCallFailed(e.to_string()))?;

        // Best-effort save.
        let _ = crate::config::save(&crate::config::config_path(), &cfg);

        Ok(wrap_content(&format!(
            "Server '{server_id}' enabled in environment '{env_id}'"
        )))
    }

    async fn handle_disable_server(
        &self,
        env_id: &str,
        server_id: &str,
    ) -> Result<Value, ProxyError> {
        self.check_permission(env_id, server_id, "disable_server")
            .await?;

        let mut cfg = self.config.write().await;
        environment::remove_server(&mut cfg, env_id, server_id)
            .map_err(|e| ProxyError::ToolCallFailed(e.to_string()))?;

        // Best-effort save.
        let _ = crate::config::save(&crate::config::config_path(), &cfg);

        Ok(wrap_content(&format!(
            "Server '{server_id}' disabled in environment '{env_id}'"
        )))
    }

    async fn handle_list_environments(&self) -> Result<Value, ProxyError> {
        let envs = self.build_environments_json().await;
        Ok(wrap_content(&envs.to_string()))
    }

    async fn handle_server_status(&self, server_id: &str) -> Result<Value, ProxyError> {
        let healthy = self.manager.is_healthy(server_id).await;
        let health = self.manager.get_health(server_id).await;
        let tool_count = match self.manager.list_tools(server_id).await {
            Ok(tools) => tools.len(),
            Err(_) => 0,
        };

        let health_str = health.as_ref().map_or("not_found", |h| h.as_str());

        let status = json!({
            "server_id": server_id,
            "healthy": healthy,
            "health": health_str,
            "tool_count": tool_count,
        });

        Ok(wrap_content(&status.to_string()))
    }

    async fn handle_confirm_action(&self, action_id: &str) -> Result<Value, ProxyError> {
        let mut pending = self.pending.lock().await;
        let action = pending.confirm(action_id).ok_or_else(|| {
            ProxyError::ToolCallFailed(
                "action expired or not found — please retry the original action".to_string(),
            )
        })?;
        drop(pending);

        match action.action.as_str() {
            "enable_server" => {
                let mut cfg = self.config.write().await;
                environment::add_server(&mut cfg, &action.env_id, &action.server_id)
                    .map_err(|e| ProxyError::ToolCallFailed(e.to_string()))?;
                let _ = crate::config::save(&crate::config::config_path(), &cfg);
                Ok(wrap_content(&format!(
                    "Confirmed: server '{}' enabled in environment '{}'",
                    action.server_id, action.env_id
                )))
            }
            "disable_server" => {
                let mut cfg = self.config.write().await;
                environment::remove_server(&mut cfg, &action.env_id, &action.server_id)
                    .map_err(|e| ProxyError::ToolCallFailed(e.to_string()))?;
                let _ = crate::config::save(&crate::config::config_path(), &cfg);
                Ok(wrap_content(&format!(
                    "Confirmed: server '{}' disabled in environment '{}'",
                    action.server_id, action.env_id
                )))
            }
            _ => Err(ProxyError::ToolCallFailed(format!(
                "unknown action: {}",
                action.action
            ))),
        }
    }

    // -----------------------------------------------------------------------
    // Permission checking (ported from gateway/tools.rs)
    // -----------------------------------------------------------------------

    async fn check_permission(
        &self,
        env_id: &str,
        server_id: &str,
        action: &str,
    ) -> Result<(), ProxyError> {
        let config = self.config.read().await;
        let level = match action {
            "enable_server" => &config.permissions.enable_server,
            "disable_server" => &config.permissions.disable_server,
            _ => {
                return Err(ProxyError::ToolCallFailed(format!(
                    "Unknown action: {action}"
                )));
            }
        };
        match level {
            PermissionLevel::Allow => Ok(()),
            PermissionLevel::Approve => {
                // Drop config lock before acquiring pending lock to avoid deadlocks.
                drop(config);

                let mut pending = self.pending.lock().await;
                let action_id =
                    if let Some(existing) = pending.find_existing(env_id, server_id, action) {
                        existing.to_string()
                    } else {
                        pending.add(env_id, server_id, action)
                    };
                Err(ProxyError::ApprovalRequired {
                    action_id,
                    message: format!(
                        "{action} '{server_id}' requires approval. Please confirm with the user."
                    ),
                })
            }
            PermissionLevel::Disable => Err(ProxyError::ToolCallFailed(
                "This action is not available".into(),
            )),
        }
    }

    // -----------------------------------------------------------------------
    // JSON builders
    // -----------------------------------------------------------------------

    async fn build_servers_json(&self) -> Value {
        let all = self.manager.list_servers().await;
        let mut entries = Vec::new();
        for (id, health) in &all {
            let tool_count = match self.manager.list_tools(id).await {
                Ok(tools) => tools.len(),
                Err(_) => 0,
            };
            let health_str = health.as_str();
            entries.push(json!({
                "id": id,
                "health": health_str,
                "tool_count": tool_count,
            }));
        }
        json!(entries)
    }

    async fn build_environments_json(&self) -> Value {
        let cfg = self.config.read().await;
        let envs: Vec<Value> = cfg
            .environments
            .iter()
            .map(|env| {
                json!({
                    "id": env.id,
                    "name": env.name,
                    "servers": env.servers,
                })
            })
            .collect();
        json!(envs)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Wrap tool output in the MCP content format.
fn wrap_content(text: &str) -> Value {
    json!({"content": [{"type": "text", "text": text}]})
}

/// Wrap resource output in the MCP ReadResourceResult format.
fn wrap_resource(uri: &str, text: &str) -> Value {
    json!({"contents": [{"uri": uri, "text": text}]})
}

/// Extract a required string field from tool arguments.
fn require_str(args: &Value, field: &str) -> Result<String, ProxyError> {
    args.get(field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| ProxyError::ToolCallFailed(format!("missing required argument: {field}")))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Environment, PermissionLevel, Permissions};

    fn config_with_permission(enable: PermissionLevel, disable: PermissionLevel) -> Config {
        Config {
            port: 4242,
            permissions: Permissions {
                enable_server: enable,
                disable_server: disable,
            },
            environments: vec![Environment {
                id: "global".to_string(),
                name: "Global".to_string(),
                servers: vec!["filesystem".to_string()],
            }],
        }
    }

    fn make_layer(config: Config) -> PlugmuxLayer {
        PlugmuxLayer::new(
            Arc::new(RwLock::new(config)),
            Arc::new(ServerManager::new()),
        )
    }

    // -----------------------------------------------------------------------
    // Tool / resource listing
    // -----------------------------------------------------------------------

    #[test]
    fn test_list_tools_returns_all_tools() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let tools = layer.list_tools();
        assert_eq!(tools.len(), 6);
    }

    #[test]
    fn test_list_resources_returns_all_resources() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let resources = layer.list_resources();
        assert_eq!(resources.len(), 4);
    }

    // -----------------------------------------------------------------------
    // call_tool dispatch
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_call_tool_unknown_returns_error() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let result = layer.call_tool("plugmux__unknown", json!({})).await;
        assert!(matches!(result, Err(ProxyError::ToolCallFailed(_))));
    }

    #[tokio::test]
    async fn test_call_tool_list_servers() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let result = layer.call_tool("plugmux__list_servers", json!({})).await;
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val["content"][0]["text"].is_string());
    }

    #[tokio::test]
    async fn test_call_tool_list_environments() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let result = layer
            .call_tool("plugmux__list_environments", json!({}))
            .await;
        assert!(result.is_ok());
        let val = result.unwrap();
        let text = val["content"][0]["text"].as_str().unwrap();
        // Should contain our "global" environment
        assert!(text.contains("global"));
    }

    #[tokio::test]
    async fn test_call_tool_server_status() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        // Server doesn't exist in manager, so health is "not_found"
        let result = layer
            .call_tool(
                "plugmux__server_status",
                json!({"server_id": "nonexistent"}),
            )
            .await;
        assert!(result.is_ok());
        let val = result.unwrap();
        let text = val["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("not_found"));
    }

    #[tokio::test]
    async fn test_call_tool_enable_server_missing_args() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let result = layer.call_tool("plugmux__enable_server", json!({})).await;
        assert!(matches!(result, Err(ProxyError::ToolCallFailed(_))));
    }

    // -----------------------------------------------------------------------
    // enable / disable with permissions
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_enable_server_allow_succeeds() {
        let config = Config {
            port: 4242,
            permissions: Permissions {
                enable_server: PermissionLevel::Allow,
                disable_server: PermissionLevel::Allow,
            },
            environments: vec![Environment {
                id: "global".to_string(),
                name: "Global".to_string(),
                servers: vec![],
            }],
        };
        let layer = make_layer(config);

        let result = layer
            .call_tool(
                "plugmux__enable_server",
                json!({"env_id": "global", "server_id": "new-srv"}),
            )
            .await;
        assert!(result.is_ok());

        // Verify server was added to config
        let cfg = layer.config.read().await;
        let ids = environment::get_server_ids(&cfg, "global").unwrap();
        assert!(ids.contains(&"new-srv".to_string()));
    }

    #[tokio::test]
    async fn test_disable_server_allow_succeeds() {
        let config = Config {
            port: 4242,
            permissions: Permissions {
                enable_server: PermissionLevel::Allow,
                disable_server: PermissionLevel::Allow,
            },
            environments: vec![Environment {
                id: "global".to_string(),
                name: "Global".to_string(),
                servers: vec!["filesystem".to_string(), "github".to_string()],
            }],
        };
        let layer = make_layer(config);

        let result = layer
            .call_tool(
                "plugmux__disable_server",
                json!({"env_id": "global", "server_id": "filesystem"}),
            )
            .await;
        assert!(result.is_ok());

        let cfg = layer.config.read().await;
        let ids = environment::get_server_ids(&cfg, "global").unwrap();
        assert!(!ids.contains(&"filesystem".to_string()));
        assert!(ids.contains(&"github".to_string()));
    }

    #[tokio::test]
    async fn test_enable_server_approve_requires_confirmation() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Approve,
            PermissionLevel::Approve,
        ));

        let result = layer
            .call_tool(
                "plugmux__enable_server",
                json!({"env_id": "global", "server_id": "new-srv"}),
            )
            .await;
        assert!(matches!(result, Err(ProxyError::ApprovalRequired { .. })));
    }

    #[tokio::test]
    async fn test_enable_server_disable_permission_blocked() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Disable,
            PermissionLevel::Disable,
        ));

        let result = layer
            .call_tool(
                "plugmux__enable_server",
                json!({"env_id": "global", "server_id": "new-srv"}),
            )
            .await;
        assert!(matches!(result, Err(ProxyError::ToolCallFailed(_))));
    }

    // -----------------------------------------------------------------------
    // confirm_action
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_confirm_action_expired_returns_error() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Approve,
            PermissionLevel::Approve,
        ));

        let result = layer
            .call_tool(
                "plugmux__confirm_action",
                json!({"action_id": "nonexistent"}),
            )
            .await;
        assert!(matches!(result, Err(ProxyError::ToolCallFailed(_))));
    }

    #[tokio::test]
    async fn test_confirm_action_executes_pending_enable() {
        let config = Config {
            port: 4242,
            permissions: Permissions {
                enable_server: PermissionLevel::Approve,
                disable_server: PermissionLevel::Approve,
            },
            environments: vec![Environment {
                id: "global".to_string(),
                name: "Global".to_string(),
                servers: vec![],
            }],
        };
        let layer = make_layer(config);

        // Trigger approval
        let err = layer
            .call_tool(
                "plugmux__enable_server",
                json!({"env_id": "global", "server_id": "new-srv"}),
            )
            .await
            .unwrap_err();
        let action_id = match err {
            ProxyError::ApprovalRequired { action_id, .. } => action_id,
            other => panic!("expected ApprovalRequired, got: {other}"),
        };

        // Confirm
        let result = layer
            .call_tool("plugmux__confirm_action", json!({"action_id": action_id}))
            .await;
        assert!(result.is_ok());

        // Verify server was added
        let cfg = layer.config.read().await;
        let ids = environment::get_server_ids(&cfg, "global").unwrap();
        assert!(ids.contains(&"new-srv".to_string()));
    }

    // -----------------------------------------------------------------------
    // read_resource
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_read_resource_servers() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let result = layer.read_resource("plugmux://servers").await;
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(
            val["contents"][0]["uri"]
                .as_str()
                .unwrap()
                .contains("servers")
        );
    }

    #[tokio::test]
    async fn test_read_resource_environments() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let result = layer.read_resource("plugmux://environments").await;
        assert!(result.is_ok());
        let val = result.unwrap();
        let text = val["contents"][0]["text"].as_str().unwrap();
        assert!(text.contains("global"));
    }

    #[tokio::test]
    async fn test_read_resource_agents_returns_empty() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let result = layer.read_resource("plugmux://agents").await;
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["contents"][0]["text"].as_str().unwrap(), "[]");
    }

    #[tokio::test]
    async fn test_read_resource_logs_returns_empty() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let result = layer.read_resource("plugmux://logs/recent").await;
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["contents"][0]["text"].as_str().unwrap(), "[]");
    }

    #[tokio::test]
    async fn test_read_resource_unknown_returns_error() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let result = layer.read_resource("plugmux://unknown").await;
        assert!(matches!(result, Err(ProxyError::Transport(_))));
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    #[test]
    fn test_wrap_content_format() {
        let val = wrap_content("hello");
        assert_eq!(val["content"][0]["type"], "text");
        assert_eq!(val["content"][0]["text"], "hello");
    }

    #[test]
    fn test_wrap_resource_format() {
        let val = wrap_resource("plugmux://test", "data");
        assert_eq!(val["contents"][0]["uri"], "plugmux://test");
        assert_eq!(val["contents"][0]["text"], "data");
    }

    #[test]
    fn test_require_str_success() {
        let args = json!({"name": "value"});
        assert_eq!(require_str(&args, "name").unwrap(), "value");
    }

    #[test]
    fn test_require_str_missing() {
        let args = json!({});
        assert!(require_str(&args, "name").is_err());
    }

    #[test]
    fn test_require_str_not_string() {
        let args = json!({"name": 42});
        assert!(require_str(&args, "name").is_err());
    }
}
