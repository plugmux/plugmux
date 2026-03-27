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
        Ok(Arc::new(Self {
            conn: Mutex::new(conn),
        }))
    }

    pub fn open_in_memory() -> Result<Arc<Self>, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        Self::init_schema(&conn)?;
        Ok(Arc::new(Self {
            conn: Mutex::new(conn),
        }))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory_creates_schema() {
        let db = Db::open_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let name: String = conn
            .query_row(
                "SELECT name FROM environments WHERE id = 'global'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(name, "Global");
    }

    #[test]
    fn test_open_in_memory_creates_all_tables() {
        let db = Db::open_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let tables = [
            "environments",
            "environment_servers",
            "bookmarks",
            "agents",
            "dismissed_agents",
            "logs",
            "active_agents",
        ];
        for table in tables {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                .unwrap();
            assert!(count >= 0, "table {table} should exist");
        }
    }
}
