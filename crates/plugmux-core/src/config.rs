use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::slug::slugify;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Environment not found: {0}")]
    EnvironmentNotFound(String),
    #[error("Cannot delete the default environment")]
    CannotDeleteDefault,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub permissions: Permissions,
    pub environments: Vec<Environment>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    /// Server IDs — referencing catalog or custom servers.
    pub servers: Vec<String>,
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

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
// Bootstrap
// ---------------------------------------------------------------------------

/// Ensures a "default" environment exists in `config`. Adds one if missing.
pub fn ensure_default(config: &mut Config) {
    if !config.environments.iter().any(|e| e.id == "default") {
        config.environments.insert(
            0,
            Environment {
                id: "default".to_string(),
                name: "Default".to_string(),
                servers: Vec::new(),
            },
        );
    }
}

// ---------------------------------------------------------------------------
// Load / Save
// ---------------------------------------------------------------------------

/// Loads config from `path`, ensuring a default environment exists.
pub fn load(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let mut config: Config = serde_json::from_str(&content)?;
    ensure_default(&mut config);
    Ok(config)
}

/// Loads config from `path`, or returns a fresh config with an empty default
/// environment if the file does not exist.
pub fn load_or_default(path: &Path) -> Config {
    match std::fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<Config>(&content) {
            Ok(mut config) => {
                ensure_default(&mut config);
                config
            }
            Err(_) => default_config(),
        },
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
        environments: vec![Environment {
            id: "default".to_string(),
            name: "Default".to_string(),
            servers: Vec::new(),
        }],
    }
}

// ---------------------------------------------------------------------------
// Environment management
// ---------------------------------------------------------------------------

/// Adds a new environment to `config` with a slug ID derived from `name`.
/// Returns a mutable reference to the new environment.
pub fn add_environment<'a>(config: &'a mut Config, name: &str) -> &'a mut Environment {
    let id = slugify(name);
    let env = Environment {
        id,
        name: name.to_string(),
        servers: Vec::new(),
    };
    config.environments.push(env);
    config.environments.last_mut().unwrap()
}

/// Finds an environment by id, returning an immutable reference.
pub fn find_environment<'a>(config: &'a Config, id: &str) -> Option<&'a Environment> {
    config.environments.iter().find(|e| e.id == id)
}

/// Finds an environment by id, returning a mutable reference.
pub fn find_environment_mut<'a>(config: &'a mut Config, id: &str) -> Option<&'a mut Environment> {
    config.environments.iter_mut().find(|e| e.id == id)
}

/// Removes an environment by id.
/// Returns `Err(CannotDeleteDefault)` if `id == "default"`.
/// Returns `Err(EnvironmentNotFound)` if the id does not exist.
pub fn remove_environment(config: &mut Config, id: &str) -> Result<(), ConfigError> {
    if id == "default" {
        return Err(ConfigError::CannotDeleteDefault);
    }
    let before = config.environments.len();
    config.environments.retain(|e| e.id != id);
    if config.environments.len() == before {
        return Err(ConfigError::EnvironmentNotFound(id.to_string()));
    }
    Ok(())
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
    // Load with port, permissions, string server IDs
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
            "environments": [
                {
                    "id": "default",
                    "name": "Default",
                    "servers": ["filesystem", "github"]
                }
            ]
        }
        "#;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, json).unwrap();

        let cfg = load(&path).unwrap();

        assert_eq!(cfg.port, 4000);
        assert_eq!(cfg.permissions.enable_server, PermissionLevel::Allow);
        assert_eq!(cfg.permissions.disable_server, PermissionLevel::Approve);
        assert_eq!(cfg.environments.len(), 1);
        assert_eq!(cfg.environments[0].servers, vec!["filesystem", "github"]);
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
        find_environment_mut(&mut cfg, "default")
            .unwrap()
            .servers
            .push("my-server".to_string());

        save(&path, &cfg).unwrap();

        let loaded = load(&path).unwrap();
        assert_eq!(loaded.port, 8080);
        assert_eq!(loaded.environments[0].servers, vec!["my-server"]);
    }

    // -----------------------------------------------------------------------
    // Default environment bootstrap (missing default gets auto-created)
    // -----------------------------------------------------------------------

    #[test]
    fn test_ensure_default_creates_missing_default() {
        let json = r#"
        {
            "port": 4242,
            "environments": [
                {
                    "id": "work",
                    "name": "Work",
                    "servers": []
                }
            ]
        }
        "#;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, json).unwrap();

        let cfg = load(&path).unwrap();

        // default env should have been injected
        let default_env = find_environment(&cfg, "default");
        assert!(default_env.is_some(), "default environment should be auto-created");
        assert_eq!(default_env.unwrap().name, "Default");
    }

    #[test]
    fn test_load_or_default_creates_default_env_when_file_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.json");

        let cfg = load_or_default(&path);
        assert!(find_environment(&cfg, "default").is_some());
        assert_eq!(cfg.port, 4242);
    }

    // -----------------------------------------------------------------------
    // delete_environment("default") returns error
    // -----------------------------------------------------------------------

    #[test]
    fn test_delete_default_environment_returns_error() {
        let mut cfg = default_config();
        let result = remove_environment(&mut cfg, "default");
        assert!(
            matches!(result, Err(ConfigError::CannotDeleteDefault)),
            "expected CannotDeleteDefault error"
        );
        // environment was not removed
        assert!(find_environment(&cfg, "default").is_some());
    }

    // -----------------------------------------------------------------------
    // add_environment creates slug ID
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_environment_creates_slug_id() {
        let mut cfg = default_config();
        let env = add_environment(&mut cfg, "My Work Project");

        assert_eq!(env.id, "my-work-project");
        assert_eq!(env.name, "My Work Project");
        assert!(env.servers.is_empty());
    }

    #[test]
    fn test_add_environment_no_endpoint() {
        // New model has no endpoint field — ensure it serializes cleanly
        let mut cfg = default_config();
        add_environment(&mut cfg, "Personal");
        let serialized = serde_json::to_string(&cfg).unwrap();
        assert!(!serialized.contains("endpoint"), "Environment must not have an endpoint field");
    }

    // -----------------------------------------------------------------------
    // find_environment_mut
    // -----------------------------------------------------------------------

    #[test]
    fn test_find_environment_mut() {
        let mut cfg = default_config();
        add_environment(&mut cfg, "Staging");

        let env = find_environment_mut(&mut cfg, "staging").unwrap();
        env.servers.push("redis".to_string());

        assert_eq!(find_environment(&cfg, "staging").unwrap().servers, vec!["redis"]);
    }

    // -----------------------------------------------------------------------
    // remove_environment non-default
    // -----------------------------------------------------------------------

    #[test]
    fn test_remove_environment_non_default() {
        let mut cfg = default_config();
        add_environment(&mut cfg, "Staging");

        remove_environment(&mut cfg, "staging").unwrap();
        assert!(find_environment(&cfg, "staging").is_none());
    }

    #[test]
    fn test_remove_environment_not_found_returns_error() {
        let mut cfg = default_config();
        let result = remove_environment(&mut cfg, "nonexistent");
        assert!(matches!(result, Err(ConfigError::EnvironmentNotFound(_))));
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
}
