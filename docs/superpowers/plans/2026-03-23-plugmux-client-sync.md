# plugmux Client-Side Sync & Remote Catalog — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect the plugmux desktop app to the cloud backend — remote catalog with icon caching, GitHub OAuth login, and change-log-based config sync across devices.

**Architecture:** New `plugmux-types` crate for shared types. New `api_client` and `sync` modules in `plugmux-core`. New auth flow and icon loader in the Tauri app. The app prefers remote catalog when online, falls back to bundled catalog when offline. Sync uses change-log merge via redb offline queue.

**Tech Stack:** Rust, `reqwest` (HTTP client), `redb` (local storage), Tauri v2, React 19, TypeScript

**Spec:** `docs/superpowers/specs/2026-03-23-plugmux-cloud-backend-design.md`

**Prerequisite:** Plan 1 (plugmux-api) must be deployed and accessible.

---

## File Map

```
crates/
├── plugmux-types/                     [NEW CRATE]
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                     — shared types: CatalogEntry, Transport, Connectivity, SyncAction, etc.
│
├── plugmux-core/
│   └── src/
│       ├── api_client.rs              [NEW] — HTTP client for plugmux-api
│       ├── sync/                      [NEW]
│       │   ├── mod.rs                 — re-exports
│       │   ├── queue.rs               — offline change queue in redb
│       │   ├── merge.rs               — apply remote changes to local config
│       │   └── client.rs              — sync push/pull/snapshot orchestration
│       ├── catalog.rs                 [MODIFY] — use plugmux-types, add remote catalog support
│       ├── config.rs                  [MODIFY] — emit sync changes on mutations
│       ├── db/
│       │   ├── mod.rs                 [MODIFY] — add new redb tables
│       │   └── sync_store.rs          [NEW] — device_id, last_sync_id, cached catalog, sync queue
│       └── server.rs                  [MODIFY] — re-export from plugmux-types
│
├── plugmux-app/
│   ├── src-tauri/src/
│   │   ├── commands.rs                [MODIFY] — add auth + sync commands
│   │   └── engine.rs                  [MODIFY] — init api_client, load remote catalog
│   └── src/
│       ├── lib/
│       │   └── commands.ts            [MODIFY] — add auth + sync invoke types
│       ├── hooks/
│       │   ├── useCatalog.ts          [MODIFY] — prefer remote catalog, fallback to bundled
│       │   ├── useAuth.ts             [NEW] — auth state, login/logout
│       │   └── useSync.ts             [NEW] — sync status, trigger sync
│       ├── components/
│       │   ├── catalog/
│       │   │   └── CatalogCard.tsx    [MODIFY] — use real icons from cache
│       │   └── settings/
│       │       └── AccountSection.tsx [NEW] — login/logout, device name, sync status
│       └── pages/
│           └── SettingsPage.tsx        [MODIFY] — add account section
```

---

### Task 1: Create plugmux-types crate

**Files:**
- Create: `crates/plugmux-types/Cargo.toml`
- Create: `crates/plugmux-types/src/lib.rs`
- Modify: `Cargo.toml` (workspace root) — add member

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "plugmux-types"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 2: Create src/lib.rs**

Extract shared types from `plugmux-core`. These must be lightweight — no tokio, axum, or heavy deps.

```rust
use serde::{Deserialize, Serialize};

// === Transport & Connectivity ===

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

// === Catalog ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_hash: Option<String>,
    /// Legacy single category field for bundled catalog compat
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Preferred multi-category field
    #[serde(default)]
    pub categories: Vec<String>,
    pub transport: Transport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default = "default_connectivity")]
    pub connectivity: Connectivity,
    #[serde(default)]
    pub official: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_score: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

fn default_connectivity() -> Connectivity {
    Connectivity::Online
}

impl CatalogEntry {
    /// Returns categories, falling back to single category field
    pub fn all_categories(&self) -> Vec<String> {
        if !self.categories.is_empty() {
            self.categories.clone()
        } else if let Some(ref cat) = self.category {
            vec![cat.clone()]
        } else {
            vec![]
        }
    }
}

// === Sync ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncAction {
    AddServerToEnv,
    RemoveServerFromEnv,
    CreateEnv,
    DeleteEnv,
    UpdateEnv,
    AddFavorite,
    RemoveFavorite,
    AddCustomServer,
    UpdateCustomServer,
    RemoveCustomServer,
}

impl SyncAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncAction::AddServerToEnv => "add_server_to_env",
            SyncAction::RemoveServerFromEnv => "remove_server_from_env",
            SyncAction::CreateEnv => "create_env",
            SyncAction::DeleteEnv => "delete_env",
            SyncAction::UpdateEnv => "update_env",
            SyncAction::AddFavorite => "add_favorite",
            SyncAction::RemoveFavorite => "remove_favorite",
            SyncAction::AddCustomServer => "add_custom_server",
            SyncAction::UpdateCustomServer => "update_custom_server",
            SyncAction::RemoveCustomServer => "remove_custom_server",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub device_id: String,
    pub timestamp: String,
    pub action: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSnapshot {
    pub environments: Vec<SnapshotEnvironment>,
    pub favorites: Vec<String>,
    pub custom_servers: Vec<serde_json::Value>,
    pub last_change_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotEnvironment {
    pub id: String,
    pub name: String,
    pub servers: Vec<String>,
}

// === Auth ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: AuthUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
}
```

- [ ] **Step 3: Add to workspace**

In root `Cargo.toml`, add to members:

```toml
members = [
  "crates/plugmux-core",
  "crates/plugmux-types",
  "crates/plugmux-cli",
  "crates/plugmux-app/src-tauri"
]
```

- [ ] **Step 4: Build to verify**

```bash
cargo build -p plugmux-types
```

Expected: Compiles with no errors.

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-types/ Cargo.toml
git commit -m "feat: add plugmux-types crate with shared types"
```

---

### Task 2: Make plugmux-core depend on plugmux-types

**Files:**
- Modify: `crates/plugmux-core/Cargo.toml`
- Modify: `crates/plugmux-core/src/server.rs` — re-export from types
- Modify: `crates/plugmux-core/src/catalog.rs` — use CatalogEntry from types

- [ ] **Step 1: Add dependency**

In `crates/plugmux-core/Cargo.toml`:

```toml
[dependencies]
plugmux-types = { path = "../plugmux-types" }
```

- [ ] **Step 2: Update server.rs**

Replace the `Transport`, `Connectivity` enums with re-exports:

```rust
pub use plugmux_types::{Transport, Connectivity};

// Keep ServerConfig and HealthStatus here (they have plugmux-core-specific logic)
```

Remove the duplicate `Transport` and `Connectivity` enum definitions from `server.rs`. Keep `ServerConfig` and `HealthStatus` in place since they depend on core logic.

- [ ] **Step 3: Update catalog.rs**

Replace the local `CatalogEntry` struct with the one from `plugmux-types`. The existing `CatalogEntry` has `icon: String` and `category: String` (singular). The new type from `plugmux-types` has `icon_key`, `icon_hash`, `categories` (plural), plus backward-compat `category` field.

Update `CatalogRegistry::load()` to handle both old bundled JSON format (`icon`, `category` singular) and new API format (`icon_key`, `categories` plural).

```rust
use plugmux_types::CatalogEntry;

// In load(), the bundled servers.json still uses old format.
// CatalogEntry handles both via serde defaults and Option fields.
// The all_categories() method provides unified access.
```

Update `to_server_config()` to work with the new `CatalogEntry` fields.

- [ ] **Step 4: Build and fix any compilation errors**

```bash
cargo build -p plugmux-core
```

Fix any type mismatches throughout the codebase (commands.rs in Tauri app, resolver.rs, etc.).

- [ ] **Step 5: Build the full workspace**

```bash
cargo build
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor: use plugmux-types for shared types in plugmux-core"
```

---

### Task 3: Add redb tables for sync state

**Files:**
- Create: `crates/plugmux-core/src/db/sync_store.rs`
- Modify: `crates/plugmux-core/src/db/mod.rs`

- [ ] **Step 1: Create sync_store.rs**

```rust
use redb::{Database, ReadableTable, TableDefinition};
use std::sync::Arc;
use crate::db::Db;

// Table definitions
pub const DEVICE_TABLE: TableDefinition<&str, &str> = TableDefinition::new("device");
pub const SYNC_META_TABLE: TableDefinition<&str, &str> = TableDefinition::new("sync_meta");
pub const SYNC_QUEUE_TABLE: TableDefinition<u64, &str> = TableDefinition::new("sync_queue");
pub const CATALOG_CACHE_TABLE: TableDefinition<&str, &str> = TableDefinition::new("catalog_cache");

/// Initialize sync-related tables
pub fn init_tables(db: &Database) -> Result<(), Box<redb::Error>> {
    let write_txn = db.begin_write()?;
    {
        let _ = write_txn.open_table(DEVICE_TABLE)?;
        let _ = write_txn.open_table(SYNC_META_TABLE)?;
        let _ = write_txn.open_table(SYNC_QUEUE_TABLE)?;
        let _ = write_txn.open_table(CATALOG_CACHE_TABLE)?;
    }
    write_txn.commit()?;
    Ok(())
}

/// Get or create device ID
pub fn get_or_create_device_id(db: &Arc<Db>) -> Result<String, Box<redb::Error>> {
    // Try to read existing
    let read_txn = db.inner.begin_read()?;
    let table = read_txn.open_table(DEVICE_TABLE)?;
    if let Some(val) = table.get("device_id")? {
        return Ok(val.value().to_string());
    }
    drop(table);
    drop(read_txn);

    // Generate new
    let id = uuid::Uuid::new_v4().to_string();
    let write_txn = db.inner.begin_write()?;
    {
        let mut table = write_txn.open_table(DEVICE_TABLE)?;
        table.insert("device_id", id.as_str())?;
    }
    write_txn.commit()?;
    Ok(id)
}

/// Get device name (defaults to hostname)
pub fn get_device_name(db: &Arc<Db>) -> Result<String, Box<redb::Error>> {
    let read_txn = db.inner.begin_read()?;
    let table = read_txn.open_table(DEVICE_TABLE)?;
    if let Some(val) = table.get("device_name")? {
        return Ok(val.value().to_string());
    }
    drop(table);
    drop(read_txn);

    // Default to hostname
    let name = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown Device".to_string());
    Ok(name)
}

/// Store auth token
pub fn set_auth_token(db: &Arc<Db>, token: &str) -> Result<(), Box<redb::Error>> {
    let write_txn = db.inner.begin_write()?;
    {
        let mut table = write_txn.open_table(SYNC_META_TABLE)?;
        table.insert("auth_token", token)?;
    }
    write_txn.commit()?;
    Ok(())
}

pub fn get_auth_token(db: &Arc<Db>) -> Result<Option<String>, Box<redb::Error>> {
    let read_txn = db.inner.begin_read()?;
    let table = read_txn.open_table(SYNC_META_TABLE)?;
    Ok(table.get("auth_token")?.map(|v| v.value().to_string()))
}

pub fn clear_auth_token(db: &Arc<Db>) -> Result<(), Box<redb::Error>> {
    let write_txn = db.inner.begin_write()?;
    {
        let mut table = write_txn.open_table(SYNC_META_TABLE)?;
        table.remove("auth_token")?;
    }
    write_txn.commit()?;
    Ok(())
}

/// Last sync ID for pull cursor
pub fn get_last_sync_id(db: &Arc<Db>) -> Result<i64, Box<redb::Error>> {
    let read_txn = db.inner.begin_read()?;
    let table = read_txn.open_table(SYNC_META_TABLE)?;
    Ok(table.get("last_sync_id")?
        .and_then(|v| v.value().parse::<i64>().ok())
        .unwrap_or(0))
}

pub fn set_last_sync_id(db: &Arc<Db>, id: i64) -> Result<(), Box<redb::Error>> {
    let write_txn = db.inner.begin_write()?;
    {
        let mut table = write_txn.open_table(SYNC_META_TABLE)?;
        table.insert("last_sync_id", id.to_string().as_str())?;
    }
    write_txn.commit()?;
    Ok(())
}

/// Offline sync queue — append changes when offline, drain on sync
pub fn queue_change(db: &Arc<Db>, change_json: &str) -> Result<(), Box<redb::Error>> {
    let write_txn = db.inner.begin_write()?;
    {
        let mut table = write_txn.open_table(SYNC_QUEUE_TABLE)?;
        // Use current timestamp as key for ordering
        let key = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        table.insert(key, change_json)?;
    }
    write_txn.commit()?;
    Ok(())
}

pub fn drain_queue(db: &Arc<Db>) -> Result<Vec<String>, Box<redb::Error>> {
    let mut entries = Vec::new();

    let read_txn = db.inner.begin_read()?;
    let table = read_txn.open_table(SYNC_QUEUE_TABLE)?;
    let iter = table.iter()?;
    for item in iter {
        let (key, val) = item?;
        entries.push((key.value(), val.value().to_string()));
    }
    drop(table);
    drop(read_txn);

    if !entries.is_empty() {
        let write_txn = db.inner.begin_write()?;
        {
            let mut table = write_txn.open_table(SYNC_QUEUE_TABLE)?;
            for (key, _) in &entries {
                table.remove(*key)?;
            }
        }
        write_txn.commit()?;
    }

    Ok(entries.into_iter().map(|(_, v)| v).collect())
}

/// Cached catalog from remote API
pub fn cache_catalog(db: &Arc<Db>, catalog_json: &str) -> Result<(), Box<redb::Error>> {
    let write_txn = db.inner.begin_write()?;
    {
        let mut table = write_txn.open_table(CATALOG_CACHE_TABLE)?;
        table.insert("catalog", catalog_json)?;
        table.insert("cached_at", chrono::Utc::now().to_rfc3339().as_str())?;
    }
    write_txn.commit()?;
    Ok(())
}

pub fn get_cached_catalog(db: &Arc<Db>) -> Result<Option<String>, Box<redb::Error>> {
    let read_txn = db.inner.begin_read()?;
    let table = read_txn.open_table(CATALOG_CACHE_TABLE)?;
    Ok(table.get("catalog")?.map(|v| v.value().to_string()))
}
```

- [ ] **Step 2: Update db/mod.rs**

Add `pub mod sync_store;` and call `sync_store::init_tables()` in `Db::open()`:

```rust
pub mod logs;
pub mod sync_store;

// In Db::open():
sync_store::init_tables(&db)?;
```

- [ ] **Step 3: Add `hostname` dependency to plugmux-core/Cargo.toml**

```toml
hostname = "0.4"
```

- [ ] **Step 4: Build**

```bash
cargo build -p plugmux-core
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add redb tables for device identity, sync queue, and catalog cache"
```

---

### Task 4: API client module

**Files:**
- Create: `crates/plugmux-core/src/api_client.rs`
- Modify: `crates/plugmux-core/Cargo.toml` — add `reqwest`

- [ ] **Step 1: Add reqwest dependency**

```toml
reqwest = { version = "0.12", features = ["json"] }
```

- [ ] **Step 2: Create api_client.rs**

```rust
use plugmux_types::*;
use reqwest::Client;
use serde::Deserialize;

const DEFAULT_API_URL: &str = "https://api.plugmux.com";

pub struct ApiClient {
    client: Client,
    base_url: String,
    auth_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CatalogListResponse {
    pub servers: Vec<CatalogEntry>,
    pub total: u32,
    pub limit: u32,
    pub cursor: u32,
}

#[derive(Debug, Deserialize)]
pub struct PullResponse {
    pub changes: Vec<ChangeEntry>,
    pub has_more: bool,
}

impl ApiClient {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.unwrap_or(DEFAULT_API_URL).trim_end_matches('/').to_string(),
            auth_token: None,
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.auth_token = Some(token);
    }

    pub fn clear_token(&mut self) {
        self.auth_token = None;
    }

    pub fn is_authenticated(&self) -> bool {
        self.auth_token.is_some()
    }

    fn auth_header(&self) -> Option<String> {
        self.auth_token.as_ref().map(|t| format!("Bearer {}", t))
    }

    // === Public endpoints ===

    pub async fn list_catalog(&self, query: Option<&str>, category: Option<&str>) -> Result<CatalogListResponse, reqwest::Error> {
        let mut url = format!("{}/v1/catalog/servers?limit=200", self.base_url);
        if let Some(q) = query {
            url.push_str(&format!("&q={}", urlencoding::encode(q)));
        }
        if let Some(cat) = category {
            url.push_str(&format!("&category={}", urlencoding::encode(cat)));
        }
        self.client.get(&url).send().await?.json().await
    }

    pub async fn get_catalog_server(&self, id: &str) -> Result<Option<CatalogEntry>, reqwest::Error> {
        let url = format!("{}/v1/catalog/servers/{}", self.base_url, id);
        let resp = self.client.get(&url).send().await?;
        if resp.status() == 404 {
            return Ok(None);
        }
        Ok(Some(resp.json().await?))
    }

    pub async fn fetch_icon(&self, icon_key: &str) -> Result<Vec<u8>, reqwest::Error> {
        let url = format!("{}/v1/icons/{}", self.base_url, icon_key);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.bytes().await?.to_vec())
    }

    // === Auth ===

    pub fn github_auth_url(&self) -> String {
        format!("{}/v1/auth/github", self.base_url)
    }

    pub async fn exchange_github_callback(&self, callback_url: &str) -> Result<AuthResponse, reqwest::Error> {
        // The callback URL contains the code; we forward it to our API
        let resp = self.client.get(callback_url).send().await?;
        resp.json().await
    }

    // === Protected endpoints ===

    pub async fn register_device(&self, device: &DeviceInfo) -> Result<(), reqwest::Error> {
        let url = format!("{}/v1/devices/register", self.base_url);
        self.client.post(&url)
            .header("Authorization", self.auth_header().unwrap_or_default())
            .json(device)
            .send().await?;
        Ok(())
    }

    pub async fn sync_push(&self, changes: &[ChangeEntry]) -> Result<(), reqwest::Error> {
        let url = format!("{}/v1/sync/push", self.base_url);
        let body = serde_json::json!({"changes": changes});
        self.client.post(&url)
            .header("Authorization", self.auth_header().unwrap_or_default())
            .json(&body)
            .send().await?;
        Ok(())
    }

    pub async fn sync_pull(&self, since_id: i64, device_id: &str) -> Result<PullResponse, reqwest::Error> {
        let url = format!(
            "{}/v1/sync/pull?since_id={}&device_id={}",
            self.base_url, since_id, device_id
        );
        self.client.get(&url)
            .header("Authorization", self.auth_header().unwrap_or_default())
            .send().await?
            .json().await
    }

    pub async fn sync_snapshot(&self) -> Result<SyncSnapshot, reqwest::Error> {
        let url = format!("{}/v1/sync/snapshot", self.base_url);
        self.client.get(&url)
            .header("Authorization", self.auth_header().unwrap_or_default())
            .send().await?
            .json().await
    }

    pub async fn get_profile(&self) -> Result<AuthUser, reqwest::Error> {
        let url = format!("{}/v1/user/profile", self.base_url);
        self.client.get(&url)
            .header("Authorization", self.auth_header().unwrap_or_default())
            .send().await?
            .json().await
    }
}
```

- [ ] **Step 3: Add `urlencoding` dependency**

```toml
urlencoding = "2"
```

- [ ] **Step 4: Add module to lib.rs**

In `plugmux-core/src/lib.rs`, add:

```rust
pub mod api_client;
```

- [ ] **Step 5: Build**

```bash
cargo build -p plugmux-core
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add API client module for plugmux cloud backend"
```

---

### Task 5: Sync client module

**Files:**
- Create: `crates/plugmux-core/src/sync/mod.rs`
- Create: `crates/plugmux-core/src/sync/client.rs`

- [ ] **Step 1: Create sync/mod.rs**

```rust
pub mod client;
```

- [ ] **Step 2: Create sync/client.rs**

```rust
use crate::api_client::ApiClient;
use crate::db::Db;
use crate::db::sync_store;
use plugmux_types::*;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SyncClient {
    api: Arc<RwLock<ApiClient>>,
    db: Arc<Db>,
    device_id: String,
}

#[derive(Debug)]
pub enum SyncError {
    NotAuthenticated,
    Network(String),
    Storage(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::NotAuthenticated => write!(f, "Not authenticated"),
            SyncError::Network(e) => write!(f, "Network error: {}", e),
            SyncError::Storage(e) => write!(f, "Storage error: {}", e),
        }
    }
}

impl SyncClient {
    pub fn new(api: Arc<RwLock<ApiClient>>, db: Arc<Db>) -> Result<Self, SyncError> {
        let device_id = sync_store::get_or_create_device_id(&db)
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        Ok(Self { api, db, device_id })
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// Queue a change locally (for offline support)
    pub fn queue_change(&self, action: SyncAction, payload: serde_json::Value) -> Result<(), SyncError> {
        let entry = ChangeEntry {
            id: None,
            device_id: self.device_id.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            action: action.as_str().to_string(),
            payload,
        };
        let json = serde_json::to_string(&entry)
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        sync_store::queue_change(&self.db, &json)
            .map_err(|e| SyncError::Storage(e.to_string()))
    }

    /// Push queued changes, then pull remote changes
    pub async fn sync(&self) -> Result<SyncResult, SyncError> {
        let api = self.api.read().await;
        if !api.is_authenticated() {
            return Err(SyncError::NotAuthenticated);
        }

        // 1. Drain and push local queue
        let queued = sync_store::drain_queue(&self.db)
            .map_err(|e| SyncError::Storage(e.to_string()))?;

        if !queued.is_empty() {
            let changes: Vec<ChangeEntry> = queued.iter()
                .filter_map(|json| serde_json::from_str(json).ok())
                .collect();

            if !changes.is_empty() {
                api.sync_push(&changes).await
                    .map_err(|e| SyncError::Network(e.to_string()))?;
            }
        }

        // 2. Pull remote changes
        let last_id = sync_store::get_last_sync_id(&self.db)
            .map_err(|e| SyncError::Storage(e.to_string()))?;

        let pull = api.sync_pull(last_id, &self.device_id).await
            .map_err(|e| SyncError::Network(e.to_string()))?;

        let changes_count = pull.changes.len();

        // Update last sync ID
        if let Some(last) = pull.changes.last() {
            if let Some(id) = last.id {
                sync_store::set_last_sync_id(&self.db, id)
                    .map_err(|e| SyncError::Storage(e.to_string()))?;
            }
        }

        Ok(SyncResult {
            pushed: queued.len(),
            pulled: changes_count,
            changes: pull.changes,
            has_more: pull.has_more,
        })
    }
}

#[derive(Debug)]
pub struct SyncResult {
    pub pushed: usize,
    pub pulled: usize,
    pub changes: Vec<ChangeEntry>,
    pub has_more: bool,
}
```

- [ ] **Step 3: Add module to lib.rs**

```rust
pub mod sync;
```

- [ ] **Step 4: Build**

```bash
cargo build -p plugmux-core
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add sync client with offline queue and push/pull"
```

---

### Task 6: Tauri auth commands

**Files:**
- Modify: `crates/plugmux-app/src-tauri/src/commands.rs`
- Modify: `crates/plugmux-app/src-tauri/src/engine.rs`

- [ ] **Step 1: Add ApiClient and SyncClient to Engine**

In `engine.rs`, add fields to the Engine struct and initialize them:

```rust
use plugmux_core::api_client::ApiClient;
use plugmux_core::sync::client::SyncClient;

// Add to Engine struct:
pub api_client: Arc<RwLock<ApiClient>>,
pub sync_client: Option<SyncClient>,

// In Engine::new() or init:
let api_client = Arc::new(RwLock::new(ApiClient::new(None)));

// Load saved auth token if exists
if let Ok(Some(token)) = sync_store::get_auth_token(&db) {
    api_client.write().await.set_token(token);
}

let sync_client = SyncClient::new(api_client.clone(), db.clone()).ok();
```

- [ ] **Step 2: Add auth commands to commands.rs**

```rust
#[tauri::command]
pub async fn get_auth_status(
    engine: State<'_, Arc<Engine>>,
) -> Result<serde_json::Value, String> {
    let api = engine.api_client.read().await;
    let authenticated = api.is_authenticated();

    if authenticated {
        match api.get_profile().await {
            Ok(user) => Ok(serde_json::json!({
                "authenticated": true,
                "user": user,
            })),
            Err(_) => Ok(serde_json::json!({
                "authenticated": false,
            })),
        }
    } else {
        Ok(serde_json::json!({
            "authenticated": false,
        }))
    }
}

#[tauri::command]
pub async fn get_github_auth_url(
    engine: State<'_, Arc<Engine>>,
) -> Result<String, String> {
    let api = engine.api_client.read().await;
    Ok(api.github_auth_url())
}

#[tauri::command]
pub async fn complete_auth(
    engine: State<'_, Arc<Engine>>,
    callback_url: String,
) -> Result<serde_json::Value, String> {
    let mut api = engine.api_client.write().await;

    let auth = api.exchange_github_callback(&callback_url).await
        .map_err(|e| format!("Auth failed: {}", e))?;

    api.set_token(auth.token.clone());

    // Save token to redb
    plugmux_core::db::sync_store::set_auth_token(&engine.db, &auth.token)
        .map_err(|e| format!("Failed to save token: {}", e))?;

    // Register device
    let device_id = plugmux_core::db::sync_store::get_or_create_device_id(&engine.db)
        .map_err(|e| format!("Failed to get device ID: {}", e))?;
    let device_name = plugmux_core::db::sync_store::get_device_name(&engine.db)
        .map_err(|e| format!("Failed to get device name: {}", e))?;

    let _ = api.register_device(&plugmux_types::DeviceInfo {
        device_id,
        name: device_name,
    }).await;

    Ok(serde_json::json!({
        "user": auth.user,
    }))
}

#[tauri::command]
pub async fn logout(
    engine: State<'_, Arc<Engine>>,
) -> Result<(), String> {
    let mut api = engine.api_client.write().await;
    api.clear_token();
    plugmux_core::db::sync_store::clear_auth_token(&engine.db)
        .map_err(|e| format!("Failed to clear token: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn trigger_sync(
    engine: State<'_, Arc<Engine>>,
) -> Result<serde_json::Value, String> {
    let sync = engine.sync_client.as_ref()
        .ok_or_else(|| "Sync not initialized".to_string())?;

    let result = sync.sync().await
        .map_err(|e| format!("Sync failed: {}", e))?;

    Ok(serde_json::json!({
        "pushed": result.pushed,
        "pulled": result.pulled,
        "has_more": result.has_more,
    }))
}
```

- [ ] **Step 3: Register commands in main.rs/lib.rs**

Add the new commands to the Tauri builder's `invoke_handler`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::get_auth_status,
    commands::get_github_auth_url,
    commands::complete_auth,
    commands::logout,
    commands::trigger_sync,
])
```

- [ ] **Step 4: Build**

```bash
cargo build -p plugmux-app
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add auth and sync Tauri commands"
```

---

### Task 7: Icon loader with local caching

**Files:**
- Create: `crates/plugmux-core/src/icon_loader.rs`
- Modify: `crates/plugmux-app/src-tauri/src/commands.rs` — add icon command

- [ ] **Step 1: Create icon_loader.rs**

```rust
use crate::api_client::ApiClient;
use std::path::PathBuf;
use tokio::sync::RwLock;
use std::sync::Arc;

pub struct IconLoader {
    api: Arc<RwLock<ApiClient>>,
    cache_dir: PathBuf,
}

impl IconLoader {
    pub fn new(api: Arc<RwLock<ApiClient>>) -> Self {
        let cache_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("plugmux")
            .join("icons");

        // Ensure cache dir exists
        let _ = std::fs::create_dir_all(&cache_dir);

        Self { api, cache_dir }
    }

    /// Get icon path, downloading if needed. Returns None if unavailable.
    pub async fn get_icon_path(&self, icon_key: &str, icon_hash: Option<&str>) -> Option<PathBuf> {
        let file_path = self.cache_dir.join(icon_key);

        // Check if cached version exists and hash matches
        if file_path.exists() {
            if let Some(hash) = icon_hash {
                let hash_file = self.cache_dir.join(format!("{}.hash", icon_key));
                if let Ok(cached_hash) = std::fs::read_to_string(&hash_file) {
                    if cached_hash.trim() == hash {
                        return Some(file_path);
                    }
                }
            } else {
                // No hash to check, cached file is fine
                return Some(file_path);
            }
        }

        // Download from API
        let api = self.api.read().await;
        match api.fetch_icon(icon_key).await {
            Ok(bytes) => {
                if std::fs::write(&file_path, &bytes).is_ok() {
                    // Save hash if provided
                    if let Some(hash) = icon_hash {
                        let hash_file = self.cache_dir.join(format!("{}.hash", icon_key));
                        let _ = std::fs::write(hash_file, hash);
                    }
                    Some(file_path)
                } else {
                    None
                }
            }
            Err(_) => {
                // Return cached version even if hash is stale
                if file_path.exists() {
                    Some(file_path)
                } else {
                    None
                }
            }
        }
    }
}
```

- [ ] **Step 2: Add module to lib.rs**

```rust
pub mod icon_loader;
```

- [ ] **Step 3: Add Tauri command for icon path**

In `commands.rs`:

```rust
#[tauri::command]
pub async fn get_icon_path(
    engine: State<'_, Arc<Engine>>,
    icon_key: String,
    icon_hash: Option<String>,
) -> Result<Option<String>, String> {
    let path = engine.icon_loader.get_icon_path(&icon_key, icon_hash.as_deref()).await;
    Ok(path.map(|p| p.to_string_lossy().to_string()))
}
```

- [ ] **Step 4: Build**

```bash
cargo build -p plugmux-app
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add icon loader with local filesystem caching"
```

---

### Task 8: Remote catalog in useCatalog hook

**Files:**
- Modify: `crates/plugmux-app/src/lib/commands.ts` — add new invoke types
- Modify: `crates/plugmux-app/src/hooks/useCatalog.ts` — fetch remote, fallback to bundled

- [ ] **Step 1: Add TypeScript types and invoke functions**

In `commands.ts`, add:

```typescript
// Auth
export interface AuthStatus {
  authenticated: boolean;
  user?: { id: string; username: string; email?: string };
}

export const getAuthStatus = () => invoke<AuthStatus>("get_auth_status");
export const getGithubAuthUrl = () => invoke<string>("get_github_auth_url");
export const completeAuth = (callbackUrl: string) =>
  invoke<{ user: { id: string; username: string; email?: string } }>("complete_auth", { callbackUrl });
export const logout = () => invoke<void>("logout");

// Sync
export const triggerSync = () =>
  invoke<{ pushed: number; pulled: number; has_more: boolean }>("trigger_sync");

// Icons
export const getIconPath = (iconKey: string, iconHash?: string) =>
  invoke<string | null>("get_icon_path", { iconKey, iconHash });

// Remote catalog (fetched via API through core)
export const fetchRemoteCatalog = () =>
  invoke<CatalogEntry[]>("fetch_remote_catalog");
```

- [ ] **Step 2: Update useCatalog.ts**

```typescript
import { useState, useEffect } from "react";
import {
  CatalogEntry,
  listCatalogServers,
  fetchRemoteCatalog,
  searchCatalog,
  Preset,
  listPresets,
} from "../lib/commands";

export function useCatalog() {
  const [servers, setServers] = useState<CatalogEntry[]>([]);
  const [presets, setPresets] = useState<Preset[]>([]);
  const [loading, setLoading] = useState(true);
  const [isRemote, setIsRemote] = useState(false);

  useEffect(() => {
    async function load() {
      try {
        // Try remote catalog first
        const remote = await fetchRemoteCatalog();
        if (remote && remote.length > 0) {
          setServers(remote);
          setIsRemote(true);
        } else {
          throw new Error("Empty remote catalog");
        }
      } catch {
        // Fallback to bundled catalog
        const [s, p] = await Promise.all([listCatalogServers(), listPresets()]);
        setServers(s);
        setPresets(p);
        setIsRemote(false);
      }
      setLoading(false);
    }
    load();
  }, []);

  const search = async (
    query: string,
    category?: string,
  ): Promise<CatalogEntry[]> => {
    // For now, search locally. Remote search can be added later.
    return await searchCatalog(query, category ?? null);
  };

  return { servers, presets, loading, search, isRemote };
}
```

- [ ] **Step 3: Add Tauri command for remote catalog fetch**

In `commands.rs`:

```rust
#[tauri::command]
pub async fn fetch_remote_catalog(
    engine: State<'_, Arc<Engine>>,
) -> Result<Vec<plugmux_types::CatalogEntry>, String> {
    let api = engine.api_client.read().await;

    match api.list_catalog(None, None).await {
        Ok(resp) => {
            // Cache in redb
            let json = serde_json::to_string(&resp.servers).unwrap_or_default();
            let _ = plugmux_core::db::sync_store::cache_catalog(&engine.db, &json);
            Ok(resp.servers)
        }
        Err(_) => {
            // Try cached catalog
            match plugmux_core::db::sync_store::get_cached_catalog(&engine.db) {
                Ok(Some(json)) => {
                    let servers: Vec<plugmux_types::CatalogEntry> = serde_json::from_str(&json)
                        .unwrap_or_default();
                    Ok(servers)
                }
                _ => Err("No catalog available".to_string()),
            }
        }
    }
}
```

- [ ] **Step 4: Build**

```bash
cargo build -p plugmux-app
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: fetch remote catalog with fallback to bundled"
```

---

### Task 9: Update CatalogCard to use real icons

**Files:**
- Modify: `crates/plugmux-app/src/components/catalog/CatalogCard.tsx`

- [ ] **Step 1: Add icon loading to CatalogCard**

Replace the first-initial fallback with an icon loader that tries the cached SVG first:

```typescript
import { useState, useEffect } from "react";
import { getIconPath } from "../../lib/commands";
import { convertFileSrc } from "@tauri-apps/api/core";

function ServerIcon({ entry }: { entry: CatalogEntry }) {
  const [iconSrc, setIconSrc] = useState<string | null>(null);

  useEffect(() => {
    if (entry.icon_key) {
      getIconPath(entry.icon_key, entry.icon_hash ?? undefined).then((path) => {
        if (path) {
          setIconSrc(convertFileSrc(path));
        }
      });
    }
  }, [entry.icon_key, entry.icon_hash]);

  if (iconSrc) {
    return (
      <img
        src={iconSrc}
        alt={entry.name}
        className="h-8 w-8 rounded"
      />
    );
  }

  // Fallback: first initial with color
  const color = colorForId(entry.id);
  return (
    <div
      className="flex h-8 w-8 items-center justify-center rounded text-sm font-bold text-white"
      style={{ backgroundColor: color }}
    >
      {entry.name.charAt(0).toUpperCase()}
    </div>
  );
}
```

Replace the existing icon rendering in the card with `<ServerIcon entry={entry} />`.

- [ ] **Step 2: Build and test**

```bash
cd crates/plugmux-app && npm run dev
```

Expected: Cards show SVG icons when available, fallback letter when not.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: load real SVG icons in CatalogCard with fallback"
```

---

### Task 10: Auth UI in Settings page

**Files:**
- Create: `crates/plugmux-app/src/components/settings/AccountSection.tsx`
- Modify: `crates/plugmux-app/src/pages/SettingsPage.tsx`
- Create: `crates/plugmux-app/src/hooks/useAuth.ts`
- Create: `crates/plugmux-app/src/hooks/useSync.ts`

- [ ] **Step 1: Create useAuth.ts**

```typescript
import { useState, useEffect, useCallback } from "react";
import {
  AuthStatus,
  getAuthStatus,
  getGithubAuthUrl,
  completeAuth,
  logout as logoutCmd,
} from "../lib/commands";
import { open } from "@tauri-apps/plugin-shell";

export function useAuth() {
  const [status, setStatus] = useState<AuthStatus>({ authenticated: false });
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getAuthStatus()
      .then(setStatus)
      .finally(() => setLoading(false));
  }, []);

  const login = useCallback(async () => {
    const url = await getGithubAuthUrl();
    // Open GitHub OAuth in system browser
    await open(url);
    // The callback URL will be handled by a deep link or manual paste
    // For now, the user completes auth in the browser and the app detects it
  }, []);

  const logout = useCallback(async () => {
    await logoutCmd();
    setStatus({ authenticated: false });
  }, []);

  const handleCallback = useCallback(async (callbackUrl: string) => {
    const result = await completeAuth(callbackUrl);
    setStatus({
      authenticated: true,
      user: result.user,
    });
  }, []);

  return { status, loading, login, logout, handleCallback };
}
```

- [ ] **Step 2: Create useSync.ts**

```typescript
import { useState, useCallback } from "react";
import { triggerSync } from "../lib/commands";

export function useSync() {
  const [syncing, setSyncing] = useState(false);
  const [lastResult, setLastResult] = useState<{
    pushed: number;
    pulled: number;
  } | null>(null);
  const [error, setError] = useState<string | null>(null);

  const sync = useCallback(async () => {
    setSyncing(true);
    setError(null);
    try {
      const result = await triggerSync();
      setLastResult({ pushed: result.pushed, pulled: result.pulled });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSyncing(false);
    }
  }, []);

  return { syncing, lastResult, error, sync };
}
```

- [ ] **Step 3: Create AccountSection.tsx**

```typescript
import { useAuth } from "../../hooks/useAuth";
import { useSync } from "../../hooks/useSync";

export function AccountSection() {
  const { status, loading, login, logout } = useAuth();
  const { syncing, lastResult, error, sync } = useSync();

  if (loading) {
    return <div className="text-sm text-zinc-500">Loading...</div>;
  }

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-medium text-zinc-300">Account</h3>

      {status.authenticated ? (
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-zinc-200">
                Signed in as <span className="font-medium">{status.user?.username}</span>
              </p>
              {status.user?.email && (
                <p className="text-xs text-zinc-500">{status.user.email}</p>
              )}
            </div>
            <button
              onClick={logout}
              className="rounded px-3 py-1 text-xs text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
            >
              Sign out
            </button>
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={sync}
              disabled={syncing}
              className="rounded bg-zinc-800 px-3 py-1.5 text-xs text-zinc-200 hover:bg-zinc-700 disabled:opacity-50"
            >
              {syncing ? "Syncing..." : "Sync now"}
            </button>
            {lastResult && (
              <span className="text-xs text-zinc-500">
                {lastResult.pushed} pushed, {lastResult.pulled} pulled
              </span>
            )}
            {error && (
              <span className="text-xs text-red-400">{error}</span>
            )}
          </div>
        </div>
      ) : (
        <div>
          <p className="mb-2 text-xs text-zinc-500">
            Sign in to sync your config across devices
          </p>
          <button
            onClick={login}
            className="flex items-center gap-2 rounded bg-zinc-800 px-4 py-2 text-sm text-zinc-200 hover:bg-zinc-700"
          >
            <svg className="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
            </svg>
            Sign in with GitHub
          </button>
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 4: Add AccountSection to SettingsPage**

Import and add `<AccountSection />` to the settings page, above or below existing settings sections.

- [ ] **Step 5: Build and test**

```bash
cd crates/plugmux-app && npm run dev
```

Expected: Settings page shows Account section with login button or user info + sync button.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add auth UI and sync button in Settings page"
```

---

*End of Plan 2 — plugmux Client-Side Sync*
