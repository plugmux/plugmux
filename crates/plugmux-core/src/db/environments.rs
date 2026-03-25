//! Environment + environment_servers CRUD.

use std::sync::Arc;

use super::Db;

pub struct EnvironmentRow {
    pub id: String,
    pub name: String,
}

/// Return all environments ordered by creation time.
pub fn list_environments(db: &Arc<Db>) -> Vec<EnvironmentRow> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, name FROM environments ORDER BY created_at ASC")
        .expect("prepare list_environments");
    stmt.query_map([], |row| Ok(EnvironmentRow { id: row.get(0)?, name: row.get(1)? }))
        .expect("query list_environments")
        .filter_map(|r| r.ok())
        .collect()
}

/// Insert a new environment. Returns an error if the id already exists.
pub fn add_environment(db: &Arc<Db>, id: &str, name: &str) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO environments (id, name) VALUES (?1, ?2)",
        rusqlite::params![id, name],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Delete an environment. The "global" environment cannot be removed.
pub fn remove_environment(db: &Arc<Db>, id: &str) -> Result<(), String> {
    if id == "global" {
        return Err("cannot remove the global environment".to_string());
    }
    let conn = db.conn.lock().unwrap();
    conn.execute("DELETE FROM environments WHERE id = ?1", rusqlite::params![id])
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Return the server ids assigned to the given environment.
pub fn get_server_ids(db: &Arc<Db>, env_id: &str) -> Result<Vec<String>, String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT server_id FROM environment_servers WHERE env_id = ?1 ORDER BY server_id ASC")
        .map_err(|e| e.to_string())?;
    let ids = stmt
        .query_map(rusqlite::params![env_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(ids)
}

/// Assign a server to an environment. Idempotent — duplicate inserts are ignored.
pub fn add_server(db: &Arc<Db>, env_id: &str, server_id: &str) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT OR IGNORE INTO environment_servers (env_id, server_id) VALUES (?1, ?2)",
        rusqlite::params![env_id, server_id],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Remove a server assignment from an environment.
pub fn remove_server(db: &Arc<Db>, env_id: &str, server_id: &str) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "DELETE FROM environment_servers WHERE env_id = ?1 AND server_id = ?2",
        rusqlite::params![env_id, server_id],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Return deduplicated server ids across all environments.
pub fn get_all_server_ids(db: &Arc<Db>) -> Vec<String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT DISTINCT server_id FROM environment_servers ORDER BY server_id ASC")
        .expect("prepare get_all_server_ids");
    stmt.query_map([], |row| row.get(0))
        .expect("query get_all_server_ids")
        .filter_map(|r| r.ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn test_list_environments_has_global() {
        let db = Db::open_in_memory().unwrap();
        let envs = list_environments(&db);
        assert_eq!(envs.len(), 1);
        assert_eq!(envs[0].id, "global");
        assert_eq!(envs[0].name, "Global");
    }

    #[test]
    fn test_add_and_list_environment() {
        let db = Db::open_in_memory().unwrap();
        add_environment(&db, "work", "Work").unwrap();
        let envs = list_environments(&db);
        assert_eq!(envs.len(), 2);
        let ids: Vec<&str> = envs.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"global"));
        assert!(ids.contains(&"work"));
    }

    #[test]
    fn test_remove_environment() {
        let db = Db::open_in_memory().unwrap();
        add_environment(&db, "work", "Work").unwrap();
        remove_environment(&db, "work").unwrap();
        let envs = list_environments(&db);
        assert_eq!(envs.len(), 1);
        assert_eq!(envs[0].id, "global");
    }

    #[test]
    fn test_cannot_remove_global() {
        let db = Db::open_in_memory().unwrap();
        let result = remove_environment(&db, "global");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("global"));
    }

    #[test]
    fn test_add_and_get_server_ids() {
        let db = Db::open_in_memory().unwrap();
        add_server(&db, "global", "figma").unwrap();
        add_server(&db, "global", "github").unwrap();
        let mut ids = get_server_ids(&db, "global").unwrap();
        ids.sort();
        assert_eq!(ids, vec!["figma", "github"]);
    }

    #[test]
    fn test_add_server_idempotent() {
        let db = Db::open_in_memory().unwrap();
        add_server(&db, "global", "figma").unwrap();
        add_server(&db, "global", "figma").unwrap();
        let ids = get_server_ids(&db, "global").unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "figma");
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
        add_server(&db, "global", "github").unwrap();
        add_server(&db, "work", "github").unwrap(); // duplicate across envs
        add_server(&db, "work", "slack").unwrap();
        let mut ids = get_all_server_ids(&db);
        ids.sort();
        assert_eq!(ids, vec!["figma", "github", "slack"]);
    }
}
