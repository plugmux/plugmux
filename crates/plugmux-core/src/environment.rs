use std::sync::Arc;

use crate::db::Db;
use crate::db::environments;

/// Get the list of server IDs for an environment.
/// Returns `None` if the query fails (e.g. environment does not exist).
pub fn get_server_ids(db: &Arc<Db>, env_id: &str) -> Option<Vec<String>> {
    environments::get_server_ids(db, env_id).ok()
}

/// Add a server ID to an environment (idempotent).
pub fn add_server(db: &Arc<Db>, env_id: &str, server_id: &str) -> Result<(), String> {
    environments::add_server(db, env_id, server_id)
}

/// Remove a server ID from an environment.
/// Returns `Ok(true)` on success.
pub fn remove_server(db: &Arc<Db>, env_id: &str, server_id: &str) -> Result<bool, String> {
    environments::remove_server(db, env_id, server_id)?;
    Ok(true)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn test_get_server_ids_returns_ids_for_environment() {
        let db = Db::open_in_memory().unwrap();
        environments::add_server(&db, "global", "filesystem").unwrap();
        environments::add_server(&db, "global", "github").unwrap();

        let ids = get_server_ids(&db, "global").expect("global should exist");
        assert!(ids.contains(&"filesystem".to_string()));
        assert!(ids.contains(&"github".to_string()));
    }

    #[test]
    fn test_add_server_adds_id_to_environment() {
        let db = Db::open_in_memory().unwrap();
        add_server(&db, "global", "postgres").unwrap();
        let ids = get_server_ids(&db, "global").unwrap();
        assert!(ids.contains(&"postgres".to_string()));
    }

    #[test]
    fn test_add_server_idempotent() {
        let db = Db::open_in_memory().unwrap();
        add_server(&db, "global", "filesystem").unwrap();
        add_server(&db, "global", "filesystem").unwrap();
        let ids = get_server_ids(&db, "global").unwrap();
        assert_eq!(ids.iter().filter(|s| *s == "filesystem").count(), 1);
    }

    #[test]
    fn test_remove_server_removes_id() {
        let db = Db::open_in_memory().unwrap();
        add_server(&db, "global", "github").unwrap();
        let removed = remove_server(&db, "global", "github").unwrap();
        assert!(removed);
        let ids = get_server_ids(&db, "global").unwrap();
        assert!(!ids.contains(&"github".to_string()));
    }
}
