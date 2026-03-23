//! Log entry storage.

use super::Db;
use redb::{ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

pub const LOGS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("logs");

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
    pub fn summarize_value(value: &Value) -> Option<String> {
        let s = serde_json::to_string(value).ok()?;
        if s.len() > 2048 {
            Some(format!("{}...", &s[..2048]))
        } else {
            Some(s)
        }
    }
}

pub fn write_log(db: &Arc<Db>, entry: &LogEntry) -> Result<(), Box<redb::Error>> {
    #[allow(clippy::result_large_err)]
    fn inner(db: &Arc<Db>, entry: &LogEntry) -> Result<(), redb::Error> {
        let json = serde_json::to_string(entry)
            .map_err(|e| redb::Error::Io(std::io::Error::other(e.to_string())))?;
        let write_txn = db.inner.begin_write()?;
        {
            let mut table = write_txn.open_table(LOGS_TABLE)?;
            table.insert(entry.id.as_str(), json.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }
    inner(db, entry).map_err(Box::new)
}

pub fn read_recent_logs(db: &Arc<Db>, limit: usize) -> Result<Vec<LogEntry>, Box<redb::Error>> {
    #[allow(clippy::result_large_err)]
    fn inner(db: &Arc<Db>, limit: usize) -> Result<Vec<LogEntry>, redb::Error> {
        let read_txn = db.inner.begin_read()?;
        let table = read_txn.open_table(LOGS_TABLE)?;
        let mut entries: Vec<LogEntry> = Vec::new();
        for item in table.iter()? {
            let (_, value) = item?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(value.value()) {
                entries.push(entry);
            }
        }
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries.truncate(limit);
        Ok(entries)
    }
    inner(db, limit).map_err(Box::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(id: &str) -> LogEntry {
        LogEntry {
            id: id.to_string(),
            timestamp: "2026-03-23T12:00:00Z".to_string(),
            env_id: "global".to_string(),
            method: "tools/list".to_string(),
            params_summary: None,
            result_summary: None,
            error: None,
            duration_ms: 5,
            agent_info: None,
        }
    }

    #[test]
    fn test_write_and_read_log() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Db::open(&dir.path().join("test.db")).unwrap();
        let entry = make_entry("test-1");
        write_log(&db, &entry).unwrap();
        let logs = read_recent_logs(&db, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].id, "test-1");
    }

    #[test]
    fn test_recent_logs_limit() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Db::open(&dir.path().join("test.db")).unwrap();
        for i in 0..5 {
            write_log(&db, &make_entry(&format!("entry-{i}"))).unwrap();
        }
        let logs = read_recent_logs(&db, 3).unwrap();
        assert_eq!(logs.len(), 3);
    }

    #[test]
    fn test_summarize_value_truncates() {
        let large = serde_json::json!({"data": "x".repeat(5000)});
        let summary = LogEntry::summarize_value(&large).unwrap();
        assert!(summary.len() <= 2051);
    }
}
