# MCP Protocol Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor plugmux from a meta-tool gateway to a transparent MCP multiplexer that proxies all MCP primitives (tools, resources, prompts) from backend servers with namespacing, logging, and agent detection.

**Architecture:** Three-layer split — proxy layer (aggregates/routes backend server primitives), plugmux layer (own management tools/resources), gateway layer (HTTP dispatch, logging, agent detection). Each environment gets its own flat tool/resource/prompt list. Global env serves plugmux management only.

**Tech Stack:** Rust 2024, axum 0.8, rmcp 1.2, redb (embedded DB), tokio, serde_json

**Spec:** `docs/superpowers/specs/2026-03-23-mcp-protocol-refactor-design.md`

---

## File Map

### New Files
| File | Responsibility |
|------|----------------|
| `crates/plugmux-core/src/proxy_layer/mod.rs` | ProxyLayer struct — aggregates across backend servers for an environment |
| `crates/plugmux-core/src/proxy_layer/tools.rs` | Tool aggregation (namespace) and routing (strip prefix, forward) |
| `crates/plugmux-core/src/proxy_layer/resources.rs` | Resource aggregation (URI rewrite) and routing (parse, forward) |
| `crates/plugmux-core/src/proxy_layer/prompts.rs` | Prompt aggregation (namespace) and routing |
| `crates/plugmux-core/src/proxy_layer/relay.rs` | Roots forwarding (agent → all backends) |
| `crates/plugmux-core/src/plugmux_layer/mod.rs` | PlugmuxLayer struct — plugmux's own MCP interface |
| `crates/plugmux-core/src/plugmux_layer/tools.rs` | plugmux__* tool implementations |
| `crates/plugmux-core/src/plugmux_layer/resources.rs` | plugmux:// resource implementations |
| `crates/plugmux-core/src/db/mod.rs` | Embedded DB initialization (redb), shared handle |
| `crates/plugmux-core/src/db/logs.rs` | LogEntry struct, write/query operations |
| `crates/plugmux-core/src/gateway/logging.rs` | Axum middleware that logs request/response to DB |
| `crates/plugmux-core/src/gateway/agent_detect.rs` | Parse User-Agent headers, detect agent type |
| `tests/proxy_layer_tests.rs` | Integration tests for proxy layer |

### Modified Files
| File | What changes |
|------|-------------|
| `crates/plugmux-core/src/proxy/mod.rs` | Extend McpClient trait + data types (ResourceInfo, PromptInfo, ToolInfo fields) |
| `crates/plugmux-core/src/proxy/stdio.rs` | Implement new trait methods (list_resources, read_resource, list_prompts, get_prompt, send_roots) |
| `crates/plugmux-core/src/proxy/http_sse.rs` | Same new trait method implementations |
| `crates/plugmux-core/src/manager.rs` | Add resource/prompt/roots delegation methods |
| `crates/plugmux-core/src/gateway/router.rs` | Rewrite dispatch to route to proxy_layer or plugmux_layer based on env_id |
| `crates/plugmux-core/src/config.rs` | Rename default→global, update ensure_default→ensure_global, error types |
| `crates/plugmux-core/src/lib.rs` | Add new module declarations |
| `crates/plugmux-core/Cargo.toml` | Add redb dependency |
| `crates/plugmux-core/src/environment.rs` | Update test references from "default" to "global" |

### Deleted Files
| File | Replaced by |
|------|-------------|
| `crates/plugmux-core/src/gateway/tools.rs` | `crates/plugmux-core/src/plugmux_layer/` |

---

## Task 1: Rename default → global in config

**Files:**
- Modify: `crates/plugmux-core/src/config.rs`
- Modify: `crates/plugmux-core/src/environment.rs`

- [ ] **Step 1: Update config.rs — rename ensure_default to ensure_global**

In `crates/plugmux-core/src/config.rs`, rename the function and update all references to "default" → "global":

```rust
/// Ensures a "global" environment exists in `config`. Adds one if missing.
/// Migrates legacy "default" environment to "global" if found.
pub fn ensure_global(config: &mut Config) {
    // Migrate legacy "default" to "global"
    if let Some(env) = config.environments.iter_mut().find(|e| e.id == "default") {
        env.id = "global".to_string();
        env.name = "Global".to_string();
    }
    // Ensure global exists
    if !config.environments.iter().any(|e| e.id == "global") {
        config.environments.insert(
            0,
            Environment {
                id: "global".to_string(),
                name: "Global".to_string(),
                servers: Vec::new(),
            },
        );
    }
}
```

Update `load()` and `load_or_default()` to call `ensure_global` instead of `ensure_default`.

Update `default_config()` to use "global" instead of "default".

Update error type: `CannotDeleteDefault` → `CannotDeleteGlobal`.

Update `remove_environment()`: check `id == "global"` instead of `id == "default"`.

- [ ] **Step 2: Update all tests in config.rs**

Replace all test references from `"default"` to `"global"`:
- `test_ensure_default_creates_missing_default` → `test_ensure_global_creates_missing_global`
- `test_load_or_default_creates_default_env_when_file_missing` → `test_load_or_default_creates_global_env_when_file_missing`
- `test_delete_default_environment_returns_error` → `test_delete_global_environment_returns_error`
- All assertions checking for `"default"` → `"global"`

- [ ] **Step 3: Update environment.rs tests**

In `crates/plugmux-core/src/environment.rs`, update `config_with_envs()` helper and `test_get_server_ids_default_environment_works` test to use `"global"` instead of `"default"`.

- [ ] **Step 4: Run tests to verify**

Run: `cargo test -p plugmux-core`
Expected: All tests pass with global references.

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-core/src/config.rs crates/plugmux-core/src/environment.rs
git commit -m "refactor: rename default environment to global"
```

---

## Task 2: Extend McpClient trait and data types

**Files:**
- Modify: `crates/plugmux-core/src/proxy/mod.rs`

- [ ] **Step 1: Write tests for new data types**

Add tests at the bottom of `crates/plugmux-core/src/proxy/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_info_with_optional_fields() {
        let tool = ToolInfo {
            name: "test".to_string(),
            description: "desc".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: Some(serde_json::json!({"type": "string"})),
            annotations: Some(serde_json::json!({"readOnlyHint": true})),
        };
        assert_eq!(tool.name, "test");
        assert!(tool.output_schema.is_some());
        assert!(tool.annotations.is_some());
    }

    #[test]
    fn test_resource_info_construction() {
        let res = ResourceInfo {
            uri: "file:///test".to_string(),
            name: "test".to_string(),
            description: Some("a test resource".to_string()),
            mime_type: Some("text/plain".to_string()),
        };
        assert_eq!(res.uri, "file:///test");
    }

    #[test]
    fn test_prompt_info_construction() {
        let prompt = PromptInfo {
            name: "code-review".to_string(),
            description: Some("Review code".to_string()),
            arguments: vec![PromptArgument {
                name: "language".to_string(),
                description: Some("Programming language".to_string()),
                required: true,
            }],
        };
        assert_eq!(prompt.arguments.len(), 1);
        assert!(prompt.arguments[0].required);
    }
}
```

- [ ] **Step 2: Run tests — verify they fail**

Run: `cargo test -p plugmux-core proxy::tests`
Expected: FAIL — `ResourceInfo`, `PromptInfo`, `PromptArgument` not defined, `ToolInfo` missing fields.

- [ ] **Step 3: Add new data types and extend ToolInfo**

In `crates/plugmux-core/src/proxy/mod.rs`, update `ToolInfo` and add new structs:

```rust
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Option<Value>,
    pub annotations: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PromptInfo {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}
```

Extend the `McpClient` trait with new methods (default implementations that return empty/unsupported so existing code doesn't break):

```rust
#[async_trait]
pub trait McpClient: Send + Sync {
    async fn initialize(&mut self) -> Result<(), ProxyError>;
    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ProxyError>;
    async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ProxyError>;
    async fn health_check(&self) -> bool;
    async fn shutdown(&mut self) -> Result<(), ProxyError>;

    // New methods with defaults
    async fn list_resources(&self) -> Result<Vec<ResourceInfo>, ProxyError> {
        Ok(Vec::new())
    }
    async fn read_resource(&self, _uri: &str) -> Result<Value, ProxyError> {
        Err(ProxyError::Transport("resources not supported".into()))
    }
    async fn list_prompts(&self) -> Result<Vec<PromptInfo>, ProxyError> {
        Ok(Vec::new())
    }
    async fn get_prompt(&self, _name: &str, _args: Value) -> Result<Value, ProxyError> {
        Err(ProxyError::Transport("prompts not supported".into()))
    }
    async fn send_roots(&self, _roots: Value) -> Result<(), ProxyError> {
        Ok(())
    }
}
```

- [ ] **Step 4: Fix ToolInfo construction sites**

Update all places that construct `ToolInfo` to include the new optional fields.

In `crates/plugmux-core/src/proxy/stdio.rs:70-79` and `crates/plugmux-core/src/proxy/http_sse.rs:63-75`, update the `list_tools()` map closure:

```rust
.map(|t| ToolInfo {
    name: t.name.to_string(),
    description: t.description.as_deref().unwrap_or("").to_string(),
    input_schema: serde_json::to_value(&*t.input_schema)
        .unwrap_or(Value::Object(Default::default())),
    output_schema: t.output_schema.as_ref()
        .and_then(|s| serde_json::to_value(&**s).ok()),
    annotations: t.annotations.as_ref()
        .and_then(|a| serde_json::to_value(a).ok()),
})
```

Also update any ToolInfo construction in `crates/plugmux-core/src/gateway/tools.rs` tests if they construct ToolInfo directly.

- [ ] **Step 5: Run tests — verify they pass**

Run: `cargo test -p plugmux-core`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/plugmux-core/src/proxy/
git commit -m "feat: extend McpClient trait with resources, prompts, roots"
```

---

## Task 3: Implement new McpClient methods in stdio and http_sse

**Files:**
- Modify: `crates/plugmux-core/src/proxy/stdio.rs`
- Modify: `crates/plugmux-core/src/proxy/http_sse.rs`

Both files follow the same pattern. The rmcp `RunningService<RoleClient, ()>` exposes `list_all_resources()`, `read_resource()`, `list_all_prompts()`, `get_prompt()`.

- [ ] **Step 1: Implement list_resources in StdioMcpClient**

```rust
async fn list_resources(&self) -> Result<Vec<ResourceInfo>, ProxyError> {
    let guard = self.service.lock().await;
    let svc = guard.as_ref().ok_or(ProxyError::NotInitialized)?;

    let resources = svc
        .list_all_resources()
        .await
        .map_err(|e| ProxyError::Transport(format!("list_resources failed: {e}")))?;

    Ok(resources
        .into_iter()
        .map(|r| ResourceInfo {
            uri: r.uri.clone(),
            name: r.name.clone(),
            description: r.description.clone(),
            mime_type: r.mime_type.clone(),
        })
        .collect())
}
```

Note: `rmcp::model::Resource` is `Annotated<RawResource>`. The fields are accessed via deref on the `Annotated` wrapper. If the compiler complains, access via `r.inner.uri`, `r.inner.name`, etc. Check the Annotated struct during implementation.

- [ ] **Step 2: Implement read_resource in StdioMcpClient**

```rust
async fn read_resource(&self, uri: &str) -> Result<Value, ProxyError> {
    let guard = self.service.lock().await;
    let svc = guard.as_ref().ok_or(ProxyError::NotInitialized)?;

    let params = rmcp::model::ReadResourceRequestParams {
        uri: uri.to_string(),
    };

    let result = svc
        .read_resource(params)
        .await
        .map_err(|e| ProxyError::Transport(format!("read_resource failed: {e}")))?;

    serde_json::to_value(&result)
        .map_err(|e| ProxyError::Transport(format!("failed to serialize resource: {e}")))
}
```

- [ ] **Step 3: Implement list_prompts and get_prompt in StdioMcpClient**

```rust
async fn list_prompts(&self) -> Result<Vec<PromptInfo>, ProxyError> {
    let guard = self.service.lock().await;
    let svc = guard.as_ref().ok_or(ProxyError::NotInitialized)?;

    let prompts = svc
        .list_all_prompts()
        .await
        .map_err(|e| ProxyError::Transport(format!("list_prompts failed: {e}")))?;

    Ok(prompts
        .into_iter()
        .map(|p| PromptInfo {
            name: p.name.clone(),
            description: p.description.clone(),
            arguments: p.arguments.as_ref().map(|args| {
                args.iter().map(|a| PromptArgument {
                    name: a.name.clone(),
                    description: a.description.clone(),
                    required: a.required.unwrap_or(false),
                }).collect()
            }).unwrap_or_default(),
        })
        .collect())
}

async fn get_prompt(&self, name: &str, args: Value) -> Result<Value, ProxyError> {
    let guard = self.service.lock().await;
    let svc = guard.as_ref().ok_or(ProxyError::NotInitialized)?;

    let arguments = match args {
        Value::Object(map) => Some(map.into_iter().map(|(k, v)| {
            (k, v.as_str().unwrap_or("").to_string())
        }).collect()),
        _ => None,
    };

    let params = rmcp::model::GetPromptRequestParams {
        name: name.to_string(),
        arguments,
    };

    let result = svc
        .get_prompt(params)
        .await
        .map_err(|e| ProxyError::Transport(format!("get_prompt failed: {e}")))?;

    serde_json::to_value(&result)
        .map_err(|e| ProxyError::Transport(format!("failed to serialize prompt: {e}")))
}
```

- [ ] **Step 4: Implement send_roots in StdioMcpClient**

`send_roots` sends a `notifications/roots/updated` notification to the backend server. The rmcp client service doesn't have a dedicated method for this, so use the low-level notification API:

```rust
async fn send_roots(&self, roots: Value) -> Result<(), ProxyError> {
    let guard = self.service.lock().await;
    let svc = guard.as_ref().ok_or(ProxyError::NotInitialized)?;

    // Send as a raw JSON-RPC notification
    let params = serde_json::from_value(roots)
        .unwrap_or_default();
    svc.send_notification::<serde_json::Value>(
        "notifications/roots/updated",
        Some(params),
    ).await.map_err(|e| ProxyError::Transport(format!("send_roots failed: {e}")))?;

    Ok(())
}
```

Note: The exact rmcp API for sending arbitrary notifications may differ. Check `RunningService` methods during implementation — look for `send_notification`, `notify`, or similar. If no such method exists, skip and leave the default no-op with a TODO comment.

- [ ] **Step 5: Copy all new method implementations to HttpSseMcpClient**

The implementations are identical — both use `self.service.lock().await` and the same rmcp API. Copy the 6 new method implementations from `StdioMcpClient` to `HttpSseMcpClient`.

- [ ] **Step 6: Build to verify**

Run: `cargo build -p plugmux-core`
Expected: Compiles. Some rmcp type access patterns may need adjustment (Annotated wrapper, field names). Fix any compiler errors.

- [ ] **Step 6: Commit**

```bash
git add crates/plugmux-core/src/proxy/stdio.rs crates/plugmux-core/src/proxy/http_sse.rs
git commit -m "feat: implement resources, prompts, roots in stdio and http clients"
```

---

## Task 4: Extend ServerManager with resource/prompt/roots delegation

**Files:**
- Modify: `crates/plugmux-core/src/manager.rs`

- [ ] **Step 1: Add new delegation methods**

Add these methods to `ServerManager` following the pattern of existing `list_tools` and `call_tool`:

```rust
/// List all resources exposed by a specific server.
pub async fn list_resources(&self, server_id: &str) -> Result<Vec<ResourceInfo>, ProxyError> {
    let map = self.servers.read().await;
    let managed = map.get(server_id).ok_or_else(|| {
        ProxyError::Transport(format!("server not found: {server_id}"))
    })?;
    managed.client.list_resources().await
}

/// Read a resource from a specific server.
pub async fn read_resource(&self, server_id: &str, uri: &str) -> Result<Value, ProxyError> {
    let map = self.servers.read().await;
    let managed = map.get(server_id).ok_or_else(|| {
        ProxyError::Transport(format!("server not found: {server_id}"))
    })?;
    managed.client.read_resource(uri).await
}

/// List all prompts exposed by a specific server.
pub async fn list_prompts(&self, server_id: &str) -> Result<Vec<PromptInfo>, ProxyError> {
    let map = self.servers.read().await;
    let managed = map.get(server_id).ok_or_else(|| {
        ProxyError::Transport(format!("server not found: {server_id}"))
    })?;
    managed.client.list_prompts().await
}

/// Get a prompt from a specific server.
pub async fn get_prompt(&self, server_id: &str, name: &str, args: Value) -> Result<Value, ProxyError> {
    let map = self.servers.read().await;
    let managed = map.get(server_id).ok_or_else(|| {
        ProxyError::Transport(format!("server not found: {server_id}"))
    })?;
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
```

- [ ] **Step 2: Update imports**

Add `ResourceInfo` and `PromptInfo` to the import from `crate::proxy`:

```rust
use crate::proxy::{McpClient, ProxyError, ToolInfo, ResourceInfo, PromptInfo};
```

- [ ] **Step 3: Build to verify**

Run: `cargo build -p plugmux-core`
Expected: Compiles.

- [ ] **Step 4: Commit**

```bash
git add crates/plugmux-core/src/manager.rs
git commit -m "feat: add resource/prompt/roots delegation to ServerManager"
```

---

## Task 5: Build proxy layer — tool aggregation and routing

**Files:**
- Create: `crates/plugmux-core/src/proxy_layer/mod.rs`
- Create: `crates/plugmux-core/src/proxy_layer/tools.rs`
- Modify: `crates/plugmux-core/src/lib.rs`

- [ ] **Step 1: Write tests for namespace helpers**

Create `crates/plugmux-core/src/proxy_layer/tools.rs`:

```rust
//! Tool aggregation and routing for the proxy layer.

use crate::proxy::ToolInfo;

/// Separator between server_id and tool_name in namespaced tool names.
pub const NS_SEP: &str = "__";

/// Prefix a tool name with a server ID: `figma__get_screenshot`
pub fn namespace_tool(server_id: &str, tool: &ToolInfo) -> ToolInfo {
    ToolInfo {
        name: format!("{server_id}{NS_SEP}{}", tool.name),
        description: format!("[{}] {}", server_id, tool.description),
        input_schema: tool.input_schema.clone(),
        output_schema: tool.output_schema.clone(),
        annotations: tool.annotations.clone(),
    }
}

/// Parse a namespaced tool name into (server_id, original_name).
/// Returns None if the name doesn't contain the separator.
pub fn parse_namespaced_tool(name: &str) -> Option<(&str, &str)> {
    name.split_once(NS_SEP)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_tool() -> ToolInfo {
        ToolInfo {
            name: "get_screenshot".to_string(),
            description: "Capture a screenshot".to_string(),
            input_schema: json!({"type": "object"}),
            output_schema: None,
            annotations: None,
        }
    }

    #[test]
    fn test_namespace_tool() {
        let namespaced = namespace_tool("figma", &sample_tool());
        assert_eq!(namespaced.name, "figma__get_screenshot");
        assert_eq!(namespaced.description, "[figma] Capture a screenshot");
    }

    #[test]
    fn test_parse_namespaced_tool() {
        let (server_id, tool_name) = parse_namespaced_tool("figma__get_screenshot").unwrap();
        assert_eq!(server_id, "figma");
        assert_eq!(tool_name, "get_screenshot");
    }

    #[test]
    fn test_parse_namespaced_tool_no_separator() {
        assert!(parse_namespaced_tool("get_screenshot").is_none());
    }

    #[test]
    fn test_parse_namespaced_tool_multiple_separators() {
        let (server_id, tool_name) = parse_namespaced_tool("figma__get__screenshot").unwrap();
        assert_eq!(server_id, "figma");
        assert_eq!(tool_name, "get__screenshot");
    }
}
```

- [ ] **Step 2: Create proxy_layer mod.rs**

Create `crates/plugmux-core/src/proxy_layer/mod.rs`:

```rust
//! Proxy layer — aggregates MCP primitives across all backend servers
//! in an environment and routes calls to the correct backend.

pub mod tools;
pub mod resources;
pub mod prompts;
pub mod relay;

use std::sync::Arc;

use serde_json::Value;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::environment;
use crate::manager::ServerManager;
use crate::proxy::{ProxyError, ToolInfo, ResourceInfo, PromptInfo};

/// Aggregates MCP primitives from all backend servers in an environment.
pub struct ProxyLayer {
    pub config: Arc<RwLock<Config>>,
    pub manager: Arc<ServerManager>,
}

impl ProxyLayer {
    pub fn new(config: Arc<RwLock<Config>>, manager: Arc<ServerManager>) -> Self {
        Self { config, manager }
    }

    /// Get server IDs for an environment.
    async fn server_ids(&self, env_id: &str) -> Result<Vec<String>, ProxyError> {
        let cfg = self.config.read().await;
        environment::get_server_ids(&cfg, env_id).ok_or_else(|| {
            ProxyError::Transport(format!("environment not found: {env_id}"))
        })
    }

    /// List all tools from all backend servers, namespaced.
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

    /// Route a tool call by parsing the namespace prefix.
    pub async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ProxyError> {
        let (server_id, tool_name) = tools::parse_namespaced_tool(name)
            .ok_or_else(|| ProxyError::Transport(format!(
                "tool name must be namespaced as server_id__{tool_name}, got: {name}"
            )))?;

        self.manager.call_tool(server_id, tool_name, args).await
    }

    /// List all resources from all backend servers, with URI rewriting.
    pub async fn list_resources(&self, env_id: &str) -> Result<Vec<ResourceInfo>, ProxyError> {
        let server_ids = self.server_ids(env_id).await?;
        let mut all_resources = Vec::new();

        for sid in &server_ids {
            match self.manager.list_resources(sid).await {
                Ok(resources) => {
                    for res in &resources {
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

    /// Route a resource read by parsing the plugmux-res:// URI.
    pub async fn read_resource(&self, uri: &str) -> Result<Value, ProxyError> {
        let (server_id, original_uri) = resources::parse_namespaced_uri(uri)
            .ok_or_else(|| ProxyError::Transport(format!(
                "resource URI must use plugmux-res://server_id/original_uri scheme, got: {uri}"
            )))?;

        self.manager.read_resource(&server_id, &original_uri).await
    }

    /// List all prompts from all backend servers, namespaced.
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

    /// Route a prompt get by parsing the namespace prefix.
    pub async fn get_prompt(&self, name: &str, args: Value) -> Result<Value, ProxyError> {
        let (server_id, prompt_name) = prompts::parse_namespaced_prompt(name)
            .ok_or_else(|| ProxyError::Transport(format!(
                "prompt name must be namespaced as server_id__{prompt_name}, got: {name}"
            )))?;

        self.manager.get_prompt(server_id, prompt_name, args).await
    }

    /// Broadcast roots to all backend servers in the environment.
    pub async fn broadcast_roots(&self, env_id: &str, roots: Value) -> Result<(), ProxyError> {
        let server_ids = self.server_ids(env_id).await?;
        self.manager.broadcast_roots(&server_ids, roots).await;
        Ok(())
    }
}
```

- [ ] **Step 3: Register the module**

Add to `crates/plugmux-core/src/lib.rs`:

```rust
pub mod proxy_layer;
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p plugmux-core proxy_layer`
Expected: Unit tests for namespace/parse helpers pass.

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-core/src/proxy_layer/ crates/plugmux-core/src/lib.rs
git commit -m "feat: add proxy layer with tool aggregation and routing"
```

---

## Task 6: Build proxy layer — resource and prompt modules

**Files:**
- Create: `crates/plugmux-core/src/proxy_layer/resources.rs`
- Create: `crates/plugmux-core/src/proxy_layer/prompts.rs`
- Create: `crates/plugmux-core/src/proxy_layer/relay.rs`

- [ ] **Step 1: Create resources.rs with tests**

```rust
//! Resource aggregation and routing for the proxy layer.
//!
//! Uses `plugmux-res://{server_id}/{original_uri}` synthetic URI scheme.

use crate::proxy::ResourceInfo;

const SCHEME: &str = "plugmux-res://";

/// Rewrite a backend resource URI to the plugmux synthetic scheme.
pub fn namespace_resource(server_id: &str, resource: &ResourceInfo) -> ResourceInfo {
    ResourceInfo {
        uri: format!("{SCHEME}{server_id}/{}", resource.uri),
        name: format!("[{}] {}", server_id, resource.name),
        description: resource.description.clone(),
        mime_type: resource.mime_type.clone(),
    }
}

/// Parse a plugmux-res:// URI into (server_id, original_uri).
/// Returns None if the URI doesn't use the plugmux-res:// scheme.
pub fn parse_namespaced_uri(uri: &str) -> Option<(String, String)> {
    let rest = uri.strip_prefix(SCHEME)?;
    let slash_pos = rest.find('/')?;
    let server_id = &rest[..slash_pos];
    let original_uri = &rest[slash_pos + 1..];
    if server_id.is_empty() || original_uri.is_empty() {
        return None;
    }
    Some((server_id.to_string(), original_uri.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_resource() -> ResourceInfo {
        ResourceInfo {
            uri: "file:///logs/app.log".to_string(),
            name: "App Log".to_string(),
            description: Some("Application log file".to_string()),
            mime_type: Some("text/plain".to_string()),
        }
    }

    #[test]
    fn test_namespace_resource() {
        let namespaced = namespace_resource("figma", &sample_resource());
        assert_eq!(namespaced.uri, "plugmux-res://figma/file:///logs/app.log");
        assert_eq!(namespaced.name, "[figma] App Log");
    }

    #[test]
    fn test_parse_namespaced_uri() {
        let (sid, orig) = parse_namespaced_uri("plugmux-res://figma/file:///logs/app.log").unwrap();
        assert_eq!(sid, "figma");
        assert_eq!(orig, "file:///logs/app.log");
    }

    #[test]
    fn test_parse_namespaced_uri_invalid() {
        assert!(parse_namespaced_uri("file:///logs/app.log").is_none());
        assert!(parse_namespaced_uri("plugmux-res://").is_none());
        assert!(parse_namespaced_uri("plugmux-res:///file:///x").is_none());
    }

    #[test]
    fn test_roundtrip() {
        let original = sample_resource();
        let namespaced = namespace_resource("myserver", &original);
        let (sid, orig_uri) = parse_namespaced_uri(&namespaced.uri).unwrap();
        assert_eq!(sid, "myserver");
        assert_eq!(orig_uri, original.uri);
    }
}
```

- [ ] **Step 2: Create prompts.rs with tests**

```rust
//! Prompt aggregation and routing for the proxy layer.

use crate::proxy::PromptInfo;

use super::tools::NS_SEP;

/// Prefix a prompt name with a server ID.
pub fn namespace_prompt(server_id: &str, prompt: &PromptInfo) -> PromptInfo {
    PromptInfo {
        name: format!("{server_id}{NS_SEP}{}", prompt.name),
        description: prompt.description.as_ref().map(|d| format!("[{}] {}", server_id, d)),
        arguments: prompt.arguments.clone(),
    }
}

/// Parse a namespaced prompt name into (server_id, original_name).
pub fn parse_namespaced_prompt(name: &str) -> Option<(&str, &str)> {
    name.split_once(NS_SEP)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::PromptArgument;

    #[test]
    fn test_namespace_prompt() {
        let prompt = PromptInfo {
            name: "code-review".to_string(),
            description: Some("Review code".to_string()),
            arguments: vec![PromptArgument {
                name: "language".to_string(),
                description: None,
                required: true,
            }],
        };
        let ns = namespace_prompt("figma", &prompt);
        assert_eq!(ns.name, "figma__code-review");
        assert_eq!(ns.description.unwrap(), "[figma] Review code");
        assert_eq!(ns.arguments.len(), 1);
    }

    #[test]
    fn test_parse_namespaced_prompt() {
        let (sid, name) = parse_namespaced_prompt("figma__code-review").unwrap();
        assert_eq!(sid, "figma");
        assert_eq!(name, "code-review");
    }
}
```

- [ ] **Step 3: Create relay.rs (roots forwarding stub)**

```rust
//! Relay module — handles roots forwarding and future sampling/elicitation relay.
//!
//! Currently implements:
//! - Roots forwarding (agent → all backend servers)
//!
//! Future (requires SSE transport upgrade):
//! - Sampling relay (backend → agent)
//! - Elicitation relay (backend → agent → user)

// Roots forwarding is handled directly by ProxyLayer::broadcast_roots()
// via ServerManager::broadcast_roots(). This module exists as a placeholder
// for future relay logic that requires SSE transport.
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p plugmux-core proxy_layer`
Expected: All proxy_layer tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-core/src/proxy_layer/
git commit -m "feat: add resource and prompt modules to proxy layer"
```

---

## Task 7: Build plugmux layer

**Files:**
- Create: `crates/plugmux-core/src/plugmux_layer/mod.rs`
- Create: `crates/plugmux-core/src/plugmux_layer/tools.rs`
- Create: `crates/plugmux-core/src/plugmux_layer/resources.rs`
- Modify: `crates/plugmux-core/src/lib.rs`

- [ ] **Step 1: Create plugmux_layer/tools.rs**

This contains the tool definitions and implementations for plugmux's own management tools. Port logic from the existing `gateway/tools.rs` but use the `plugmux__` prefix.

```rust
//! Plugmux's own MCP tools — management interface.

use serde_json::{Value, json};

use crate::proxy::ToolInfo;

/// Return the list of plugmux management tool definitions.
pub fn list_tools() -> Vec<ToolInfo> {
    vec![
        ToolInfo {
            name: "plugmux__list_servers".to_string(),
            description: "List all MCP servers available across all environments".to_string(),
            input_schema: json!({"type": "object", "properties": {}, "required": []}),
            output_schema: None,
            annotations: None,
        },
        ToolInfo {
            name: "plugmux__enable_server".to_string(),
            description: "Add a server to an environment".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "env_id": {"type": "string", "description": "The environment ID"},
                    "server_id": {"type": "string", "description": "The server identifier"}
                },
                "required": ["env_id", "server_id"]
            }),
            output_schema: None,
            annotations: None,
        },
        ToolInfo {
            name: "plugmux__disable_server".to_string(),
            description: "Remove a server from an environment".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "env_id": {"type": "string", "description": "The environment ID"},
                    "server_id": {"type": "string", "description": "The server identifier"}
                },
                "required": ["env_id", "server_id"]
            }),
            output_schema: None,
            annotations: None,
        },
        ToolInfo {
            name: "plugmux__list_environments".to_string(),
            description: "List all environments".to_string(),
            input_schema: json!({"type": "object", "properties": {}, "required": []}),
            output_schema: None,
            annotations: None,
        },
        ToolInfo {
            name: "plugmux__server_status".to_string(),
            description: "Get detailed status of a specific server".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "server_id": {"type": "string", "description": "The server identifier"}
                },
                "required": ["server_id"]
            }),
            output_schema: None,
            annotations: None,
        },
        ToolInfo {
            name: "plugmux__confirm_action".to_string(),
            description: "Confirm a pending action that requires user approval".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "action_id": {"type": "string", "description": "The action ID"}
                },
                "required": ["action_id"]
            }),
            output_schema: None,
            annotations: None,
        },
    ]
}
```

- [ ] **Step 2: Create plugmux_layer/resources.rs**

```rust
//! Plugmux's own MCP resources — state exposure.

use serde_json::{Value, json};

use crate::proxy::ResourceInfo;

/// Return the list of plugmux resource definitions.
pub fn list_resources() -> Vec<ResourceInfo> {
    vec![
        ResourceInfo {
            uri: "plugmux://servers".to_string(),
            name: "Servers".to_string(),
            description: Some("All servers with health and connection status".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        ResourceInfo {
            uri: "plugmux://environments".to_string(),
            name: "Environments".to_string(),
            description: Some("All environments with their server lists".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        ResourceInfo {
            uri: "plugmux://agents".to_string(),
            name: "Agents".to_string(),
            description: Some("Connected and detected agents".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        ResourceInfo {
            uri: "plugmux://logs/recent".to_string(),
            name: "Recent Logs".to_string(),
            description: Some("Recent gateway activity log".to_string()),
            mime_type: Some("application/json".to_string()),
        },
    ]
}
```

- [ ] **Step 3: Create plugmux_layer/mod.rs**

```rust
//! Plugmux layer — plugmux's own MCP interface.
//!
//! Serves on `/env/global` only. Provides management tools and state resources.

pub mod tools;
pub mod resources;

use std::sync::Arc;

use serde_json::{Value, json};
use tokio::sync::RwLock;

use crate::config::Config;
use crate::environment;
use crate::manager::ServerManager;
use crate::pending_actions::PendingActions;
use crate::proxy::{ProxyError, ToolInfo, ResourceInfo};
use tokio::sync::Mutex;

pub struct PlugmuxLayer {
    pub config: Arc<RwLock<Config>>,
    pub manager: Arc<ServerManager>,
    pub pending: Mutex<PendingActions>,
}

impl PlugmuxLayer {
    pub fn new(config: Arc<RwLock<Config>>, manager: Arc<ServerManager>) -> Self {
        Self {
            config,
            manager,
            pending: Mutex::new(PendingActions::new()),
        }
    }

    pub fn list_tools(&self) -> Vec<ToolInfo> {
        tools::list_tools()
    }

    pub fn list_resources(&self) -> Vec<ResourceInfo> {
        resources::list_resources()
    }

    pub async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ProxyError> {
        // Strip plugmux__ prefix for internal routing
        let tool_name = name.strip_prefix("plugmux__").unwrap_or(name);

        match tool_name {
            "list_servers" => self.handle_list_servers().await,
            "enable_server" => {
                let env_id = args.get("env_id").and_then(|v| v.as_str())
                    .ok_or_else(|| ProxyError::Transport("missing 'env_id'".into()))?;
                let server_id = args.get("server_id").and_then(|v| v.as_str())
                    .ok_or_else(|| ProxyError::Transport("missing 'server_id'".into()))?;
                self.handle_enable_server(env_id, server_id).await
            }
            "disable_server" => {
                let env_id = args.get("env_id").and_then(|v| v.as_str())
                    .ok_or_else(|| ProxyError::Transport("missing 'env_id'".into()))?;
                let server_id = args.get("server_id").and_then(|v| v.as_str())
                    .ok_or_else(|| ProxyError::Transport("missing 'server_id'".into()))?;
                self.handle_disable_server(env_id, server_id).await
            }
            "list_environments" => self.handle_list_environments().await,
            "server_status" => {
                let server_id = args.get("server_id").and_then(|v| v.as_str())
                    .ok_or_else(|| ProxyError::Transport("missing 'server_id'".into()))?;
                self.handle_server_status(server_id).await
            }
            "confirm_action" => {
                let action_id = args.get("action_id").and_then(|v| v.as_str())
                    .ok_or_else(|| ProxyError::Transport("missing 'action_id'".into()))?;
                self.handle_confirm_action(action_id).await
            }
            _ => Err(ProxyError::Transport(format!("unknown plugmux tool: {name}"))),
        }
    }

    pub async fn read_resource(&self, uri: &str) -> Result<Value, ProxyError> {
        match uri {
            "plugmux://servers" => self.resource_servers().await,
            "plugmux://environments" => self.resource_environments().await,
            "plugmux://agents" => Ok(json!([])), // TODO: wire agent state
            "plugmux://logs/recent" => Ok(json!([])), // TODO: wire DB logs
            _ => Err(ProxyError::Transport(format!("unknown plugmux resource: {uri}"))),
        }
    }

    // --- Tool handlers (ported from gateway/tools.rs) ---

    async fn handle_list_servers(&self) -> Result<Value, ProxyError> {
        let servers = self.manager.list_servers().await;
        let result: Vec<Value> = servers
            .into_iter()
            .map(|(id, health)| json!({"id": id, "health": format!("{:?}", health)}))
            .collect();
        Ok(wrap_content(&serde_json::to_string(&result).unwrap()))
    }

    async fn handle_enable_server(&self, env_id: &str, server_id: &str) -> Result<Value, ProxyError> {
        let mut cfg = self.config.write().await;
        environment::add_server(&mut cfg, env_id, server_id)
            .map_err(|e| ProxyError::ToolCallFailed(e.to_string()))?;
        let _ = crate::config::save(&crate::config::config_path(), &cfg);
        Ok(wrap_content("server added to environment"))
    }

    async fn handle_disable_server(&self, env_id: &str, server_id: &str) -> Result<Value, ProxyError> {
        let mut cfg = self.config.write().await;
        environment::remove_server(&mut cfg, env_id, server_id)
            .map_err(|e| ProxyError::ToolCallFailed(e.to_string()))?;
        let _ = crate::config::save(&crate::config::config_path(), &cfg);
        Ok(wrap_content("server removed from environment"))
    }

    async fn handle_list_environments(&self) -> Result<Value, ProxyError> {
        let cfg = self.config.read().await;
        let envs: Vec<Value> = cfg.environments.iter().map(|e| {
            json!({"id": e.id, "name": e.name, "server_count": e.servers.len()})
        }).collect();
        Ok(wrap_content(&serde_json::to_string(&envs).unwrap()))
    }

    async fn handle_server_status(&self, server_id: &str) -> Result<Value, ProxyError> {
        let health = self.manager.get_health(server_id).await;
        let tool_count = self.manager.list_tools(server_id).await
            .map(|t| t.len()).unwrap_or(0);
        Ok(wrap_content(&serde_json::to_string(&json!({
            "id": server_id,
            "health": health.map(|h| format!("{:?}", h)),
            "tool_count": tool_count,
        })).unwrap()))
    }

    async fn handle_confirm_action(&self, action_id: &str) -> Result<Value, ProxyError> {
        let mut pending = self.pending.lock().await;
        let action = pending.confirm(action_id).ok_or_else(|| {
            ProxyError::ToolCallFailed("action expired or not found".into())
        })?;
        drop(pending);

        match action.action.as_str() {
            "enable_server" => {
                self.handle_enable_server(&action.env_id, &action.server_id).await
            }
            "disable_server" => {
                self.handle_disable_server(&action.env_id, &action.server_id).await
            }
            _ => Err(ProxyError::ToolCallFailed(format!("unknown action: {}", action.action))),
        }
    }

    // --- Resource handlers ---

    async fn resource_servers(&self) -> Result<Value, ProxyError> {
        let servers = self.manager.list_servers().await;
        let result: Vec<Value> = servers.into_iter().map(|(id, health)| {
            json!({"id": id, "health": format!("{:?}", health)})
        }).collect();
        Ok(json!({"contents": [{"uri": "plugmux://servers", "text": serde_json::to_string(&result).unwrap()}]}))
    }

    async fn resource_environments(&self) -> Result<Value, ProxyError> {
        let cfg = self.config.read().await;
        let envs: Vec<Value> = cfg.environments.iter().map(|e| {
            json!({"id": e.id, "name": e.name, "servers": e.servers})
        }).collect();
        Ok(json!({"contents": [{"uri": "plugmux://environments", "text": serde_json::to_string(&envs).unwrap()}]}))
    }
}

/// Wrap a text string into the MCP tool result content format.
fn wrap_content(text: &str) -> Value {
    json!({
        "content": [{"type": "text", "text": text}]
    })
}
```

- [ ] **Step 4: Register module in lib.rs**

Add to `crates/plugmux-core/src/lib.rs`:

```rust
pub mod plugmux_layer;
```

- [ ] **Step 5: Build to verify**

Run: `cargo build -p plugmux-core`
Expected: Compiles.

- [ ] **Step 6: Commit**

```bash
git add crates/plugmux-core/src/plugmux_layer/ crates/plugmux-core/src/lib.rs
git commit -m "feat: add plugmux layer with management tools and resources"
```

---

## Task 8: Rewrite gateway router

**Files:**
- Modify: `crates/plugmux-core/src/gateway/router.rs`
- Modify: `crates/plugmux-core/src/gateway/mod.rs` (if it exists)

- [ ] **Step 1: Rewrite router.rs**

Replace the current `handle_jsonrpc` and `dispatch` with the new env-aware routing:

```rust
//! HTTP router for the plugmux gateway.
//!
//! Routes MCP JSON-RPC requests to either the plugmux layer (global env)
//! or the proxy layer (project envs) based on the environment ID.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::{Value, json};
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::config::Config;
use crate::manager::ServerManager;
use crate::plugmux_layer::PlugmuxLayer;
use crate::proxy::ProxyError;
use crate::proxy_layer::ProxyLayer;

const GLOBAL_ENV: &str = "global";

#[derive(Clone)]
struct AppState {
    plugmux: Arc<PlugmuxLayer>,
    proxy: Arc<ProxyLayer>,
}

pub fn build_router(
    config: Arc<RwLock<Config>>,
    manager: Arc<ServerManager>,
) -> Router {
    let plugmux = Arc::new(PlugmuxLayer::new(config.clone(), manager.clone()));
    let proxy = Arc::new(ProxyLayer::new(config, manager));
    let state = AppState { plugmux, proxy };

    Router::new()
        .route("/env/{env_id}", post(handle_jsonrpc))
        .route("/health", get(handle_health))
        .with_state(state)
}

pub async fn start_server(
    config: Arc<RwLock<Config>>,
    manager: Arc<ServerManager>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let router = build_router(config, manager);
    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("plugmux gateway listening on http://{addr}");
    axum::serve(listener, router).await?;
    Ok(())
}

async fn handle_health() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

async fn handle_jsonrpc(
    State(state): State<AppState>,
    Path(env_id): Path<String>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let id = body.get("id").cloned().unwrap_or(Value::Null);
    let method = body.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = body.get("params").cloned().unwrap_or(Value::Null);

    let result = dispatch(&state, &env_id, method, &params).await;

    match result {
        Ok(value) => (
            StatusCode::OK,
            Json(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": value,
            })),
        ),
        Err(err) => {
            error!(method = %method, env = %env_id, error = %err, "JSON-RPC error");
            (
                StatusCode::OK,
                Json(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32603,
                        "message": err.to_string(),
                    },
                })),
            )
        }
    }
}

async fn dispatch(
    state: &AppState,
    env_id: &str,
    method: &str,
    params: &Value,
) -> Result<Value, ProxyError> {
    match method {
        "initialize" => Ok(handle_initialize()),
        "notifications/initialized" => Ok(Value::Null),
        "ping" => Ok(json!({})),
        "tools/list" => dispatch_tools_list(state, env_id).await,
        "tools/call" => dispatch_tools_call(state, env_id, params).await,
        "resources/list" => dispatch_resources_list(state, env_id).await,
        "resources/read" => dispatch_resources_read(state, env_id, params).await,
        "prompts/list" => dispatch_prompts_list(state, env_id).await,
        "prompts/get" => dispatch_prompts_get(state, env_id, params).await,
        "notifications/roots/updated" => {
            if env_id != GLOBAL_ENV {
                state.proxy.broadcast_roots(env_id, params.clone()).await?;
            }
            Ok(Value::Null)
        }
        _ => Err(ProxyError::Transport(format!("unknown method: {method}"))),
    }
}

fn handle_initialize() -> Value {
    json!({
        "protocolVersion": "2025-03-26",
        "capabilities": {
            "tools": { "listChanged": true },
            "resources": { "subscribe": false, "listChanged": true },
            "prompts": { "listChanged": true }
        },
        "serverInfo": {
            "name": "plugmux",
            "version": "0.2.0"
        }
    })
}

// --- Dispatch helpers ---

async fn dispatch_tools_list(state: &AppState, env_id: &str) -> Result<Value, ProxyError> {
    let tools = if env_id == GLOBAL_ENV {
        state.plugmux.list_tools()
    } else {
        state.proxy.list_tools(env_id).await?
    };

    let tool_values: Vec<Value> = tools.iter().map(|t| {
        let mut tool = json!({
            "name": t.name,
            "description": t.description,
            "inputSchema": t.input_schema,
        });
        if let Some(ref os) = t.output_schema {
            tool["outputSchema"] = os.clone();
        }
        if let Some(ref ann) = t.annotations {
            tool["annotations"] = ann.clone();
        }
        tool
    }).collect();

    Ok(json!({ "tools": tool_values }))
}

async fn dispatch_tools_call(state: &AppState, env_id: &str, params: &Value) -> Result<Value, ProxyError> {
    let tool_name = params.get("name").and_then(|n| n.as_str())
        .ok_or_else(|| ProxyError::Transport("missing 'name' in tools/call".into()))?;
    let args = params.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));

    if env_id == GLOBAL_ENV {
        state.plugmux.call_tool(tool_name, args).await
    } else {
        state.proxy.call_tool(tool_name, args).await
    }
}

async fn dispatch_resources_list(state: &AppState, env_id: &str) -> Result<Value, ProxyError> {
    let resources = if env_id == GLOBAL_ENV {
        state.plugmux.list_resources()
    } else {
        state.proxy.list_resources(env_id).await?
    };

    let res_values: Vec<Value> = resources.iter().map(|r| {
        let mut res = json!({"uri": r.uri, "name": r.name});
        if let Some(ref d) = r.description { res["description"] = json!(d); }
        if let Some(ref m) = r.mime_type { res["mimeType"] = json!(m); }
        res
    }).collect();

    Ok(json!({ "resources": res_values }))
}

async fn dispatch_resources_read(state: &AppState, env_id: &str, params: &Value) -> Result<Value, ProxyError> {
    let uri = params.get("uri").and_then(|u| u.as_str())
        .ok_or_else(|| ProxyError::Transport("missing 'uri' in resources/read".into()))?;

    if env_id == GLOBAL_ENV {
        state.plugmux.read_resource(uri).await
    } else {
        state.proxy.read_resource(uri).await
    }
}

async fn dispatch_prompts_list(state: &AppState, env_id: &str) -> Result<Value, ProxyError> {
    if env_id == GLOBAL_ENV {
        Ok(json!({ "prompts": [] })) // plugmux has no prompts
    } else {
        let prompts = state.proxy.list_prompts(env_id).await?;
        let prompt_values: Vec<Value> = prompts.iter().map(|p| {
            let mut prompt = json!({"name": p.name});
            if let Some(ref d) = p.description { prompt["description"] = json!(d); }
            if !p.arguments.is_empty() {
                prompt["arguments"] = json!(p.arguments.iter().map(|a| {
                    let mut arg = json!({"name": a.name, "required": a.required});
                    if let Some(ref d) = a.description { arg["description"] = json!(d); }
                    arg
                }).collect::<Vec<Value>>());
            }
            prompt
        }).collect();
        Ok(json!({ "prompts": prompt_values }))
    }
}

async fn dispatch_prompts_get(state: &AppState, env_id: &str, params: &Value) -> Result<Value, ProxyError> {
    let name = params.get("name").and_then(|n| n.as_str())
        .ok_or_else(|| ProxyError::Transport("missing 'name' in prompts/get".into()))?;
    let args = params.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));

    if env_id == GLOBAL_ENV {
        Err(ProxyError::Transport("no prompts available on global".into()))
    } else {
        state.proxy.get_prompt(name, args).await
    }
}
```

- [ ] **Step 2: Build to verify**

Run: `cargo build -p plugmux-core`
Expected: Compiles. May need to update gateway/mod.rs if it re-exports tools.rs.

- [ ] **Step 3: Run existing tests**

Run: `cargo test -p plugmux-core`
Expected: Gateway tools tests will fail (they reference old GatewayTools). This is expected — we'll address in task 10.

- [ ] **Step 4: Commit**

```bash
git add crates/plugmux-core/src/gateway/
git commit -m "refactor: rewrite router to dispatch to proxy and plugmux layers"
```

---

## Task 9: Add embedded database for logging

**Files:**
- Create: `crates/plugmux-core/src/db/mod.rs`
- Create: `crates/plugmux-core/src/db/logs.rs`
- Modify: `crates/plugmux-core/Cargo.toml`
- Modify: `crates/plugmux-core/src/lib.rs`

- [ ] **Step 1: Add redb dependency**

In `crates/plugmux-core/Cargo.toml`, add:

```toml
redb = "2"
```

- [ ] **Step 2: Create db/mod.rs**

```rust
//! Embedded database module (redb).
//!
//! Currently stores request/response logs.
//! Future phases: config, catalog, agents, sync metadata.

pub mod logs;

use std::path::Path;
use std::sync::Arc;

use redb::Database;

/// Shared database handle.
pub struct Db {
    pub inner: Database,
}

impl Db {
    /// Open or create the database at the given path.
    pub fn open(path: &Path) -> Result<Arc<Self>, redb::DatabaseError> {
        let db = Database::create(path)?;

        // Ensure tables exist
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(logs::LOGS_TABLE);
        }
        write_txn.commit()?;

        Ok(Arc::new(Self { inner: db }))
    }

    /// Default database path: ~/.config/plugmux/plugmux.db
    pub fn default_path() -> std::path::PathBuf {
        crate::config::config_dir().join("plugmux.db")
    }
}
```

- [ ] **Step 3: Create db/logs.rs**

```rust
//! Log entry storage.

use std::sync::Arc;

use redb::{ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Db;

/// Table definition: key = UUID string, value = JSON-serialized LogEntry.
pub const LOGS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("logs");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: String,
    pub env_id: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_info: Option<AgentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub session_id: String,
}

impl LogEntry {
    /// Summarize params for storage (truncate large payloads).
    pub fn summarize_value(value: &Value) -> Option<String> {
        let s = serde_json::to_string(value).ok()?;
        if s.len() > 2048 {
            Some(format!("{}...", &s[..2048]))
        } else {
            Some(s)
        }
    }
}

/// Write a log entry to the database.
pub fn write_log(db: &Arc<Db>, entry: &LogEntry) -> Result<(), redb::Error> {
    let json = serde_json::to_string(entry).map_err(|e| {
        redb::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    })?;

    let write_txn = db.inner.begin_write()?;
    {
        let mut table = write_txn.open_table(LOGS_TABLE)?;
        table.insert(entry.id.as_str(), json.as_str())?;
    }
    write_txn.commit()?;
    Ok(())
}

/// Read recent log entries (last N).
pub fn read_recent_logs(db: &Arc<Db>, limit: usize) -> Result<Vec<LogEntry>, redb::Error> {
    let read_txn = db.inner.begin_read()?;
    let table = read_txn.open_table(LOGS_TABLE)?;

    let mut entries: Vec<LogEntry> = Vec::new();
    for item in table.iter()? {
        let (_, value) = item?;
        if let Ok(entry) = serde_json::from_str::<LogEntry>(value.value()) {
            entries.push(entry);
        }
    }

    // Sort by timestamp descending, take last N
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    entries.truncate(limit);

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(id: &str) -> LogEntry {
        LogEntry {
            id: id.to_string(),
            timestamp: "2026-03-23T12:00:00Z".to_string(),
            env_id: "global".to_string(),
            method: "tools/list".to_string(),
            params_summary: None,
            result_summary: None,
            error: None,
            duration_ms: 5,
            agent_info: None,
        }
    }

    #[test]
    fn test_write_and_read_log() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Db::open(&dir.path().join("test.db")).unwrap();

        let entry = make_entry("test-1");
        write_log(&db, &entry).unwrap();

        let logs = read_recent_logs(&db, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].id, "test-1");
    }

    #[test]
    fn test_recent_logs_limit() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Db::open(&dir.path().join("test.db")).unwrap();

        for i in 0..5 {
            write_log(&db, &make_entry(&format!("entry-{i}"))).unwrap();
        }

        let logs = read_recent_logs(&db, 3).unwrap();
        assert_eq!(logs.len(), 3);
    }

    #[test]
    fn test_summarize_value_truncates() {
        let large = serde_json::json!({"data": "x".repeat(5000)});
        let summary = LogEntry::summarize_value(&large).unwrap();
        assert!(summary.len() <= 2051); // 2048 + "..."
    }
}
```

- [ ] **Step 4: Register module in lib.rs**

Add to `crates/plugmux-core/src/lib.rs`:

```rust
pub mod db;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p plugmux-core db`
Expected: All DB tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/plugmux-core/Cargo.toml crates/plugmux-core/src/db/ crates/plugmux-core/src/lib.rs
git commit -m "feat: add embedded database (redb) with log storage"
```

---

## Task 10: Add logging middleware and agent detection

**Files:**
- Create: `crates/plugmux-core/src/gateway/logging.rs`
- Create: `crates/plugmux-core/src/gateway/agent_detect.rs`
- Modify: `crates/plugmux-core/src/gateway/router.rs`

- [ ] **Step 1: Create agent_detect.rs**

```rust
//! Agent detection from HTTP headers.

/// Known agent patterns in User-Agent headers.
const AGENT_PATTERNS: &[(&str, &str)] = &[
    ("claude-code", "claude-code"),
    ("claude-desktop", "claude-desktop"),
    ("cursor", "cursor"),
    ("windsurf", "windsurf"),
    ("codex", "codex"),
    ("vscode", "vscode"),
    ("zed", "zed"),
    ("continue", "continue"),
];

/// Try to detect an agent ID from a User-Agent string.
pub fn detect_agent(user_agent: &str) -> Option<String> {
    let ua_lower = user_agent.to_lowercase();
    for (pattern, agent_id) in AGENT_PATTERNS {
        if ua_lower.contains(pattern) {
            return Some(agent_id.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_claude_code() {
        assert_eq!(
            detect_agent("Claude-Code/1.0"),
            Some("claude-code".to_string())
        );
    }

    #[test]
    fn test_detect_cursor() {
        assert_eq!(
            detect_agent("Mozilla/5.0 Cursor/0.48"),
            Some("cursor".to_string())
        );
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect_agent("SomeRandomAgent/1.0"), None);
    }
}
```

- [ ] **Step 2: Create logging.rs**

```rust
//! Request/response logging middleware.

use std::sync::Arc;
use std::time::Instant;

use serde_json::Value;

use crate::db::Db;
use crate::db::logs::{AgentInfo, LogEntry, write_log};

/// Log a request/response pair to the database.
pub fn log_request(
    db: &Arc<Db>,
    env_id: &str,
    method: &str,
    params: &Value,
    result: &Result<Value, String>,
    duration: std::time::Duration,
    user_agent: Option<&str>,
    agent_id: Option<&str>,
    session_id: &str,
) {
    let entry = LogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        env_id: env_id.to_string(),
        method: method.to_string(),
        params_summary: LogEntry::summarize_value(params),
        result_summary: match result {
            Ok(v) => LogEntry::summarize_value(v),
            Err(_) => None,
        },
        error: match result {
            Err(e) => Some(e.clone()),
            Ok(_) => None,
        },
        duration_ms: duration.as_millis() as u64,
        agent_info: Some(AgentInfo {
            user_agent: user_agent.map(String::from),
            agent_id: agent_id.map(String::from),
            session_id: session_id.to_string(),
        }),
    };

    if let Err(e) = write_log(db, &entry) {
        tracing::warn!(error = %e, "failed to write log entry");
    }
}
```

- [ ] **Step 3: Wire logging into router**

Update `handle_jsonrpc` in `router.rs` to accept the DB handle and log each request. Add `db: Option<Arc<Db>>` to `AppState`:

```rust
use crate::db::Db;
use crate::gateway::agent_detect;
use crate::gateway::logging;
use axum::http::HeaderMap;

#[derive(Clone)]
struct AppState {
    plugmux: Arc<PlugmuxLayer>,
    proxy: Arc<ProxyLayer>,
    db: Option<Arc<Db>>,
}
```

Update `build_router` to accept optional `Arc<Db>`:

```rust
pub fn build_router(
    config: Arc<RwLock<Config>>,
    manager: Arc<ServerManager>,
    db: Option<Arc<Db>>,
) -> Router {
    // ...
    let state = AppState { plugmux, proxy, db };
    // ...
}
```

Update `handle_jsonrpc` to extract headers, measure time, and log:

```rust
async fn handle_jsonrpc(
    State(state): State<AppState>,
    Path(env_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let id = body.get("id").cloned().unwrap_or(Value::Null);
    let method = body.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = body.get("params").cloned().unwrap_or(Value::Null);

    let user_agent = headers.get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let agent_id = user_agent.as_deref()
        .and_then(agent_detect::detect_agent);

    let result = dispatch(&state, &env_id, method, &params).await;
    let duration = start.elapsed();

    // Log to DB
    if let Some(ref db) = state.db {
        let log_result = match &result {
            Ok(v) => Ok(v.clone()),
            Err(e) => Err(e.to_string()),
        };
        logging::log_request(
            db, &env_id, method, &params, &log_result,
            duration, user_agent.as_deref(), agent_id.as_deref(),
            "default-session", // TODO: session tracking
        );
    }

    // ... rest of response handling unchanged
}
```

Update `start_server` signature to accept `Option<Arc<Db>>` and pass it through.

- [ ] **Step 4: Update call sites of build_router and start_server**

Check `crates/plugmux-cli` and `crates/plugmux-app/src-tauri` for calls to `build_router` or `start_server` and add the `db` parameter (pass `None` initially to keep things compiling, then wire in real DB).

- [ ] **Step 5: Build and run tests**

Run: `cargo build` (full workspace)
Expected: Compiles.

Run: `cargo test -p plugmux-core`
Expected: Tests pass (some old gateway tests may need updating — see Task 11).

- [ ] **Step 6: Commit**

```bash
git add crates/plugmux-core/src/gateway/ crates/plugmux-core/src/lib.rs
git commit -m "feat: add logging middleware and agent detection to gateway"
```

---

## Task 11: Clean up — remove old gateway/tools.rs and fix tests

**Files:**
- Delete: `crates/plugmux-core/src/gateway/tools.rs`
- Modify: `crates/plugmux-core/src/gateway/mod.rs`
- Update all broken test references

- [ ] **Step 1: Remove gateway/tools.rs**

Delete the file. Its functionality is now in `plugmux_layer/`.

- [ ] **Step 2: Update gateway/mod.rs**

Remove `pub mod tools;` from the module declaration. Add:

```rust
pub mod router;
pub mod logging;
pub mod agent_detect;
```

- [ ] **Step 3: Fix any remaining compilation errors**

Run: `cargo build`

Fix any references to `GatewayTools`, `gateway::tools::`, etc. These should now point to `plugmux_layer::PlugmuxLayer`.

- [ ] **Step 4: Run full test suite**

Run: `cargo test`
Expected: All tests pass. If old gateway tests reference deleted types, either port them to test PlugmuxLayer or remove them (they're tested via the new plugmux_layer tests).

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor: remove old gateway/tools.rs, wire up new layers"
```

---

## Task 12: Integration test — end-to-end JSON-RPC

**Files:**
- Create: `crates/plugmux-core/tests/integration_test.rs` (or inline in relevant modules)

- [ ] **Step 1: Write integration test for global env**

Test that `/env/global` returns plugmux management tools:

```rust
#[tokio::test]
async fn test_global_env_returns_plugmux_tools() {
    // Build router with empty config + no DB
    let config = Arc::new(RwLock::new(test_config()));
    let manager = Arc::new(ServerManager::new());
    let router = build_router(config, manager, None);

    // Send tools/list request
    let body = json!({
        "jsonrpc": "2.0",
        "id": "1",
        "method": "tools/list"
    });

    let response = router
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/env/global")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body: Value = // parse response body
    let tools = body["result"]["tools"].as_array().unwrap();
    assert!(tools.iter().any(|t| t["name"] == "plugmux__list_servers"));
    assert!(tools.iter().any(|t| t["name"] == "plugmux__enable_server"));
}
```

- [ ] **Step 2: Write integration test for ping**

```rust
#[tokio::test]
async fn test_ping_returns_empty() {
    // ... setup router ...
    // Send ping, assert result is {}
}
```

- [ ] **Step 3: Write integration test for initialize**

Verify `protocolVersion`, `capabilities`, and `serverInfo` fields.

- [ ] **Step 4: Run tests**

Run: `cargo test -p plugmux-core integration`
Expected: All pass.

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-core/tests/
git commit -m "test: add integration tests for gateway routing"
```

---

## Task 13: Wire DB into CLI and Tauri app

**Files:**
- Modify: `crates/plugmux-cli/src/commands/start.rs` (or wherever `start_server` is called)
- Modify: `crates/plugmux-app/src-tauri/src/engine.rs`

- [ ] **Step 1: Find and update CLI start command**

Locate where `gateway::router::start_server` is called in the CLI. Initialize the DB and pass it:

```rust
let db = plugmux_core::db::Db::open(&plugmux_core::db::Db::default_path())
    .map_err(|e| /* handle */)?;
start_server(config, manager, port, Some(db)).await?;
```

- [ ] **Step 2: Find and update Tauri engine**

Same pattern — initialize DB in `engine.rs` and pass to the gateway.

- [ ] **Step 3: Build full workspace**

Run: `cargo build`
Expected: Compiles.

- [ ] **Step 4: Commit**

```bash
git add crates/plugmux-cli/ crates/plugmux-app/src-tauri/
git commit -m "feat: wire embedded DB into CLI and Tauri app"
```

---

## Task 14: Manual verification with MCP Inspector

- [ ] **Step 1: Start plugmux**

Run: `cargo run --bin plugmux-cli -- start`

- [ ] **Step 2: Open MCP Inspector**

Connect to `http://localhost:4242/env/global` using Streamable HTTP transport.

- [ ] **Step 3: Verify**

- **Ping:** Should return `{}`
- **Initialize:** Should show protocolVersion `2025-03-26`, capabilities with tools/resources/prompts
- **Tools tab:** Should show `plugmux__list_servers`, `plugmux__enable_server`, etc.
- **Resources tab:** Should show `plugmux://servers`, `plugmux://environments`, etc.

- [ ] **Step 4: Test a project environment**

If you have a backend server configured in any project env, connect to `http://localhost:4242/env/{project}` and verify namespaced tools appear.

- [ ] **Step 5: Check logs**

Verify `~/.config/plugmux/plugmux.db` was created and contains log entries.

---

## Summary

| Task | What it does | Depends on |
|------|-------------|------------|
| 1 | Rename default → global | — |
| 2 | Extend McpClient trait + data types | — |
| 3 | Implement new methods in stdio/http clients | 2 |
| 4 | Extend ServerManager | 2 |
| 5 | Build proxy layer — tools | 2, 4 |
| 6 | Build proxy layer — resources, prompts | 5 |
| 7 | Build plugmux layer | 2 |
| 8 | Rewrite gateway router | 5, 6, 7 |
| 9 | Add embedded DB | — |
| 10 | Add logging + agent detection | 8, 9 |
| 11 | Clean up old code | 8, 10 |
| 12 | Integration tests | 11 |
| 13 | Wire DB into CLI/Tauri | 9, 11 |
| 14 | Manual verification | 13 |
