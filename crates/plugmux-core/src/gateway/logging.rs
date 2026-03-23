//! Request/response logging to embedded DB.

use crate::db::Db;
use crate::db::logs::{AgentInfo, LogEntry, write_log};
use serde_json::Value;
use std::sync::Arc;

/// Bundles the arguments for [`log_request`] to stay under the clippy argument limit.
pub struct LogRequestParams<'a> {
    pub db: &'a Arc<Db>,
    pub env_id: &'a str,
    pub method: &'a str,
    pub params: &'a Value,
    pub result: &'a Result<Value, String>,
    pub duration: std::time::Duration,
    pub user_agent: Option<&'a str>,
    pub agent_id: Option<&'a str>,
    pub session_id: &'a str,
}

pub fn log_request(p: &LogRequestParams<'_>) {
    let entry = LogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        env_id: p.env_id.to_string(),
        method: p.method.to_string(),
        params_summary: LogEntry::summarize_value(p.params),
        result_summary: match p.result {
            Ok(v) => LogEntry::summarize_value(v),
            Err(_) => None,
        },
        error: match p.result {
            Err(e) => Some(e.clone()),
            Ok(_) => None,
        },
        duration_ms: p.duration.as_millis() as u64,
        agent_info: Some(AgentInfo {
            user_agent: p.user_agent.map(String::from),
            agent_id: p.agent_id.map(String::from),
            session_id: p.session_id.to_string(),
        }),
    };

    if let Err(e) = write_log(p.db, &entry) {
        tracing::warn!(error = %e, "failed to write log entry");
    }
}
