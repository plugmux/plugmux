//! Tracks which agents have made at least one MCP call through the gateway.

use super::Db;
use redb::{ReadableTable, TableDefinition};
use std::collections::HashSet;
use std::sync::Arc;

pub const ACTIVE_AGENTS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("active_agents");

/// Record that an agent has been seen. Returns `true` if this is a new entry.
pub fn mark_active(db: &Arc<Db>, agent_id: &str) -> Result<bool, Box<redb::Error>> {
    #[allow(clippy::result_large_err)]
    fn inner(db: &Arc<Db>, agent_id: &str) -> Result<bool, redb::Error> {
        // Check if already exists
        {
            let read_txn = db.inner.begin_read()?;
            let table = read_txn.open_table(ACTIVE_AGENTS_TABLE)?;
            if table.get(agent_id)?.is_some() {
                return Ok(false);
            }
        }
        // Insert
        let write_txn = db.inner.begin_write()?;
        {
            let mut table = write_txn.open_table(ACTIVE_AGENTS_TABLE)?;
            table.insert(agent_id, "")?;
        }
        write_txn.commit()?;
        Ok(true)
    }
    inner(db, agent_id).map_err(Box::new)
}

/// Load all active agent IDs.
pub fn load_active(db: &Arc<Db>) -> Result<HashSet<String>, Box<redb::Error>> {
    #[allow(clippy::result_large_err)]
    fn inner(db: &Arc<Db>) -> Result<HashSet<String>, redb::Error> {
        let read_txn = db.inner.begin_read()?;
        let table = read_txn.open_table(ACTIVE_AGENTS_TABLE)?;
        let mut ids = HashSet::new();
        for item in table.iter()? {
            let (key, _) = item?;
            ids.insert(key.value().to_string());
        }
        Ok(ids)
    }
    inner(db).map_err(Box::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_active_returns_true_for_new() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Db::open(&dir.path().join("test.db")).unwrap();
        assert!(mark_active(&db, "claude-code").unwrap());
    }

    #[test]
    fn test_mark_active_returns_false_for_existing() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Db::open(&dir.path().join("test.db")).unwrap();
        assert!(mark_active(&db, "claude-code").unwrap());
        assert!(!mark_active(&db, "claude-code").unwrap());
    }

    #[test]
    fn test_load_active_returns_all() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Db::open(&dir.path().join("test.db")).unwrap();
        mark_active(&db, "claude-code").unwrap();
        mark_active(&db, "cursor").unwrap();
        let ids = load_active(&db).unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains("claude-code"));
        assert!(ids.contains("cursor"));
    }
}
