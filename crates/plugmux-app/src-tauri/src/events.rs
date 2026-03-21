use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatusPayload {
    pub status: String, // "running", "stopped", "conflict"
}

#[allow(dead_code)] // Used when health checker emits events (wired in future)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHealthPayload {
    pub server_id: String,
    pub healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerChangedPayload {
    pub server_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerToggledPayload {
    pub server_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_id: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentChangedPayload {
    pub env_id: String,
}

// Event name constants
pub const ENGINE_STATUS_CHANGED: &str = "engine_status_changed";
#[allow(dead_code)] // Used when health checker emits events (wired in future)
pub const SERVER_HEALTH_CHANGED: &str = "server_health_changed";
pub const SERVER_ADDED: &str = "server_added";
pub const SERVER_REMOVED: &str = "server_removed";
pub const SERVER_TOGGLED: &str = "server_toggled";
pub const ENVIRONMENT_CREATED: &str = "environment_created";
pub const ENVIRONMENT_DELETED: &str = "environment_deleted";
pub const CONFIG_RELOADED: &str = "config_reloaded";
