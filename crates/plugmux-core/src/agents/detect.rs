use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{
    AgentEntry, AgentRegistry, AgentSource, AgentState, AgentTier, ConfigFormat,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Green,  // plugmux is the only MCP entry
    Yellow, // plugmux present + other MCPs also present
    Gray,   // not connected (no plugmux key, or not installed)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedAgent {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub config_path: Option<String>,
    pub installed: bool,
    pub status: AgentStatus,
    pub source: String, // "auto", "registry", "custom"
}

/// Determines the plugmux connection status by inspecting an agent's config file.
///
/// - If the file doesn't exist, returns Gray.
/// - Parses the file according to `config_format` and looks for the `mcp_key` section.
/// - If that section contains a "plugmux" key and nothing else -> Green.
/// - If "plugmux" is present alongside other keys -> Yellow.
/// - Otherwise -> Gray.
pub fn detect_agent_status(
    config_path: &Path,
    config_format: &ConfigFormat,
    mcp_key: &str,
) -> AgentStatus {
    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return AgentStatus::Gray,
    };

    match config_format {
        ConfigFormat::Json => detect_status_json(&content, mcp_key),
        ConfigFormat::Toml => detect_status_toml(&content, mcp_key),
    }
}

fn detect_status_json(content: &str, mcp_key: &str) -> AgentStatus {
    let value: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return AgentStatus::Gray,
    };

    let mcp_section = match value.get(mcp_key) {
        Some(serde_json::Value::Object(map)) => map,
        _ => return AgentStatus::Gray,
    };

    if mcp_section.is_empty() {
        return AgentStatus::Gray;
    }

    if mcp_section.contains_key("plugmux") {
        if mcp_section.len() == 1 {
            AgentStatus::Green
        } else {
            AgentStatus::Yellow
        }
    } else {
        AgentStatus::Gray
    }
}

fn detect_status_toml(content: &str, mcp_key: &str) -> AgentStatus {
    let value: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return AgentStatus::Gray,
    };

    let mcp_section = match value.get(mcp_key) {
        Some(toml::Value::Table(table)) => table,
        _ => return AgentStatus::Gray,
    };

    if mcp_section.is_empty() {
        return AgentStatus::Gray;
    }

    if mcp_section.contains_key("plugmux") {
        if mcp_section.len() == 1 {
            AgentStatus::Green
        } else {
            AgentStatus::Yellow
        }
    } else {
        AgentStatus::Gray
    }
}

/// Detects a single agent from a registry entry.
///
/// Resolves the config path for the current OS, checks if it exists,
/// and determines the connection status.
pub fn detect_agent(entry: &AgentEntry, registry: &AgentRegistry) -> DetectedAgent {
    let config_path = registry.resolve_config_path(entry);
    let installed = config_path.as_ref().is_some_and(|p| p.exists());

    let status = match &config_path {
        Some(p) if installed => detect_agent_status(p, &entry.config_format, &entry.mcp_key),
        _ => AgentStatus::Gray,
    };

    DetectedAgent {
        id: entry.id.clone(),
        name: entry.name.clone(),
        icon: entry.icon.clone(),
        config_path: config_path.map(|p| p.to_string_lossy().to_string()),
        installed,
        status,
        source: match entry.tier {
            AgentTier::Auto => "auto".to_string(),
            AgentTier::Manual => "registry".to_string(),
        },
    }
}

/// Scans all agents from the registry and state, returning a deduplicated list
/// of detected agents.
///
/// - All auto-tier agents from the registry are included.
/// - Agents from state with registry/custom sources are also included.
/// - Dismissed agents are excluded.
pub fn detect_all(registry: &AgentRegistry, state: &AgentState) -> Vec<DetectedAgent> {
    let mut seen = std::collections::HashSet::new();
    let mut results = Vec::new();

    // Include all agents from registry (auto + manual)
    for entry in registry.list_agents() {
        if state.is_dismissed(&entry.id) {
            continue;
        }
        seen.insert(entry.id.clone());
        results.push(detect_agent(entry, registry));
    }

    // Include agents from state (registry/custom sources) that aren't already covered
    for state_entry in &state.agents {
        if seen.contains(&state_entry.id) {
            continue;
        }
        if state.is_dismissed(&state_entry.id) {
            continue;
        }

        // Try to find in registry first for full metadata
        if let Some(reg_entry) = registry.get_agent(&state_entry.id) {
            results.push(detect_agent(reg_entry, registry));
        } else {
            // Custom agent — build DetectedAgent from state entry
            let config_path = state_entry.config_path.clone();
            let config_path_buf = config_path.as_ref().map(std::path::PathBuf::from);
            let installed = config_path_buf.as_ref().is_some_and(|p| p.exists());

            let status = match (&config_path_buf, &state_entry.config_format, &state_entry.mcp_key)
            {
                (Some(p), Some(fmt), Some(key)) if installed => {
                    detect_agent_status(p, fmt, key)
                }
                _ => AgentStatus::Gray,
            };

            let source = match state_entry.source {
                AgentSource::Auto => "auto",
                AgentSource::Registry => "registry",
                AgentSource::Custom => "custom",
            };

            results.push(DetectedAgent {
                id: state_entry.id.clone(),
                name: state_entry
                    .name
                    .clone()
                    .unwrap_or_else(|| state_entry.id.clone()),
                icon: None,
                config_path,
                installed,
                status,
                source: source.to_string(),
            });
        }

        seen.insert(state_entry.id.clone());
    }

    // Sort by registry order — agents in the registry come first in their defined order,
    // custom/unknown agents sort to the end.
    results.sort_by_key(|a| registry.position(&a.id).unwrap_or(usize::MAX));
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_missing_file_returns_gray() {
        let status = detect_agent_status(
            Path::new("/tmp/does-not-exist-plugmux-test.json"),
            &ConfigFormat::Json,
            "mcpServers",
        );
        assert_eq!(status, AgentStatus::Gray);
    }

    #[test]
    fn test_empty_json_returns_gray() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        std::fs::write(&path, "{}").unwrap();

        let status = detect_agent_status(&path, &ConfigFormat::Json, "mcpServers");
        assert_eq!(status, AgentStatus::Gray);
    }

    #[test]
    fn test_json_plugmux_only_returns_green() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        std::fs::write(
            &path,
            r#"{"mcpServers": {"plugmux": {"url": "http://localhost:4242/env/default"}}}"#,
        )
        .unwrap();

        let status = detect_agent_status(&path, &ConfigFormat::Json, "mcpServers");
        assert_eq!(status, AgentStatus::Green);
    }

    #[test]
    fn test_json_plugmux_plus_other_returns_yellow() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        std::fs::write(
            &path,
            r#"{"mcpServers": {"plugmux": {"url": "http://localhost:4242/env/default"}, "github": {"command": "gh"}}}"#,
        )
        .unwrap();

        let status = detect_agent_status(&path, &ConfigFormat::Json, "mcpServers");
        assert_eq!(status, AgentStatus::Yellow);
    }

    #[test]
    fn test_json_no_plugmux_returns_gray() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        std::fs::write(
            &path,
            r#"{"mcpServers": {"github": {"command": "gh"}}}"#,
        )
        .unwrap();

        let status = detect_agent_status(&path, &ConfigFormat::Json, "mcpServers");
        assert_eq!(status, AgentStatus::Gray);
    }

    #[test]
    fn test_toml_plugmux_only_returns_green() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(
            &path,
            "[mcp_servers.plugmux]\nurl = \"http://localhost:4242/env/default\"\n",
        )
        .unwrap();

        let status = detect_agent_status(&path, &ConfigFormat::Toml, "mcp_servers");
        assert_eq!(status, AgentStatus::Green);
    }

    #[test]
    fn test_toml_plugmux_plus_other_returns_yellow() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(
            &path,
            "[mcp_servers.plugmux]\nurl = \"http://localhost:4242/env/default\"\n\n[mcp_servers.github]\ncommand = \"gh\"\n",
        )
        .unwrap();

        let status = detect_agent_status(&path, &ConfigFormat::Toml, "mcp_servers");
        assert_eq!(status, AgentStatus::Yellow);
    }

    #[test]
    fn test_toml_no_plugmux_returns_gray() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(
            &path,
            "[mcp_servers.github]\ncommand = \"gh\"\n",
        )
        .unwrap();

        let status = detect_agent_status(&path, &ConfigFormat::Toml, "mcp_servers");
        assert_eq!(status, AgentStatus::Gray);
    }

    #[test]
    fn test_json_empty_mcp_section_returns_gray() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        std::fs::write(&path, r#"{"mcpServers": {}}"#).unwrap();

        let status = detect_agent_status(&path, &ConfigFormat::Json, "mcpServers");
        assert_eq!(status, AgentStatus::Gray);
    }

    #[test]
    fn test_detect_all_excludes_dismissed_agents() {
        let registry_json = r#"{
            "version": 1,
            "agents": [
                {
                    "id": "agent-a",
                    "name": "Agent A",
                    "icon": null,
                    "config_format": "json",
                    "mcp_key": "mcpServers",
                    "tier": "auto",
                    "config_paths": {
                        "macos": "/tmp/nonexistent-a.json",
                        "linux": "/tmp/nonexistent-a.json",
                        "windows": null
                    }
                },
                {
                    "id": "agent-b",
                    "name": "Agent B",
                    "icon": null,
                    "config_format": "json",
                    "mcp_key": "mcpServers",
                    "tier": "auto",
                    "config_paths": {
                        "macos": "/tmp/nonexistent-b.json",
                        "linux": "/tmp/nonexistent-b.json",
                        "windows": null
                    }
                }
            ]
        }"#;

        let registry = AgentRegistry::load(registry_json).unwrap();
        let mut state = AgentState::default();
        state.dismiss_agent("agent-a");

        let detected = detect_all(&registry, &state);
        let ids: Vec<&str> = detected.iter().map(|d| d.id.as_str()).collect();

        assert!(!ids.contains(&"agent-a"), "dismissed agent should be excluded");
        assert!(ids.contains(&"agent-b"), "non-dismissed agent should be included");
    }

    #[test]
    fn test_detect_all_includes_custom_state_agents() {
        let registry_json = r#"{
            "version": 1,
            "agents": [
                {
                    "id": "agent-a",
                    "name": "Agent A",
                    "icon": null,
                    "config_format": "json",
                    "mcp_key": "mcpServers",
                    "tier": "auto",
                    "config_paths": {
                        "macos": "/tmp/nonexistent-a.json",
                        "linux": "/tmp/nonexistent-a.json",
                        "windows": null
                    }
                }
            ]
        }"#;

        let registry = AgentRegistry::load(registry_json).unwrap();
        let mut state = AgentState::default();
        state.add_agent(super::super::AgentStateEntry {
            id: "custom-agent".to_string(),
            source: AgentSource::Custom,
            name: Some("My Custom Agent".to_string()),
            config_path: Some("/tmp/nonexistent-custom.json".to_string()),
            config_format: Some(ConfigFormat::Json),
            mcp_key: Some("mcpServers".to_string()),
        });

        let detected = detect_all(&registry, &state);
        let ids: Vec<&str> = detected.iter().map(|d| d.id.as_str()).collect();

        assert!(ids.contains(&"agent-a"));
        assert!(ids.contains(&"custom-agent"));

        let custom = detected.iter().find(|d| d.id == "custom-agent").unwrap();
        assert_eq!(custom.name, "My Custom Agent");
        assert_eq!(custom.source, "custom");
        assert!(!custom.installed);
        assert_eq!(custom.status, AgentStatus::Gray);
    }
}
