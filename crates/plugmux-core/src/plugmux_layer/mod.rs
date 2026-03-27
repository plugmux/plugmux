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
use crate::db::Db;
use crate::db::environments as db_envs;
use crate::manager::ServerManager;
use crate::pending_actions::PendingActions;
use crate::proxy::{ProxyError, ResourceInfo, ToolInfo};
use crate::slug::slugify;

/// The plugmux management layer — served only on `/env/global`.
pub struct PlugmuxLayer {
    pub config: Arc<RwLock<Config>>,
    pub manager: Arc<ServerManager>,
    pub pending: Mutex<PendingActions>,
    pub db: Option<Arc<Db>>,
}

impl PlugmuxLayer {
    /// Create a new `PlugmuxLayer`.
    pub fn new(
        config: Arc<RwLock<Config>>,
        manager: Arc<ServerManager>,
        db: Option<Arc<Db>>,
    ) -> Self {
        Self {
            config,
            manager,
            pending: Mutex::new(PendingActions::new()),
            db,
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
            "add_environment" => {
                let name = require_str(&args, "name")?;
                let servers = args
                    .get("servers")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                self.handle_add_environment(&name, &servers).await
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
                let agents = self.build_agents_json();
                Ok(wrap_resource(uri, &agents.to_string()))
            }
            "plugmux://logs/recent" => {
                let logs = self.build_logs_json();
                Ok(wrap_resource(uri, &logs.to_string()))
            }
            _ => Err(ProxyError::Transport(format!(
                "unknown plugmux resource: {uri}"
            ))),
        }
    }

    // -----------------------------------------------------------------------
    // Tool handlers
    // -----------------------------------------------------------------------

    async fn handle_enable_server(
        &self,
        env_id: &str,
        server_id: &str,
    ) -> Result<Value, ProxyError> {
        self.check_permission(env_id, server_id, "enable_server")
            .await?;

        if let Some(ref db) = self.db {
            db_envs::add_server(db, env_id, server_id).map_err(ProxyError::ToolCallFailed)?;
        } else {
            return Err(ProxyError::ToolCallFailed("database not available".into()));
        }

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

        if let Some(ref db) = self.db {
            db_envs::remove_server(db, env_id, server_id).map_err(ProxyError::ToolCallFailed)?;
        } else {
            return Err(ProxyError::ToolCallFailed("database not available".into()));
        }

        Ok(wrap_content(&format!(
            "Server '{server_id}' disabled in environment '{env_id}'"
        )))
    }

    async fn handle_add_environment(
        &self,
        name: &str,
        servers: &[String],
    ) -> Result<Value, ProxyError> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| ProxyError::ToolCallFailed("database not available".into()))?;

        let env_id = slugify(name);
        db_envs::add_environment(db, &env_id, name).map_err(ProxyError::ToolCallFailed)?;

        for server_id in servers {
            db_envs::add_server(db, &env_id, server_id).map_err(ProxyError::ToolCallFailed)?;
        }

        let msg = if servers.is_empty() {
            format!(
                "Environment '{}' (id: {}) created. You can add servers later with enable_server.",
                name, env_id
            )
        } else {
            format!(
                "Environment '{}' (id: {}) created with servers: {}",
                name,
                env_id,
                servers.join(", ")
            )
        };

        Ok(wrap_content(&msg))
    }

    async fn handle_confirm_action(&self, action_id: &str) -> Result<Value, ProxyError> {
        let mut pending = self.pending.lock().await;
        let action = pending.confirm(action_id).ok_or_else(|| {
            ProxyError::ToolCallFailed(
                "action expired or not found — please retry the original action".to_string(),
            )
        })?;
        drop(pending);

        let db = self
            .db
            .as_ref()
            .ok_or_else(|| ProxyError::ToolCallFailed("database not available".into()))?;

        match action.action.as_str() {
            "enable_server" => {
                db_envs::add_server(db, &action.env_id, &action.server_id)
                    .map_err(ProxyError::ToolCallFailed)?;
                Ok(wrap_content(&format!(
                    "Confirmed: server '{}' enabled in environment '{}'",
                    action.server_id, action.env_id
                )))
            }
            "disable_server" => {
                db_envs::remove_server(db, &action.env_id, &action.server_id)
                    .map_err(ProxyError::ToolCallFailed)?;
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
        if let Some(ref db) = self.db {
            let env_rows = db_envs::list_environments(db);
            let envs: Vec<Value> = env_rows
                .iter()
                .map(|env| {
                    let servers = db_envs::get_server_ids(db, &env.id).unwrap_or_default();
                    json!({
                        "id": env.id,
                        "name": env.name,
                        "servers": servers,
                    })
                })
                .collect();
            json!(envs)
        } else {
            json!([])
        }
    }

    fn build_agents_json(&self) -> Value {
        let registry = crate::agents::AgentRegistry::load_bundled();
        let entries: Vec<Value> = registry
            .list_agents()
            .iter()
            .map(|a| {
                json!({
                    "id": a.id,
                    "name": a.name,
                    "tier": a.tier,
                })
            })
            .collect();
        json!(entries)
    }

    fn build_logs_json(&self) -> Value {
        if let Some(ref db) = self.db {
            match crate::db::logs::read_recent_logs(db, 20) {
                Ok(entries) => {
                    let simplified: Vec<Value> = entries
                        .iter()
                        .map(|e| {
                            let mut obj = json!({
                                "method": e.method,
                                "env_id": e.env_id,
                                "duration_ms": e.duration_ms,
                                "timestamp": e.timestamp,
                            });
                            if let Some(ref err) = e.error {
                                obj["error"] = json!(err);
                            }
                            if let Some(ref info) = e.agent_info
                                && let Some(ref agent_id) = info.agent_id
                            {
                                obj["agent"] = json!(agent_id);
                            }
                            obj
                        })
                        .collect();
                    json!(simplified)
                }
                Err(_) => json!([]),
            }
        } else {
            json!([])
        }
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
    json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": text}]})
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
    use crate::config::{Config, PermissionLevel, Permissions};
    use crate::db::Db;

    fn config_with_permission(enable: PermissionLevel, disable: PermissionLevel) -> Config {
        Config {
            port: 4242,
            permissions: Permissions {
                enable_server: enable,
                disable_server: disable,
            },
            device_id: "test-device".to_string(),
            onboarding_shown: false,
            api_url: "http://test".to_string(),
        }
    }

    fn make_layer(config: Config) -> PlugmuxLayer {
        let db = Db::open_in_memory().unwrap();
        // Pre-populate global env with "filesystem" server for tests that expect it
        db_envs::add_server(&db, "global", "filesystem").unwrap();
        PlugmuxLayer::new(
            Arc::new(RwLock::new(config)),
            Arc::new(ServerManager::new()),
            Some(db),
        )
    }

    fn make_layer_with_db(config: Config, db: Arc<Db>) -> PlugmuxLayer {
        PlugmuxLayer::new(
            Arc::new(RwLock::new(config)),
            Arc::new(ServerManager::new()),
            Some(db),
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
        assert_eq!(tools.len(), 4);
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
    async fn test_call_tool_add_environment() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let result = layer
            .call_tool(
                "plugmux__add_environment",
                json!({"name": "Work", "servers": ["figma", "github"]}),
            )
            .await;
        assert!(result.is_ok());
        let val = result.unwrap();
        let text = val["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Work"));
        assert!(text.contains("figma"));

        // Verify it was added to db
        let db = layer.db.as_ref().unwrap();
        let servers = db_envs::get_server_ids(db, "work").unwrap();
        assert!(servers.contains(&"figma".to_string()));
        assert!(servers.contains(&"github".to_string()));
    }

    #[tokio::test]
    async fn test_call_tool_add_environment_no_servers() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let result = layer
            .call_tool("plugmux__add_environment", json!({"name": "Personal"}))
            .await;
        assert!(result.is_ok());
        let val = result.unwrap();
        let text = val["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Personal"));
        assert!(text.contains("add servers later"));
    }

    #[tokio::test]
    async fn test_call_tool_add_environment_missing_name() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let result = layer.call_tool("plugmux__add_environment", json!({})).await;
        assert!(matches!(result, Err(ProxyError::ToolCallFailed(_))));
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
            device_id: "test-device".to_string(),
            onboarding_shown: false,
            api_url: "http://test".to_string(),
        };
        let db = Db::open_in_memory().unwrap();
        let layer = make_layer_with_db(config, db);

        let result = layer
            .call_tool(
                "plugmux__enable_server",
                json!({"env_id": "global", "server_id": "new-srv"}),
            )
            .await;
        assert!(result.is_ok());

        // Verify server was added to db
        let db = layer.db.as_ref().unwrap();
        let ids = db_envs::get_server_ids(db, "global").unwrap();
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
            device_id: "test-device".to_string(),
            onboarding_shown: false,
            api_url: "http://test".to_string(),
        };
        let db = Db::open_in_memory().unwrap();
        db_envs::add_server(&db, "global", "filesystem").unwrap();
        db_envs::add_server(&db, "global", "github").unwrap();
        let layer = make_layer_with_db(config, db);

        let result = layer
            .call_tool(
                "plugmux__disable_server",
                json!({"env_id": "global", "server_id": "filesystem"}),
            )
            .await;
        assert!(result.is_ok());

        let db = layer.db.as_ref().unwrap();
        let ids = db_envs::get_server_ids(db, "global").unwrap();
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
            device_id: "test-device".to_string(),
            onboarding_shown: false,
            api_url: "http://test".to_string(),
        };
        let db = Db::open_in_memory().unwrap();
        let layer = make_layer_with_db(config, db);

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

        // Verify server was added to db
        let db = layer.db.as_ref().unwrap();
        let ids = db_envs::get_server_ids(db, "global").unwrap();
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
    async fn test_read_resource_agents() {
        let layer = make_layer(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Allow,
        ));
        let result = layer.read_resource("plugmux://agents").await;
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val["contents"][0]["text"].as_str().is_some());
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
