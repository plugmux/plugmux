use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Not found: {0}")]
    EnvironmentNotFound(String),
    #[error("ID collision: {0}")]
    IdCollision(String),
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

fn default_port() -> u16 {
    4242
}

fn default_approve() -> PermissionLevel {
    PermissionLevel::Approve
}

fn default_device_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default = "default_device_id")]
    pub device_id: String,
    #[serde(default)]
    pub onboarding_shown: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    #[serde(default = "default_approve")]
    pub enable_server: PermissionLevel,
    #[serde(default = "default_approve")]
    pub disable_server: PermissionLevel,
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            enable_server: PermissionLevel::Approve,
            disable_server: PermissionLevel::Approve,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    Allow,
    Approve,
    Disable,
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// The global environment ID — always present, cannot be deleted.
pub const GLOBAL_ENV: &str = "global";

/// Build the plugmux gateway URL for a given port and environment.
pub fn gateway_url(port: u16, env_id: &str) -> String {
    format!("http://localhost:{port}/env/{env_id}")
}

/// Build the global management endpoint URL for a given port.
pub fn global_url(port: u16) -> String {
    gateway_url(port, GLOBAL_ENV)
}

/// Returns the plugmux config directory: `~/.config/plugmux/`.
pub fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    base.join("plugmux")
}

/// Returns the canonical config path: `~/.config/plugmux/config.json`.
pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

// ---------------------------------------------------------------------------
// Load / Save
// ---------------------------------------------------------------------------

/// Loads config from `path`.
pub fn load(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

/// Loads config from `path`, or returns a fresh default config if the file
/// does not exist or cannot be parsed.
pub fn load_or_default(path: &Path) -> Config {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            serde_json::from_str::<Config>(&content).unwrap_or_else(|_| default_config())
        }
        Err(_) => default_config(),
    }
}

/// Serialises `config` to `path`, creating parent directories as needed.
pub fn save(path: &Path, config: &Config) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}

fn default_config() -> Config {
    Config {
        port: default_port(),
        permissions: Permissions::default(),
        device_id: default_device_id(),
        onboarding_shown: false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // -----------------------------------------------------------------------
    // config_path
    // -----------------------------------------------------------------------

    #[test]
    fn test_config_path_ends_with_config_json() {
        let path = config_path();
        assert!(
            path.to_str().unwrap().ends_with("plugmux/config.json"),
            "expected path ending in plugmux/config.json, got: {}",
            path.display()
        );
    }

    // -----------------------------------------------------------------------
    // Load with port, permissions
    // -----------------------------------------------------------------------

    #[test]
    fn test_load_config_json() {
        let json = r#"
        {
            "port": 4000,
            "permissions": {
                "enable_server": "allow",
                "disable_server": "approve"
            },
            "device_id": "test-device-123"
        }
        "#;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, json).unwrap();

        let cfg = load(&path).unwrap();

        assert_eq!(cfg.port, 4000);
        assert_eq!(cfg.permissions.enable_server, PermissionLevel::Allow);
        assert_eq!(cfg.permissions.disable_server, PermissionLevel::Approve);
        assert_eq!(cfg.device_id, "test-device-123");
        assert!(!cfg.onboarding_shown);
    }

    // -----------------------------------------------------------------------
    // Save and reload roundtrip
    // -----------------------------------------------------------------------

    #[test]
    fn test_save_and_reload_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");

        let mut cfg = default_config();
        cfg.port = 8080;

        save(&path, &cfg).unwrap();

        let loaded = load(&path).unwrap();
        assert_eq!(loaded.port, 8080);
        assert_eq!(loaded.device_id, cfg.device_id);
    }

    // -----------------------------------------------------------------------
    // load_or_default
    // -----------------------------------------------------------------------

    #[test]
    fn test_load_or_default_returns_default_when_file_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.json");

        let cfg = load_or_default(&path);
        assert_eq!(cfg.port, 4242);
        assert!(!cfg.device_id.is_empty());
    }

    // -----------------------------------------------------------------------
    // Permissions defaults
    // -----------------------------------------------------------------------

    #[test]
    fn test_permissions_default_to_approve() {
        let cfg = default_config();
        assert_eq!(cfg.permissions.enable_server, PermissionLevel::Approve);
        assert_eq!(cfg.permissions.disable_server, PermissionLevel::Approve);
    }

    #[test]
    fn test_port_defaults_to_4242() {
        let cfg = default_config();
        assert_eq!(cfg.port, 4242);
    }

    // -----------------------------------------------------------------------
    // device_id generated on default
    // -----------------------------------------------------------------------

    #[test]
    fn test_device_id_generated_on_default() {
        let cfg = default_config();
        assert!(!cfg.device_id.is_empty());
        // Should be a valid UUID
        assert!(uuid::Uuid::parse_str(&cfg.device_id).is_ok());
    }

    #[test]
    fn test_device_id_generated_when_missing_from_json() {
        let json = r#"{"port": 4242}"#;
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, json).unwrap();

        let cfg = load(&path).unwrap();
        assert!(!cfg.device_id.is_empty());
    }
}
