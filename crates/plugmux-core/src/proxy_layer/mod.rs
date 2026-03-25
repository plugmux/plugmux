//! Proxy layer — aggregates MCP primitives across all backend servers
//! in an environment and routes calls to the correct backend.

pub mod prompts;
pub mod resources;
pub mod tools;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::db::Db;
use crate::db::environments as db_envs;
use crate::manager::ServerManager;
use crate::proxy::{PromptInfo, ProxyError, ResourceInfo, ToolInfo};

pub struct ProxyLayer {
    pub config: Arc<RwLock<Config>>,
    pub manager: Arc<ServerManager>,
    pub db: Option<Arc<Db>>,
}

impl ProxyLayer {
    pub fn new(config: Arc<RwLock<Config>>, manager: Arc<ServerManager>, db: Option<Arc<Db>>) -> Self {
        Self { config, manager, db }
    }

    async fn server_ids(&self, env_id: &str) -> Result<Vec<String>, ProxyError> {
        if let Some(ref db) = self.db {
            db_envs::get_server_ids(db, env_id)
                .map_err(|e| ProxyError::Transport(format!("environment error: {e}")))
        } else {
            Err(ProxyError::Transport("database not available".into()))
        }
    }

    /// Aggregate items from all servers in an environment, skipping failures.
    async fn aggregate<T, F>(
        &self,
        env_id: &str,
        kind: &str,
        fetch: F,
    ) -> Result<Vec<T>, ProxyError>
    where
        F: for<'a> Fn(
            &'a ServerManager,
            &'a str,
        )
            -> Pin<Box<dyn Future<Output = Result<Vec<T>, ProxyError>> + Send + 'a>>,
    {
        let server_ids = self.server_ids(env_id).await?;
        let mut all = Vec::new();
        for sid in &server_ids {
            match fetch(&self.manager, sid).await {
                Ok(items) => all.extend(items),
                Err(e) => {
                    tracing::warn!(server_id = %sid, error = %e, "failed to list {kind}");
                }
            }
        }
        Ok(all)
    }

    pub async fn list_tools(&self, env_id: &str) -> Result<Vec<ToolInfo>, ProxyError> {
        self.aggregate(env_id, "tools", |mgr, sid| {
            Box::pin(async move {
                mgr.list_tools(sid).await.map(|items| {
                    items
                        .iter()
                        .map(|t| tools::namespace_tool(sid, t))
                        .collect()
                })
            })
        })
        .await
    }

    pub async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ProxyError> {
        let (server_id, tool_name) = tools::parse_namespaced_tool(name).ok_or_else(|| {
            ProxyError::Transport(format!(
                "tool name must be namespaced as server_id__tool_name, got: {name}"
            ))
        })?;
        self.manager.call_tool(server_id, tool_name, args).await
    }

    pub async fn list_resources(&self, env_id: &str) -> Result<Vec<ResourceInfo>, ProxyError> {
        self.aggregate(env_id, "resources", |mgr, sid| {
            Box::pin(async move {
                mgr.list_resources(sid).await.map(|items| {
                    items
                        .iter()
                        .map(|r| resources::namespace_resource(sid, r))
                        .collect()
                })
            })
        })
        .await
    }

    pub async fn read_resource(&self, uri: &str) -> Result<Value, ProxyError> {
        let (server_id, original_uri) = resources::parse_namespaced_uri(uri).ok_or_else(|| {
            ProxyError::Transport(format!(
                "resource URI must use plugmux-res://server_id/original_uri, got: {uri}"
            ))
        })?;
        self.manager.read_resource(&server_id, &original_uri).await
    }

    pub async fn list_prompts(&self, env_id: &str) -> Result<Vec<PromptInfo>, ProxyError> {
        self.aggregate(env_id, "prompts", |mgr, sid| {
            Box::pin(async move {
                mgr.list_prompts(sid).await.map(|items| {
                    items
                        .iter()
                        .map(|p| prompts::namespace_prompt(sid, p))
                        .collect()
                })
            })
        })
        .await
    }

    pub async fn get_prompt(&self, name: &str, args: Value) -> Result<Value, ProxyError> {
        let (server_id, prompt_name) = prompts::parse_namespaced_prompt(name).ok_or_else(|| {
            ProxyError::Transport(format!(
                "prompt name must be namespaced as server_id__prompt_name, got: {name}"
            ))
        })?;
        self.manager.get_prompt(server_id, prompt_name, args).await
    }

    pub async fn broadcast_roots(&self, env_id: &str, roots: Value) -> Result<(), ProxyError> {
        let server_ids = self.server_ids(env_id).await?;
        self.manager.broadcast_roots(&server_ids, roots).await;
        Ok(())
    }
}
