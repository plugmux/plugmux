use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::server::{Connectivity, ServerConfig, Transport};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogData {
    pub version: u32,
    pub servers: Vec<CatalogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub transport: Transport,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub connectivity: Connectivity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetData {
    pub version: u32,
    pub presets: Vec<Preset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub servers: Vec<String>,
}

pub struct CatalogRegistry {
    servers: HashMap<String, CatalogEntry>,
    all_servers: Vec<CatalogEntry>,
    presets: HashMap<String, Preset>,
    all_presets: Vec<Preset>,
}

impl CatalogRegistry {
    pub fn load(servers_json: &str, presets_json: &str) -> Result<Self, serde_json::Error> {
        let catalog: CatalogData = serde_json::from_str(servers_json)?;
        let preset_data: PresetData = serde_json::from_str(presets_json)?;

        let mut servers = HashMap::new();
        for entry in &catalog.servers {
            servers.insert(entry.id.clone(), entry.clone());
        }

        let mut presets = HashMap::new();
        for preset in &preset_data.presets {
            presets.insert(preset.id.clone(), preset.clone());
        }

        Ok(Self {
            all_servers: catalog.servers,
            servers,
            all_presets: preset_data.presets,
            presets,
        })
    }

    pub fn load_bundled() -> Self {
        let servers = include_str!("../../../catalog/servers.json");
        let presets = include_str!("../../../catalog/presets.json");
        Self::load(servers, presets).expect("Bundled catalog is valid JSON")
    }

    pub fn get_server(&self, id: &str) -> Option<&CatalogEntry> {
        self.servers.get(id)
    }

    pub fn has_server(&self, id: &str) -> bool {
        self.servers.contains_key(id)
    }

    pub fn search(&self, query: &str, category: Option<&str>) -> Vec<&CatalogEntry> {
        let query_lower = query.to_lowercase();
        self.all_servers
            .iter()
            .filter(|entry| {
                let matches_query = query_lower.is_empty()
                    || entry.name.to_lowercase().contains(&query_lower)
                    || entry.description.to_lowercase().contains(&query_lower);
                let matches_category = category.map(|c| entry.category == c).unwrap_or(true);
                matches_query && matches_category
            })
            .collect()
    }

    pub fn list_servers(&self) -> &[CatalogEntry] {
        &self.all_servers
    }

    pub fn list_presets(&self) -> &[Preset] {
        &self.all_presets
    }

    pub fn get_preset(&self, id: &str) -> Option<&Preset> {
        self.presets.get(id)
    }

    pub fn to_server_config(entry: &CatalogEntry) -> ServerConfig {
        ServerConfig {
            id: entry.id.clone(),
            name: entry.name.clone(),
            transport: entry.transport.clone(),
            command: entry.command.clone(),
            args: entry.args.clone(),
            url: entry.url.clone(),
            connectivity: entry.connectivity.clone(),
            description: Some(entry.description.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SERVERS_JSON: &str = r#"{
        "version": 1,
        "servers": [
            {
                "id": "figma",
                "name": "Figma",
                "description": "Read and inspect Figma designs",
                "icon": "figma.svg",
                "category": "design",
                "transport": "stdio",
                "command": "npx",
                "args": ["-y", "@anthropic/figma-mcp"],
                "connectivity": "online"
            },
            {
                "id": "github",
                "name": "GitHub",
                "description": "Interact with GitHub repositories and code",
                "icon": "github.svg",
                "category": "dev-tools",
                "transport": "stdio",
                "command": "npx",
                "args": ["-y", "@modelcontextprotocol/server-github"],
                "connectivity": "online"
            },
            {
                "id": "context7",
                "name": "Context7",
                "description": "Up-to-date documentation for any library",
                "icon": "context7.svg",
                "category": "dev-tools",
                "transport": "http",
                "url": "https://mcp.context7.com/mcp",
                "connectivity": "online"
            },
            {
                "id": "postgres",
                "name": "PostgreSQL",
                "description": "Query PostgreSQL databases",
                "icon": "postgres.svg",
                "category": "database",
                "transport": "stdio",
                "command": "npx",
                "args": ["-y", "@modelcontextprotocol/server-postgres"],
                "connectivity": "local"
            }
        ]
    }"#;

    const PRESETS_JSON: &str = r#"{
        "version": 1,
        "presets": [
            {
                "id": "web-dev",
                "name": "Web Development",
                "description": "Frontend and full-stack web development",
                "icon": "web-dev.svg",
                "servers": ["figma", "context7"]
            },
            {
                "id": "data-work",
                "name": "Data Work",
                "description": "Database querying and data exploration",
                "icon": "data-work.svg",
                "servers": ["postgres"]
            }
        ]
    }"#;

    fn make_registry() -> CatalogRegistry {
        CatalogRegistry::load(SERVERS_JSON, PRESETS_JSON).expect("test JSON is valid")
    }

    #[test]
    fn test_load_catalog_from_json_string() {
        let registry = make_registry();
        assert_eq!(registry.list_servers().len(), 4);
    }

    #[test]
    fn test_get_server_by_id_returns_correct_entry() {
        let registry = make_registry();
        let entry = registry.get_server("figma").expect("figma should exist");
        assert_eq!(entry.id, "figma");
        assert_eq!(entry.name, "Figma");
        assert_eq!(entry.category, "design");
        assert_eq!(entry.transport, Transport::Stdio);
        assert_eq!(entry.connectivity, Connectivity::Online);
    }

    #[test]
    fn test_get_server_unknown_id_returns_none() {
        let registry = make_registry();
        assert!(registry.get_server("nonexistent-server").is_none());
    }

    #[test]
    fn test_has_server_returns_true_for_existing() {
        let registry = make_registry();
        assert!(registry.has_server("figma"));
        assert!(registry.has_server("github"));
        assert!(registry.has_server("context7"));
        assert!(registry.has_server("postgres"));
    }

    #[test]
    fn test_has_server_returns_false_for_unknown() {
        let registry = make_registry();
        assert!(!registry.has_server("nonexistent"));
        assert!(!registry.has_server(""));
    }

    #[test]
    fn test_search_matches_name_case_insensitive() {
        let registry = make_registry();
        let results = registry.search("figma", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "figma");

        let results_upper = registry.search("FIGMA", None);
        assert_eq!(results_upper.len(), 1);
        assert_eq!(results_upper[0].id, "figma");

        let results_mixed = registry.search("FiGmA", None);
        assert_eq!(results_mixed.len(), 1);
        assert_eq!(results_mixed[0].id, "figma");
    }

    #[test]
    fn test_search_matches_description_case_insensitive() {
        let registry = make_registry();
        // "repositories" appears in github description
        let results = registry.search("repositories", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "github");

        let results_upper = registry.search("REPOSITORIES", None);
        assert_eq!(results_upper.len(), 1);
        assert_eq!(results_upper[0].id, "github");
    }

    #[test]
    fn test_search_by_category_filters_correctly() {
        let registry = make_registry();
        let dev_tools = registry.search("", Some("dev-tools"));
        assert_eq!(dev_tools.len(), 2);
        assert!(dev_tools.iter().any(|e| e.id == "github"));
        assert!(dev_tools.iter().any(|e| e.id == "context7"));

        let design = registry.search("", Some("design"));
        assert_eq!(design.len(), 1);
        assert_eq!(design[0].id, "figma");

        let database = registry.search("", Some("database"));
        assert_eq!(database.len(), 1);
        assert_eq!(database[0].id, "postgres");
    }

    #[test]
    fn test_search_by_query_and_category_combined() {
        let registry = make_registry();
        // "documentation" matches context7 description, which is also dev-tools
        let results = registry.search("documentation", Some("dev-tools"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "context7");

        // "documentation" in design category should return nothing
        let results_none = registry.search("documentation", Some("design"));
        assert!(results_none.is_empty());
    }

    #[test]
    fn test_search_empty_query_returns_all() {
        let registry = make_registry();
        let results = registry.search("", None);
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_search_no_match_returns_empty() {
        let registry = make_registry();
        let results = registry.search("xyzzy-not-a-real-tool", None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_list_servers_returns_all_servers() {
        let registry = make_registry();
        let servers = registry.list_servers();
        assert_eq!(servers.len(), 4);
        let ids: Vec<&str> = servers.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"figma"));
        assert!(ids.contains(&"github"));
        assert!(ids.contains(&"context7"));
        assert!(ids.contains(&"postgres"));
    }

    #[test]
    fn test_list_presets_returns_presets() {
        let registry = make_registry();
        let presets = registry.list_presets();
        assert_eq!(presets.len(), 2);
    }

    #[test]
    fn test_get_preset_by_id() {
        let registry = make_registry();
        let preset = registry
            .get_preset("web-dev")
            .expect("web-dev should exist");
        assert_eq!(preset.id, "web-dev");
        assert_eq!(preset.name, "Web Development");
        assert!(preset.servers.contains(&"figma".to_string()));
        assert!(preset.servers.contains(&"context7".to_string()));
    }

    #[test]
    fn test_get_preset_unknown_id_returns_none() {
        let registry = make_registry();
        assert!(registry.get_preset("nonexistent-preset").is_none());
    }

    #[test]
    fn test_to_server_config_stdio() {
        let registry = make_registry();
        let entry = registry.get_server("figma").unwrap();
        let config = CatalogRegistry::to_server_config(entry);

        assert_eq!(config.id, "figma");
        assert_eq!(config.name, "Figma");
        assert_eq!(config.transport, Transport::Stdio);
        assert_eq!(config.command, Some("npx".to_string()));
        assert_eq!(
            config.args,
            Some(vec!["-y".to_string(), "@anthropic/figma-mcp".to_string()])
        );
        assert_eq!(config.url, None);
        assert_eq!(config.connectivity, Connectivity::Online);
        assert_eq!(
            config.description,
            Some("Read and inspect Figma designs".to_string())
        );
    }

    #[test]
    fn test_to_server_config_http() {
        let registry = make_registry();
        let entry = registry.get_server("context7").unwrap();
        let config = CatalogRegistry::to_server_config(entry);

        assert_eq!(config.id, "context7");
        assert_eq!(config.transport, Transport::Http);
        assert_eq!(config.url, Some("https://mcp.context7.com/mcp".to_string()));
        assert_eq!(config.command, None);
        assert_eq!(config.args, None);
        assert_eq!(config.connectivity, Connectivity::Online);
    }

    #[test]
    fn test_load_bundled() {
        // Verifies the bundled JSON files are valid and load correctly
        let registry = CatalogRegistry::load_bundled();
        assert!(!registry.list_servers().is_empty());
        assert!(!registry.list_presets().is_empty());
        assert!(registry.has_server("figma"));
        assert!(registry.has_server("github"));
        assert!(registry.get_preset("web-dev").is_some());
    }
}
