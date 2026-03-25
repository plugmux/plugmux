//! Log entry storage (SQLite).

use super::Db;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: String,
    pub env_id: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_info: Option<AgentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub session_id: String,
}

impl LogEntry {
    pub fn summarize_value(value: &serde_json::Value) -> Option<String> {
        let s = serde_json::to_string(value).ok()?;
        if s.len() > 2048 {
            let boundary = s.floor_char_boundary(2048);
            Some(format!("{}...", &s[..boundary]))
        } else {
            Some(s)
        }
    }
}

pub fn write_log(db: &Arc<Db>, entry: &LogEntry) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    let (user_agent, agent_id, session_id) = match &entry.agent_info {
        Some(info) => (
            info.user_agent.as_deref(),
            info.agent_id.as_deref(),
            Some(info.session_id.as_str()),
        ),
        None => (None, None, None),
    };
    conn.execute(
        "INSERT INTO logs (id, timestamp, env_id, method, params_summary, result_summary, error, duration_ms, user_agent, agent_id, session_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            entry.id,
            entry.timestamp,
            entry.env_id,
            entry.method,
            entry.params_summary,
            entry.result_summary,
            entry.error,
            entry.duration_ms as i64,
            user_agent,
            agent_id,
            session_id,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn read_recent_logs(db: &Arc<Db>, limit: usize) -> Result<Vec<LogEntry>, String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, env_id, method, params_summary, result_summary, error, duration_ms, user_agent, agent_id, session_id
             FROM logs ORDER BY timestamp DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let entries = stmt
        .query_map(rusqlite::params![limit as i64], |row| {
            let user_agent: Option<String> = row.get(8)?;
            let agent_id: Option<String> = row.get(9)?;
            let session_id: Option<String> = row.get(10)?;
            let agent_info = session_id.map(|sid| AgentInfo {
                user_agent,
                agent_id,
                session_id: sid,
            });
            Ok(LogEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                env_id: row.get(2)?,
                method: row.get(3)?,
                params_summary: row.get(4)?,
                result_summary: row.get(5)?,
                error: row.get(6)?,
                duration_ms: row.get::<_, i64>(7)? as u64,
                agent_info,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    fn make_entry(id: &str, timestamp: &str) -> LogEntry {
        LogEntry {
            id: id.to_string(),
            timestamp: timestamp.to_string(),
            env_id: "global".to_string(),
            method: "tools/call".to_string(),
            params_summary: Some(r#"{"tool":"test"}"#.to_string()),
            result_summary: Some(r#"{"ok":true}"#.to_string()),
            error: None,
            duration_ms: 42,
            agent_info: Some(AgentInfo {
                user_agent: Some("test-agent/1.0".to_string()),
                agent_id: Some("agent-abc".to_string()),
                session_id: "session-123".to_string(),
            }),
        }
    }

    #[test]
    fn test_write_and_read_log() {
        let db = Db::open_in_memory().unwrap();
        let entry = make_entry("test-1", "2026-03-25T10:00:00Z");
        write_log(&db, &entry).unwrap();

        let logs = read_recent_logs(&db, 10).unwrap();
        assert_eq!(logs.len(), 1);

        let got = &logs[0];
        assert_eq!(got.id, "test-1");
        assert_eq!(got.timestamp, "2026-03-25T10:00:00Z");
        assert_eq!(got.env_id, "global");
        assert_eq!(got.method, "tools/call");
        assert_eq!(got.params_summary.as_deref(), Some(r#"{"tool":"test"}"#));
        assert_eq!(got.result_summary.as_deref(), Some(r#"{"ok":true}"#));
        assert_eq!(got.error, None);
        assert_eq!(got.duration_ms, 42);

        let info = got.agent_info.as_ref().unwrap();
        assert_eq!(info.user_agent.as_deref(), Some("test-agent/1.0"));
        assert_eq!(info.agent_id.as_deref(), Some("agent-abc"));
        assert_eq!(info.session_id, "session-123");
    }

    #[test]
    fn test_recent_logs_limit() {
        let db = Db::open_in_memory().unwrap();

        for i in 1..=5 {
            let ts = format!("2026-03-25T10:00:0{}Z", i);
            let id = format!("test-{}", i);
            write_log(&db, &make_entry(&id, &ts)).unwrap();
        }

        let logs = read_recent_logs(&db, 3).unwrap();
        assert_eq!(logs.len(), 3);
    }

    #[test]
    fn test_summarize_value_truncates() {
        // Build a JSON string value longer than 2048 chars.
        let big_string = "x".repeat(6000);
        let value = serde_json::json!({ "data": big_string });

        let summary = LogEntry::summarize_value(&value).unwrap();
        // Must be truncated and end with "..."
        assert!(summary.ends_with("..."), "expected summary to end with '...'");
        // The raw serialised form is >2048, so the summary must be shorter.
        let raw = serde_json::to_string(&value).unwrap();
        assert!(summary.len() < raw.len());
        // The non-truncated prefix plus "..." should be at most 2048 + 3 chars.
        assert!(summary.len() <= 2048 + 3);
    }
}
