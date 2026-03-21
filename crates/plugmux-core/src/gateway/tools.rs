//! Gateway tool implementations — the 5 LLM-facing operations that the
//! HTTP router (Task 8) will expose as MCP tools.

use std::sync::Arc;

use serde_json::Value;
use tokio::sync::RwLock;

use crate::config::{Permission, PlugmuxConfig, ServerOverride};
use crate::environment::resolve_named;
use crate::manager::ServerManager;
use crate::proxy::{ProxyError, ToolInfo};

/// The business logic layer for the gateway's LLM-facing tools.
pub struct GatewayTools {
    pub config: Arc<RwLock<PlugmuxConfig>>,
    pub manager: Arc<ServerManager>,
}

/// Summary information about a server, returned by `list_servers`.
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    pub healthy: bool,
    pub tool_count: usize,
}

/// The resolved permission level for an action.
#[derive(Debug, Clone, PartialEq)]
enum PermissionLevel {
    Allow,
    Approve,
    Deny,
}

impl GatewayTools {
    /// Create a new `GatewayTools` instance.
    pub fn new(config: Arc<RwLock<PlugmuxConfig>>, manager: Arc<ServerManager>) -> Self {
        Self { config, manager }
    }

    /// List servers available in an environment, filtered to healthy ones,
    /// with tool counts.
    pub async fn list_servers(&self, env_id: &str) -> Result<Vec<ServerInfo>, ProxyError> {
        let cfg = self.config.read().await;
        let resolved = resolve_named(&cfg, env_id).ok_or_else(|| {
            ProxyError::Transport(format!("environment not found: {env_id}"))
        })?;

        let mut infos = Vec::new();
        for rs in &resolved {
            let id = &rs.config.id;
            let healthy = self.manager.is_healthy(id).await;
            if !healthy {
                continue;
            }

            let tool_count = match self.manager.list_tools(id).await {
                Ok(tools) => tools.len(),
                Err(_) => 0,
            };

            infos.push(ServerInfo {
                id: id.clone(),
                name: rs.config.name.clone(),
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

    /// Enable a server in an environment (adds/updates an override with `enabled: true`).
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
        let env = cfg
            .environments
            .iter_mut()
            .find(|e| e.id == env_id)
            .ok_or_else(|| {
                ProxyError::Transport(format!("environment not found: {env_id}"))
            })?;

        // Update existing override or create a new one.
        if let Some(ov) = env
            .overrides
            .iter_mut()
            .find(|o| o.server_id == server_id)
        {
            ov.enabled = Some(true);
        } else {
            env.overrides.push(ServerOverride {
                server_id: server_id.to_string(),
                enabled: Some(true),
                url: None,
                permissions: None,
            });
        }

        Ok(())
    }

    /// Disable a server in an environment (adds/updates an override with `enabled: false`).
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
        let env = cfg
            .environments
            .iter_mut()
            .find(|e| e.id == env_id)
            .ok_or_else(|| {
                ProxyError::Transport(format!("environment not found: {env_id}"))
            })?;

        // Update existing override or create a new one.
        if let Some(ov) = env
            .overrides
            .iter_mut()
            .find(|o| o.server_id == server_id)
        {
            ov.enabled = Some(false);
        } else {
            env.overrides.push(ServerOverride {
                server_id: server_id.to_string(),
                enabled: Some(false),
                url: None,
                permissions: None,
            });
        }

        Ok(())
    }

    /// Check the permission for an action on a server within an environment.
    ///
    /// Permission resolution:
    /// - Look up the `ServerOverride` for this server in the environment.
    /// - If the override has a `permissions` field, check the action against
    ///   the `allow` and `deny` lists.
    /// - If the action is in the `deny` list → `Deny` (error: action disabled).
    /// - If the action is in the `allow` list → `Allow` (proceed).
    /// - Otherwise → `Approve` (error: requires user approval).
    async fn check_permission(
        &self,
        env_id: &str,
        server_id: &str,
        action: &str,
    ) -> Result<(), ProxyError> {
        let level = self.resolve_permission(env_id, server_id, action).await;
        match level {
            PermissionLevel::Allow => Ok(()),
            PermissionLevel::Approve => Err(ProxyError::ToolCallFailed(format!(
                "action '{action}' on server '{server_id}' requires user approval"
            ))),
            PermissionLevel::Deny => Err(ProxyError::ToolCallFailed(format!(
                "action '{action}' on server '{server_id}' is disabled"
            ))),
        }
    }

    /// Resolve what permission level applies for a given action.
    async fn resolve_permission(
        &self,
        env_id: &str,
        server_id: &str,
        action: &str,
    ) -> PermissionLevel {
        let cfg = self.config.read().await;
        let env = match cfg.environments.iter().find(|e| e.id == env_id) {
            Some(e) => e,
            None => return PermissionLevel::Approve,
        };

        let ov = match env.overrides.iter().find(|o| o.server_id == server_id) {
            Some(o) => o,
            None => return PermissionLevel::Approve,
        };

        match &ov.permissions {
            Some(perm) => resolve_permission_level(perm, action),
            None => PermissionLevel::Approve,
        }
    }
}

/// Determine the permission level for a specific action based on the allow/deny lists.
fn resolve_permission_level(perm: &Permission, action: &str) -> PermissionLevel {
    // Deny takes precedence.
    if perm.deny.as_ref().is_some_and(|deny| deny.iter().any(|a| a == action)) {
        return PermissionLevel::Deny;
    }

    if perm.allow.as_ref().is_some_and(|allow| allow.iter().any(|a| a == action)) {
        return PermissionLevel::Allow;
    }

    // Default: requires approval.
    PermissionLevel::Approve
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Permission;

    #[test]
    fn test_resolve_permission_deny() {
        let perm = Permission {
            allow: None,
            deny: Some(vec!["disable_server".to_string()]),
        };
        assert_eq!(
            resolve_permission_level(&perm, "disable_server"),
            PermissionLevel::Deny
        );
    }

    #[test]
    fn test_resolve_permission_allow() {
        let perm = Permission {
            allow: Some(vec!["enable_server".to_string()]),
            deny: None,
        };
        assert_eq!(
            resolve_permission_level(&perm, "enable_server"),
            PermissionLevel::Allow
        );
    }

    #[test]
    fn test_resolve_permission_default_approve() {
        let perm = Permission {
            allow: None,
            deny: None,
        };
        assert_eq!(
            resolve_permission_level(&perm, "enable_server"),
            PermissionLevel::Approve
        );
    }

    #[test]
    fn test_resolve_permission_deny_takes_precedence() {
        let perm = Permission {
            allow: Some(vec!["enable_server".to_string()]),
            deny: Some(vec!["enable_server".to_string()]),
        };
        // Deny should take precedence over allow.
        assert_eq!(
            resolve_permission_level(&perm, "enable_server"),
            PermissionLevel::Deny
        );
    }
}
