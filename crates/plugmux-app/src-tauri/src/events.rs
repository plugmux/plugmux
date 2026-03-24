use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatusPayload {
    pub status: String, // "running", "stopped", "conflict"
}

#[allow(dead_code)] // Wired when health checker emits events to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHealthPayload {
    pub server_id: String,
    pub status: String, // "healthy", "degraded", "unavailable"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerChangedPayload {
    pub server_id: String,
    pub env_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentChangedPayload {
    pub env_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomServerChangedPayload {
    pub server_id: String,
}

// Event name constants
pub const ENGINE_STATUS_CHANGED: &str = "engine_status_changed";
#[allow(dead_code)] // Wired when health checker emits events to frontend
pub const SERVER_HEALTH_CHANGED: &str = "server_health_changed";
pub const SERVER_ADDED: &str = "server_added";
pub const SERVER_REMOVED: &str = "server_removed";
pub const ENVIRONMENT_CREATED: &str = "environment_created";
pub const ENVIRONMENT_DELETED: &str = "environment_deleted";
pub const CONFIG_RELOADED: &str = "config_reloaded";
pub const CUSTOM_SERVER_ADDED: &str = "custom_server_added";
pub const CUSTOM_SERVER_UPDATED: &str = "custom_server_updated";
pub const CUSTOM_SERVER_REMOVED: &str = "custom_server_removed";
pub const AGENT_ACTIVITY: &str = "agent_activity";
pub const LOG_ADDED: &str = "log_added";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentActivityPayload {
    pub agent_id: String,
    /// True when this agent is seen for the first time this session.
    pub is_new: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAddedPayload {
    pub agent_id: Option<String>,
    pub method: String,
    pub env_id: String,
    pub duration_ms: u64,
    pub error: Option<String>,
}
