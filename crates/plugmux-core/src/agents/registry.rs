use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConfigFormat {
    Json,
    Toml,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentTier {
    Auto,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPaths {
    pub macos: Option<String>,
    pub linux: Option<String>,
    pub windows: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEntry {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub config_format: ConfigFormat,
    pub mcp_key: String,
    pub tier: AgentTier,
    #[serde(default)]
    pub install_url: Option<String>,
    #[serde(default)]
    pub setup_hint: Option<String>,
    pub config_paths: Option<ConfigPaths>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentData {
    version: u32,
    agents: Vec<AgentEntry>,
}

pub struct AgentRegistry {
    ordered: Vec<AgentEntry>,
    index: HashMap<String, usize>,
}

impl AgentRegistry {
    pub fn load(json: &str) -> Result<Self, serde_json::Error> {
        let data: AgentData = serde_json::from_str(json)?;

        let mut index = HashMap::new();
        for (i, entry) in data.agents.iter().enumerate() {
            index.insert(entry.id.clone(), i);
        }

        Ok(Self {
            ordered: data.agents,
            index,
        })
    }

    pub fn load_bundled() -> Self {
        let json = include_str!("../../../../agents/agents.json");
        Self::load(json).expect("Bundled agents.json is valid JSON")
    }

    /// Returns all agents in registry order (as defined in agents.json).
    pub fn list_agents(&self) -> Vec<&AgentEntry> {
        self.ordered.iter().collect()
    }

    /// Returns auto-tier agents in registry order.
    pub fn list_auto_agents(&self) -> Vec<&AgentEntry> {
        self.ordered
            .iter()
            .filter(|a| a.tier == AgentTier::Auto)
            .collect()
    }

    pub fn get_agent(&self, id: &str) -> Option<&AgentEntry> {
        self.index.get(id).map(|&i| &self.ordered[i])
    }

    /// Returns the position of an agent in the registry order.
    pub fn position(&self, id: &str) -> Option<usize> {
        self.index.get(id).copied()
    }

    pub fn resolve_config_path(&self, agent: &AgentEntry) -> Option<PathBuf> {
        let paths = agent.config_paths.as_ref()?;

        let raw = if cfg!(target_os = "macos") {
            paths.macos.as_deref()
        } else if cfg!(target_os = "linux") {
            paths.linux.as_deref()
        } else if cfg!(target_os = "windows") {
            paths.windows.as_deref()
        } else {
            None
        }?;

        Some(Self::expand_path(raw))
    }

    fn expand_path(raw: &str) -> PathBuf {
        let mut path = raw.to_string();

        // Expand ~ to home directory
        if (path.starts_with("~/") || path == "~")
            && let Some(home) = dirs::home_dir()
        {
            let home_str = home.to_string_lossy();
            path = if path == "~" {
                home_str.to_string()
            } else {
                format!("{}{}", home_str, &path[1..])
            };
        }

        // Expand Windows environment variables
        if cfg!(target_os = "windows") {
            if let Ok(user_profile) = std::env::var("USERPROFILE") {
                path = path.replace("%USERPROFILE%", &user_profile);
            }
            if let Ok(appdata) = std::env::var("APPDATA") {
                path = path.replace("%APPDATA%", &appdata);
            }
        }

        PathBuf::from(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_JSON: &str = r#"{
        "version": 1,
        "agents": [
            {
                "id": "claude-code",
                "name": "Claude Code",
                "icon": "claudecode",
                "config_format": "json",
                "mcp_key": "mcpServers",
                "tier": "auto",
                "config_paths": {
                    "macos": "~/.claude.json",
                    "linux": "~/.claude.json",
                    "windows": "%USERPROFILE%\\.claude\\settings.json"
                }
            },
            {
                "id": "cursor",
                "name": "Cursor",
                "icon": "cursor",
                "config_format": "json",
                "mcp_key": "mcpServers",
                "tier": "auto",
                "config_paths": {
                    "macos": "~/.cursor/mcp.json",
                    "linux": "~/.cursor/mcp.json",
                    "windows": "%USERPROFILE%\\.cursor\\mcp.json"
                }
            },
            {
                "id": "codex",
                "name": "Codex",
                "icon": "codex",
                "config_format": "toml",
                "mcp_key": "mcp_servers",
                "tier": "auto",
                "config_paths": {
                    "macos": "~/.codex/config.toml",
                    "linux": "~/.codex/config.toml",
                    "windows": "%USERPROFILE%\\.codex\\config.toml"
                }
            },
            {
                "id": "cherrystudio",
                "name": "Cherry Studio",
                "icon": "cherrystudio",
                "config_format": "json",
                "mcp_key": "mcpServers",
                "tier": "manual",
                "config_paths": null
            }
        ]
    }"#;

    fn make_registry() -> AgentRegistry {
        AgentRegistry::load(TEST_JSON).expect("test JSON is valid")
    }

    #[test]
    fn test_load_from_json_string() {
        let registry = make_registry();
        assert_eq!(registry.list_agents().len(), 4);
    }

    #[test]
    fn test_list_agents_returns_all() {
        let registry = make_registry();
        let agents = registry.list_agents();
        assert_eq!(agents.len(), 4);

        let ids: Vec<&str> = agents.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"claude-code"));
        assert!(ids.contains(&"cursor"));
        assert!(ids.contains(&"codex"));
        assert!(ids.contains(&"cherrystudio"));
    }

    #[test]
    fn test_list_auto_agents_returns_only_auto_tier() {
        let registry = make_registry();
        let auto_agents = registry.list_auto_agents();
        assert_eq!(auto_agents.len(), 3);

        for agent in &auto_agents {
            assert_eq!(agent.tier, AgentTier::Auto);
        }

        let ids: Vec<&str> = auto_agents.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"claude-code"));
        assert!(ids.contains(&"cursor"));
        assert!(ids.contains(&"codex"));
        assert!(!ids.contains(&"cherrystudio"));
    }

    #[test]
    fn test_get_agent_returns_correct_entry() {
        let registry = make_registry();
        let agent = registry
            .get_agent("claude-code")
            .expect("claude-code should exist");
        assert_eq!(agent.id, "claude-code");
        assert_eq!(agent.name, "Claude Code");
        assert_eq!(agent.icon, Some("claudecode".to_string()));
        assert_eq!(agent.config_format, ConfigFormat::Json);
        assert_eq!(agent.mcp_key, "mcpServers");
        assert_eq!(agent.tier, AgentTier::Auto);
        assert!(agent.config_paths.is_some());
    }

    #[test]
    fn test_get_agent_returns_none_for_unknown() {
        let registry = make_registry();
        assert!(registry.get_agent("nonexistent-agent").is_none());
    }

    #[test]
    fn test_resolve_config_path_expands_tilde() {
        let registry = make_registry();
        let agent = registry.get_agent("claude-code").unwrap();
        let path = registry.resolve_config_path(agent);

        // On macOS/Linux the path should be expanded; on any OS it should be Some
        if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
            let path = path.expect("should resolve on macOS/Linux");
            let path_str = path.to_string_lossy();
            // Should NOT start with ~
            assert!(
                !path_str.starts_with('~'),
                "tilde should be expanded: {}",
                path_str
            );
            // Should end with the expected suffix
            assert!(path_str.ends_with(".claude.json"));
            // Should start with home dir
            let home = dirs::home_dir().expect("home dir exists");
            assert!(path.starts_with(&home));
        }
    }

    #[test]
    fn test_resolve_config_path_returns_none_for_no_paths() {
        let registry = make_registry();
        let agent = registry.get_agent("cherrystudio").unwrap();
        let path = registry.resolve_config_path(agent);
        assert!(
            path.is_none(),
            "manual agents with null config_paths should return None"
        );
    }

    #[test]
    fn test_config_format_toml() {
        let registry = make_registry();
        let agent = registry.get_agent("codex").unwrap();
        assert_eq!(agent.config_format, ConfigFormat::Toml);
    }

    #[test]
    fn test_load_bundled_does_not_panic() {
        let registry = AgentRegistry::load_bundled();
        let agents = registry.list_agents();
        assert!(
            !agents.is_empty(),
            "bundled agents.json should have entries"
        );

        // Verify some known agents exist
        assert!(registry.get_agent("claude-code").is_some());
        assert!(registry.get_agent("cursor").is_some());

        // Verify auto agents exist
        let auto = registry.list_auto_agents();
        assert!(!auto.is_empty());
    }
}
