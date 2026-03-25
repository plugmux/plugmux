# plugmux — Replace redb with SQLite Design Spec

**Author:** Lasha Kvantaliani
**Date:** 2026-03-25
**Status:** Draft v1

---

## 1. Goal

Replace redb with SQLite as the single embedded database for all local user state. This aligns the local storage format with the cloud backend (Cloudflare D1 = SQLite), making future sync trivial — same schema, same SQL, no translation layer.

---

## 2. Why SQLite over redb

- **Sync simplicity**: D1 is SQLite. Same schema on client and server means sync is row-level diffing, not format translation.
- **Relational queries**: Logs filtering, environment-server joins — SQL is natural for this.
- **Inspectable**: `sqlite3 plugmux.db` for debugging. redb has no CLI tool.
- **Mature**: Battle-tested, WAL mode handles concurrent reads during writes.
- **Size**: ~1-2MB added to binary via `rusqlite` bundled feature. Negligible for a Tauri app.

---

## 3. What Moves Where

### SQLite (`plugmux.db`) — user data + local operational data

| Data | Currently | Synced later? |
|------|-----------|---------------|
| Environments | config.json | Yes |
| Server assignments per env | config.json | Yes |
| Bookmarked servers | N/A (new) | Yes |
| Agent state | agents_state.json | Yes |
| Dismissed agents | agents_state.json | Yes |
| Logs | redb | No |
| Active agent flags | redb | No |

### config.json — machine-local settings only (never synced)

```json
{
  "port": 4242,
  "permissions": {
    "enable_server": "allow",
    "disable_server": "allow"
  },
  "device_id": "uuid",
  "onboarding_shown": false
}
```

---

## 4. Schema

```sql
-- =============================================
-- Synced tables (will sync to cloud later)
-- =============================================

CREATE TABLE environments (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE environment_servers (
    env_id      TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
    server_id   TEXT NOT NULL,
    PRIMARY KEY (env_id, server_id)
);

CREATE TABLE bookmarks (
    server_id   TEXT PRIMARY KEY,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE agents (
    id              TEXT PRIMARY KEY,
    source          TEXT NOT NULL,   -- 'auto' | 'registry' | 'custom'
    name            TEXT,
    icon            TEXT,
    config_path     TEXT,
    config_format   TEXT,            -- 'json' | 'toml'
    mcp_key         TEXT
);

CREATE TABLE dismissed_agents (
    agent_id    TEXT PRIMARY KEY
);

-- =============================================
-- Local-only tables (never synced)
-- =============================================

CREATE TABLE logs (
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

CREATE TABLE active_agents (
    agent_id    TEXT PRIMARY KEY
);
```

### Indexes

```sql
CREATE INDEX idx_logs_timestamp ON logs(timestamp DESC);
CREATE INDEX idx_logs_env_id ON logs(env_id);
CREATE INDEX idx_logs_agent_id ON logs(agent_id);
```

### Bootstrap

On first launch, insert the "global" environment:

```sql
INSERT OR IGNORE INTO environments (id, name) VALUES ('global', 'Global');
```

---

## 5. Database Module Structure

```
crates/plugmux-core/src/db/
├── mod.rs              -- Db struct, open(), init schema, WAL mode
├── environments.rs     -- CRUD for environments + environment_servers
├── bookmarks.rs        -- add/remove/list bookmarks
├── agents.rs           -- agent state + dismissed agents
├── active_agents.rs    -- mark_active / load_active (rewrite from redb)
└── logs.rs             -- write_log / read_recent_logs (rewrite from redb)
```

### Db struct

```rust
pub struct Db {
    pub conn: rusqlite::Connection,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Self::init_schema(&conn)?;
        Ok(Self { conn })
    }
}
```

Wrapped in `Arc<Mutex<Db>>` for thread-safe access from async handlers. SQLite in WAL mode supports concurrent reads, and the mutex serializes writes. This is sufficient for plugmux's workload.

---

## 6. Config Changes

### Remove from Config struct
- `environments: Vec<Environment>`

### Add to Config struct
- `device_id: String` (generated on first launch if missing)
- `onboarding_shown: bool`

### Environment struct moves to db module
The `Environment` struct and related functions (`add_environment`, `find_environment`, etc.) move from `config.rs` to `db/environments.rs`, operating on SQLite instead of in-memory Vec.

---

## 7. Code Changes Summary

### Cargo.toml (plugmux-core)
- Remove: `redb`
- Add: `rusqlite = { version = "0.34", features = ["bundled"] }`

### Files to rewrite
- `db/mod.rs` — new Db struct with SQLite connection
- `db/logs.rs` — SQL queries instead of redb table ops
- `db/active_agents.rs` — SQL queries instead of redb table ops

### Files to create
- `db/environments.rs` — environment + server assignment CRUD
- `db/bookmarks.rs` — bookmark CRUD
- `db/agents.rs` — agent state CRUD (replaces agents/state.rs)

### Files to modify
- `config.rs` — remove Environment from Config, add device_id + onboarding_shown
- `environment.rs` — rewrite to use Db instead of Config
- `agents/state.rs` — remove (replaced by db/agents.rs)
- `plugmux_layer/mod.rs` — update handlers to query SQLite
- `gateway/router.rs` — pass Db to handlers (already does this)
- All consumers of environments/agents that currently read from config or agents_state.json

### Files to delete
- agents/state.rs (logic moves to db/agents.rs)

---

## 8. No Migration

No production users exist yet. On update:
- Old redb file and agents_state.json are simply ignored
- Fresh SQLite database is created
- User re-configures (minimal friction since no production installs)

---

## 9. Testing

Each db module gets unit tests using a temporary in-memory SQLite database (`Connection::open_in_memory()`). This is fast, isolated, and doesn't touch disk.

---

## 10. Downstream Impact on Cloud Sync

When the cloud backend ships, sync becomes:
1. Pull: fetch changed rows from API, upsert into local SQLite
2. Push: read local changes, POST to API
3. Conflict resolution: server wins (last-write-wins with timestamps)

Same SQL schema on both sides. No format translation needed.
