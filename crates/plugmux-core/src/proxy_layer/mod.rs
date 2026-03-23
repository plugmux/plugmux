//! Proxy layer — aggregates MCP primitives across all backend servers
//! in an environment and routes calls to the correct backend.

pub mod prompts;
pub mod relay;
pub mod resources;
pub mod tools;

use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::environment;
use crate::manager::ServerManager;
use crate::proxy::{PromptInfo, ProxyError, ResourceInfo, ToolInfo};

pub struct ProxyLayer {
    pub config: Arc<RwLock<Config>>,
    pub manager: Arc<ServerManager>,
}

impl ProxyLayer {
    pub fn new(config: Arc<RwLock<Config>>, manager: Arc<ServerManager>) -> Self {
        Self { config, manager }
    }

    async fn server_ids(&self, env_id: &str) -> Result<Vec<String>, ProxyError> {
        let cfg = self.config.read().await;
        environment::get_server_ids(&cfg, env_id)
            .ok_or_else(|| ProxyError::Transport(format!("environment not found: {env_id}")))
    }

    pub async fn list_tools(&self, env_id: &str) -> Result<Vec<ToolInfo>, ProxyError> {
        let server_ids = self.server_ids(env_id).await?;
        let mut all_tools = Vec::new();
        for sid in &server_ids {
            match self.manager.list_tools(sid).await {
                Ok(tools) => {
                    for tool in &tools {
                        all_tools.push(tools::namespace_tool(sid, tool));
                    }
                }
                Err(e) => {
                    tracing::warn!(server_id = %sid, error = %e, "failed to list tools");
                }
            }
        }
        Ok(all_tools)
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
        let server_ids = self.server_ids(env_id).await?;
        let mut all_resources = Vec::new();
        for sid in &server_ids {
            match self.manager.list_resources(sid).await {
                Ok(res_list) => {
                    for res in &res_list {
                        all_resources.push(resources::namespace_resource(sid, res));
                    }
                }
                Err(e) => {
                    tracing::warn!(server_id = %sid, error = %e, "failed to list resources");
                }
            }
        }
        Ok(all_resources)
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
        let server_ids = self.server_ids(env_id).await?;
        let mut all_prompts = Vec::new();
        for sid in &server_ids {
            match self.manager.list_prompts(sid).await {
                Ok(prompt_list) => {
                    for prompt in &prompt_list {
                        all_prompts.push(prompts::namespace_prompt(sid, prompt));
                    }
                }
                Err(e) => {
                    tracing::warn!(server_id = %sid, error = %e, "failed to list prompts");
                }
            }
        }
        Ok(all_prompts)
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
