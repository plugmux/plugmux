//! Tracks which agents have made at least one MCP call through the gateway.
//! TODO: Rewrite with SQLite in Task 7

use std::collections::HashSet;
use std::sync::Arc;
use super::Db;

pub fn mark_active(db: &Arc<Db>, agent_id: &str) -> Result<bool, String> {
    let conn = db.conn.lock().unwrap();
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM active_agents WHERE agent_id = ?1",
            rusqlite::params![agent_id],
            |_| Ok(true),
        )
        .unwrap_or(false);
    if exists {
        return Ok(false);
    }
    conn.execute(
        "INSERT INTO active_agents (agent_id) VALUES (?1)",
        rusqlite::params![agent_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(true)
}

pub fn load_active(db: &Arc<Db>) -> Result<HashSet<String>, String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT agent_id FROM active_agents")
        .map_err(|e| e.to_string())?;
    let ids = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(ids)
}
