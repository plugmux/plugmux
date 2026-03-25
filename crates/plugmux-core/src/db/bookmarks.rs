//! Bookmark CRUD — implemented in Task 4

use std::sync::Arc;

use crate::db::Db;

pub fn add_bookmark(db: &Arc<Db>, server_id: &str) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO bookmarks (server_id) VALUES (?1)",
        rusqlite::params![server_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn remove_bookmark(db: &Arc<Db>, server_id: &str) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM bookmarks WHERE server_id = ?1",
        rusqlite::params![server_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_bookmarks(db: &Arc<Db>) -> Vec<String> {
    let conn = db.conn.lock().expect("db lock poisoned");
    let mut stmt = conn
        .prepare("SELECT server_id FROM bookmarks ORDER BY created_at ASC")
        .expect("prepare failed");
    stmt.query_map([], |row| row.get::<_, String>(0))
        .expect("query failed")
        .filter_map(|r| r.ok())
        .collect()
}

pub fn is_bookmarked(db: &Arc<Db>, server_id: &str) -> bool {
    let conn = db.conn.lock().expect("db lock poisoned");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM bookmarks WHERE server_id = ?1",
            rusqlite::params![server_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    count > 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn test_add_and_list_bookmarks() {
        let db = Db::open_in_memory().unwrap();
        add_bookmark(&db, "figma").unwrap();
        add_bookmark(&db, "github").unwrap();
        let bookmarks = list_bookmarks(&db);
        assert_eq!(bookmarks.len(), 2);
        assert!(bookmarks.contains(&"figma".to_string()));
        assert!(bookmarks.contains(&"github".to_string()));
    }

    #[test]
    fn test_add_bookmark_idempotent() {
        let db = Db::open_in_memory().unwrap();
        add_bookmark(&db, "figma").unwrap();
        add_bookmark(&db, "figma").unwrap();
        let bookmarks = list_bookmarks(&db);
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0], "figma");
    }

    #[test]
    fn test_remove_bookmark() {
        let db = Db::open_in_memory().unwrap();
        add_bookmark(&db, "figma").unwrap();
        remove_bookmark(&db, "figma").unwrap();
        let bookmarks = list_bookmarks(&db);
        assert!(bookmarks.is_empty());
    }

    #[test]
    fn test_is_bookmarked() {
        let db = Db::open_in_memory().unwrap();
        assert!(!is_bookmarked(&db, "figma"));
        add_bookmark(&db, "figma").unwrap();
        assert!(is_bookmarked(&db, "figma"));
    }
}
