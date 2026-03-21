use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::server::ServerConfig;
use crate::slug::slugify;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Environment not found: {0}")]
    EnvironmentNotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Permission {
    pub allow: Option<Vec<String>>,
    pub deny: Option<Vec<String>>,
}

/// Per-environment override for a specific server (identified by server id).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerOverride {
    pub server_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permission>,
}

/// Top-level "main" section — servers shared across all environments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MainConfig {
    #[serde(default)]
    pub servers: Vec<ServerConfig>,
}

/// A named environment with its own server list and optional overrides.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentConfig {
    pub id: String,
    pub name: String,
    /// The HTTP endpoint that plugmux exposes for this environment.
    pub endpoint: String,
    #[serde(default)]
    pub servers: Vec<ServerConfig>,
    #[serde(default)]
    pub overrides: Vec<ServerOverride>,
}

/// Root config file structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PlugmuxConfig {
    #[serde(default)]
    pub main: MainConfig,
    #[serde(default)]
    pub environments: Vec<EnvironmentConfig>,
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// Returns the canonical config path: `~/.config/plugmux/plugmux.json`.
pub fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    base.join("plugmux").join("plugmux.json")
}

// ---------------------------------------------------------------------------
// CRUD helpers
// ---------------------------------------------------------------------------

/// Creates and returns an empty `PlugmuxConfig`.
pub fn default_config() -> PlugmuxConfig {
    PlugmuxConfig::default()
}

/// Loads config from `path`.
pub fn load(path: &Path) -> Result<PlugmuxConfig, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config = serde_json::from_str(&content)?;
    Ok(config)
}

/// Loads config from `path`, or returns an empty config if the file does not exist.
pub fn load_or_default(path: &Path) -> Result<PlugmuxConfig, ConfigError> {
    match load(path) {
        Ok(cfg) => Ok(cfg),
        Err(ConfigError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(default_config())
        }
        Err(e) => Err(e),
    }
}

/// Serialises `config` to `path`, creating parent directories as needed.
pub fn save(path: &Path, config: &PlugmuxConfig) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Environment management
// ---------------------------------------------------------------------------

/// Adds a new environment to `config` with a slugified id and a default endpoint URL.
/// Returns a mutable reference to the new environment.
pub fn add_environment<'a>(config: &'a mut PlugmuxConfig, name: &str) -> &'a mut EnvironmentConfig {
    let id = slugify(name);
    let endpoint = format!("http://localhost:3000/{id}");
    let env = EnvironmentConfig {
        id,
        name: name.to_string(),
        endpoint,
        servers: Vec::new(),
        overrides: Vec::new(),
    };
    config.environments.push(env);
    config.environments.last_mut().unwrap()
}

/// Finds an environment by id, returning an immutable reference.
pub fn find_environment<'a>(
    config: &'a PlugmuxConfig,
    id: &str,
) -> Option<&'a EnvironmentConfig> {
    config.environments.iter().find(|e| e.id == id)
}

/// Removes an environment by id. Returns `true` if it was found and removed.
pub fn remove_environment(config: &mut PlugmuxConfig, id: &str) -> bool {
    let before = config.environments.len();
    config.environments.retain(|e| e.id != id);
    config.environments.len() < before
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{Connectivity, Transport};
    use tempfile::TempDir;

    fn make_server(id: &str, name: &str) -> ServerConfig {
        ServerConfig {
            id: id.to_string(),
            name: name.to_string(),
            transport: Transport::Stdio,
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "some-server".to_string()]),
            url: None,
            connectivity: Connectivity::Local,
            enabled: true,
            description: None,
        }
    }

    #[test]
    fn test_default_config() {
        let cfg = default_config();
        assert!(cfg.main.servers.is_empty());
        assert!(cfg.environments.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("plugmux.json");

        let mut cfg = default_config();
        cfg.main.servers.push(make_server("s1", "Server One"));
        save(&path, &cfg).unwrap();

        let loaded = load(&path).unwrap();
        assert_eq!(loaded, cfg);
    }

    #[test]
    fn test_add_environment() {
        let mut cfg = default_config();
        let env = add_environment(&mut cfg, "My Project");

        assert_eq!(env.id, "my-project");
        assert_eq!(env.name, "My Project");
        assert_eq!(env.endpoint, "http://localhost:3000/my-project");
        assert_eq!(cfg.environments.len(), 1);
    }

    #[test]
    fn test_find_environment() {
        let mut cfg = default_config();
        add_environment(&mut cfg, "Alpha");
        add_environment(&mut cfg, "Beta");

        let found = find_environment(&cfg, "alpha");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Alpha");

        assert!(find_environment(&cfg, "gamma").is_none());
    }

    #[test]
    fn test_remove_environment() {
        let mut cfg = default_config();
        add_environment(&mut cfg, "Alpha");
        add_environment(&mut cfg, "Beta");

        assert!(remove_environment(&mut cfg, "alpha"));
        assert_eq!(cfg.environments.len(), 1);
        assert_eq!(cfg.environments[0].id, "beta");

        // Removing again returns false
        assert!(!remove_environment(&mut cfg, "alpha"));
    }

    #[test]
    fn test_roundtrip_full_config() {
        let json = r#"
        {
            "main": {
                "servers": [
                    {
                        "id": "fs",
                        "name": "Filesystem",
                        "transport": "stdio",
                        "command": "npx",
                        "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
                        "connectivity": "local",
                        "enabled": true
                    }
                ]
            },
            "environments": [
                {
                    "id": "work",
                    "name": "Work",
                    "endpoint": "http://localhost:3000/work",
                    "servers": [
                        {
                            "id": "gh",
                            "name": "GitHub",
                            "transport": "http",
                            "url": "https://api.githubcopilot.com/mcp/",
                            "connectivity": "online",
                            "enabled": true
                        }
                    ],
                    "overrides": [
                        {
                            "server_id": "fs",
                            "enabled": false
                        }
                    ]
                }
            ]
        }
        "#;

        let cfg: PlugmuxConfig = serde_json::from_str(json).unwrap();

        assert_eq!(cfg.main.servers.len(), 1);
        assert_eq!(cfg.main.servers[0].id, "fs");
        assert_eq!(cfg.main.servers[0].transport, Transport::Stdio);

        assert_eq!(cfg.environments.len(), 1);
        let env = &cfg.environments[0];
        assert_eq!(env.id, "work");
        assert_eq!(env.servers.len(), 1);
        assert_eq!(env.servers[0].transport, Transport::Http);
        assert_eq!(env.overrides.len(), 1);
        assert_eq!(env.overrides[0].server_id, "fs");
        assert_eq!(env.overrides[0].enabled, Some(false));

        // Ensure round-trip serialisation works
        let re_serialised = serde_json::to_string(&cfg).unwrap();
        let re_parsed: PlugmuxConfig = serde_json::from_str(&re_serialised).unwrap();
        assert_eq!(cfg, re_parsed);
    }
}
