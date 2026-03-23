# Phase 3 — Catalog & Community Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor plugmux's config model (no inheritance, string-based server IDs, global permissions), add a bundled MCP server catalog with icons and categories, add environment presets, and enable community contributions via GitHub PRs.

**Architecture:** The config model is simplified first (Tasks 1-3), then the catalog/resolver layer is built (Tasks 4-6), then migration (Task 7, needs catalog), followed by UI updates (Tasks 8-12), CLI updates (Task 13), and community infrastructure (Task 14). Each task produces working, testable code. The existing integration test is updated at the end to validate the full flow.

**Note on async:** The codebase uses `tokio::sync::RwLock` (not `std::sync`). All `.read()` and `.write()` calls are async and use `.await`, not `.unwrap()`. Code snippets in this plan show simplified sync patterns — the implementer must use `tokio::sync` equivalents throughout.

**Tech Stack:** Rust (plugmux-core, plugmux-cli, Tauri backend), React + TypeScript + shadcn/ui (Tauri frontend), serde_json, axum, rmcp

**Spec:** `docs/superpowers/specs/2026-03-21-plugmux-phase3-design.md`

---

## File Structure

### New Files

| File | Purpose |
|------|---------|
| `crates/plugmux-core/src/catalog.rs` | CatalogRegistry — load bundled servers.json, lookup by ID, search |
| `crates/plugmux-core/src/custom_servers.rs` | CustomServerStore — load/save custom_servers.json, CRUD |
| `crates/plugmux-core/src/resolver.rs` | ServerResolver — resolve server ID to full ServerConfig |
| `crates/plugmux-core/src/migration.rs` | Migrate Phase 2 plugmux.json to new config format |
| `catalog/servers.json` | Bundled catalog of curated MCP servers |
| `catalog/presets.json` | Bundled environment preset templates |
| `catalog/icons/*.svg` | Monochrome SVG icons for catalog servers |
| `catalog/CONTRIBUTING.md` | Guide for community server submissions |
| `crates/plugmux-cli/src/commands/custom.rs` | CLI commands for custom server management |
| `crates/plugmux-cli/src/commands/catalog.rs` | CLI commands for catalog browsing |
| `crates/plugmux-app/src/hooks/useCatalog.ts` | React hook for catalog data |
| `crates/plugmux-app/src/hooks/useCustomServers.ts` | React hook for custom server management |
| `crates/plugmux-app/src/components/catalog/CatalogCard.tsx` | Server card for catalog grid |
| `crates/plugmux-app/src/components/catalog/CatalogDetail.tsx` | Expanded server detail view |
| `crates/plugmux-app/src/components/catalog/CategoryFilter.tsx` | Category filter pills |
| `crates/plugmux-app/src/components/settings/PermissionsSection.tsx` | Global permissions UI |
| `crates/plugmux-app/src/components/settings/CustomServersSection.tsx` | Custom server management UI |

### Modified Files

| File | Changes |
|------|---------|
| `crates/plugmux-core/src/lib.rs` | Add new module exports |
| `crates/plugmux-core/src/config.rs` | Rewrite: new Config struct, config.json format, default bootstrap |
| `crates/plugmux-core/src/server.rs` | Add HealthStatus enum, remove `enabled` field from ServerConfig |
| `crates/plugmux-core/src/environment.rs` | Simplify: remove inheritance/override logic |
| `crates/plugmux-core/src/gateway/router.rs` | Update tool schemas, dispatch to new tool implementations |
| `crates/plugmux-core/src/gateway/tools.rs` | Rewrite enable/disable as add/remove, global permissions |
| `crates/plugmux-core/src/pending_actions.rs` | Minor: PendingAction struct already matches spec |
| `crates/plugmux-core/src/manager.rs` | Add HealthStatus to ManagedServer |
| `crates/plugmux-core/src/health.rs` | Return HealthStatus instead of bool |
| `crates/plugmux-app/src-tauri/src/lib.rs` | Register new commands, remove old ones |
| `crates/plugmux-app/src-tauri/src/commands.rs` | Rewrite for new command set |
| `crates/plugmux-app/src-tauri/src/engine.rs` | Use new config + resolver |
| `crates/plugmux-app/src-tauri/src/events.rs` | Update event payloads |
| `crates/plugmux-app/src-tauri/src/watcher.rs` | Watch both config.json and custom_servers.json |
| `crates/plugmux-app/src/lib/commands.ts` | New TypeScript command wrappers |
| `crates/plugmux-app/src/hooks/useConfig.ts` | Updated for new config shape |
| `crates/plugmux-app/src/App.tsx` | Remove MainPage routing, update navigation |
| `crates/plugmux-app/src/components/layout/Sidebar.tsx` | "Main" → "Default", aggregate health dots per env |
| `crates/plugmux-app/src/pages/EnvironmentPage.tsx` | Simplified: flat server list, no inheritance |
| `crates/plugmux-app/src/pages/CatalogPage.tsx` | Full implementation |
| `crates/plugmux-app/src/pages/PresetsPage.tsx` | Minimal implementation |
| `crates/plugmux-app/src/pages/SettingsPage.tsx` | Add Permissions + Custom Servers sections |
| `crates/plugmux-app/src/components/servers/ServerCard.tsx` | Add health dot, icon support |
| `crates/plugmux-app/src/components/servers/AddServerDialog.tsx` | Repurpose for custom servers only (full ServerConfig minus `enabled`) |
| `crates/plugmux-app/src/components/environments/CreateEnvironmentDialog.tsx` | Add optional preset selection |
| `crates/plugmux-app/src-tauri/src/tray.rs` | Update tray icon aggregate health (worst across all envs) |
| `crates/plugmux-cli/src/main.rs` | New CLI command structure |
| `crates/plugmux-cli/src/commands/stop.rs` | Read port from config.json |
| `crates/plugmux-cli/src/commands/status.rs` | Read port from config.json |
| `crates/plugmux-cli/src/commands/mod.rs` | Add catalog, custom modules |
| `crates/plugmux-cli/src/commands/start.rs` | Use new config + resolver |
| `crates/plugmux-cli/src/commands/env.rs` | Updated for new config, --preset flag |
| `crates/plugmux-cli/src/commands/server.rs` | Rewrite: always env-scoped, string IDs |
| `crates/plugmux-cli/src/commands/config.rs` | Add migrate subcommand |
| `crates/plugmux-cli/tests/gateway_integration.rs` | Update for new config model |

### Deleted Files

| File | Reason |
|------|--------|
| `crates/plugmux-app/src/pages/MainPage.tsx` | Default is just an environment |
| `crates/plugmux-app/src/components/environments/InheritedServers.tsx` | No inheritance |
| `crates/plugmux-app/src/components/environments/PermissionsPanel.tsx` | Moved to Settings |
| `crates/plugmux-app/src/components/environments/EnvironmentServers.tsx` | Merged into simplified EnvironmentPage |

---

## Task 1: Rewrite Config Model (plugmux-core)

**Files:**
- Modify: `crates/plugmux-core/src/config.rs`
- Modify: `crates/plugmux-core/src/server.rs`
- Modify: `crates/plugmux-core/src/lib.rs`

**Context:** The current `config.rs` (294 lines) has `PlugmuxConfig` with `MainConfig`, `EnvironmentConfig` (with overrides, endpoint, full ServerConfig objects), `ServerOverride`, and `Permission`. Replace all of this with the new simplified model. The current `server.rs` (44 lines) has `ServerConfig`, `Transport`, and `Connectivity` which remain mostly unchanged.

- [ ] **Step 1: Write tests for new Config struct**

Create tests at the bottom of `config.rs` for the new config model:
- Test: load config.json with port, permissions, environments (string server IDs)
- Test: save and reload roundtrip
- Test: default environment bootstrap (missing default gets auto-created)
- Test: delete_environment("default") returns error
- Test: add_environment creates slug ID
- Test: config_path() returns `~/.config/plugmux/config.json` (new name)

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p plugmux-core config`
Expected: Compilation errors (new structs don't exist yet)

- [ ] **Step 3: Rewrite config.rs with new structs**

Replace the entire file. New structs:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub permissions: Permissions,
    pub environments: Vec<Environment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    #[serde(default = "default_approve")]
    pub enable_server: PermissionLevel,
    #[serde(default = "default_approve")]
    pub disable_server: PermissionLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    Allow,
    Approve,
    Disable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub servers: Vec<String>,  // server IDs — catalog or custom
}
```

Functions:
- `config_dir() -> PathBuf` — returns `~/.config/plugmux/`
- `config_path() -> PathBuf` — returns `~/.config/plugmux/config.json`
- `load(path) -> Result<Config, ConfigError>` — load, ensure default env exists
- `load_or_default(path) -> Config` — load or create with empty default env
- `save(path, config) -> Result<(), ConfigError>`
- `ensure_default(config: &mut Config)` — add default env if missing
- `add_environment(config, name) -> &mut Environment` — slug ID, no endpoint
- `find_environment(config, id) -> Option<&Environment>`
- `find_environment_mut(config, id) -> Option<&mut Environment>`
- `remove_environment(config, id) -> Result<(), ConfigError>` — error on "default"

- [ ] **Step 4: Update server.rs — remove `enabled`, add HealthStatus**

Remove the `enabled: bool` field from `ServerConfig`. In the new model, a server is either in an environment's server list or it isn't — there is no toggle. Also remove the `default_true()` serde default.

Add `HealthStatus` enum:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unavailable { reason: String },
}
```

Note: `HealthStatus` serializes as `{"status": "healthy"}` or `{"status": "degraded", "reason": "..."}`. The frontend receives this object format in events.

- [ ] **Step 5: Update lib.rs exports**

Add to `crates/plugmux-core/src/lib.rs`:
```rust
pub mod catalog;
pub mod custom_servers;
pub mod resolver;
pub mod migration;
```

(These modules will be empty stubs for now to avoid compilation errors — create empty files with `// TODO: implement in Task 5/6/7`)

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p plugmux-core config`
Expected: All new config tests pass

- [ ] **Step 7: Commit**

```bash
git add -A crates/plugmux-core/src/
git commit -m "refactor(core): rewrite config model — no inheritance, string server IDs, global permissions"
```

---

## Task 2: Simplify Environment Resolution (plugmux-core)

**Files:**
- Modify: `crates/plugmux-core/src/environment.rs`

**Context:** Current `environment.rs` (175 lines) has `resolve_environment()` that merges Main + Environment servers, applies overrides, filters disabled. Replace with simple ID-based lookup. The `ServerSource` and `ResolvedServer` types are still useful but simplified.

- [ ] **Step 1: Write tests for simplified environment resolution**

Tests:
- Test: resolve environment returns server IDs from the environment
- Test: resolve non-existent environment returns None
- Test: resolve default environment works like any other

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p plugmux-core environment`
Expected: Compilation errors (old types referenced)

- [ ] **Step 3: Rewrite environment.rs**

Remove `ServerSource`, `ResolvedServer`. New simple functions:
```rust
/// Get the list of server IDs for an environment
pub fn get_server_ids(config: &Config, env_id: &str) -> Option<Vec<String>> {
    config::find_environment(config, env_id)
        .map(|env| env.servers.clone())
}

/// Add a server ID to an environment (if not already present)
pub fn add_server(config: &mut Config, env_id: &str, server_id: &str) -> Result<(), ConfigError> {
    let env = config::find_environment_mut(config, env_id)
        .ok_or(ConfigError::EnvironmentNotFound(env_id.to_string()))?;
    if !env.servers.contains(&server_id.to_string()) {
        env.servers.push(server_id.to_string());
    }
    Ok(())
}

/// Remove a server ID from an environment
pub fn remove_server(config: &mut Config, env_id: &str, server_id: &str) -> Result<bool, ConfigError> {
    let env = config::find_environment_mut(config, env_id)
        .ok_or(ConfigError::EnvironmentNotFound(env_id.to_string()))?;
    let before = env.servers.len();
    env.servers.retain(|s| s != server_id);
    Ok(env.servers.len() < before)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p plugmux-core environment`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-core/src/environment.rs
git commit -m "refactor(core): simplify environment resolution — no inheritance, no overrides"
```

---

## Task 3: Update Gateway Tools for New Model (plugmux-core)

**Files:**
- Modify: `crates/plugmux-core/src/gateway/tools.rs`
- Modify: `crates/plugmux-core/src/gateway/router.rs`
- Modify: `crates/plugmux-core/src/pending_actions.rs`
- Modify: `crates/plugmux-core/src/manager.rs`
- Modify: `crates/plugmux-core/src/health.rs`

**Context:** `tools.rs` (370 lines) has `GatewayTools` with `enable_server`/`disable_server` that toggle overrides. Rewrite to add/remove server IDs from environments. Permission check becomes global (read `config.permissions`). `router.rs` (414 lines) dispatches JSON-RPC — update tool schemas. `manager.rs` (173 lines) needs HealthStatus. `health.rs` (54 lines) needs HealthStatus.

- [ ] **Step 1: Write tests for new permission check**

Tests in `tools.rs`:
- Test: global permission "allow" permits action
- Test: global permission "approve" returns ApprovalRequired
- Test: global permission "disable" returns error
- Test: enable_server adds server ID to environment
- Test: disable_server removes server ID from environment

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p plugmux-core gateway`
Expected: Failures due to changed types

- [ ] **Step 3: Update manager.rs — add HealthStatus tracking**

Change `ManagedServer.healthy: bool` to `health: HealthStatus`. Update `is_healthy()` to check `matches!(health, HealthStatus::Healthy)`. Add `get_health(id) -> Option<HealthStatus>` method. Update `set_health()` to take `HealthStatus`.

- [ ] **Step 4: Update health.rs — use HealthStatus**

Update `check_server_health()` to return `HealthStatus` instead of bool. If `client.health_check()` returns false, set `HealthStatus::Unavailable { reason: "Health check failed".into() }`.

- [ ] **Step 5: Rewrite tools.rs — GatewayTools**

`GatewayTools` fields change:
```rust
pub struct GatewayTools {
    pub config: Arc<RwLock<Config>>,        // was PlugmuxConfig
    pub manager: Arc<ServerManager>,
    pub pending: Mutex<PendingActions>,
}
```

Rewrite `check_permission()` to read `config.permissions` (global, not per-override). Uses `tokio::sync::RwLock` (async `.read().await`):
```rust
async fn check_permission(&self, env_id: &str, server_id: &str, action: &str) -> Result<(), ProxyError> {
    let config = self.config.read().await;
    let level = match action {
        "enable_server" => &config.permissions.enable_server,
        "disable_server" => &config.permissions.disable_server,
        _ => return Err(ProxyError::ToolCallFailed(format!("Unknown action: {action}"))),
    };
    match level {
        PermissionLevel::Allow => Ok(()),
        PermissionLevel::Approve => {
            // Check for existing pending action first
            let mut pending = self.pending.lock().await;
            let action_id = if let Some(existing) = pending.find_existing(env_id, server_id, action) {
                existing.to_string()
            } else {
                pending.add(env_id.to_string(), server_id.to_string(), action.to_string())
            };
            Err(ProxyError::ApprovalRequired {
                action_id,
                message: format!("{action} '{server_id}' requires approval. Please confirm with the user."),
            })
        }
        PermissionLevel::Disable => Err(ProxyError::ToolCallFailed("This action is not available".into())),
    }
}
```

Rewrite `enable_server()` to call `environment::add_server()` instead of toggling overrides.
Rewrite `disable_server()` to call `environment::remove_server()`.
Rewrite `list_servers()` to read environment's server ID list, check each against manager for health.

- [ ] **Step 6: Update router.rs — tool schemas**

Update the `tools/list` response: tool descriptions for `enable_server` say "Add a server to this environment" and `disable_server` say "Remove a server from this environment". Remove references to "toggle" in descriptions.

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test -p plugmux-core`
Expected: All pass

- [ ] **Step 8: Commit**

```bash
git add crates/plugmux-core/src/
git commit -m "refactor(core): gateway tools use global permissions, enable/disable as add/remove"
```

---

## Task 4: Catalog Registry (plugmux-core)

**Files:**
- Create: `crates/plugmux-core/src/catalog.rs`
- Create: `catalog/servers.json`
- Create: `catalog/presets.json`

**Context:** The catalog is a JSON file bundled at compile time via `include_str!`. `CatalogRegistry` parses it once and provides lookup/search methods.

- [ ] **Step 1: Create initial catalog/servers.json**

Create `catalog/servers.json` at the repo root with an initial curated set. Research current MCP servers available and include ~10-15 well-known ones to start. Use Claude Desktop's server list as a reference for which servers to include. Each entry needs: id, name, description, icon (filename), category, transport, command/args or url, connectivity.

Categories to use: `design`, `dev-tools`, `database`, `browser`, `ai`, `productivity`, `testing`, `infrastructure`, `marketing`, `content`

- [ ] **Step 2: Create catalog/presets.json**

Create with the structure defined in the spec. Include 1-2 placeholder presets (e.g., "web-dev"). Content will be refined later after MCP server landscape research.

```json
{
  "version": 1,
  "presets": [
    {
      "id": "web-dev",
      "name": "Web Development",
      "description": "Frontend and full-stack web development",
      "icon": "web-dev.svg",
      "servers": ["figma", "shadcn", "context7"]
    }
  ]
}
```

- [ ] **Step 3: Create placeholder SVG icons**

Create `catalog/icons/` directory with simple placeholder SVGs for each server in the catalog. Use a generic monochrome circle/square icon as placeholder — real icons will be sourced from official brand assets later.

- [ ] **Step 4: Write tests for CatalogRegistry**

Tests:
- Test: load catalog from JSON string
- Test: get_server by ID returns correct entry
- Test: get_server with unknown ID returns None
- Test: search by query matches name and description (case-insensitive)
- Test: search by category filters correctly
- Test: list_all returns all servers
- Test: list_presets returns presets
- Test: get_preset by ID

- [ ] **Step 5: Run tests to verify they fail**

Run: `cargo test -p plugmux-core catalog`
Expected: Module empty

- [ ] **Step 6: Implement catalog.rs**

```rust
use crate::server::{Transport, Connectivity, ServerConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogData {
    pub version: u32,
    pub servers: Vec<CatalogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub transport: Transport,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub connectivity: Connectivity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetData {
    pub version: u32,
    pub presets: Vec<Preset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub servers: Vec<String>,
}

pub struct CatalogRegistry {
    servers: HashMap<String, CatalogEntry>,
    all_servers: Vec<CatalogEntry>,
    presets: HashMap<String, Preset>,
    all_presets: Vec<Preset>,
}

impl CatalogRegistry {
    pub fn load(servers_json: &str, presets_json: &str) -> Result<Self, serde_json::Error> { ... }
    pub fn load_bundled() -> Self {
        let servers = include_str!("../../../catalog/servers.json");
        let presets = include_str!("../../../catalog/presets.json");
        Self::load(servers, presets).expect("Bundled catalog is valid JSON")
    }
    pub fn get_server(&self, id: &str) -> Option<&CatalogEntry> { ... }
    pub fn has_server(&self, id: &str) -> bool { ... }
    pub fn search(&self, query: &str, category: Option<&str>) -> Vec<&CatalogEntry> { ... }
    pub fn list_servers(&self) -> &[CatalogEntry] { ... }
    pub fn to_server_config(entry: &CatalogEntry) -> ServerConfig { ... }
    pub fn get_preset(&self, id: &str) -> Option<&Preset> { ... }
    pub fn list_presets(&self) -> &[Preset] { ... }
}
```

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test -p plugmux-core catalog`
Expected: All pass

- [ ] **Step 8: Commit**

```bash
git add catalog/ crates/plugmux-core/src/catalog.rs
git commit -m "feat(core): add CatalogRegistry with bundled servers and presets"
```

---

## Task 5: Custom Server Store (plugmux-core)

**Files:**
- Create: `crates/plugmux-core/src/custom_servers.rs`

**Context:** Custom servers are stored in `~/.config/plugmux/custom_servers.json`. The store provides CRUD operations and validates no ID collisions with catalog.

- [ ] **Step 1: Write tests for CustomServerStore**

Tests:
- Test: load from JSON string
- Test: load returns empty when file doesn't exist
- Test: add server and save
- Test: add server with catalog ID collision returns error
- Test: update existing server
- Test: remove server
- Test: get_server by ID

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p plugmux-core custom_servers`

- [ ] **Step 3: Implement custom_servers.rs**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomServersData {
    pub version: u32,
    pub servers: Vec<ServerConfig>,
}

pub struct CustomServerStore {
    servers: HashMap<String, ServerConfig>,
    path: PathBuf,
}

impl CustomServerStore {
    pub fn load(path: PathBuf) -> Result<Self, ConfigError> { ... }
    pub fn load_or_default(path: PathBuf) -> Self { ... }
    pub fn save(&self) -> Result<(), ConfigError> { ... }
    pub fn get(&self, id: &str) -> Option<&ServerConfig> { ... }
    pub fn has(&self, id: &str) -> bool { ... }
    pub fn list(&self) -> Vec<&ServerConfig> { ... }
    pub fn add(&mut self, config: ServerConfig, catalog: &CatalogRegistry) -> Result<(), ConfigError> {
        // Validate: ID not in catalog
        if catalog.has_server(&config.id) {
            return Err(ConfigError::IdCollision(config.id));
        }
        ...
    }
    pub fn update(&mut self, id: &str, config: ServerConfig) -> Result<(), ConfigError> { ... }
    pub fn remove(&mut self, id: &str) -> Result<bool, ConfigError> { ... }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p plugmux-core custom_servers`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-core/src/custom_servers.rs
git commit -m "feat(core): add CustomServerStore for user-defined MCP servers"
```

---

## Task 6: Server Resolver (plugmux-core)

**Files:**
- Create: `crates/plugmux-core/src/resolver.rs`

**Context:** The resolver ties catalog + custom_servers together. Given a server ID, it returns the full ServerConfig needed to start the MCP client.

- [ ] **Step 1: Write tests for ServerResolver**

Tests:
- Test: resolve catalog server ID returns catalog ServerConfig
- Test: resolve custom server ID returns custom ServerConfig
- Test: resolve unknown ID returns Unavailable health status
- Test: resolve all IDs for an environment (mix of catalog + custom)
- Test: catalog takes priority (lookup order)

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p plugmux-core resolver`

- [ ] **Step 3: Implement resolver.rs**

```rust
pub struct ServerResolver {
    catalog: Arc<CatalogRegistry>,
    custom: Arc<RwLock<CustomServerStore>>,
}

impl ServerResolver {
    pub fn new(catalog: Arc<CatalogRegistry>, custom: Arc<RwLock<CustomServerStore>>) -> Self { ... }

    pub fn resolve(&self, server_id: &str) -> ResolvedServer { ... }

    pub fn resolve_all(&self, server_ids: &[String]) -> Vec<ResolvedServer> { ... }
}

pub struct ResolvedServer {
    pub id: String,
    pub config: Option<ServerConfig>,  // None if unresolvable
    pub source: ServerSource,
    pub health: HealthStatus,          // Unavailable if unresolvable
}

pub enum ServerSource {
    Catalog,
    Custom,
    Unknown,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p plugmux-core resolver`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-core/src/resolver.rs
git commit -m "feat(core): add ServerResolver — catalog-first ID resolution"
```

---

## Task 7: Config Migration (plugmux-core)

**Files:**
- Create: `crates/plugmux-core/src/migration.rs`

**Context:** Users upgrading from Phase 2 have `~/.config/plugmux/plugmux.json`. We need to migrate to `config.json` + `custom_servers.json`. The old format has `main.servers` (Vec<ServerConfig>), `environments` with overrides and per-env permissions. This task comes after Task 4 (CatalogRegistry) because migration needs the catalog to match old server configs to catalog IDs.

- [ ] **Step 1: Write tests for migration**

Tests:
- Test: migrate old config with main servers → default environment with server IDs
- Test: migrate old config with environments → environments with string server IDs
- Test: servers not matching catalog → moved to custom_servers
- Test: old file renamed to .backup
- Test: no-op if config.json already exists

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p plugmux-core migration`
Expected: Compilation errors (module is stub)

- [ ] **Step 3: Implement migration.rs**

```rust
pub fn needs_migration() -> bool {
    let old = config_dir().join("plugmux.json");
    let new = config_path();
    old.exists() && !new.exists()
}

pub fn migrate(catalog: &CatalogRegistry) -> Result<(), ConfigError> {
    // 1. Load old plugmux.json (using serde_json::Value for flexibility)
    // 2. Extract main.servers → create "default" env with matched IDs
    // 3. Extract environments → create envs with matched IDs
    // 4. For each ServerConfig: if catalog has matching command/url → use catalog ID
    //    else → add to custom_servers list, use custom ID
    // 5. Write config.json and custom_servers.json
    // 6. Rename plugmux.json → plugmux.json.backup
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p plugmux-core migration`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-core/src/migration.rs
git commit -m "feat(core): add Phase 2 to Phase 3 config migration"
```

---

## Task 8: Update Tauri Backend (plugmux-app)

**Files:**
- Modify: `crates/plugmux-app/src-tauri/src/engine.rs`
- Modify: `crates/plugmux-app/src-tauri/src/commands.rs`
- Modify: `crates/plugmux-app/src-tauri/src/events.rs`
- Modify: `crates/plugmux-app/src-tauri/src/lib.rs`
- Modify: `crates/plugmux-app/src-tauri/src/watcher.rs`

**Context:** `engine.rs` (158 lines) creates Engine with old PlugmuxConfig. `commands.rs` (470 lines) has 20 Tauri command handlers using old model. Both need full rewrite for new config + resolver.

- [ ] **Step 1: Update engine.rs**

Engine now holds:
```rust
pub struct Engine {
    pub config: Arc<RwLock<Config>>,
    pub catalog: Arc<CatalogRegistry>,
    pub custom_servers: Arc<RwLock<CustomServerStore>>,
    pub resolver: Arc<ServerResolver>,
    pub manager: Arc<ServerManager>,
    pub status: Arc<RwLock<EngineStatus>>,
    pub port: Arc<RwLock<u16>>,
    pub shutdown_tx: Arc<RwLock<Option<tokio::sync::oneshot::Sender<()>>>>,
}
```

Update `new()`: load Config from config_path(), load CatalogRegistry::load_bundled(), load CustomServerStore, create ServerResolver. Check for migration on startup.

Update `start()`: for each environment, resolve server IDs via resolver, start servers via manager.

Update `reload_config()`: reload config.json and custom_servers.json.

- [ ] **Step 2: Rewrite commands.rs**

Remove old commands: `get_main_servers`, `add_main_server`, `remove_main_server`, `toggle_main_server`, `rename_server`, `toggle_env_override`, per-env `get_permissions`/`set_permission`.

Add new commands per spec Section 7:
- `get_config`, `get_port`, `set_port` (persists to config.json)
- `get_permissions`, `set_permission` (global)
- `list_environments`, `create_environment`, `delete_environment` (guard "default"), `rename_environment`
- `add_server_to_env`, `remove_server_from_env`
- `list_custom_servers`, `add_custom_server`, `update_custom_server`, `remove_custom_server`
- `list_catalog_servers`, `search_catalog`, `get_catalog_entry`
- `list_presets`, `create_env_from_preset`
- `get_server_health`
- `migrate_config`

Each command accesses engine state, performs the operation, saves config if changed, emits appropriate event.

- [ ] **Step 3: Update events.rs**

Update payloads:
- `ServerChangedPayload`: `env_id` is now always required (not optional)
- Remove `ServerToggledPayload` (no more toggle concept)
- Update `ServerHealthPayload` to use string status: "healthy" | "degraded" | "unavailable"

- [ ] **Step 4: Update lib.rs — register new commands**

Update the `invoke_handler` in `run()` to register new command set and remove old ones.

- [ ] **Step 5: Update watcher.rs — watch both files**

Watch `~/.config/plugmux/` directory. On change to either `config.json` or `custom_servers.json`, reload the appropriate store and emit `config_reloaded` event.

- [ ] **Step 6: Update tray.rs — aggregate health status**

Update tray icon health dot to reflect the worst health status across all environments' servers. Listen to `server_health_changed` events. Green if all healthy, yellow if any degraded, red if any unavailable, grey if engine stopped.

- [ ] **Step 7: Build and verify compilation**

Run: `cd crates/plugmux-app && cargo build`
Expected: Compiles without errors (UI will be broken until Task 9)

- [ ] **Step 8: Commit**

```bash
git add crates/plugmux-app/src-tauri/src/
git commit -m "refactor(app): rewrite Tauri backend for Phase 3 config model + catalog"
```

---

## Task 9: Update TypeScript Commands & Hooks

**Files:**
- Modify: `crates/plugmux-app/src/lib/commands.ts`
- Modify: `crates/plugmux-app/src/hooks/useConfig.ts`
- Create: `crates/plugmux-app/src/hooks/useCatalog.ts`
- Create: `crates/plugmux-app/src/hooks/useCustomServers.ts`

**Context:** `commands.ts` (69 lines) has TypeScript types and invoke wrappers for old model. `useConfig.ts` (67 lines) fetches config and listens to events.

- [ ] **Step 1: Rewrite commands.ts**

New types:
```typescript
export interface Config {
  port: number;
  permissions: Permissions;
  environments: Environment[];
}

export interface Permissions {
  enable_server: "allow" | "approve" | "disable";
  disable_server: "allow" | "approve" | "disable";
}

export interface Environment {
  id: string;
  name: string;
  servers: string[];
}

export interface ServerConfig {
  id: string;
  name: string;
  transport: "stdio" | "http";
  command?: string;
  args?: string[];
  url?: string;
  connectivity: "local" | "online";
  description?: string;
}

export interface CatalogEntry extends ServerConfig {
  icon: string;
  category: string;
}

export interface Preset {
  id: string;
  name: string;
  description: string;
  icon: string;
  servers: string[];
}

export type HealthStatus = "healthy" | "degraded" | "unavailable";
```

New command wrappers matching Tauri commands from Task 8.

- [ ] **Step 2: Update useConfig.ts**

Update to use new `Config` type. Listen to same events. Expose: config, loading, reload, and wrapped functions for environment CRUD and server add/remove.

- [ ] **Step 3: Create useCatalog.ts**

```typescript
export function useCatalog() {
  const [servers, setServers] = useState<CatalogEntry[]>([]);
  const [presets, setPresets] = useState<Preset[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([listCatalogServers(), listPresets()])
      .then(([s, p]) => { setServers(s); setPresets(p); })
      .finally(() => setLoading(false));
  }, []);

  const search = async (query: string, category?: string) => { ... };

  return { servers, presets, loading, search };
}
```

- [ ] **Step 4: Create useCustomServers.ts**

```typescript
export function useCustomServers() {
  const [servers, setServers] = useState<ServerConfig[]>([]);
  // Load, add, update, remove wrappers
  return { servers, loading, addServer, updateServer, removeServer, reload };
}
```

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-app/src/lib/ crates/plugmux-app/src/hooks/
git commit -m "refactor(app): update TypeScript commands and hooks for Phase 3"
```

---

## Task 10: Update Sidebar & Navigation

**Files:**
- Modify: `crates/plugmux-app/src/components/layout/Sidebar.tsx`
- Modify: `crates/plugmux-app/src/App.tsx`
- Delete: `crates/plugmux-app/src/pages/MainPage.tsx`

**Context:** `Sidebar.tsx` (150 lines) has "Main" as first nav item and environment list. `App.tsx` (54 lines) routes to MainPage for "main". MainPage (64 lines) is deleted — Default is just an environment.

- [ ] **Step 1: Update Sidebar.tsx**

- Replace "Main" nav item with "Default" that navigates to `env:default`
- Pin Default at top of environment list (don't show it in the dynamic env list again)
- Remove the separate "Main" icon/button
- Catalog and Presets nav items stay the same
- Add aggregate health dot per environment badge: compute worst health status across all servers in the environment (green if all healthy, yellow if any degraded, red if any unavailable). Listen to `server_health_changed` events to update.

- [ ] **Step 2: Update App.tsx**

- Remove `"main"` case from page routing
- Default active page should be `"env:default"` instead of `"main"`
- Remove MainPage import

- [ ] **Step 3: Delete MainPage.tsx**

Remove `crates/plugmux-app/src/pages/MainPage.tsx`

- [ ] **Step 4: Verify app compiles**

Run: `cd crates/plugmux-app && npm run build`
Expected: TypeScript compiles (runtime may have issues until environment page is updated)

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-app/src/
git rm crates/plugmux-app/src/pages/MainPage.tsx
git commit -m "refactor(app): Main → Default, remove MainPage"
```

---

## Task 11: Simplified Environment Page + Catalog Page + Presets Page

**Files:**
- Modify: `crates/plugmux-app/src/pages/EnvironmentPage.tsx`
- Modify: `crates/plugmux-app/src/pages/CatalogPage.tsx`
- Modify: `crates/plugmux-app/src/pages/PresetsPage.tsx`
- Create: `crates/plugmux-app/src/components/catalog/CatalogCard.tsx`
- Create: `crates/plugmux-app/src/components/catalog/CatalogDetail.tsx`
- Create: `crates/plugmux-app/src/components/catalog/CategoryFilter.tsx`
- Modify: `crates/plugmux-app/src/components/servers/ServerCard.tsx`
- Delete: `crates/plugmux-app/src/components/environments/InheritedServers.tsx`
- Delete: `crates/plugmux-app/src/components/environments/EnvironmentServers.tsx`
- Delete: `crates/plugmux-app/src/components/environments/PermissionsPanel.tsx`

**Context:** This is the largest UI task. EnvironmentPage (133 lines) currently has InheritedServers + EnvironmentServers + PermissionsPanel sections. Replace with flat server list. CatalogPage (10 lines) and PresetsPage (10 lines) are placeholders.

- [ ] **Step 1: Update ServerCard.tsx**

Add health dot (green/yellow/red circle) and icon support. The card now shows:
- Server icon (SVG from catalog, or generic icon for custom)
- Server name
- Description
- Health dot
- Remove button (no more toggle switch — server is either present or not)

Use the `useCatalog` hook or pass resolved server info as props.

- [ ] **Step 2: Delete old environment components**

Remove:
- `InheritedServers.tsx`
- `EnvironmentServers.tsx`
- `PermissionsPanel.tsx`

- [ ] **Step 3: Update AddServerDialog.tsx for custom servers only**

The current `AddServerDialog.tsx` (178 lines) creates a full `ServerConfig` with transport, command, url, connectivity, and an `enabled` field. Repurpose it for custom server creation only:
- Remove the `enabled` toggle (no longer exists on ServerConfig)
- This dialog is now only used from the Settings page (Custom Servers section) and from Environment page's "Add Custom Server" button
- The "Add Server" flow (catalog servers) navigates to the Catalog page instead

- [ ] **Step 4: Update CreateEnvironmentDialog.tsx for preset support**

The current `CreateEnvironmentDialog.tsx` (84 lines) only asks for a name. Add an optional preset selector:
- Show a dropdown/radio of available presets (from `useCatalog().presets`) + "Empty" option
- If a preset is selected, call `createEnvFromPreset(presetId, name)` instead of `createEnvironment(name)`

- [ ] **Step 5: Rewrite EnvironmentPage.tsx**

Simplified layout:
- Header: env name (editable, unless "default") + computed URL (`http://localhost:{port}/env/{id}`) + copy button
- Flat server list using ServerCard for each server ID in `env.servers`
- Resolve each server ID via catalog/custom to get name, description, icon
- "Add Server" button → navigate to Catalog page
- "Add Custom Server" button → open AddServerDialog (modified for custom servers)
- "Delete Environment" button (hidden for "default")

- [ ] **Step 6: Create CatalogCard.tsx**

Card component for catalog grid:
```tsx
interface CatalogCardProps {
  entry: CatalogEntry;
  installedIn: string[];  // environment names where installed
  onAdd: (envId: string) => void;
  onClick: () => void;
}
```

Shows: icon, name, description (2 lines), category badge, installed indicator, "Add" dropdown.

- [ ] **Step 7: Create CategoryFilter.tsx**

Row of filter pills:
```tsx
interface CategoryFilterProps {
  categories: string[];
  selected: string | null;  // null = "All"
  onSelect: (category: string | null) => void;
}
```

- [ ] **Step 8: Create CatalogDetail.tsx**

Expanded view when clicking a catalog card:
```tsx
interface CatalogDetailProps {
  entry: CatalogEntry;
  installedIn: string[];
  environments: Environment[];
  onAdd: (envId: string) => void;
  onClose: () => void;
}
```

Shows: full description, config preview, installed environments list, "Add to..." button.

- [ ] **Step 9: Implement CatalogPage.tsx**

Full implementation:
- Search bar (controlled input, debounced)
- CategoryFilter pills
- Grid of CatalogCards
- CatalogDetail modal/panel on card click
- Use `useCatalog` hook for data, `useConfig` for environment list
- Search: call `searchCatalog(query, category)` command

- [ ] **Step 10: Implement PresetsPage.tsx**

Minimal implementation:
- Grid of preset cards from `useCatalog().presets`
- Each card: icon, name, description, server list
- "Create Environment" button → dialog for name → `createEnvFromPreset(presetId, name)`

- [ ] **Step 11: Verify UI compiles and renders**

Run: `cd crates/plugmux-app && npm run build`
Expected: Compiles without errors

- [ ] **Step 12: Commit**

```bash
git add crates/plugmux-app/src/
git rm crates/plugmux-app/src/components/environments/InheritedServers.tsx
git rm crates/plugmux-app/src/components/environments/EnvironmentServers.tsx
git rm crates/plugmux-app/src/components/environments/PermissionsPanel.tsx
git commit -m "feat(app): implement Catalog page, Presets page, simplified Environment page"
```

---

## Task 12: Update Settings Page

**Files:**
- Modify: `crates/plugmux-app/src/pages/SettingsPage.tsx`
- Create: `crates/plugmux-app/src/components/settings/PermissionsSection.tsx`
- Create: `crates/plugmux-app/src/components/settings/CustomServersSection.tsx`

**Context:** SettingsPage (140 lines) currently has Gateway, Startup, Appearance, About sections. Add Permissions (global) and Custom Servers sections.

- [ ] **Step 1: Create PermissionsSection.tsx**

Global permissions UI:
- Table with action name + dropdown (allow/approve/disable)
- Actions: `enable_server`, `disable_server`
- On change: call `setPermission(action, level)` command

- [ ] **Step 2: Create CustomServersSection.tsx**

Custom server management:
- List of custom servers from `useCustomServers()`
- Each row: name, transport, command/url, edit button, delete button
- "Add Custom Server" button → AddServerDialog (repurposed)
- Edit opens dialog with pre-filled values

- [ ] **Step 3: Update SettingsPage.tsx**

Add new sections between Gateway and Startup:
- Permissions section
- Custom Servers section

Update Gateway section: port now persists via `setPort()`.

- [ ] **Step 4: Verify UI compiles**

Run: `cd crates/plugmux-app && npm run build`
Expected: Compiles

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-app/src/pages/SettingsPage.tsx crates/plugmux-app/src/components/settings/
git commit -m "feat(app): add Permissions and Custom Servers sections to Settings"
```

---

## Task 13: Update CLI

**Files:**
- Modify: `crates/plugmux-cli/src/main.rs`
- Modify: `crates/plugmux-cli/src/commands/mod.rs`
- Modify: `crates/plugmux-cli/src/commands/start.rs`
- Modify: `crates/plugmux-cli/src/commands/env.rs`
- Modify: `crates/plugmux-cli/src/commands/server.rs`
- Modify: `crates/plugmux-cli/src/commands/config.rs`
- Create: `crates/plugmux-cli/src/commands/custom.rs`
- Create: `crates/plugmux-cli/src/commands/catalog.rs`

**Context:** CLI (main.rs 69 lines) defines clap commands. All command handlers need updating for new config model.

- [ ] **Step 1: Update main.rs — new CLI structure**

New top-level commands: Start, Stop, Status, Env, Server, Custom, Catalog, Config.

Server commands now always require `--env <env-id>`.
New Custom subcommands: add, edit, remove, list.
New Catalog subcommands: list, search, browse.
Config gets new `migrate` subcommand.

- [ ] **Step 2: Update commands/mod.rs**

Add `pub mod catalog;` and `pub mod custom;`

- [ ] **Step 3: Update start.rs**

Use new Config + CatalogRegistry + CustomServerStore + ServerResolver. On start:
1. Check for migration, run if needed
2. Load config, catalog, custom servers
3. Create resolver
4. For each environment, resolve server IDs, start servers
5. Print banner with environment URLs

- [ ] **Step 4: Update env.rs**

Use new Config. Add `--preset` flag for create:
- Load catalog, find preset, create environment with preset's server list
- `delete` guards against "default"

- [ ] **Step 5: Rewrite server.rs**

Server commands are always env-scoped:
- `add <server-id> --env <env-id>` — add string ID to environment
- `remove <server-id> --env <env-id>` — remove from environment
- `list --env <env-id>` — list servers in environment, show resolved names from catalog/custom

- [ ] **Step 6: Create custom.rs**

Custom server commands:
- `add --id --name --transport --command/--url` — add to custom_servers.json, validate no catalog collision
- `edit <id>` — update fields
- `remove <id>` — remove
- `list` — list all custom servers

- [ ] **Step 7: Create catalog.rs**

Catalog commands (read-only):
- `list` — list all catalog servers
- `search <query>` — search by name/description
- `browse --category <cat>` — filter by category

- [ ] **Step 8: Update config.rs**

Add `migrate` subcommand that calls `migration::migrate()`.
Add `show` subcommand that pretty-prints the current `config.json` content. The existing `Export` and `Import` commands remain.

- [ ] **Step 9: Update stop.rs and status.rs**

Both currently use a hardcoded or CLI-flag port. Update to read port from `config.json` (via `Config::load_or_default()`) as the default, with `--port` flag as override.

- [ ] **Step 10: Run CLI help to verify structure**

Run: `cargo run -p plugmux-cli -- --help`
Expected: Shows new command structure

- [ ] **Step 11: Commit**

```bash
git add crates/plugmux-cli/src/
git commit -m "refactor(cli): update CLI for Phase 3 — env-scoped servers, catalog, custom servers"
```

---

## Task 14: Community Infrastructure

**Files:**
- Create: `catalog/CONTRIBUTING.md`
- Create: `.github/workflows/catalog-validate.yml` (if GitHub Actions desired)

**Context:** Community members submit servers via GitHub PRs. We need a contribution guide and optionally CI validation.

- [ ] **Step 1: Write CONTRIBUTING.md**

Cover:
- How to add a server (step by step)
- Required fields in servers.json entry
- Icon guidelines (monochrome SVG, 24x24 viewbox, max 2KB)
- Available categories
- Full example entry
- PR title format: `catalog: add <server-name>`
- What to expect during review

- [ ] **Step 2: Create CI validation script (optional)**

Create a simple script or GitHub Action that:
- Validates servers.json is valid JSON
- Validates all required fields exist
- Checks no duplicate IDs
- Checks referenced icon files exist
- Checks SVG files are valid

This can be a shell script or a small Rust binary.

- [ ] **Step 3: Commit**

```bash
git add catalog/CONTRIBUTING.md
git commit -m "docs: add catalog contribution guide"
```

---

## Task 15: Update Integration Test

**Files:**
- Modify: `crates/plugmux-cli/tests/gateway_integration.rs`

**Context:** The existing integration test (262 lines) creates a config with MainConfig + environments, starts a mock server, and tests JSON-RPC through the gateway. It needs to be updated for the new config model.

- [ ] **Step 1: Update test config setup**

Replace `PlugmuxConfig` with new `Config`. Create environment with server ID strings. The mock server needs to be registered as a custom server (since it won't be in the catalog).

- [ ] **Step 2: Update assertions**

- `tools/list` returns 6 tools: list_servers, get_tools, execute, enable_server, disable_server, confirm_action. **Note:** The current test incorrectly asserts 5 tools (missing confirm_action from Phase 2). Fix this to assert 6.
- Tool descriptions updated: enable_server → "Add a server to this environment", disable_server → "Remove a server from this environment"
- `list_servers` response format may change (server IDs instead of full configs)
- `enable_server`/`disable_server` now add/remove from environment

- [ ] **Step 3: Add catalog resolution test**

Add a test that creates a config referencing a catalog server ID and verifies it resolves correctly (may need to mock the catalog or use a test-only catalog).

- [ ] **Step 4: Run full test suite**

Run: `cargo test --workspace`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-cli/tests/
git commit -m "test: update integration test for Phase 3 config model"
```

---

## Task 16: Final Verification

**Files:** None (verification only)

- [ ] **Step 1: Run all Rust tests**

Run: `cargo test --workspace`
Expected: All pass

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --workspace --all-targets`
Expected: No warnings

- [ ] **Step 3: Build release binaries**

Run: `cargo build --release`
Expected: Builds successfully

- [ ] **Step 4: Build Tauri app**

Run: `cd crates/plugmux-app && npm run build && cargo build -p plugmux-app`
Expected: Both frontend and backend build

- [ ] **Step 5: Manual smoke test**

1. Delete `~/.config/plugmux/` (fresh start)
2. Run `cargo run -p plugmux-cli -- env list` → should show "default" environment
3. Run `cargo run -p plugmux-cli -- catalog list` → should show catalog servers
4. Run `cargo run -p plugmux-cli -- server add figma --env default` → should add figma
5. Run `cargo run -p plugmux-cli -- server list --env default` → should show figma
6. Run `cargo run -p plugmux-cli -- custom add --id test --name "Test" --transport stdio --command echo` → should add custom server
7. Run `cargo run -p plugmux-cli -- custom list` → should show test server

- [ ] **Step 6: Final commit if any remaining changes**

```bash
git status
# If any uncommitted changes, commit them
```
