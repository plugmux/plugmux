# plugmux Phase 1 — Core Gateway CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a working CLI gateway that proxies MCP calls through per-environment endpoints, so an agent connects to one URL and accesses only the servers configured for that environment.

**Architecture:** Rust workspace with a shared `plugmux-core` library crate and a `plugmux-cli` binary crate. The core handles config, environment resolution, MCP client connections (stdio + HTTP+SSE), and the gateway MCP server. The CLI provides start/stop/status and config management commands. An axum HTTP server exposes per-environment MCP endpoints.

**Tech Stack:** Rust, `rmcp` (official MCP SDK), `axum` (HTTP server), `tokio` (async runtime), `clap` (CLI), `serde`/`serde_json` (config), `slug` (URL slugs)

**Spec:** `docs/superpowers/specs/2026-03-20-plugmux-design.md`

---

## File Structure

```
plugmux/
├── Cargo.toml                          # Workspace root
├── crates/
│   ├── plugmux-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # Re-exports all public modules
│   │       ├── config.rs               # Config structs, load/save, defaults
│   │       ├── server.rs               # Server definition types, connectivity
│   │       ├── environment.rs          # Environment resolver (merge Main + env + overrides)
│   │       ├── slug.rs                 # Slug generation from environment names
│   │       ├── proxy/
│   │       │   ├── mod.rs              # McpClient trait definition
│   │       │   ├── stdio.rs           # Stdio transport MCP client
│   │       │   └── http_sse.rs        # HTTP+SSE transport MCP client
│   │       ├── health.rs               # Health checker, connectivity-aware exclusion
│   │       ├── manager.rs              # Server lifecycle manager (start/stop/track)
│   │       └── gateway/
│   │           ├── mod.rs              # Gateway MCP server setup
│   │           ├── tools.rs           # 5 LLM-facing tool implementations
│   │           └── router.rs          # Axum HTTP server, per-environment routing
│   └── plugmux-cli/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs                 # Entry point, clap setup
│           └── commands/
│               ├── mod.rs
│               ├── start.rs           # Start gateway (foreground/daemon)
│               ├── stop.rs            # Stop gateway
│               ├── status.rs          # Show server health
│               ├── env.rs             # Environment CRUD commands
│               ├── server.rs          # Server add/remove/toggle/rename/list
│               └── config.rs         # Config path/export/import
└── tests/
    └── integration/
        ├── mock_mcp_server.rs          # Simple MCP server for testing
        └── gateway_test.rs            # End-to-end gateway tests
```

---

## Task 1: Project Scaffold

**Files:**
- Create: `plugmux/Cargo.toml`
- Create: `plugmux/crates/plugmux-core/Cargo.toml`
- Create: `plugmux/crates/plugmux-core/src/lib.rs`
- Create: `plugmux/crates/plugmux-cli/Cargo.toml`
- Create: `plugmux/crates/plugmux-cli/src/main.rs`

- [ ] **Step 1: Create workspace Cargo.toml**

```toml
[workspace]
resolver = "2"
members = ["crates/plugmux-core", "crates/plugmux-cli"]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
```

- [ ] **Step 2: Create plugmux-core crate**

`crates/plugmux-core/Cargo.toml`:
```toml
[package]
name = "plugmux-core"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
slug = "0.1"
thiserror = "2"
tracing = "0.1"
dirs = "6"
```

`crates/plugmux-core/src/lib.rs`:
```rust
pub mod config;
pub mod server;
pub mod environment;
pub mod slug;
```

- [ ] **Step 3: Create plugmux-cli crate**

`crates/plugmux-cli/Cargo.toml`:
```toml
[package]
name = "plugmux-cli"
version.workspace = true
edition.workspace = true

[[bin]]
name = "plugmux"
path = "src/main.rs"

[dependencies]
plugmux-core = { path = "../plugmux-core"
# Note: relative from crates/plugmux-cli/ to crates/plugmux-core/ }
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
tracing-subscriber = "0.3"
```

`crates/plugmux-cli/src/main.rs`:
```rust
fn main() {
    println!("plugmux v0.1.0");
}
```

- [ ] **Step 4: Verify workspace compiles**

Run: `cd plugmux && cargo build`
Expected: Compiles successfully, produces `target/debug/plugmux` binary

- [ ] **Step 5: Commit**

```bash
cd plugmux && git init
git add -A
git commit -m "feat: initialize plugmux rust workspace with core and cli crates"
```

---

## Task 2: Config Data Model + Persistence

**Files:**
- Create: `crates/plugmux-core/src/server.rs`
- Create: `crates/plugmux-core/src/config.rs`
- Create: `crates/plugmux-core/src/slug.rs`
- Test: `crates/plugmux-core/src/config.rs` (inline `#[cfg(test)]` module)

- [ ] **Step 1: Write tests for server types and slug generation**

In `crates/plugmux-core/src/slug.rs`:
```rust
pub fn slugify(name: &str) -> String {
    slug::slugify(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("My SaaS App"), "my-saas-app");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("Rust & Embedded"), "rust-embedded");
    }

    #[test]
    fn test_slugify_already_slug() {
        assert_eq!(slugify("my-project"), "my-project");
    }
}
```

- [ ] **Step 2: Run slug tests to verify they pass**

Run: `cd plugmux && cargo test -p plugmux-core slug`
Expected: 3 tests PASS

- [ ] **Step 3: Define server types**

In `crates/plugmux-core/src/server.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    Stdio,
    #[serde(rename = "http")]
    Http,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Connectivity {
    Local,
    Online,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub transport: Transport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default = "default_connectivity")]
    pub connectivity: Connectivity,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_connectivity() -> Connectivity {
    Connectivity::Local
}

fn default_true() -> bool {
    true
}
```

- [ ] **Step 4: Write config tests**

In `crates/plugmux-core/src/config.rs`:
```rust
use crate::server::{Connectivity, ServerConfig, Transport};
use crate::slug::slugify;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config: {0}")]
    Read(#[from] std::io::Error),
    #[error("failed to parse config: {0}")]
    Parse(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    Allow,
    Approve,
    Disable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerOverride {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MainConfig {
    pub servers: Vec<ServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentConfig {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub servers: Vec<ServerConfig>,
    #[serde(default)]
    pub overrides: HashMap<String, ServerOverride>,
    #[serde(default)]
    pub permissions: HashMap<String, Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlugmuxConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    pub main: MainConfig,
    #[serde(default)]
    pub environments: Vec<EnvironmentConfig>,
}

fn default_version() -> u32 {
    1
}

impl PlugmuxConfig {
    pub fn default_config() -> Self {
        Self {
            version: 1,
            main: MainConfig { servers: vec![] },
            environments: vec![],
        }
    }

    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("plugmux")
            .join("plugmux.json")
    }

    pub fn load(path: &PathBuf) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn load_or_default(path: &PathBuf) -> Self {
        Self::load(path).unwrap_or_else(|_| Self::default_config())
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn add_environment(&mut self, name: &str) -> &EnvironmentConfig {
        let id = slugify(name);
        let endpoint = format!("http://localhost:4242/env/{}", id);
        let env = EnvironmentConfig {
            id: id.clone(),
            name: name.to_string(),
            endpoint: Some(endpoint),
            servers: vec![],
            overrides: HashMap::new(),
            permissions: HashMap::new(),
        };
        self.environments.push(env);
        self.environments.last().unwrap()
    }

    pub fn find_environment(&self, id: &str) -> Option<&EnvironmentConfig> {
        self.environments.iter().find(|e| e.id == id)
    }

    pub fn remove_environment(&mut self, id: &str) -> bool {
        let len = self.environments.len();
        self.environments.retain(|e| e.id != id);
        self.environments.len() < len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = PlugmuxConfig::default_config();
        assert_eq!(config.version, 1);
        assert!(config.main.servers.is_empty());
        assert!(config.environments.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("plugmux.json");

        let mut config = PlugmuxConfig::default_config();
        config.main.servers.push(ServerConfig {
            id: "test".into(),
            name: "Test Server".into(),
            transport: Transport::Stdio,
            command: Some("echo".into()),
            args: Some(vec!["hello".into()]),
            url: None,
            connectivity: Connectivity::Local,
            enabled: true,
            description: Some("A test server".into()),
        });

        config.save(&path).unwrap();
        let loaded = PlugmuxConfig::load(&path).unwrap();
        assert_eq!(config, loaded);
    }

    #[test]
    fn test_add_environment() {
        let mut config = PlugmuxConfig::default_config();
        let env = config.add_environment("My SaaS App");
        assert_eq!(env.id, "my-saas-app");
        assert_eq!(env.name, "My SaaS App");
        assert_eq!(
            env.endpoint.as_deref(),
            Some("http://localhost:4242/env/my-saas-app")
        );
    }

    #[test]
    fn test_find_environment() {
        let mut config = PlugmuxConfig::default_config();
        config.add_environment("My Project");
        assert!(config.find_environment("my-project").is_some());
        assert!(config.find_environment("nonexistent").is_none());
    }

    #[test]
    fn test_remove_environment() {
        let mut config = PlugmuxConfig::default_config();
        config.add_environment("My Project");
        assert!(config.remove_environment("my-project"));
        assert!(!config.remove_environment("my-project"));
    }

    #[test]
    fn test_roundtrip_full_config() {
        let json = r#"{
            "version": 1,
            "main": {
                "servers": [
                    {
                        "id": "figma",
                        "name": "Design",
                        "transport": "stdio",
                        "command": "npx",
                        "args": ["-y", "@anthropic/figma-mcp"],
                        "connectivity": "online",
                        "enabled": true
                    }
                ]
            },
            "environments": [
                {
                    "id": "my-saas-app",
                    "name": "My SaaS App",
                    "endpoint": "http://localhost:4242/env/my-saas-app",
                    "servers": [],
                    "overrides": {
                        "figma": { "enabled": false }
                    },
                    "permissions": {
                        "enable_server": "approve"
                    }
                }
            ]
        }"#;

        let config: PlugmuxConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.main.servers.len(), 1);
        assert_eq!(config.environments[0].overrides["figma"].enabled, false);
        assert_eq!(
            config.environments[0].permissions["enable_server"],
            Permission::Approve
        );
    }
}
```

- [ ] **Step 5: Add tempfile dev dependency and run tests**

Add to `crates/plugmux-core/Cargo.toml`:
```toml
[dev-dependencies]
tempfile = "3"
```

Run: `cd plugmux && cargo test -p plugmux-core config`
Expected: 6 tests PASS

- [ ] **Step 6: Update lib.rs exports**

```rust
pub mod config;
pub mod server;
pub mod slug;
```

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: add config data model with load/save and environment management"
```

---

## Task 3: Environment Resolver

**Files:**
- Create: `crates/plugmux-core/src/environment.rs`
- Test: inline `#[cfg(test)]` module

The environment resolver merges Main servers + environment-specific servers + overrides into a single flat list of active servers.

- [ ] **Step 1: Write failing tests**

In `crates/plugmux-core/src/environment.rs`:
```rust
use crate::config::{EnvironmentConfig, MainConfig, PlugmuxConfig, ServerOverride};
use crate::server::ServerConfig;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ResolvedServer {
    pub config: ServerConfig,
    pub source: ServerSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerSource {
    Main,
    Environment,
}

pub fn resolve_environment(
    main: &MainConfig,
    env: Option<&EnvironmentConfig>,
) -> Vec<ResolvedServer> {
    let mut servers: Vec<ResolvedServer> = vec![];

    // Add Main servers, applying overrides if environment is specified
    for server in &main.servers {
        let mut s = server.clone();

        if let Some(env) = env {
            if let Some(ov) = env.overrides.get(&server.id) {
                s.enabled = ov.enabled;
            }
        }

        if s.enabled {
            servers.push(ResolvedServer {
                config: s,
                source: ServerSource::Main,
            });
        }
    }

    // Add environment-specific servers
    if let Some(env) = env {
        for server in &env.servers {
            if server.enabled {
                servers.push(ResolvedServer {
                    config: server.clone(),
                    source: ServerSource::Environment,
                });
            }
        }
    }

    servers
}

/// Resolve "main" environment — just the Main servers
pub fn resolve_main(config: &PlugmuxConfig) -> Vec<ResolvedServer> {
    resolve_environment(&config.main, None)
}

/// Resolve a named environment
pub fn resolve_named(config: &PlugmuxConfig, env_id: &str) -> Option<Vec<ResolvedServer>> {
    let env = config.find_environment(env_id)?;
    Some(resolve_environment(&config.main, Some(env)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{Connectivity, Transport};

    fn make_server(id: &str, enabled: bool) -> ServerConfig {
        ServerConfig {
            id: id.into(),
            name: id.into(),
            transport: Transport::Stdio,
            command: Some("echo".into()),
            args: None,
            url: None,
            connectivity: Connectivity::Local,
            enabled,
            description: None,
        }
    }

    #[test]
    fn test_resolve_main_only() {
        let main = MainConfig {
            servers: vec![make_server("figma", true), make_server("github", true)],
        };
        let resolved = resolve_environment(&main, None);
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].config.id, "figma");
        assert_eq!(resolved[0].source, ServerSource::Main);
    }

    #[test]
    fn test_resolve_excludes_disabled_main() {
        let main = MainConfig {
            servers: vec![make_server("figma", true), make_server("github", false)],
        };
        let resolved = resolve_environment(&main, None);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].config.id, "figma");
    }

    #[test]
    fn test_resolve_env_inherits_main() {
        let main = MainConfig {
            servers: vec![make_server("figma", true)],
        };
        let env = EnvironmentConfig {
            id: "my-app".into(),
            name: "My App".into(),
            endpoint: None,
            servers: vec![make_server("shadcn", true)],
            overrides: HashMap::new(),
            permissions: HashMap::new(),
        };
        let resolved = resolve_environment(&main, Some(&env));
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].config.id, "figma");
        assert_eq!(resolved[0].source, ServerSource::Main);
        assert_eq!(resolved[1].config.id, "shadcn");
        assert_eq!(resolved[1].source, ServerSource::Environment);
    }

    #[test]
    fn test_resolve_env_override_disables_main_server() {
        let main = MainConfig {
            servers: vec![make_server("figma", true), make_server("github", true)],
        };
        let mut overrides = HashMap::new();
        overrides.insert("figma".into(), ServerOverride { enabled: false });

        let env = EnvironmentConfig {
            id: "my-app".into(),
            name: "My App".into(),
            endpoint: None,
            servers: vec![],
            overrides,
            permissions: HashMap::new(),
        };
        let resolved = resolve_environment(&main, Some(&env));
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].config.id, "github");
    }

    #[test]
    fn test_resolve_named_not_found() {
        let config = PlugmuxConfig::default_config();
        assert!(resolve_named(&config, "nonexistent").is_none());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd plugmux && cargo test -p plugmux-core environment`
Expected: 5 tests PASS

- [ ] **Step 3: Update lib.rs**

Add `pub mod environment;` to `lib.rs`.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add environment resolver with Main inheritance and overrides"
```

---

## Task 4: MCP Client Trait + Stdio Implementation

**Files:**
- Create: `crates/plugmux-core/src/proxy/mod.rs`
- Create: `crates/plugmux-core/src/proxy/stdio.rs`
- Modify: `crates/plugmux-core/src/lib.rs`
- Modify: `crates/plugmux-core/Cargo.toml`

This task defines the MCP client abstraction and implements the stdio transport. We need to:
- Spawn a child process
- Send JSON-RPC messages via stdin
- Read responses from stdout
- Support `initialize`, `tools/list`, and `tools/call`

- [ ] **Step 1: Add rmcp dependency**

Add to `crates/plugmux-core/Cargo.toml`:
```toml
rmcp = { version = "1", features = ["client", "transport-child-process"] }
```

Note: Check the exact feature flags available in `rmcp` at build time. The crate may use different feature names. Consult `rmcp` docs if build fails: https://docs.rs/rmcp/latest

- [ ] **Step 2: Define the MCP client trait**

In `crates/plugmux-core/src/proxy/mod.rs`:
```rust
pub mod stdio;

use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("failed to start server process: {0}")]
    SpawnFailed(String),
    #[error("server not initialized")]
    NotInitialized,
    #[error("tool call failed: {0}")]
    ToolCallFailed(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("timeout")]
    Timeout,
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Abstraction over an MCP client connection to an upstream server
#[async_trait]
pub trait McpClient: Send + Sync {
    /// Initialize the connection and perform MCP handshake
    async fn initialize(&mut self) -> Result<(), ProxyError>;

    /// List all tools available on this server
    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ProxyError>;

    /// Call a tool on this server
    async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ProxyError>;

    /// Check if the server is healthy/responsive
    async fn health_check(&self) -> bool;

    /// Shut down the connection
    async fn shutdown(&mut self) -> Result<(), ProxyError>;
}
```

Add to `crates/plugmux-core/Cargo.toml`:
```toml
async-trait = "0.1"
```

- [ ] **Step 3: Implement stdio MCP client**

In `crates/plugmux-core/src/proxy/stdio.rs`:
```rust
use super::{McpClient, ProxyError, ToolInfo};
use async_trait::async_trait;
use serde_json::Value;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

/// MCP client that communicates with an upstream server via stdio (stdin/stdout).
///
/// Implementation notes:
/// - This wraps `rmcp` client transport if compatible, otherwise uses raw JSON-RPC.
/// - The child process is spawned on `initialize()` and killed on `shutdown()`.
/// - All communication uses newline-delimited JSON-RPC 2.0 over stdin/stdout.
///
/// If `rmcp`'s built-in child process transport works for our use case, prefer
/// using it directly rather than this manual implementation. Check rmcp docs at
/// https://docs.rs/rmcp/latest for `TokioChildProcess` or similar.
///
/// TODO: Replace this with rmcp's built-in transport if it handles our proxy
/// pattern (we need to hold the connection open and make multiple calls).

pub struct StdioMcpClient {
    command: String,
    args: Vec<String>,
    child: Option<Mutex<Child>>,
    initialized: bool,
}

impl StdioMcpClient {
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self {
            command,
            args,
            child: None,
            initialized: false,
        }
    }
}

#[async_trait]
impl McpClient for StdioMcpClient {
    async fn initialize(&mut self) -> Result<(), ProxyError> {
        // Spawn child process with piped stdin/stdout
        let child = Command::new(&self.command)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| ProxyError::SpawnFailed(e.to_string()))?;

        self.child = Some(Mutex::new(child));

        // Send MCP initialize request
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "plugmux",
                    "version": "0.1.0"
                }
            }
        });

        let response = self.send_request(&init_request).await?;

        // Send initialized notification
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        self.send_notification(&notification).await?;

        self.initialized = true;
        Ok(())
    }

    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ProxyError> {
        if !self.initialized {
            return Err(ProxyError::NotInitialized);
        }

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        });

        let response = self.send_request(&request).await?;
        let tools = response["result"]["tools"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|t| ToolInfo {
                name: t["name"].as_str().unwrap_or("").to_string(),
                description: t["description"].as_str().unwrap_or("").to_string(),
                input_schema: t["inputSchema"].clone(),
            })
            .collect();

        Ok(tools)
    }

    async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ProxyError> {
        if !self.initialized {
            return Err(ProxyError::NotInitialized);
        }

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": args
            }
        });

        let response = self.send_request(&request).await?;

        if let Some(error) = response.get("error") {
            return Err(ProxyError::ToolCallFailed(error.to_string()));
        }

        Ok(response["result"].clone())
    }

    async fn health_check(&self) -> bool {
        if let Some(child) = &self.child {
            let mut guard = child.lock().await;
            // Check if process is still running
            match guard.try_wait() {
                Ok(None) => true,  // Still running
                _ => false,        // Exited or error
            }
        } else {
            false
        }
    }

    async fn shutdown(&mut self) -> Result<(), ProxyError> {
        if let Some(child) = self.child.take() {
            let mut guard = child.lock().await;
            let _ = guard.kill().await;
        }
        self.initialized = false;
        Ok(())
    }
}

impl StdioMcpClient {
    async fn send_request(&self, request: &Value) -> Result<Value, ProxyError> {
        let child_mutex = self.child.as_ref().ok_or(ProxyError::NotInitialized)?;
        let mut child = child_mutex.lock().await;

        let stdin = child.stdin.as_mut().ok_or(ProxyError::Transport("no stdin".into()))?;
        let stdout = child.stdout.as_mut().ok_or(ProxyError::Transport("no stdout".into()))?;

        let mut msg = serde_json::to_string(request)
            .map_err(|e| ProxyError::Transport(e.to_string()))?;
        msg.push('\n');

        stdin
            .write_all(msg.as_bytes())
            .await
            .map_err(|e| ProxyError::Transport(e.to_string()))?;
        stdin
            .flush()
            .await
            .map_err(|e| ProxyError::Transport(e.to_string()))?;

        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .map_err(|e| ProxyError::Transport(e.to_string()))?;

        serde_json::from_str(&line).map_err(|e| ProxyError::Transport(e.to_string()))
    }

    async fn send_notification(&self, notification: &Value) -> Result<(), ProxyError> {
        let child_mutex = self.child.as_ref().ok_or(ProxyError::NotInitialized)?;
        let mut child = child_mutex.lock().await;

        let stdin = child.stdin.as_mut().ok_or(ProxyError::Transport("no stdin".into()))?;

        let mut msg = serde_json::to_string(notification)
            .map_err(|e| ProxyError::Transport(e.to_string()))?;
        msg.push('\n');

        stdin
            .write_all(msg.as_bytes())
            .await
            .map_err(|e| ProxyError::Transport(e.to_string()))?;
        stdin
            .flush()
            .await
            .map_err(|e| ProxyError::Transport(e.to_string()))?;

        Ok(())
    }
}
```

**Important:** The above is a raw JSON-RPC implementation. Before implementing, check if `rmcp`'s `ClientHandler` with `TokioChildProcess` transport can replace this entirely. If so, use rmcp directly — it handles protocol details, message IDs, and error handling. The trait `McpClient` stays as our abstraction regardless.

- [ ] **Step 4: Update lib.rs**

Add `pub mod proxy;` to `lib.rs`.

- [ ] **Step 5: Verify it compiles**

Run: `cd plugmux && cargo build -p plugmux-core`
Expected: Compiles. No runtime test yet — stdio clients need a real MCP server process to test against (covered in Task 10).

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add MCP client trait and stdio transport implementation"
```

---

## Task 5: MCP Client — HTTP+SSE Transport

**Files:**
- Create: `crates/plugmux-core/src/proxy/http_sse.rs`
- Modify: `crates/plugmux-core/src/proxy/mod.rs`
- Modify: `crates/plugmux-core/Cargo.toml`

- [ ] **Step 1: Add HTTP client dependency**

Add to `crates/plugmux-core/Cargo.toml`:
```toml
reqwest = { version = "0.12", features = ["json"] }
```

Also check if `rmcp` has a built-in HTTP+SSE client transport feature (e.g., `transport-sse`). If so, prefer that.

- [ ] **Step 2: Implement HTTP+SSE client**

In `crates/plugmux-core/src/proxy/http_sse.rs`:
```rust
use super::{McpClient, ProxyError, ToolInfo};
use async_trait::async_trait;
use serde_json::Value;

/// MCP client that communicates with an upstream server via HTTP+SSE.
///
/// The MCP HTTP+SSE transport works as follows:
/// - Client sends POST requests with JSON-RPC messages
/// - Server responds with SSE events for streaming, or direct JSON for request/response
///
/// As with StdioMcpClient, check if rmcp provides a built-in HTTP+SSE client
/// transport. If so, wrap it rather than implementing from scratch.
///
/// Key rmcp features to check: "transport-sse-client" or similar.

pub struct HttpSseMcpClient {
    url: String,
    client: reqwest::Client,
    initialized: bool,
}

impl HttpSseMcpClient {
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
            initialized: false,
        }
    }
}

#[async_trait]
impl McpClient for HttpSseMcpClient {
    async fn initialize(&mut self) -> Result<(), ProxyError> {
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "plugmux",
                    "version": "0.1.0"
                }
            }
        });

        let response = self
            .client
            .post(&self.url)
            .json(&init_request)
            .send()
            .await
            .map_err(|e| ProxyError::Transport(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProxyError::Transport(format!(
                "HTTP {}",
                response.status()
            )));
        }

        // Send initialized notification (fire-and-forget)
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        let _ = self.client.post(&self.url).json(&notification).send().await;

        self.initialized = true;
        Ok(())
    }

    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ProxyError> {
        if !self.initialized {
            return Err(ProxyError::NotInitialized);
        }

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        });

        let response: Value = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ProxyError::Transport(e.to_string()))?
            .json()
            .await
            .map_err(|e| ProxyError::Transport(e.to_string()))?;

        let tools = response["result"]["tools"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|t| ToolInfo {
                name: t["name"].as_str().unwrap_or("").to_string(),
                description: t["description"].as_str().unwrap_or("").to_string(),
                input_schema: t["inputSchema"].clone(),
            })
            .collect();

        Ok(tools)
    }

    async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ProxyError> {
        if !self.initialized {
            return Err(ProxyError::NotInitialized);
        }

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": args
            }
        });

        let response: Value = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ProxyError::Transport(e.to_string()))?
            .json()
            .await
            .map_err(|e| ProxyError::Transport(e.to_string()))?;

        if let Some(error) = response.get("error") {
            return Err(ProxyError::ToolCallFailed(error.to_string()));
        }

        Ok(response["result"].clone())
    }

    async fn health_check(&self) -> bool {
        // Try a simple HTTP request to check if server is reachable
        self.client
            .get(&self.url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .is_ok()
    }

    async fn shutdown(&mut self) -> Result<(), ProxyError> {
        self.initialized = false;
        Ok(())
    }
}
```

**Note:** The actual MCP HTTP+SSE transport may use a different pattern than simple POST (e.g., SSE endpoint for server→client, POST for client→server). Consult the MCP spec and `rmcp` docs during implementation. The Streamable HTTP transport (newer spec) uses a single endpoint. Adjust accordingly.

- [ ] **Step 3: Export from mod.rs**

Add `pub mod http_sse;` to `crates/plugmux-core/src/proxy/mod.rs`.

- [ ] **Step 4: Verify it compiles**

Run: `cd plugmux && cargo build -p plugmux-core`
Expected: Compiles.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add HTTP+SSE transport MCP client"
```

---

## Task 6: Server Manager + Health Checker

**Files:**
- Create: `crates/plugmux-core/src/manager.rs`
- Create: `crates/plugmux-core/src/health.rs`
- Modify: `crates/plugmux-core/src/lib.rs`

The manager owns all running MCP client connections. The health checker periodically pings them and marks online servers as excluded when offline.

- [ ] **Step 1: Implement the server manager**

In `crates/plugmux-core/src/manager.rs`:
```rust
use crate::proxy::stdio::StdioMcpClient;
use crate::proxy::http_sse::HttpSseMcpClient;
use crate::proxy::{McpClient, ProxyError, ToolInfo};
use crate::server::{ServerConfig, Transport};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub struct ManagedServer {
    pub config: ServerConfig,
    pub client: Box<dyn McpClient>,
    pub healthy: bool,
}

pub struct ServerManager {
    servers: Arc<RwLock<HashMap<String, ManagedServer>>>,
}

impl ServerManager {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start and initialize an MCP client for a server config
    pub async fn start_server(&self, config: ServerConfig) -> Result<(), ProxyError> {
        let id = config.id.clone();
        let mut client: Box<dyn McpClient> = match config.transport {
            Transport::Stdio => {
                let command = config.command.clone().ok_or_else(|| {
                    ProxyError::SpawnFailed("stdio server missing 'command'".into())
                })?;
                let args = config.args.clone().unwrap_or_default();
                Box::new(StdioMcpClient::new(command, args))
            }
            Transport::Http => {
                let url = config.url.clone().ok_or_else(|| {
                    ProxyError::Transport("http server missing 'url'".into())
                })?;
                Box::new(HttpSseMcpClient::new(url))
            }
        };

        client.initialize().await?;
        info!("Server '{}' initialized", id);

        let managed = ManagedServer {
            config,
            client,
            healthy: true,
        };

        self.servers.write().await.insert(id, managed);
        Ok(())
    }

    /// Stop and remove a server
    pub async fn stop_server(&self, id: &str) -> Result<(), ProxyError> {
        if let Some(mut server) = self.servers.write().await.remove(id) {
            server.client.shutdown().await?;
            info!("Server '{}' stopped", id);
        }
        Ok(())
    }

    /// List tools for a specific server
    pub async fn list_tools(&self, server_id: &str) -> Result<Vec<ToolInfo>, ProxyError> {
        let servers = self.servers.read().await;
        let server = servers
            .get(server_id)
            .ok_or_else(|| ProxyError::Transport(format!("server '{}' not found", server_id)))?;
        server.client.list_tools().await
    }

    /// Execute a tool on a specific server
    pub async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        args: Value,
    ) -> Result<Value, ProxyError> {
        let servers = self.servers.read().await;
        let server = servers
            .get(server_id)
            .ok_or_else(|| ProxyError::Transport(format!("server '{}' not found", server_id)))?;
        server.client.call_tool(tool_name, args).await
    }

    /// Get list of running server IDs with health status
    pub async fn list_servers(&self) -> Vec<(String, bool)> {
        let servers = self.servers.read().await;
        servers
            .iter()
            .map(|(id, s)| (id.clone(), s.healthy))
            .collect()
    }

    /// Check if a specific server is running and healthy
    pub async fn is_healthy(&self, server_id: &str) -> bool {
        let servers = self.servers.read().await;
        servers
            .get(server_id)
            .map(|s| s.healthy)
            .unwrap_or(false)
    }

    /// Shut down all servers
    pub async fn shutdown_all(&self) {
        let mut servers = self.servers.write().await;
        for (id, server) in servers.iter_mut() {
            if let Err(e) = server.client.shutdown().await {
                warn!("Failed to shutdown server '{}': {}", id, e);
            }
        }
        servers.clear();
    }
}
```

- [ ] **Step 2: Implement the health checker**

In `crates/plugmux-core/src/health.rs`:
```rust
use crate::manager::ServerManager;
use crate::server::Connectivity;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Periodically checks server health.
/// Online servers that fail health checks are marked unhealthy.
/// Local servers are assumed healthy if the process is running.
pub async fn start_health_checker(
    manager: Arc<ServerManager>,
    interval: Duration,
) {
    loop {
        tokio::time::sleep(interval).await;

        let server_ids = manager.list_servers().await;
        for (id, _) in server_ids {
            let healthy = {
                let servers = manager.servers_ref().await;
                if let Some(server) = servers.get(&id) {
                    server.client.health_check().await
                } else {
                    continue;
                }
            };

            if !healthy {
                warn!("Server '{}' health check failed", id);
            }
            // Update health status
            manager.set_health(&id, healthy).await;
        }
    }
}
```

Add these helper methods to `ServerManager`:
```rust
// Add to ServerManager impl
pub async fn servers_ref(&self) -> tokio::sync::RwLockReadGuard<'_, HashMap<String, ManagedServer>> {
    self.servers.read().await
}

pub async fn set_health(&self, id: &str, healthy: bool) {
    if let Some(server) = self.servers.write().await.get_mut(id) {
        server.healthy = healthy;
    }
}
```

- [ ] **Step 3: Update lib.rs**

Add `pub mod manager;` and `pub mod health;`.

- [ ] **Step 4: Verify it compiles**

Run: `cd plugmux && cargo build -p plugmux-core`
Expected: Compiles.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add server manager with lifecycle control and health checker"
```

---

## Task 7: Gateway MCP Server — Tool Implementations

**Files:**
- Create: `crates/plugmux-core/src/gateway/mod.rs`
- Create: `crates/plugmux-core/src/gateway/tools.rs`
- Modify: `crates/plugmux-core/src/lib.rs`
- Modify: `crates/plugmux-core/Cargo.toml`

This implements the 5 LLM-facing tools. The gateway acts as an MCP server that the agent connects to.

- [ ] **Step 1: Define tool response types**

In `crates/plugmux-core/src/gateway/tools.rs`:
```rust
use crate::config::{Permission, PlugmuxConfig};
use crate::environment::{resolve_environment, resolve_named, ResolvedServer};
use crate::manager::ServerManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub tools_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ListServersResponse {
    pub servers: Vec<ServerInfo>,
    pub environment: String,
    pub total_servers: usize,
    pub total_tools: usize,
}

#[derive(Debug, Serialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Debug, Serialize)]
pub struct GetToolsResponse {
    pub server: String,
    pub tools: Vec<ToolSchema>,
}

pub struct GatewayTools {
    pub config: Arc<RwLock<PlugmuxConfig>>,
    pub manager: Arc<ServerManager>,
}

impl GatewayTools {
    pub fn new(config: Arc<RwLock<PlugmuxConfig>>, manager: Arc<ServerManager>) -> Self {
        Self { config, manager }
    }

    /// list_servers — returns all enabled + healthy servers in this environment
    pub async fn list_servers(&self, env_id: &str) -> Result<ListServersResponse, String> {
        let config = self.config.read().await;

        let resolved = if env_id == "main" {
            crate::environment::resolve_main(&config)
        } else {
            resolve_named(&config, env_id)
                .ok_or_else(|| format!("environment '{}' not found", env_id))?
        };

        let mut servers = vec![];
        let mut total_tools = 0;

        for rs in &resolved {
            let healthy = self.manager.is_healthy(&rs.config.id).await;
            if !healthy {
                continue; // Exclude unhealthy servers
            }

            let tools_count = self
                .manager
                .list_tools(&rs.config.id)
                .await
                .map(|t| t.len())
                .unwrap_or(0);

            total_tools += tools_count;

            servers.push(ServerInfo {
                id: rs.config.id.clone(),
                name: rs.config.name.clone(),
                description: rs.config.description.clone(),
                status: "ready".to_string(),
                tools_count,
            });
        }

        let total_servers = servers.len();
        Ok(ListServersResponse {
            servers,
            environment: env_id.to_string(),
            total_servers,
            total_tools,
        })
    }

    /// get_tools — returns full tool schemas for a specific server
    pub async fn get_tools(&self, server_id: &str) -> Result<GetToolsResponse, String> {
        let tools = self
            .manager
            .list_tools(server_id)
            .await
            .map_err(|e| e.to_string())?;

        let tool_schemas: Vec<ToolSchema> = tools
            .into_iter()
            .map(|t| ToolSchema {
                name: t.name,
                description: t.description,
                input_schema: t.input_schema,
            })
            .collect();

        Ok(GetToolsResponse {
            server: server_id.to_string(),
            tools: tool_schemas,
        })
    }

    /// execute — proxy a tool call to an upstream server
    pub async fn execute(
        &self,
        server_id: &str,
        tool_name: &str,
        args: Value,
    ) -> Result<Value, String> {
        self.manager
            .call_tool(server_id, tool_name, args)
            .await
            .map_err(|e| e.to_string())
    }

    /// enable_server — enable a server in this environment (checks permissions)
    pub async fn enable_server(
        &self,
        env_id: &str,
        server_id: &str,
    ) -> Result<String, String> {
        let permission = self.check_permission(env_id, "enable_server").await?;
        match permission {
            Permission::Allow => {
                self.set_server_override(env_id, server_id, true).await?;
                Ok(format!("Server '{}' enabled", server_id))
            }
            Permission::Approve => Err("Action requires user approval (pending)".to_string()),
            Permission::Disable => Err("Action is disabled for this environment".to_string()),
        }
    }

    /// disable_server — disable a server in this environment (checks permissions)
    pub async fn disable_server(
        &self,
        env_id: &str,
        server_id: &str,
    ) -> Result<String, String> {
        let permission = self.check_permission(env_id, "disable_server").await?;
        match permission {
            Permission::Allow => {
                self.set_server_override(env_id, server_id, false).await?;
                Ok(format!("Server '{}' disabled", server_id))
            }
            Permission::Approve => Err("Action requires user approval (pending)".to_string()),
            Permission::Disable => Err("Action is disabled for this environment".to_string()),
        }
    }

    async fn check_permission(
        &self,
        env_id: &str,
        action: &str,
    ) -> Result<Permission, String> {
        let config = self.config.read().await;
        let env = config
            .find_environment(env_id)
            .ok_or_else(|| format!("environment '{}' not found", env_id))?;

        Ok(env
            .permissions
            .get(action)
            .cloned()
            .unwrap_or(Permission::Approve)) // Default to Approve
    }

    async fn set_server_override(
        &self,
        env_id: &str,
        server_id: &str,
        enabled: bool,
    ) -> Result<(), String> {
        let mut config = self.config.write().await;
        let env = config
            .environments
            .iter_mut()
            .find(|e| e.id == env_id)
            .ok_or_else(|| format!("environment '{}' not found", env_id))?;

        env.overrides.insert(
            server_id.to_string(),
            crate::config::ServerOverride { enabled },
        );

        // Persist config
        let path = PlugmuxConfig::config_path();
        config.save(&path).map_err(|e| e.to_string())?;

        Ok(())
    }
}
```

- [ ] **Step 2: Create gateway mod.rs**

In `crates/plugmux-core/src/gateway/mod.rs`:
```rust
pub mod tools;
pub mod router;
```

- [ ] **Step 3: Update lib.rs**

Add `pub mod gateway;`.

- [ ] **Step 4: Verify it compiles**

Run: `cd plugmux && cargo build -p plugmux-core`
Expected: Compiles.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add gateway tool implementations (list_servers, get_tools, execute, enable/disable)"
```

---

## Task 8: HTTP Server + Environment Router

**Files:**
- Create: `crates/plugmux-core/src/gateway/router.rs`
- Modify: `crates/plugmux-core/Cargo.toml`

This sets up the axum HTTP server that exposes per-environment MCP endpoints. Each environment URL serves an MCP server (via HTTP+SSE or Streamable HTTP transport).

- [ ] **Step 1: Add axum dependency**

Add to `crates/plugmux-core/Cargo.toml`:
```toml
axum = "0.8"
rmcp = { version = "1", features = ["client", "server", "transport-child-process", "transport-sse-server"] }
```

Note: Check exact rmcp feature names. We need server-side SSE/HTTP transport support.

- [ ] **Step 2: Implement the router**

In `crates/plugmux-core/src/gateway/router.rs`:
```rust
use crate::config::PlugmuxConfig;
use crate::gateway::tools::GatewayTools;
use crate::manager::ServerManager;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub tools: Arc<GatewayTools>,
}

#[derive(Deserialize)]
pub struct ToolCallRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl JsonRpcResponse {
    fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Value, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message }),
        }
    }
}

/// Build the axum router
pub fn build_router(
    config: Arc<RwLock<PlugmuxConfig>>,
    manager: Arc<ServerManager>,
) -> Router {
    let tools = Arc::new(GatewayTools::new(config, manager));
    let state = AppState { tools };

    Router::new()
        .route("/env/{env_id}", post(handle_mcp_request))
        .route("/health", get(health_check))
        .with_state(state)
}

async fn health_check() -> &'static str {
    "ok"
}

/// Handle MCP JSON-RPC requests for a specific environment
async fn handle_mcp_request(
    Path(env_id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<ToolCallRequest>,
) -> Json<JsonRpcResponse> {
    let response = match request.method.as_str() {
        "initialize" => {
            let result = serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "plugmux",
                    "version": "0.1.0"
                }
            });
            JsonRpcResponse::success(request.id, result)
        }

        "tools/list" => {
            // Return our 5 gateway tools
            let tools = serde_json::json!({
                "tools": [
                    {
                        "name": "list_servers",
                        "description": "List all available MCP servers in this environment",
                        "inputSchema": {
                            "type": "object",
                            "properties": {},
                            "required": []
                        }
                    },
                    {
                        "name": "get_tools",
                        "description": "Get full tool schemas for a specific server",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "server_id": { "type": "string", "description": "The server ID to get tools for" }
                            },
                            "required": ["server_id"]
                        }
                    },
                    {
                        "name": "execute",
                        "description": "Execute a tool on a specific server",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "server_id": { "type": "string", "description": "The server to call" },
                                "tool_name": { "type": "string", "description": "The tool to execute" },
                                "args": { "type": "object", "description": "Arguments to pass to the tool" }
                            },
                            "required": ["server_id", "tool_name"]
                        }
                    },
                    {
                        "name": "enable_server",
                        "description": "Enable a server in this environment",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "server_id": { "type": "string" }
                            },
                            "required": ["server_id"]
                        }
                    },
                    {
                        "name": "disable_server",
                        "description": "Disable a server in this environment",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "server_id": { "type": "string" }
                            },
                            "required": ["server_id"]
                        }
                    }
                ]
            });
            JsonRpcResponse::success(request.id, tools)
        }

        "tools/call" => {
            let tool_name = request.params["name"].as_str().unwrap_or("");
            let args = &request.params["arguments"];

            match tool_name {
                "list_servers" => match state.tools.list_servers(&env_id).await {
                    Ok(resp) => JsonRpcResponse::success(
                        request.id,
                        serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string(&resp).unwrap() }] }),
                    ),
                    Err(e) => JsonRpcResponse::error(request.id, -32000, e),
                },

                "get_tools" => {
                    let server_id = args["server_id"].as_str().unwrap_or("");
                    match state.tools.get_tools(server_id).await {
                        Ok(resp) => JsonRpcResponse::success(
                            request.id,
                            serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string(&resp).unwrap() }] }),
                        ),
                        Err(e) => JsonRpcResponse::error(request.id, -32000, e),
                    }
                }

                "execute" => {
                    let server_id = args["server_id"].as_str().unwrap_or("");
                    let tool = args["tool_name"].as_str().unwrap_or("");
                    let tool_args = args.get("args").cloned().unwrap_or(Value::Object(Default::default()));
                    match state.tools.execute(server_id, tool, tool_args).await {
                        Ok(result) => JsonRpcResponse::success(request.id, result),
                        Err(e) => JsonRpcResponse::error(request.id, -32000, e),
                    }
                }

                "enable_server" => {
                    let server_id = args["server_id"].as_str().unwrap_or("");
                    match state.tools.enable_server(&env_id, server_id).await {
                        Ok(msg) => JsonRpcResponse::success(
                            request.id,
                            serde_json::json!({ "content": [{ "type": "text", "text": msg }] }),
                        ),
                        Err(e) => JsonRpcResponse::error(request.id, -32000, e),
                    }
                }

                "disable_server" => {
                    let server_id = args["server_id"].as_str().unwrap_or("");
                    match state.tools.disable_server(&env_id, server_id).await {
                        Ok(msg) => JsonRpcResponse::success(
                            request.id,
                            serde_json::json!({ "content": [{ "type": "text", "text": msg }] }),
                        ),
                        Err(e) => JsonRpcResponse::error(request.id, -32000, e),
                    }
                }

                _ => JsonRpcResponse::error(
                    request.id,
                    -32601,
                    format!("unknown tool: {}", tool_name),
                ),
            }
        }

        _ => JsonRpcResponse::error(
            request.id,
            -32601,
            format!("unknown method: {}", request.method),
        ),
    };

    Json(response)
}

/// Start the HTTP server
pub async fn start_server(
    config: Arc<RwLock<PlugmuxConfig>>,
    manager: Arc<ServerManager>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let router = build_router(config, manager);
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("plugmux gateway listening on http://{}", addr);
    axum::serve(listener, router).await?;
    Ok(())
}
```

**Important:** This implements a simplified JSON-RPC handler. The actual MCP HTTP transport may require SSE for server-initiated messages. For Phase 1 (CLI, no streaming), this request-response model is sufficient. Phase 2 can upgrade to full SSE using `rmcp`'s server transport.

- [ ] **Step 3: Verify it compiles**

Run: `cd plugmux && cargo build -p plugmux-core`
Expected: Compiles.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add axum HTTP server with per-environment MCP routing"
```

---

## Task 9: CLI Commands

**Files:**
- Create: `crates/plugmux-cli/src/commands/mod.rs`
- Create: `crates/plugmux-cli/src/commands/start.rs`
- Create: `crates/plugmux-cli/src/commands/stop.rs`
- Create: `crates/plugmux-cli/src/commands/status.rs`
- Create: `crates/plugmux-cli/src/commands/env.rs`
- Create: `crates/plugmux-cli/src/commands/server.rs`
- Create: `crates/plugmux-cli/src/commands/config.rs`
- Modify: `crates/plugmux-cli/src/main.rs`

- [ ] **Step 1: Define CLI structure with clap**

In `crates/plugmux-cli/src/main.rs`:
```rust
mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "plugmux", version, about = "MCP gateway — one URL, all your servers")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the gateway server
    Start {
        /// Run as background daemon
        #[arg(long)]
        daemon: bool,
        /// Port to listen on
        #[arg(long, default_value = "4242")]
        port: u16,
    },
    /// Stop the gateway server
    Stop,
    /// Show server health status
    Status,
    /// Manage environments
    Env {
        #[command(subcommand)]
        action: commands::env::EnvAction,
    },
    /// Manage servers
    Server {
        #[command(subcommand)]
        action: commands::server::ServerAction,
    },
    /// Config operations
    Config {
        #[command(subcommand)]
        action: commands::config::ConfigAction,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Start { daemon, port } => commands::start::run(daemon, port).await,
        Commands::Stop => commands::stop::run().await,
        Commands::Status => commands::status::run().await,
        Commands::Env { action } => commands::env::run(action).await,
        Commands::Server { action } => commands::server::run(action).await,
        Commands::Config { action } => commands::config::run(action).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

- [ ] **Step 2: Implement commands/mod.rs**

```rust
pub mod start;
pub mod stop;
pub mod status;
pub mod env;
pub mod server;
pub mod config;
```

- [ ] **Step 3: Implement start command**

In `crates/plugmux-cli/src/commands/start.rs`:
```rust
use plugmux_core::config::PlugmuxConfig;
use plugmux_core::environment::resolve_named;
use plugmux_core::gateway::router;
use plugmux_core::manager::ServerManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub async fn run(daemon: bool, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = PlugmuxConfig::config_path();
    let config = PlugmuxConfig::load_or_default(&config_path);

    info!("Loaded config from {:?}", config_path);
    info!(
        "Main: {} servers, {} environments",
        config.main.servers.len(),
        config.environments.len()
    );

    let manager = Arc::new(ServerManager::new());
    let config = Arc::new(RwLock::new(config));

    // Start all enabled Main servers
    {
        let cfg = config.read().await;
        for server in &cfg.main.servers {
            if server.enabled {
                info!("Starting server '{}'...", server.id);
                if let Err(e) = manager.start_server(server.clone()).await {
                    eprintln!("Warning: failed to start '{}': {}", server.id, e);
                }
            }
        }

        // Start environment-specific servers
        for env in &cfg.environments {
            for server in &env.servers {
                if server.enabled {
                    info!("Starting server '{}' (env: {})...", server.id, env.id);
                    if let Err(e) = manager.start_server(server.clone()).await {
                        eprintln!("Warning: failed to start '{}': {}", server.id, e);
                    }
                }
            }
        }
    }

    println!("plugmux gateway starting on http://127.0.0.1:{}", port);
    println!("Environments:");
    {
        let cfg = config.read().await;
        println!("  main: http://127.0.0.1:{}/env/main", port);
        for env in &cfg.environments {
            println!("  {}: http://127.0.0.1:{}/env/{}", env.name, port, env.id);
        }
    }

    router::start_server(config, manager, port).await?;
    Ok(())
}
```

- [ ] **Step 4: Implement env commands**

In `crates/plugmux-cli/src/commands/env.rs`:
```rust
use clap::Subcommand;
use plugmux_core::config::PlugmuxConfig;

#[derive(Subcommand)]
pub enum EnvAction {
    /// List all environments
    List,
    /// Create a new environment
    Create {
        name: String,
        #[arg(long)]
        preset: Option<String>,
    },
    /// Delete an environment
    Delete { name: String },
    /// Show environment URL
    Url { name: String },
}

pub async fn run(action: EnvAction) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = PlugmuxConfig::config_path();
    let mut config = PlugmuxConfig::load_or_default(&config_path);

    match action {
        EnvAction::List => {
            if config.environments.is_empty() {
                println!("No environments configured. Use 'plugmux env create <name>' to create one.");
            } else {
                for env in &config.environments {
                    let server_count = env.servers.len();
                    println!(
                        "  {} ({} servers) → http://localhost:4242/env/{}",
                        env.name, server_count, env.id
                    );
                }
            }
        }
        EnvAction::Create { name, preset: _ } => {
            let env = config.add_environment(&name);
            let id = env.id.clone();
            config.save(&config_path)?;
            println!("Created environment '{}' → http://localhost:4242/env/{}", name, id);
        }
        EnvAction::Delete { name } => {
            let slug = plugmux_core::slug::slugify(&name);
            if config.remove_environment(&slug) {
                config.save(&config_path)?;
                println!("Deleted environment '{}'", name);
            } else {
                eprintln!("Environment '{}' not found", name);
            }
        }
        EnvAction::Url { name } => {
            let slug = plugmux_core::slug::slugify(&name);
            match config.find_environment(&slug) {
                Some(env) => println!("{}", env.endpoint.as_deref().unwrap_or("no endpoint")),
                None => eprintln!("Environment '{}' not found", name),
            }
        }
    }

    Ok(())
}
```

- [ ] **Step 5: Implement server commands**

In `crates/plugmux-cli/src/commands/server.rs`:
```rust
use clap::Subcommand;
use plugmux_core::config::PlugmuxConfig;
use plugmux_core::server::{Connectivity, ServerConfig, Transport};

#[derive(Subcommand)]
pub enum ServerAction {
    /// Add a server
    Add {
        id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long, default_value = "stdio")]
        transport: String,
        #[arg(long)]
        command: Option<String>,
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        env: Option<String>,
        #[arg(long, default_value = "local")]
        connectivity: String,
    },
    /// Remove a server
    Remove {
        id: String,
        #[arg(long)]
        env: Option<String>,
    },
    /// List servers
    List {
        #[arg(long)]
        env: Option<String>,
    },
    /// Toggle server enabled/disabled
    Toggle {
        id: String,
        #[arg(long)]
        env: Option<String>,
    },
    /// Rename a server's display name
    Rename {
        id: String,
        #[arg(long)]
        name: String,
    },
}

pub async fn run(action: ServerAction) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = PlugmuxConfig::config_path();
    let mut config = PlugmuxConfig::load_or_default(&config_path);

    match action {
        ServerAction::Add {
            id,
            name,
            transport,
            command,
            url,
            env,
            connectivity,
        } => {
            let transport = match transport.as_str() {
                "stdio" => Transport::Stdio,
                "http" => Transport::Http,
                _ => return Err("transport must be 'stdio' or 'http'".into()),
            };
            let connectivity = match connectivity.as_str() {
                "local" => Connectivity::Local,
                "online" => Connectivity::Online,
                _ => return Err("connectivity must be 'local' or 'online'".into()),
            };

            let server = ServerConfig {
                id: id.clone(),
                name: name.unwrap_or_else(|| id.clone()),
                transport,
                command,
                args: None, // Could parse from command string
                url,
                connectivity,
                enabled: true,
                description: None,
            };

            if let Some(env_id) = env {
                let slug = plugmux_core::slug::slugify(&env_id);
                let env = config
                    .environments
                    .iter_mut()
                    .find(|e| e.id == slug)
                    .ok_or_else(|| format!("environment '{}' not found", env_id))?;
                env.servers.push(server);
                println!("Added '{}' to environment '{}'", id, env_id);
            } else {
                config.main.servers.push(server);
                println!("Added '{}' to Main", id);
            }
            config.save(&config_path)?;
        }

        ServerAction::Remove { id, env } => {
            if let Some(env_id) = env {
                let slug = plugmux_core::slug::slugify(&env_id);
                let env = config
                    .environments
                    .iter_mut()
                    .find(|e| e.id == slug)
                    .ok_or_else(|| format!("environment '{}' not found", env_id))?;
                env.servers.retain(|s| s.id != id);
            } else {
                config.main.servers.retain(|s| s.id != id);
            }
            config.save(&config_path)?;
            println!("Removed '{}'", id);
        }

        ServerAction::List { env } => {
            let servers = if let Some(env_id) = env {
                let slug = plugmux_core::slug::slugify(&env_id);
                let resolved = plugmux_core::environment::resolve_named(&config, &slug)
                    .ok_or_else(|| format!("environment '{}' not found", env_id))?;
                println!("Servers in environment '{}':", env_id);
                for rs in &resolved {
                    let source = match rs.source {
                        plugmux_core::environment::ServerSource::Main => "(from Main)",
                        plugmux_core::environment::ServerSource::Environment => "(local)",
                    };
                    let status = if rs.config.enabled { "✓" } else { "✗" };
                    println!("  {} {} — {} {}", status, rs.config.name, rs.config.id, source);
                }
                return Ok(());
            } else {
                &config.main.servers
            };

            println!("Main servers:");
            for s in servers {
                let status = if s.enabled { "✓" } else { "✗" };
                println!("  {} {} — {}", status, s.name, s.id);
            }
        }

        ServerAction::Toggle { id, env } => {
            if let Some(env_id) = env {
                let slug = plugmux_core::slug::slugify(&env_id);
                let env = config
                    .environments
                    .iter_mut()
                    .find(|e| e.id == slug)
                    .ok_or_else(|| format!("environment '{}' not found", env_id))?;

                // Check if it's an env-local server
                if let Some(s) = env.servers.iter_mut().find(|s| s.id == id) {
                    s.enabled = !s.enabled;
                    println!("'{}' is now {}", id, if s.enabled { "enabled" } else { "disabled" });
                } else {
                    // It's a Main server — use override
                    let current = env
                        .overrides
                        .get(&id)
                        .map(|o| o.enabled)
                        .unwrap_or(true);
                    env.overrides.insert(
                        id.clone(),
                        plugmux_core::config::ServerOverride { enabled: !current },
                    );
                    println!(
                        "'{}' is now {} in '{}'",
                        id,
                        if !current { "enabled" } else { "disabled" },
                        env_id
                    );
                }
            } else {
                let server = config
                    .main
                    .servers
                    .iter_mut()
                    .find(|s| s.id == id)
                    .ok_or_else(|| format!("server '{}' not found in Main", id))?;
                server.enabled = !server.enabled;
                println!(
                    "'{}' is now {}",
                    id,
                    if server.enabled { "enabled" } else { "disabled" }
                );
            }
            config.save(&config_path)?;
        }

        ServerAction::Rename { id, name } => {
            let server = config
                .main
                .servers
                .iter_mut()
                .find(|s| s.id == id)
                .ok_or_else(|| format!("server '{}' not found in Main", id))?;
            server.name = name.clone();
            config.save(&config_path)?;
            println!("Renamed '{}' to '{}'", id, name);
        }
    }

    Ok(())
}
```

- [ ] **Step 6: Implement config commands**

In `crates/plugmux-cli/src/commands/config.rs`:
```rust
use clap::Subcommand;
use plugmux_core::config::PlugmuxConfig;

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show config file path
    Path,
    /// Export current config to stdout
    Export,
    /// Import config from a file
    Import { file: String },
}

pub async fn run(action: ConfigAction) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = PlugmuxConfig::config_path();

    match action {
        ConfigAction::Path => {
            println!("{}", config_path.display());
        }
        ConfigAction::Export => {
            let config = PlugmuxConfig::load_or_default(&config_path);
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        ConfigAction::Import { file } => {
            let path = std::path::PathBuf::from(&file);
            let config = PlugmuxConfig::load(&path)?;
            config.save(&config_path)?;
            println!("Imported config from '{}'", file);
        }
    }

    Ok(())
}
```

- [ ] **Step 7: Implement stop and status stubs**

In `crates/plugmux-cli/src/commands/stop.rs`:
```rust
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // For Phase 1: send shutdown signal to running gateway
    // Simple approach: check if gateway is running via health endpoint
    let client = reqwest::Client::new();
    match client
        .get("http://127.0.0.1:4242/health")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
    {
        Ok(_) => {
            println!("Gateway is running. Use Ctrl+C to stop the foreground process.");
            println!("Daemon stop will be implemented in Phase 2.");
        }
        Err(_) => {
            println!("No gateway running on port 4242.");
        }
    }
    Ok(())
}
```

In `crates/plugmux-cli/src/commands/status.rs`:
```rust
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    match client
        .get("http://127.0.0.1:4242/health")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
    {
        Ok(_) => println!("plugmux gateway: running on http://127.0.0.1:4242"),
        Err(_) => println!("plugmux gateway: not running"),
    }
    Ok(())
}
```

Add `reqwest` to `crates/plugmux-cli/Cargo.toml`:
```toml
reqwest = { version = "0.12", features = ["json"] }
```

- [ ] **Step 8: Verify full CLI compiles**

Run: `cd plugmux && cargo build`
Expected: Both crates compile. `target/debug/plugmux` binary exists.

- [ ] **Step 9: Test CLI help output**

Run: `cd plugmux && cargo run -- --help`
Expected: Shows usage with Start, Stop, Status, Env, Server, Config subcommands.

Run: `cd plugmux && cargo run -- env --help`
Expected: Shows List, Create, Delete, Url subcommands.

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "feat: add CLI with start, env, server, config, stop, and status commands"
```

---

## Task 10: Integration Test

**Files:**
- Create: `plugmux/tests/integration/mock_mcp_server.rs`
- Create: `plugmux/tests/integration/gateway_test.rs`

This task creates a minimal mock MCP server and tests the full gateway flow end-to-end.

- [ ] **Step 1: Create a mock MCP server**

The mock server is a simple Rust binary that speaks MCP over stdio. It exposes one tool: `echo` that returns its input.

In `plugmux/tests/integration/mock_mcp_server.rs`:
```rust
//! A minimal MCP server for integration testing.
//! Reads JSON-RPC from stdin, responds on stdout.

use std::io::{self, BufRead, Write};
use serde_json::{json, Value};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let method = request["method"].as_str().unwrap_or("");
        let id = request.get("id").cloned();

        // Notifications (no id) — just acknowledge silently
        if id.is_none() {
            continue;
        }

        let response = match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "mock-mcp", "version": "0.1.0" }
                }
            }),
            "tools/list" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": [{
                        "name": "echo",
                        "description": "Echoes back the input",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "message": { "type": "string" }
                            },
                            "required": ["message"]
                        }
                    }]
                }
            }),
            "tools/call" => {
                let args = &request["params"]["arguments"];
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{
                            "type": "text",
                            "text": format!("echo: {}", args["message"].as_str().unwrap_or(""))
                        }]
                    }
                })
            }
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("unknown method: {}", method) }
            }),
        };

        let mut resp_str = serde_json::to_string(&response).unwrap();
        resp_str.push('\n');
        out.write_all(resp_str.as_bytes()).unwrap();
        out.flush().unwrap();
    }
}
```

Add the mock server as a binary in the workspace. Create `plugmux/tests/integration/Cargo.toml` or add as a `[[bin]]` in the workspace.

Simplest approach — add to `crates/plugmux-cli/Cargo.toml`:
```toml
[[bin]]
name = "mock-mcp-server"
path = "../tests/integration/mock_mcp_server.rs"
```

- [ ] **Step 2: Build the mock server**

Run: `cd plugmux && cargo build --bin mock-mcp-server`
Expected: Produces `target/debug/mock-mcp-server`

- [ ] **Step 3: Write integration test**

In `plugmux/tests/integration/gateway_test.rs`:
```rust
//! Integration test: starts the gateway with a mock MCP server and
//! verifies the full flow: list_servers → get_tools → execute

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use plugmux_core::config::{PlugmuxConfig, MainConfig, EnvironmentConfig};
use plugmux_core::server::{ServerConfig, Transport, Connectivity};
use plugmux_core::manager::ServerManager;
use plugmux_core::gateway::router;
use std::collections::HashMap;

#[tokio::test]
async fn test_full_gateway_flow() {
    // Find mock server binary
    let mock_path = PathBuf::from(env!("CARGO_BIN_EXE_mock-mcp-server"));

    // Create config with mock server
    let config = PlugmuxConfig {
        version: 1,
        main: MainConfig {
            servers: vec![ServerConfig {
                id: "mock".into(),
                name: "Mock Server".into(),
                transport: Transport::Stdio,
                command: Some(mock_path.to_str().unwrap().into()),
                args: Some(vec![]),
                url: None,
                connectivity: Connectivity::Local,
                enabled: true,
                description: Some("A test server".into()),
            }],
        },
        environments: vec![EnvironmentConfig {
            id: "test-env".into(),
            name: "Test Env".into(),
            endpoint: Some("http://localhost:0/env/test-env".into()),
            servers: vec![],
            overrides: HashMap::new(),
            permissions: HashMap::new(),
        }],
    };

    let manager = Arc::new(ServerManager::new());
    let config = Arc::new(RwLock::new(config));

    // Start the mock server
    {
        let cfg = config.read().await;
        for server in &cfg.main.servers {
            manager.start_server(server.clone()).await.expect("failed to start mock server");
        }
    }

    // Build router and test with a client
    let app = router::build_router(config.clone(), manager.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give server a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{}/env/main", port);

    // Test 1: Initialize
    let resp: serde_json::Value = client
        .post(&base)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "0.1.0" }
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(resp["result"]["serverInfo"]["name"].as_str().unwrap() == "plugmux");

    // Test 2: tools/list
    let resp: serde_json::Value = client
        .post(&base)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let tools = resp["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 5); // Our 5 gateway tools

    // Test 3: Call list_servers
    let resp: serde_json::Value = client
        .post(&base)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "list_servers",
                "arguments": {}
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(resp["result"]["content"][0]["text"].as_str().unwrap().contains("mock"));

    // Test 4: Call get_tools for mock server
    let resp: serde_json::Value = client
        .post(&base)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "get_tools",
                "arguments": { "server_id": "mock" }
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(resp["result"]["content"][0]["text"].as_str().unwrap().contains("echo"));

    // Test 5: Execute echo tool
    let resp: serde_json::Value = client
        .post(&base)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "execute",
                "arguments": {
                    "server_id": "mock",
                    "tool_name": "echo",
                    "args": { "message": "hello plugmux" }
                }
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("hello plugmux"));

    // Cleanup
    manager.shutdown_all().await;
}
```

- [ ] **Step 4: Run integration test**

Run: `cd plugmux && cargo test --test gateway_test`
Expected: All 5 assertions pass — full flow works end-to-end.

Note: The test file location may need adjustment depending on how the workspace test structure is set up. If `tests/integration/` doesn't work as an auto-discovered integration test, move `gateway_test.rs` to `plugmux-cli/tests/gateway_test.rs` instead.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "test: add integration test with mock MCP server for full gateway flow"
```

---

## Task 11: README + Final Polish

**Files:**
- Create: `plugmux/README.md`

- [ ] **Step 1: Write README**

```markdown
# plugmux

One URL, all your MCP servers.

plugmux is a local gateway that sits between your AI coding agent (Claude, Cursor, Codex) and your MCP servers. Instead of configuring each server separately, you point your agent at plugmux — it manages everything.

## Quick Start

```bash
# Create a config
plugmux server add figma --transport stdio --command "npx -y @anthropic/figma-mcp"
plugmux server add context7 --transport http --url "https://context7.dev/mcp"

# Create an environment
plugmux env create my-project

# Start the gateway
plugmux start
```

Then configure your agent to connect to:
```
http://localhost:4242/env/my-project
```

## Concepts

- **Main** — your base servers, inherited by all environments
- **Environment** — a project workspace with its own servers + a unique URL
- **Presets** — one-click templates to bootstrap environments

## CLI Reference

```bash
plugmux start [--port 4242]     # Start gateway
plugmux status                   # Check if running
plugmux env list                 # List environments
plugmux env create <name>        # Create environment
plugmux server add <id> ...      # Add server
plugmux server list [--env name] # List servers
plugmux config path              # Show config location
```

## License

MIT
```

- [ ] **Step 2: Run full test suite**

Run: `cd plugmux && cargo test`
Expected: All unit + integration tests pass.

Run: `cd plugmux && cargo clippy`
Expected: No warnings.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "docs: add README and polish for Phase 1 release"
```

---

## Completion Checklist

After all tasks are done, verify:

- [ ] `cargo build` succeeds
- [ ] `cargo test` — all tests pass
- [ ] `cargo clippy` — no warnings
- [ ] `plugmux --help` shows all commands
- [ ] `plugmux env create test-project` creates environment in config
- [ ] `plugmux server add mock --transport stdio --command "echo"` adds to Main
- [ ] `plugmux server list` shows the server
- [ ] `plugmux server list --env test-project` shows inherited + local
- [ ] `plugmux start` starts the gateway, environment URLs are printed
- [ ] HTTP POST to `/env/main` with `initialize` returns plugmux server info
- [ ] HTTP POST to `/env/main` with `tools/list` returns 5 tools
