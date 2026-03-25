//! Agent state CRUD — implemented in Task 5

use std::sync::Arc;

use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::Db;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentSource {
    Auto,
    Registry,
    Custom,
}

impl AgentSource {
    fn as_str(&self) -> &'static str {
        match self {
            AgentSource::Auto => "auto",
            AgentSource::Registry => "registry",
            AgentSource::Custom => "custom",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "registry" => AgentSource::Registry,
            "custom" => AgentSource::Custom,
            _ => AgentSource::Auto,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateEntry {
    pub id: String,
    pub source: AgentSource,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub config_path: Option<String>,
    pub config_format: Option<String>,
    pub mcp_key: Option<String>,
}

/// Insert or replace an agent record (upsert semantics).
pub fn add_agent(db: &Arc<Db>, entry: &AgentStateEntry) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO agents
            (id, source, name, icon, config_path, config_format, mcp_key)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            entry.id,
            entry.source.as_str(),
            entry.name,
            entry.icon,
            entry.config_path,
            entry.config_format,
            entry.mcp_key,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Delete an agent by id.
pub fn remove_agent(db: &Arc<Db>, id: &str) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute("DELETE FROM agents WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Fetch a single agent by id, returning `None` if not found.
pub fn get_agent(db: &Arc<Db>, id: &str) -> Option<AgentStateEntry> {
    let conn = db.conn.lock().unwrap();
    conn.query_row(
        "SELECT id, source, name, icon, config_path, config_format, mcp_key
         FROM agents WHERE id = ?1",
        params![id],
        row_to_entry,
    )
    .ok()
}

/// Return all agents.
pub fn list_agents(db: &Arc<Db>) -> Vec<AgentStateEntry> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = match conn.prepare(
        "SELECT id, source, name, icon, config_path, config_format, mcp_key FROM agents",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    match stmt.query_map([], row_to_entry) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => vec![],
    }
}

/// Remove the agent from `agents` and record it in `dismissed_agents`.
pub fn dismiss_agent(db: &Arc<Db>, id: &str) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute("DELETE FROM agents WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO dismissed_agents (agent_id) VALUES (?1)",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Returns `true` if the agent id appears in `dismissed_agents`.
pub fn is_dismissed(db: &Arc<Db>, id: &str) -> bool {
    let conn = db.conn.lock().unwrap();
    conn.query_row(
        "SELECT 1 FROM dismissed_agents WHERE agent_id = ?1",
        params![id],
        |_| Ok(true),
    )
    .unwrap_or(false)
}

/// Return all dismissed agent ids.
pub fn list_dismissed(db: &Arc<Db>) -> Vec<String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = match conn.prepare("SELECT agent_id FROM dismissed_agents") {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    match stmt.query_map([], |row| row.get(0)) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => vec![],
    }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentStateEntry> {
    let source_str: String = row.get(1)?;
    Ok(AgentStateEntry {
        id: row.get(0)?,
        source: AgentSource::from_str(&source_str),
        name: row.get(2)?,
        icon: row.get(3)?,
        config_path: row.get(4)?,
        config_format: row.get(5)?,
        mcp_key: row.get(6)?,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(id: &str) -> AgentStateEntry {
        AgentStateEntry {
            id: id.to_string(),
            source: AgentSource::Auto,
            name: Some("Test Agent".to_string()),
            icon: Some("icon.png".to_string()),
            config_path: Some("/etc/agent.toml".to_string()),
            config_format: Some("toml".to_string()),
            mcp_key: Some("key-abc".to_string()),
        }
    }

    #[test]
    fn test_add_and_get_agent() {
        let db = Db::open_in_memory().unwrap();
        let entry = sample_entry("agent-1");
        add_agent(&db, &entry).unwrap();

        let got = get_agent(&db, "agent-1").expect("should exist");
        assert_eq!(got.id, "agent-1");
        assert_eq!(got.source, AgentSource::Auto);
        assert_eq!(got.name.as_deref(), Some("Test Agent"));
        assert_eq!(got.icon.as_deref(), Some("icon.png"));
        assert_eq!(got.config_path.as_deref(), Some("/etc/agent.toml"));
        assert_eq!(got.config_format.as_deref(), Some("toml"));
        assert_eq!(got.mcp_key.as_deref(), Some("key-abc"));
    }

    #[test]
    fn test_add_agent_replaces_existing() {
        let db = Db::open_in_memory().unwrap();
        add_agent(&db, &sample_entry("agent-1")).unwrap();

        let updated = AgentStateEntry {
            id: "agent-1".to_string(),
            source: AgentSource::Registry,
            name: Some("Updated".to_string()),
            icon: None,
            config_path: None,
            config_format: None,
            mcp_key: None,
        };
        add_agent(&db, &updated).unwrap();

        let got = get_agent(&db, "agent-1").expect("should exist");
        assert_eq!(got.source, AgentSource::Registry);
        assert_eq!(got.name.as_deref(), Some("Updated"));
        assert!(got.icon.is_none());

        // Confirm only one row
        assert_eq!(list_agents(&db).len(), 1);
    }

    #[test]
    fn test_remove_agent() {
        let db = Db::open_in_memory().unwrap();
        add_agent(&db, &sample_entry("agent-1")).unwrap();
        remove_agent(&db, "agent-1").unwrap();
        assert!(get_agent(&db, "agent-1").is_none());
    }

    #[test]
    fn test_list_agents() {
        let db = Db::open_in_memory().unwrap();
        add_agent(&db, &sample_entry("agent-1")).unwrap();
        add_agent(&db, &sample_entry("agent-2")).unwrap();

        let agents = list_agents(&db);
        assert_eq!(agents.len(), 2);
        let ids: Vec<&str> = agents.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"agent-1"));
        assert!(ids.contains(&"agent-2"));
    }

    #[test]
    fn test_dismiss_agent_removes_and_adds_to_dismissed() {
        let db = Db::open_in_memory().unwrap();
        add_agent(&db, &sample_entry("agent-1")).unwrap();

        dismiss_agent(&db, "agent-1").unwrap();

        assert!(get_agent(&db, "agent-1").is_none(), "should be removed from agents");
        assert!(list_dismissed(&db).contains(&"agent-1".to_string()));
    }

    #[test]
    fn test_is_dismissed() {
        let db = Db::open_in_memory().unwrap();
        add_agent(&db, &sample_entry("agent-1")).unwrap();

        assert!(!is_dismissed(&db, "agent-1"), "not dismissed yet");
        dismiss_agent(&db, "agent-1").unwrap();
        assert!(is_dismissed(&db, "agent-1"), "should be dismissed");
    }

    #[test]
    fn test_dismiss_is_idempotent() {
        let db = Db::open_in_memory().unwrap();
        add_agent(&db, &sample_entry("agent-1")).unwrap();

        dismiss_agent(&db, "agent-1").unwrap();
        // Second dismiss — agent no longer in agents table, but should not panic
        dismiss_agent(&db, "agent-1").unwrap();

        assert!(is_dismissed(&db, "agent-1"));
        assert_eq!(list_dismissed(&db).len(), 1, "no duplicates in dismissed_agents");
    }
}
