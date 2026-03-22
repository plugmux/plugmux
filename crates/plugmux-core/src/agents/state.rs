use std::path::Path;

use serde::{Deserialize, Serialize};

use super::ConfigFormat;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentSource {
    Auto,
    Registry,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateEntry {
    pub id: String,
    pub source: AgentSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_format: Option<ConfigFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentState {
    #[serde(default)]
    pub agents: Vec<AgentStateEntry>,
    #[serde(default)]
    pub dismissed_agents: Vec<String>,
}

const STATE_FILE: &str = "agents_state.json";

impl AgentState {
    /// Loads state from `agents_state.json` in the given directory.
    /// Returns an empty state if the file doesn't exist or fails to parse.
    pub fn load(config_dir: &Path) -> Self {
        let path = config_dir.join(STATE_FILE);
        match std::fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Writes state to `agents_state.json` in the given directory.
    pub fn save(&self, config_dir: &Path) -> Result<(), String> {
        let path = config_dir.join(STATE_FILE);
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize agent state: {e}"))?;
        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write {}: {e}", path.display()))
    }

    /// Adds an agent entry. Replaces any existing entry with the same id.
    pub fn add_agent(&mut self, entry: AgentStateEntry) {
        self.agents.retain(|a| a.id != entry.id);
        self.agents.push(entry);
    }

    /// Removes an agent from the agents list by id.
    pub fn remove_agent(&mut self, id: &str) {
        self.agents.retain(|a| a.id != id);
    }

    /// Removes an agent from the agents list and adds it to dismissed_agents.
    pub fn dismiss_agent(&mut self, id: &str) {
        self.remove_agent(id);
        if !self.dismissed_agents.contains(&id.to_string()) {
            self.dismissed_agents.push(id.to_string());
        }
    }

    /// Returns true if the agent id is in the dismissed list.
    pub fn is_dismissed(&self, id: &str) -> bool {
        self.dismissed_agents.iter().any(|d| d == id)
    }

    /// Returns a reference to the agent entry with the given id, if it exists.
    pub fn get_agent(&self, id: &str) -> Option<&AgentStateEntry> {
        self.agents.iter().find(|a| a.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_entry(id: &str, source: AgentSource) -> AgentStateEntry {
        AgentStateEntry {
            id: id.to_string(),
            source,
            name: None,
            config_path: None,
            config_format: None,
            mcp_key: None,
        }
    }

    #[test]
    fn test_empty_state_when_file_missing() {
        let tmp = TempDir::new().unwrap();
        let state = AgentState::load(tmp.path());
        assert!(state.agents.is_empty());
        assert!(state.dismissed_agents.is_empty());
    }

    #[test]
    fn test_save_and_load_round_trip() {
        let tmp = TempDir::new().unwrap();
        let mut state = AgentState::default();
        state.add_agent(AgentStateEntry {
            id: "claude-code".to_string(),
            source: AgentSource::Auto,
            name: None,
            config_path: Some("/home/user/.claude/settings.json".to_string()),
            config_format: Some(ConfigFormat::Json),
            mcp_key: Some("mcpServers".to_string()),
        });
        state.dismissed_agents.push("old-agent".to_string());

        state.save(tmp.path()).expect("save should succeed");

        let loaded = AgentState::load(tmp.path());
        assert_eq!(loaded.agents.len(), 1);
        assert_eq!(loaded.agents[0].id, "claude-code");
        assert_eq!(loaded.agents[0].source, AgentSource::Auto);
        assert_eq!(
            loaded.agents[0].config_path,
            Some("/home/user/.claude/settings.json".to_string())
        );
        assert_eq!(loaded.agents[0].config_format, Some(ConfigFormat::Json));
        assert_eq!(
            loaded.agents[0].mcp_key,
            Some("mcpServers".to_string())
        );
        assert_eq!(loaded.dismissed_agents, vec!["old-agent"]);
    }

    #[test]
    fn test_add_agent_and_get_agent() {
        let mut state = AgentState::default();
        let entry = make_entry("cursor", AgentSource::Registry);
        state.add_agent(entry);

        let agent = state.get_agent("cursor");
        assert!(agent.is_some());
        assert_eq!(agent.unwrap().id, "cursor");
        assert_eq!(agent.unwrap().source, AgentSource::Registry);
    }

    #[test]
    fn test_add_agent_replaces_existing() {
        let mut state = AgentState::default();
        state.add_agent(make_entry("cursor", AgentSource::Auto));
        state.add_agent(make_entry("cursor", AgentSource::Custom));

        assert_eq!(state.agents.len(), 1);
        assert_eq!(state.get_agent("cursor").unwrap().source, AgentSource::Custom);
    }

    #[test]
    fn test_dismiss_agent_removes_and_adds_to_dismissed() {
        let mut state = AgentState::default();
        state.add_agent(make_entry("cursor", AgentSource::Auto));
        state.add_agent(make_entry("codex", AgentSource::Auto));

        state.dismiss_agent("cursor");

        assert!(state.get_agent("cursor").is_none());
        assert!(state.get_agent("codex").is_some());
        assert!(state.is_dismissed("cursor"));
        assert!(!state.is_dismissed("codex"));
    }

    #[test]
    fn test_is_dismissed() {
        let mut state = AgentState::default();
        assert!(!state.is_dismissed("foo"));

        state.dismissed_agents.push("foo".to_string());
        assert!(state.is_dismissed("foo"));
        assert!(!state.is_dismissed("bar"));
    }

    #[test]
    fn test_remove_agent() {
        let mut state = AgentState::default();
        state.add_agent(make_entry("a", AgentSource::Auto));
        state.add_agent(make_entry("b", AgentSource::Registry));

        state.remove_agent("a");

        assert!(state.get_agent("a").is_none());
        assert!(state.get_agent("b").is_some());
        // remove_agent should NOT add to dismissed
        assert!(!state.is_dismissed("a"));
    }

    #[test]
    fn test_dismiss_agent_is_idempotent() {
        let mut state = AgentState::default();
        state.add_agent(make_entry("x", AgentSource::Auto));

        state.dismiss_agent("x");
        state.dismiss_agent("x"); // second call shouldn't duplicate

        assert_eq!(
            state
                .dismissed_agents
                .iter()
                .filter(|d| *d == "x")
                .count(),
            1
        );
    }

    #[test]
    fn test_load_returns_default_on_invalid_json() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("agents_state.json"), "not valid json")
            .unwrap();

        let state = AgentState::load(tmp.path());
        assert!(state.agents.is_empty());
        assert!(state.dismissed_agents.is_empty());
    }
}
