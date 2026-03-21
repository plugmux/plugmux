//! Gateway tool implementations — the 6 LLM-facing operations that the
//! HTTP router exposes as MCP tools.
//!
//! Rewritten for the new Config model: permissions are global (not per-override),
//! and enable/disable operate by adding/removing server IDs from environments.

use std::sync::Arc;

use serde_json::Value;
use tokio::sync::{Mutex, RwLock};

use crate::config::{Config, PermissionLevel};
use crate::environment;
use crate::manager::ServerManager;
use crate::pending_actions::PendingActions;
use crate::proxy::{ProxyError, ToolInfo};

/// The business logic layer for the gateway's LLM-facing tools.
pub struct GatewayTools {
    pub config: Arc<RwLock<Config>>,
    pub manager: Arc<ServerManager>,
    pub pending: Mutex<PendingActions>,
}

/// Summary information about a server, returned by `list_servers`.
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    pub healthy: bool,
    pub tool_count: usize,
}

impl GatewayTools {
    /// Create a new `GatewayTools` instance.
    pub fn new(config: Arc<RwLock<Config>>, manager: Arc<ServerManager>) -> Self {
        Self {
            config,
            manager,
            pending: Mutex::new(PendingActions::new()),
        }
    }

    /// List servers in an environment with health and tool counts.
    pub async fn list_servers(&self, env_id: &str) -> Result<Vec<ServerInfo>, ProxyError> {
        let cfg = self.config.read().await;
        let server_ids = environment::get_server_ids(&cfg, env_id).ok_or_else(|| {
            ProxyError::Transport(format!("environment not found: {env_id}"))
        })?;
        // Drop the config lock before doing async work on the manager.
        drop(cfg);

        let mut infos = Vec::new();
        for id in &server_ids {
            let healthy = self.manager.is_healthy(id).await;
            let tool_count = match self.manager.list_tools(id).await {
                Ok(tools) => tools.len(),
                Err(_) => 0,
            };

            infos.push(ServerInfo {
                id: id.clone(),
                name: id.clone(), // use the ID as name; manager doesn't expose config
                healthy,
                tool_count,
            });
        }

        Ok(infos)
    }

    /// Get the full tool list for a specific server.
    pub async fn get_tools(&self, server_id: &str) -> Result<Vec<ToolInfo>, ProxyError> {
        self.manager.list_tools(server_id).await
    }

    /// Execute a tool on a specific server.
    pub async fn execute(
        &self,
        server_id: &str,
        tool_name: &str,
        args: Value,
    ) -> Result<Value, ProxyError> {
        self.manager.call_tool(server_id, tool_name, args).await
    }

    /// Add a server to an environment.
    ///
    /// Checks permissions first — the action name is `"enable_server"`.
    pub async fn enable_server(
        &self,
        env_id: &str,
        server_id: &str,
    ) -> Result<(), ProxyError> {
        self.check_permission(env_id, server_id, "enable_server")
            .await?;

        let mut cfg = self.config.write().await;
        environment::add_server(&mut cfg, env_id, server_id).map_err(|e| {
            ProxyError::ToolCallFailed(e.to_string())
        })?;

        // Best-effort save — don't fail the action if the file can't be written.
        let _ = crate::config::save(&crate::config::config_path(), &cfg);

        Ok(())
    }

    /// Remove a server from an environment.
    ///
    /// Checks permissions first — the action name is `"disable_server"`.
    pub async fn disable_server(
        &self,
        env_id: &str,
        server_id: &str,
    ) -> Result<(), ProxyError> {
        self.check_permission(env_id, server_id, "disable_server")
            .await?;

        let mut cfg = self.config.write().await;
        environment::remove_server(&mut cfg, env_id, server_id).map_err(|e| {
            ProxyError::ToolCallFailed(e.to_string())
        })?;

        // Best-effort save.
        let _ = crate::config::save(&crate::config::config_path(), &cfg);

        Ok(())
    }

    /// Confirm a pending action that requires user approval.
    pub async fn confirm_action(&self, action_id: &str) -> Result<(), ProxyError> {
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
                Ok(())
            }
            "disable_server" => {
                let mut cfg = self.config.write().await;
                environment::remove_server(&mut cfg, &action.env_id, &action.server_id)
                    .map_err(|e| ProxyError::ToolCallFailed(e.to_string()))?;
                let _ = crate::config::save(&crate::config::config_path(), &cfg);
                Ok(())
            }
            _ => Err(ProxyError::ToolCallFailed(format!(
                "unknown action: {}",
                action.action
            ))),
        }
    }

    /// Check the permission for an action on a server within an environment.
    ///
    /// Reads the global `config.permissions` (not per-override).
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
                )))
            }
        };
        match level {
            PermissionLevel::Allow => Ok(()),
            PermissionLevel::Approve => {
                // Must drop config lock before acquiring pending lock to avoid
                // potential deadlocks with different lock orderings.
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
            PermissionLevel::Disable => {
                Err(ProxyError::ToolCallFailed("This action is not available".into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Environment, Permissions, PermissionLevel};

    /// Helper: build a Config with a given permission level for enable/disable.
    fn config_with_permission(
        enable: PermissionLevel,
        disable: PermissionLevel,
    ) -> Config {
        Config {
            port: 4242,
            permissions: Permissions {
                enable_server: enable,
                disable_server: disable,
            },
            environments: vec![Environment {
                id: "default".to_string(),
                name: "Default".to_string(),
                servers: vec!["filesystem".to_string()],
            }],
        }
    }

    fn make_tools(config: Config) -> GatewayTools {
        GatewayTools::new(
            Arc::new(RwLock::new(config)),
            Arc::new(ServerManager::new()),
        )
    }

    // -----------------------------------------------------------------------
    // Permission checks
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_permission_allow_permits_action() {
        let tools = make_tools(config_with_permission(
            PermissionLevel::Allow,
            PermissionLevel::Approve,
        ));

        let result = tools
            .check_permission("default", "filesystem", "enable_server")
            .await;
        assert!(result.is_ok(), "Allow permission should permit the action");
    }

    #[tokio::test]
    async fn test_permission_approve_returns_approval_required() {
        let tools = make_tools(config_with_permission(
            PermissionLevel::Approve,
            PermissionLevel::Approve,
        ));

        let result = tools
            .check_permission("default", "filesystem", "enable_server")
            .await;
        assert!(
            matches!(result, Err(ProxyError::ApprovalRequired { .. })),
            "Approve permission should return ApprovalRequired"
        );
    }

    #[tokio::test]
    async fn test_permission_disable_returns_tool_call_failed() {
        let tools = make_tools(config_with_permission(
            PermissionLevel::Disable,
            PermissionLevel::Disable,
        ));

        let result = tools
            .check_permission("default", "filesystem", "enable_server")
            .await;
        assert!(
            matches!(result, Err(ProxyError::ToolCallFailed(_))),
            "Disable permission should return ToolCallFailed"
        );
    }

    // -----------------------------------------------------------------------
    // enable_server / disable_server modify the environment's server list
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_enable_server_adds_server_id_to_environment() {
        let config = Config {
            port: 4242,
            permissions: Permissions {
                enable_server: PermissionLevel::Allow,
                disable_server: PermissionLevel::Allow,
            },
            environments: vec![Environment {
                id: "default".to_string(),
                name: "Default".to_string(),
                servers: vec![],
            }],
        };
        let tools = make_tools(config);

        tools.enable_server("default", "new-server").await.unwrap();

        let cfg = tools.config.read().await;
        let ids = environment::get_server_ids(&cfg, "default").unwrap();
        assert!(ids.contains(&"new-server".to_string()));
    }

    #[tokio::test]
    async fn test_disable_server_removes_server_id_from_environment() {
        let config = Config {
            port: 4242,
            permissions: Permissions {
                enable_server: PermissionLevel::Allow,
                disable_server: PermissionLevel::Allow,
            },
            environments: vec![Environment {
                id: "default".to_string(),
                name: "Default".to_string(),
                servers: vec!["filesystem".to_string(), "github".to_string()],
            }],
        };
        let tools = make_tools(config);

        tools
            .disable_server("default", "filesystem")
            .await
            .unwrap();

        let cfg = tools.config.read().await;
        let ids = environment::get_server_ids(&cfg, "default").unwrap();
        assert!(!ids.contains(&"filesystem".to_string()));
        assert!(ids.contains(&"github".to_string()));
    }

    // -----------------------------------------------------------------------
    // confirm_action
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_confirm_action_expired_returns_error() {
        let tools = make_tools(config_with_permission(
            PermissionLevel::Approve,
            PermissionLevel::Approve,
        ));

        let result = tools.confirm_action("nonexistent-id").await;
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
                id: "default".to_string(),
                name: "Default".to_string(),
                servers: vec![],
            }],
        };
        let tools = make_tools(config);

        // Trigger approval required
        let err = tools
            .enable_server("default", "new-srv")
            .await
            .unwrap_err();
        let action_id = match err {
            ProxyError::ApprovalRequired { action_id, .. } => action_id,
            other => panic!("expected ApprovalRequired, got: {other}"),
        };

        // Confirm the action
        tools.confirm_action(&action_id).await.unwrap();

        // Server should now be in the environment
        let cfg = tools.config.read().await;
        let ids = environment::get_server_ids(&cfg, "default").unwrap();
        assert!(ids.contains(&"new-srv".to_string()));
    }
}
