//! Tracks which agents have made at least one MCP call through the gateway.
//! TODO: Rewrite with SQLite in Task 7

use super::Db;
use std::collections::HashSet;
use std::sync::Arc;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_active_returns_true_for_new() {
        let db = Db::open_in_memory().unwrap();
        let result = mark_active(&db, "agent-1").unwrap();
        assert!(result, "first mark_active call should return true");
    }

    #[test]
    fn test_mark_active_returns_false_for_existing() {
        let db = Db::open_in_memory().unwrap();
        let first = mark_active(&db, "agent-1").unwrap();
        let second = mark_active(&db, "agent-1").unwrap();
        assert!(first, "first call should return true");
        assert!(!second, "second call with same id should return false");
    }

    #[test]
    fn test_load_active_returns_all() {
        let db = Db::open_in_memory().unwrap();
        mark_active(&db, "agent-a").unwrap();
        mark_active(&db, "agent-b").unwrap();
        let active = load_active(&db).unwrap();
        assert_eq!(active.len(), 2);
        assert!(active.contains("agent-a"));
        assert!(active.contains("agent-b"));
    }
}
