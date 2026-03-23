//! Request/response logging to embedded DB.

use std::sync::Arc;
use serde_json::Value;
use crate::db::Db;
use crate::db::logs::{AgentInfo, LogEntry, write_log};

pub fn log_request(
    db: &Arc<Db>,
    env_id: &str,
    method: &str,
    params: &Value,
    result: &Result<Value, String>,
    duration: std::time::Duration,
    user_agent: Option<&str>,
    agent_id: Option<&str>,
    session_id: &str,
) {
    let entry = LogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        env_id: env_id.to_string(),
        method: method.to_string(),
        params_summary: LogEntry::summarize_value(params),
        result_summary: match result {
            Ok(v) => LogEntry::summarize_value(v),
            Err(_) => None,
        },
        error: match result {
            Err(e) => Some(e.clone()),
            Ok(_) => None,
        },
        duration_ms: duration.as_millis() as u64,
        agent_info: Some(AgentInfo {
            user_agent: user_agent.map(String::from),
            agent_id: agent_id.map(String::from),
            session_id: session_id.to_string(),
        }),
    };

    if let Err(e) = write_log(db, &entry) {
        tracing::warn!(error = %e, "failed to write log entry");
    }
}
