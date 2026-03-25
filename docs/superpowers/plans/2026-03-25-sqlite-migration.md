# plugmux SQLite Migration — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace redb with rusqlite (bundled SQLite) as the single embedded database for all user state — environments, server assignments, bookmarks, agent state, logs, and active agents.

**Architecture:** A single `Db` struct wraps a `rusqlite::Connection`, initializes schema on open, and provides per-module query functions. `Arc<Mutex<Db>>` for thread-safe access. Config.json shrinks to machine-local-only settings (port, permissions, device_id, onboarding_shown). Environment/server data moves entirely to SQLite.

**Tech Stack:** `rusqlite` with `bundled` feature (embeds SQLite), raw SQL (no ORM)

**Spec:** `docs/superpowers/specs/2026-03-25-sqlite-migration-design.md`

---

## File Structure

```
crates/plugmux-core/
├── Cargo.toml                          — swap redb → rusqlite
├── src/
│   ├── db/
│   │   ├── mod.rs                      — REWRITE: Db struct with rusqlite, schema init, WAL
│   │   ├── environments.rs             — NEW: environment + environment_servers CRUD
│   │   ├── bookmarks.rs                — NEW: bookmark CRUD
│   │   ├── agents.rs                   — NEW: agent state + dismissed agents (replaces agents/state.rs)
│   │   ├── active_agents.rs            — REWRITE: SQLite instead of redb
│   │   └── logs.rs                     — REWRITE: SQLite instead of redb
│   ├── config.rs                       — MODIFY: remove Environment, add device_id + onboarding_shown
│   ├── environment.rs                  — REWRITE: query Db instead of Config
│   ├── agents/
│   │   ├── mod.rs                      — MODIFY: remove state module re-export
│   │   ├── state.rs                    — DELETE (replaced by db/agents.rs)
│   │   ├── detect.rs                   — MODIFY: use db::agents instead of AgentState
│   │   └── migrate.rs                  — MODIFY: use db::agents
│   ├── plugmux_layer/mod.rs            — MODIFY: environment queries via Db
│   ├── proxy_layer/mod.rs              — MODIFY: environment queries via Db
│   ├── gateway/router.rs               — MODIFY: pass Db, environment lookup from Db
│   ├── gateway/logging.rs              — MODIFY: use new logs API
│   ├── migration.rs                    — MODIFY: write to Db instead of config.json for envs
│   └── lib.rs                          — no change
├── crates/plugmux-app/src-tauri/src/
│   ├── engine.rs                       — MODIFY: open SQLite Db, remove redb, pass to router
│   ├── commands.rs                     — MODIFY: environment/agent commands use Db
│   └── watcher.rs                      — MODIFY: only watch config.json for port/permissions
├── crates/plugmux-cli/src/commands/
│   ├── env.rs                          — MODIFY: use Db for environment CRUD
│   ├── server.rs                       — MODIFY: use Db for server assignment
│   └── agents.rs                       — MODIFY: use db::agents
```

---

## Task 1: Swap redb for rusqlite in Cargo.toml

**Files:**
- Modify: `crates/plugmux-core/Cargo.toml`

- [ ] **Step 1: Replace redb with rusqlite**

In `crates/plugmux-core/Cargo.toml`, change:
```toml
# Remove:
redb = "2"
# Add:
rusqlite = { version = "0.34", features = ["bundled"] }
```

- [ ] **Step 2: Verify it compiles (will have errors — that's expected)**

Run: `cargo check -p plugmux-core 2>&1 | head -5`
Expected: Errors about `redb` imports — confirms the swap happened.

- [ ] **Step 3: Commit**

```bash
git add crates/plugmux-core/Cargo.toml Cargo.lock
git commit -m "chore: swap redb for rusqlite in plugmux-core"
```

---

## Task 2: Rewrite db/mod.rs — SQLite Db struct + schema init

**Files:**
- Rewrite: `crates/plugmux-core/src/db/mod.rs`

- [ ] **Step 1: Write the new Db struct with schema initialization**

```rust
//! Embedded database module (SQLite).
//!
//! Stores environments, server assignments, bookmarks, agent state, logs,
//! and active agent tracking.

pub mod active_agents;
pub mod agents;
pub mod bookmarks;
pub mod environments;
pub mod logs;

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

pub struct Db {
    pub conn: Mutex<Connection>,
}

impl Db {
    pub fn open(path: &Path) -> Result<Arc<Self>, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Self::init_schema(&conn)?;
        Ok(Arc::new(Self { conn: Mutex::new(conn) }))
    }

    pub fn open_in_memory() -> Result<Arc<Self>, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        Self::init_schema(&conn)?;
        Ok(Arc::new(Self { conn: Mutex::new(conn) }))
    }

    fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS environments (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS environment_servers (
                env_id      TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
                server_id   TEXT NOT NULL,
                PRIMARY KEY (env_id, server_id)
            );

            CREATE TABLE IF NOT EXISTS bookmarks (
                server_id   TEXT PRIMARY KEY,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS agents (
                id              TEXT PRIMARY KEY,
                source          TEXT NOT NULL,
                name            TEXT,
                icon            TEXT,
                config_path     TEXT,
                config_format   TEXT,
                mcp_key         TEXT
            );

            CREATE TABLE IF NOT EXISTS dismissed_agents (
                agent_id    TEXT PRIMARY KEY
            );

            CREATE TABLE IF NOT EXISTS logs (
                id              TEXT PRIMARY KEY,
                timestamp       TEXT NOT NULL,
                env_id          TEXT NOT NULL,
                method          TEXT NOT NULL,
                params_summary  TEXT,
                result_summary  TEXT,
                error           TEXT,
                duration_ms     INTEGER NOT NULL,
                user_agent      TEXT,
                agent_id        TEXT,
                session_id      TEXT
            );

            CREATE TABLE IF NOT EXISTS active_agents (
                agent_id    TEXT PRIMARY KEY
            );

            CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_logs_env_id ON logs(env_id);
            CREATE INDEX IF NOT EXISTS idx_logs_agent_id ON logs(agent_id);

            INSERT OR IGNORE INTO environments (id, name) VALUES ('global', 'Global');
            ",
        )?;
        Ok(())
    }

    pub fn default_path() -> std::path::PathBuf {
        crate::config::config_dir().join("plugmux.db")
    }
}
```

- [ ] **Step 2: Verify module compiles (sub-modules will be empty stubs)**

Create empty files for new modules:
- `crates/plugmux-core/src/db/environments.rs` — `// TODO`
- `crates/plugmux-core/src/db/bookmarks.rs` — `// TODO`
- `crates/plugmux-core/src/db/agents.rs` — `// TODO`

Run: `cargo check -p plugmux-core 2>&1 | grep "^error" | head -10`

- [ ] **Step 3: Write test for Db::open_in_memory**

Add to `db/mod.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory_creates_schema() {
        let db = Db::open_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        // Verify global environment exists
        let name: String = conn
            .query_row("SELECT name FROM environments WHERE id = 'global'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "Global");
    }

    #[test]
    fn test_open_in_memory_creates_all_tables() {
        let db = Db::open_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let tables = ["environments", "environment_servers", "bookmarks", "agents",
                       "dismissed_agents", "logs", "active_agents"];
        for table in tables {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                .unwrap();
            assert!(count >= 0, "table {table} should exist");
        }
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p plugmux-core -- db::tests -v`
Expected: 2 tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-core/src/db/
git commit -m "feat: rewrite db/mod.rs with SQLite schema init"
```

---

## Task 3: Implement db/environments.rs

**Files:**
- Create: `crates/plugmux-core/src/db/environments.rs`

- [ ] **Step 1: Write tests first**

```rust
#[cfg(test)]
mod tests {
    use crate::db::Db;
    use super::*;

    #[test]
    fn test_list_environments_has_global() {
        let db = Db::open_in_memory().unwrap();
        let envs = list_environments(&db);
        assert_eq!(envs.len(), 1);
        assert_eq!(envs[0].id, "global");
    }

    #[test]
    fn test_add_and_list_environment() {
        let db = Db::open_in_memory().unwrap();
        add_environment(&db, "work", "Work").unwrap();
        let envs = list_environments(&db);
        assert_eq!(envs.len(), 2);
    }

    #[test]
    fn test_remove_environment() {
        let db = Db::open_in_memory().unwrap();
        add_environment(&db, "work", "Work").unwrap();
        remove_environment(&db, "work").unwrap();
        let envs = list_environments(&db);
        assert_eq!(envs.len(), 1); // only global
    }

    #[test]
    fn test_cannot_remove_global() {
        let db = Db::open_in_memory().unwrap();
        let result = remove_environment(&db, "global");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_and_get_server_ids() {
        let db = Db::open_in_memory().unwrap();
        add_server(&db, "global", "figma").unwrap();
        add_server(&db, "global", "github").unwrap();
        let ids = get_server_ids(&db, "global").unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"figma".to_string()));
    }

    #[test]
    fn test_add_server_idempotent() {
        let db = Db::open_in_memory().unwrap();
        add_server(&db, "global", "figma").unwrap();
        add_server(&db, "global", "figma").unwrap();
        let ids = get_server_ids(&db, "global").unwrap();
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn test_remove_server() {
        let db = Db::open_in_memory().unwrap();
        add_server(&db, "global", "figma").unwrap();
        remove_server(&db, "global", "figma").unwrap();
        let ids = get_server_ids(&db, "global").unwrap();
        assert!(ids.is_empty());
    }

    #[test]
    fn test_get_all_server_ids_across_environments() {
        let db = Db::open_in_memory().unwrap();
        add_environment(&db, "work", "Work").unwrap();
        add_server(&db, "global", "figma").unwrap();
        add_server(&db, "work", "github").unwrap();
        add_server(&db, "work", "figma").unwrap(); // duplicate across envs
        let all = get_all_server_ids(&db);
        assert_eq!(all.len(), 2); // figma + github, deduplicated
    }
}
```

- [ ] **Step 2: Implement the module**

```rust
use std::sync::Arc;
use crate::db::Db;

pub struct EnvironmentRow {
    pub id: String,
    pub name: String,
}

pub fn list_environments(db: &Arc<Db>) -> Vec<EnvironmentRow> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name FROM environments ORDER BY rowid").unwrap();
    stmt.query_map([], |row| {
        Ok(EnvironmentRow {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}

pub fn add_environment(db: &Arc<Db>, id: &str, name: &str) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT OR IGNORE INTO environments (id, name) VALUES (?1, ?2)",
        rusqlite::params![id, name],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn remove_environment(db: &Arc<Db>, id: &str) -> Result<(), String> {
    if id == "global" {
        return Err("Cannot delete the global environment".to_string());
    }
    let conn = db.conn.lock().unwrap();
    let affected = conn
        .execute("DELETE FROM environments WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    if affected == 0 {
        return Err(format!("Environment not found: {id}"));
    }
    Ok(())
}

pub fn get_server_ids(db: &Arc<Db>, env_id: &str) -> Result<Vec<String>, String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT server_id FROM environment_servers WHERE env_id = ?1")
        .map_err(|e| e.to_string())?;
    let ids = stmt
        .query_map(rusqlite::params![env_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(ids)
}

pub fn add_server(db: &Arc<Db>, env_id: &str, server_id: &str) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT OR IGNORE INTO environment_servers (env_id, server_id) VALUES (?1, ?2)",
        rusqlite::params![env_id, server_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn remove_server(db: &Arc<Db>, env_id: &str, server_id: &str) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "DELETE FROM environment_servers WHERE env_id = ?1 AND server_id = ?2",
        rusqlite::params![env_id, server_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Returns all unique server IDs across all environments.
pub fn get_all_server_ids(db: &Arc<Db>) -> Vec<String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT DISTINCT server_id FROM environment_servers")
        .unwrap();
    stmt.query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p plugmux-core -- db::environments -v`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/plugmux-core/src/db/environments.rs
git commit -m "feat: add db/environments.rs — environment + server CRUD"
```

---

## Task 4: Implement db/bookmarks.rs

**Files:**
- Create: `crates/plugmux-core/src/db/bookmarks.rs`

- [ ] **Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use crate::db::Db;
    use super::*;

    #[test]
    fn test_add_and_list_bookmarks() {
        let db = Db::open_in_memory().unwrap();
        add_bookmark(&db, "figma").unwrap();
        add_bookmark(&db, "github").unwrap();
        let bookmarks = list_bookmarks(&db);
        assert_eq!(bookmarks.len(), 2);
    }

    #[test]
    fn test_add_bookmark_idempotent() {
        let db = Db::open_in_memory().unwrap();
        add_bookmark(&db, "figma").unwrap();
        add_bookmark(&db, "figma").unwrap();
        assert_eq!(list_bookmarks(&db).len(), 1);
    }

    #[test]
    fn test_remove_bookmark() {
        let db = Db::open_in_memory().unwrap();
        add_bookmark(&db, "figma").unwrap();
        remove_bookmark(&db, "figma").unwrap();
        assert!(list_bookmarks(&db).is_empty());
    }

    #[test]
    fn test_is_bookmarked() {
        let db = Db::open_in_memory().unwrap();
        assert!(!is_bookmarked(&db, "figma"));
        add_bookmark(&db, "figma").unwrap();
        assert!(is_bookmarked(&db, "figma"));
    }
}
```

- [ ] **Step 2: Implement**

```rust
use std::sync::Arc;
use crate::db::Db;

pub fn add_bookmark(db: &Arc<Db>, server_id: &str) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT OR IGNORE INTO bookmarks (server_id) VALUES (?1)",
        rusqlite::params![server_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn remove_bookmark(db: &Arc<Db>, server_id: &str) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "DELETE FROM bookmarks WHERE server_id = ?1",
        rusqlite::params![server_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_bookmarks(db: &Arc<Db>) -> Vec<String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT server_id FROM bookmarks ORDER BY created_at").unwrap();
    stmt.query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
}

pub fn is_bookmarked(db: &Arc<Db>, server_id: &str) -> bool {
    let conn = db.conn.lock().unwrap();
    conn.query_row(
        "SELECT 1 FROM bookmarks WHERE server_id = ?1",
        rusqlite::params![server_id],
        |_| Ok(()),
    )
    .is_ok()
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p plugmux-core -- db::bookmarks -v`

- [ ] **Step 4: Commit**

```bash
git add crates/plugmux-core/src/db/bookmarks.rs
git commit -m "feat: add db/bookmarks.rs — bookmark CRUD"
```

---

## Task 5: Implement db/agents.rs (replaces agents/state.rs)

**Files:**
- Create: `crates/plugmux-core/src/db/agents.rs`

- [ ] **Step 1: Write tests**

Tests should cover: add_agent, get_agent, remove_agent, list_agents, dismiss_agent, is_dismissed, add replaces existing.

- [ ] **Step 2: Implement**

Functions: `add_agent`, `remove_agent`, `get_agent`, `list_agents`, `dismiss_agent`, `is_dismissed`, `list_dismissed`. Same semantics as the old `AgentState` struct but backed by SQL.

The `AgentStateEntry` struct moves here (or is re-exported). Keep the same fields: `id`, `source`, `name`, `icon`, `config_path`, `config_format`, `mcp_key`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p plugmux-core -- db::agents -v`

- [ ] **Step 4: Commit**

```bash
git add crates/plugmux-core/src/db/agents.rs
git commit -m "feat: add db/agents.rs — agent state CRUD"
```

---

## Task 6: Rewrite db/logs.rs with SQLite

**Files:**
- Rewrite: `crates/plugmux-core/src/db/logs.rs`

- [ ] **Step 1: Rewrite using SQL**

Keep the same `LogEntry` and `AgentInfo` structs. Replace redb table operations with SQL INSERT/SELECT. The `write_log` function inserts a row. The `read_recent_logs` function uses `ORDER BY timestamp DESC LIMIT ?`.

- [ ] **Step 2: Update tests to use Db::open_in_memory()**

Replace `tempfile::TempDir` + `Db::open(&dir)` with `Db::open_in_memory()`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p plugmux-core -- db::logs -v`

- [ ] **Step 4: Commit**

```bash
git add crates/plugmux-core/src/db/logs.rs
git commit -m "feat: rewrite db/logs.rs with SQLite"
```

---

## Task 7: Rewrite db/active_agents.rs with SQLite

**Files:**
- Rewrite: `crates/plugmux-core/src/db/active_agents.rs`

- [ ] **Step 1: Rewrite using SQL**

Same API: `mark_active(db, agent_id) -> Result<bool>` and `load_active(db) -> Result<HashSet<String>>`. Replace redb with SQL INSERT/SELECT.

- [ ] **Step 2: Update tests**

- [ ] **Step 3: Run tests**

Run: `cargo test -p plugmux-core -- db::active_agents -v`

- [ ] **Step 4: Commit**

```bash
git add crates/plugmux-core/src/db/active_agents.rs
git commit -m "feat: rewrite db/active_agents.rs with SQLite"
```

---

## Task 8: Update config.rs — remove environments, add device_id + onboarding

**Files:**
- Modify: `crates/plugmux-core/src/config.rs`

- [ ] **Step 1: Remove Environment from Config struct**

Remove:
- `Environment` struct
- `environments` field from `Config`
- `ensure_global`, `add_environment`, `find_environment`, `find_environment_mut`, `remove_environment` functions
- `GLOBAL_ENV` constant (move to `db/environments.rs` or keep as a shared constant)
- All tests related to environments

Keep `GLOBAL_ENV` as a public constant (used by router and other modules).

- [ ] **Step 2: Add device_id and onboarding_shown**

```rust
fn default_device_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default = "default_device_id")]
    pub device_id: String,
    #[serde(default)]
    pub onboarding_shown: bool,
}
```

- [ ] **Step 3: Update default_config() and tests**

- [ ] **Step 4: Run tests**

Run: `cargo test -p plugmux-core -- config -v`

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-core/src/config.rs
git commit -m "refactor: remove environments from config, add device_id + onboarding"
```

---

## Task 9: Rewrite environment.rs to use Db

**Files:**
- Rewrite: `crates/plugmux-core/src/environment.rs`

- [ ] **Step 1: Replace all functions**

The old `get_server_ids(&Config, env_id)`, `add_server(&mut Config, env_id, server_id)`, `remove_server(&mut Config, env_id, server_id)` now delegate to `db::environments::*`.

```rust
use std::sync::Arc;
use crate::db::Db;
use crate::db::environments;

pub fn get_server_ids(db: &Arc<Db>, env_id: &str) -> Option<Vec<String>> {
    environments::get_server_ids(db, env_id).ok()
}

pub fn add_server(db: &Arc<Db>, env_id: &str, server_id: &str) -> Result<(), String> {
    environments::add_server(db, env_id, server_id)
}

pub fn remove_server(db: &Arc<Db>, env_id: &str, server_id: &str) -> Result<bool, String> {
    environments::remove_server(db, env_id, server_id)?;
    Ok(true)
}
```

Or consider removing `environment.rs` entirely and having callers use `db::environments` directly. Choose whichever produces less churn.

- [ ] **Step 2: Update tests**

- [ ] **Step 3: Commit**

```bash
git add crates/plugmux-core/src/environment.rs
git commit -m "refactor: environment.rs delegates to db::environments"
```

---

## Task 10: Remove agents/state.rs, update agents/mod.rs

**Files:**
- Delete: `crates/plugmux-core/src/agents/state.rs`
- Modify: `crates/plugmux-core/src/agents/mod.rs`
- Modify: `crates/plugmux-core/src/agents/detect.rs`
- Modify: `crates/plugmux-core/src/agents/migrate.rs`

- [ ] **Step 1: Remove state module from agents/mod.rs**

```rust
mod registry;
pub use registry::*;

mod detect;
pub use detect::*;

mod migrate;
pub use migrate::*;

// Re-export db agent types for convenience
pub use crate::db::agents::{AgentStateEntry, AgentSource};
```

- [ ] **Step 2: Delete agents/state.rs**

- [ ] **Step 3: Update detect.rs and migrate.rs**

These files import `AgentState`, `AgentStateEntry`, `AgentSource`, `ConfigFormat` from the old state module. Update to use `crate::db::agents` instead. The `detect_all` function currently takes `&AgentState` — change it to take `&Arc<Db>`.

- [ ] **Step 4: Run all tests**

Run: `cargo test -p plugmux-core -- agents -v`

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-core/src/agents/
git commit -m "refactor: remove agents/state.rs, use db::agents"
```

---

## Task 11: Update plugmux_layer/mod.rs

**Files:**
- Modify: `crates/plugmux-core/src/plugmux_layer/mod.rs`

- [ ] **Step 1: Update environment handlers**

The `PlugmuxLayer` already has `db: Option<Arc<Db>>`. Change:
- `handle_enable_server` → use `db::environments::add_server` instead of `environment::add_server(&mut cfg, ...)`
- `handle_disable_server` → use `db::environments::remove_server`
- `handle_add_environment` → use `db::environments::add_environment`
- `build_environments_json` → use `db::environments::list_environments` + `get_server_ids`
- Remove `config` writes for environment operations (no longer needed)

- [ ] **Step 2: Update permission checking**

Permissions still come from `Config` (that stays). Only environment mutations move to Db.

- [ ] **Step 3: Update tests**

Tests that construct `PlugmuxLayer` need to pass a `Db::open_in_memory()` and pre-populate environments via `db::environments::add_environment`.

- [ ] **Step 4: Run tests**

Run: `cargo test -p plugmux-core -- plugmux_layer -v`

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-core/src/plugmux_layer/
git commit -m "refactor: plugmux_layer uses Db for environment operations"
```

---

## Task 12: Update proxy_layer/mod.rs

**Files:**
- Modify: `crates/plugmux-core/src/proxy_layer/mod.rs`

- [ ] **Step 1: Change server_ids() to query Db**

The `ProxyLayer` currently holds `Arc<RwLock<Config>>` and calls `environment::get_server_ids(&cfg, env_id)`. Add a `db: Option<Arc<Db>>` field and query from Db instead.

- [ ] **Step 2: Commit**

```bash
git add crates/plugmux-core/src/proxy_layer/
git commit -m "refactor: proxy_layer queries Db for server IDs"
```

---

## Task 13: Update gateway/router.rs

**Files:**
- Modify: `crates/plugmux-core/src/gateway/router.rs`

- [ ] **Step 1: Ensure Db is threaded through correctly**

The router already passes `db: Option<Arc<Db>>` to `PlugmuxLayer`. Ensure `ProxyLayer` also gets it. The `AppState` struct already has `db` — just make sure it's used consistently.

- [ ] **Step 2: Update build_router signature if needed**

- [ ] **Step 3: Commit**

```bash
git add crates/plugmux-core/src/gateway/
git commit -m "refactor: router passes Db to proxy_layer"
```

---

## Task 14: Update gateway/logging.rs

**Files:**
- Modify: `crates/plugmux-core/src/gateway/logging.rs`

- [ ] **Step 1: Update log_request to use new SQLite-based write_log**

The function signature should remain the same. The underlying `write_log` function now uses SQL.

- [ ] **Step 2: Commit**

```bash
git add crates/plugmux-core/src/gateway/logging.rs
git commit -m "refactor: logging uses SQLite-backed write_log"
```

---

## Task 15: Update migration.rs

**Files:**
- Modify: `crates/plugmux-core/src/migration.rs`

- [ ] **Step 1: Update to write environments to Db instead of Config**

The old migration writes `Environment` structs to `Config.environments`. Now it should call `db::environments::add_environment` and `db::environments::add_server`. This means `migrate()` needs a `&Arc<Db>` parameter.

Alternatively, since there are no production users, consider simplifying or removing the Phase-2 migration entirely.

- [ ] **Step 2: Update tests**

- [ ] **Step 3: Commit**

```bash
git add crates/plugmux-core/src/migration.rs
git commit -m "refactor: migration writes environments to SQLite"
```

---

## Task 16: Update Tauri engine.rs

**Files:**
- Modify: `crates/plugmux-app/src-tauri/src/engine.rs`

- [ ] **Step 1: Change Db type**

The `Engine.db` field changes from `Arc<RwLock<Option<Arc<Db>>>>` (where Db was redb) to the new SQLite `Arc<Db>`. Open the database in `Engine::new()` rather than in `start()`, since environment data is needed before the gateway starts.

- [ ] **Step 2: Update start() to use Db for environment iteration**

Instead of `cfg.environments`, use `db::environments::list_environments` + `get_server_ids` to collect all server IDs to start.

- [ ] **Step 3: Commit**

```bash
git add crates/plugmux-app/src-tauri/src/engine.rs
git commit -m "refactor: engine uses SQLite Db"
```

---

## Task 17: Update Tauri commands.rs

**Files:**
- Modify: `crates/plugmux-app/src-tauri/src/commands.rs`

- [ ] **Step 1: Update environment commands**

Commands that read/write environments (`get_environments`, `add_environment`, etc.) now use `db::environments`. Commands that read/write agent state use `db::agents`.

- [ ] **Step 2: Update agent commands**

`add_custom_agent`, `add_agent_from_registry`, `dismiss_agent` etc. now use `db::agents`.

- [ ] **Step 3: Remove imports of old types**

Remove `Environment` from `plugmux_core::config` imports. Remove `AgentState`, `AgentStateEntry` from `plugmux_core::agents` if they moved.

- [ ] **Step 4: Compile and fix**

Run: `cargo check -p plugmux-app`

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-app/src-tauri/src/commands.rs
git commit -m "refactor: Tauri commands use SQLite for environments + agents"
```

---

## Task 18: Update CLI commands (env.rs, server.rs, agents.rs)

**Files:**
- Modify: `crates/plugmux-cli/src/commands/env.rs`
- Modify: `crates/plugmux-cli/src/commands/server.rs`
- Modify: `crates/plugmux-cli/src/commands/agents.rs`

- [ ] **Step 1: CLI commands open a Db connection**

Each CLI command that needs environment/agent data opens a `Db::open(&Db::default_path())` and queries it directly.

- [ ] **Step 2: Update env.rs**

`EnvCommands::List` → `db::environments::list_environments`
`EnvCommands::Create` → `db::environments::add_environment` + optionally `add_server` for presets
`EnvCommands::Delete` → `db::environments::remove_environment`

- [ ] **Step 3: Update server.rs**

`ServerCommands::Add` → `db::environments::add_server`
`ServerCommands::Remove` → `db::environments::remove_server`
`ServerCommands::List` → `db::environments::get_server_ids`

- [ ] **Step 4: Update agents.rs**

Use `db::agents` instead of `AgentState::load`.

- [ ] **Step 5: Run full build**

Run: `cargo check`

- [ ] **Step 6: Commit**

```bash
git add crates/plugmux-cli/src/commands/
git commit -m "refactor: CLI commands use SQLite for environments + agents"
```

---

## Task 19: Update integration test

**Files:**
- Modify: `crates/plugmux-cli/tests/gateway_integration.rs`

- [ ] **Step 1: Update test to use Db for environment setup**

The integration test currently constructs `Config` with `Environment` structs. Update to create a `Db::open_in_memory()` and populate environments via `db::environments`.

- [ ] **Step 2: Run integration test**

Run: `cargo test -p plugmux-cli -- gateway_integration -v`

- [ ] **Step 3: Commit**

```bash
git add crates/plugmux-cli/tests/
git commit -m "refactor: integration test uses SQLite"
```

---

## Task 20: Final cleanup and full test run

**Files:**
- Delete: `crates/plugmux-core/src/agents/state.rs` (if not done in Task 10)
- Verify: no remaining `redb` imports anywhere

- [ ] **Step 1: Search for any remaining redb references**

Run: `rg "redb" crates/ --type rust`
Expected: No matches.

- [ ] **Step 2: Search for remaining agents/state.rs references**

Run: `rg "agents::state\b|AgentState::load" crates/ --type rust`
Expected: No matches (all migrated to db::agents).

- [ ] **Step 3: Full test suite**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 4: Full clippy check**

Run: `cargo clippy -- -D warnings`

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore: cleanup — remove all redb references, final SQLite migration"
```
