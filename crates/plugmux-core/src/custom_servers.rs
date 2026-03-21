use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::catalog::CatalogRegistry;
use crate::config::ConfigError;
use crate::server::ServerConfig;

// ---------------------------------------------------------------------------
// Persistence format
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomServersData {
    pub version: u32,
    pub servers: Vec<ServerConfig>,
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

pub struct CustomServerStore {
    servers: HashMap<String, ServerConfig>,
    path: PathBuf,
}

impl CustomServerStore {
    /// Load from `path`. Returns `ConfigError::Io` if the file is missing or
    /// unreadable, and `ConfigError::Json` if the content is malformed.
    pub fn load(path: PathBuf) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(&path)?;
        let data: CustomServersData = serde_json::from_str(&content)?;
        let servers = data
            .servers
            .into_iter()
            .map(|s| (s.id.clone(), s))
            .collect();
        Ok(Self { servers, path })
    }

    /// Load from `path`, returning an empty store if the file does not exist
    /// or cannot be parsed.
    pub fn load_or_default(path: PathBuf) -> Self {
        match Self::load(path.clone()) {
            Ok(store) => store,
            Err(_) => Self {
                servers: HashMap::new(),
                path,
            },
        }
    }

    /// Persist the store to its file, creating parent directories as needed.
    pub fn save(&self) -> Result<(), ConfigError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut servers: Vec<ServerConfig> = self.servers.values().cloned().collect();
        // Deterministic order so diffs are clean.
        servers.sort_by(|a, b| a.id.cmp(&b.id));
        let data = CustomServersData {
            version: 1,
            servers,
        };
        let content = serde_json::to_string_pretty(&data)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Read accessors
    // -----------------------------------------------------------------------

    pub fn get(&self, id: &str) -> Option<&ServerConfig> {
        self.servers.get(id)
    }

    pub fn has(&self, id: &str) -> bool {
        self.servers.contains_key(id)
    }

    /// Returns all custom servers in an unspecified order.
    pub fn list(&self) -> Vec<&ServerConfig> {
        self.servers.values().collect()
    }

    // -----------------------------------------------------------------------
    // Mutations
    // -----------------------------------------------------------------------

    /// Add a new custom server.
    ///
    /// Returns `ConfigError::IdCollision` if the ID already exists in the
    /// catalog registry **or** in the custom store.
    pub fn add(
        &mut self,
        config: ServerConfig,
        catalog: &CatalogRegistry,
    ) -> Result<(), ConfigError> {
        if catalog.has_server(&config.id) {
            return Err(ConfigError::IdCollision(config.id.clone()));
        }
        if self.servers.contains_key(&config.id) {
            return Err(ConfigError::IdCollision(config.id.clone()));
        }
        self.servers.insert(config.id.clone(), config);
        Ok(())
    }

    /// Replace an existing custom server.
    ///
    /// Returns `ConfigError::EnvironmentNotFound` (reused for "not found")
    /// if `id` does not exist in the store.
    pub fn update(&mut self, id: &str, config: ServerConfig) -> Result<(), ConfigError> {
        if !self.servers.contains_key(id) {
            return Err(ConfigError::EnvironmentNotFound(id.to_string()));
        }
        self.servers.insert(id.to_string(), config);
        Ok(())
    }

    /// Remove a custom server.  Returns `true` if it existed, `false`
    /// otherwise.
    pub fn remove(&mut self, id: &str) -> bool {
        self.servers.remove(id).is_some()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{Connectivity, Transport};
    use tempfile::TempDir;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    const CATALOG_SERVERS_JSON: &str = r#"{
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
            }
        ]
    }"#;

    const CATALOG_PRESETS_JSON: &str = r#"{"version": 1, "presets": []}"#;

    fn make_catalog() -> CatalogRegistry {
        CatalogRegistry::load(CATALOG_SERVERS_JSON, CATALOG_PRESETS_JSON)
            .expect("test catalog JSON is valid")
    }

    fn make_server(id: &str, name: &str) -> ServerConfig {
        ServerConfig {
            id: id.to_string(),
            name: name.to_string(),
            transport: Transport::Stdio,
            command: Some("node".to_string()),
            args: Some(vec!["./mcp-server.js".to_string()]),
            url: None,
            connectivity: Connectivity::Local,
            description: None,
        }
    }

    fn valid_json(id: &str) -> String {
        format!(
            r#"{{
                "version": 1,
                "servers": [
                    {{
                        "id": "{id}",
                        "name": "Internal DB",
                        "transport": "stdio",
                        "command": "node",
                        "args": ["./mcp-server.js"],
                        "connectivity": "local"
                    }}
                ]
            }}"#,
            id = id
        )
    }

    // -----------------------------------------------------------------------
    // load from valid JSON
    // -----------------------------------------------------------------------

    #[test]
    fn test_load_from_valid_json_string() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("custom_servers.json");
        std::fs::write(&path, valid_json("internal-db")).unwrap();

        let store = CustomServerStore::load(path).unwrap();
        assert_eq!(store.list().len(), 1);
        let server = store.get("internal-db").expect("internal-db should exist");
        assert_eq!(server.name, "Internal DB");
    }

    // -----------------------------------------------------------------------
    // load returns empty when file doesn't exist
    // -----------------------------------------------------------------------

    #[test]
    fn test_load_or_default_returns_empty_when_file_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.json");

        let store = CustomServerStore::load_or_default(path);
        assert!(store.list().is_empty());
    }

    // -----------------------------------------------------------------------
    // load returns error when file doesn't exist (strict load)
    // -----------------------------------------------------------------------

    #[test]
    fn test_load_returns_error_when_file_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.json");

        let result = CustomServerStore::load(path);
        assert!(matches!(result, Err(ConfigError::Io(_))));
    }

    // -----------------------------------------------------------------------
    // add server and verify it's stored
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_server_and_verify_stored() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("custom_servers.json");
        let mut store = CustomServerStore::load_or_default(path);
        let catalog = make_catalog();

        store.add(make_server("my-db", "My DB"), &catalog).unwrap();

        assert!(store.has("my-db"));
        let s = store.get("my-db").unwrap();
        assert_eq!(s.name, "My DB");
    }

    // -----------------------------------------------------------------------
    // add with catalog ID collision returns ConfigError::IdCollision
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_catalog_id_collision_returns_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("custom_servers.json");
        let mut store = CustomServerStore::load_or_default(path);
        let catalog = make_catalog();

        // "figma" exists in the catalog
        let result = store.add(make_server("figma", "My Figma Clone"), &catalog);
        assert!(
            matches!(result, Err(ConfigError::IdCollision(ref id)) if id == "figma"),
            "expected IdCollision(\"figma\"), got {:?}",
            result
        );
    }

    // -----------------------------------------------------------------------
    // add with duplicate custom ID returns ConfigError::IdCollision
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_duplicate_custom_id_returns_collision() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("custom_servers.json");
        let mut store = CustomServerStore::load_or_default(path);
        let catalog = make_catalog();

        store.add(make_server("my-db", "My DB"), &catalog).unwrap();
        let result = store.add(make_server("my-db", "My DB Again"), &catalog);
        assert!(matches!(result, Err(ConfigError::IdCollision(_))));
    }

    // -----------------------------------------------------------------------
    // update existing server
    // -----------------------------------------------------------------------

    #[test]
    fn test_update_existing_server() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("custom_servers.json");
        let mut store = CustomServerStore::load_or_default(path);
        let catalog = make_catalog();

        store.add(make_server("my-db", "My DB"), &catalog).unwrap();

        let updated = ServerConfig {
            id: "my-db".to_string(),
            name: "My Updated DB".to_string(),
            transport: Transport::Stdio,
            command: Some("node".to_string()),
            args: Some(vec!["./updated.js".to_string()]),
            url: None,
            connectivity: Connectivity::Local,
            description: Some("updated".to_string()),
        };
        store.update("my-db", updated).unwrap();

        let s = store.get("my-db").unwrap();
        assert_eq!(s.name, "My Updated DB");
        assert_eq!(s.description, Some("updated".to_string()));
    }

    // -----------------------------------------------------------------------
    // update non-existent server returns error
    // -----------------------------------------------------------------------

    #[test]
    fn test_update_nonexistent_server_returns_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("custom_servers.json");
        let mut store = CustomServerStore::load_or_default(path);

        let result = store.update("ghost", make_server("ghost", "Ghost"));
        assert!(
            matches!(result, Err(ConfigError::EnvironmentNotFound(ref id)) if id == "ghost"),
            "expected EnvironmentNotFound(\"ghost\"), got {:?}",
            result
        );
    }

    // -----------------------------------------------------------------------
    // remove server returns true
    // -----------------------------------------------------------------------

    #[test]
    fn test_remove_server_returns_true() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("custom_servers.json");
        let mut store = CustomServerStore::load_or_default(path);
        let catalog = make_catalog();

        store.add(make_server("my-db", "My DB"), &catalog).unwrap();
        assert!(store.remove("my-db"));
        assert!(!store.has("my-db"));
    }

    // -----------------------------------------------------------------------
    // remove non-existent server returns false
    // -----------------------------------------------------------------------

    #[test]
    fn test_remove_nonexistent_server_returns_false() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("custom_servers.json");
        let mut store = CustomServerStore::load_or_default(path);

        assert!(!store.remove("does-not-exist"));
    }

    // -----------------------------------------------------------------------
    // get_server by ID returns correct server
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_server_by_id_returns_correct_server() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("custom_servers.json");
        let mut store = CustomServerStore::load_or_default(path);
        let catalog = make_catalog();

        store.add(make_server("alpha", "Alpha"), &catalog).unwrap();
        store.add(make_server("beta", "Beta"), &catalog).unwrap();

        let s = store.get("alpha").unwrap();
        assert_eq!(s.id, "alpha");
        assert_eq!(s.name, "Alpha");

        assert!(store.get("gamma").is_none());
    }

    // -----------------------------------------------------------------------
    // list returns all servers
    // -----------------------------------------------------------------------

    #[test]
    fn test_list_returns_all_servers() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("custom_servers.json");
        let mut store = CustomServerStore::load_or_default(path);
        let catalog = make_catalog();

        store.add(make_server("alpha", "Alpha"), &catalog).unwrap();
        store.add(make_server("beta", "Beta"), &catalog).unwrap();
        store.add(make_server("gamma", "Gamma"), &catalog).unwrap();

        let list = store.list();
        assert_eq!(list.len(), 3);

        let ids: Vec<&str> = list.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"alpha"));
        assert!(ids.contains(&"beta"));
        assert!(ids.contains(&"gamma"));
    }

    // -----------------------------------------------------------------------
    // save and reload roundtrip
    // -----------------------------------------------------------------------

    #[test]
    fn test_save_and_reload_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("custom_servers.json");
        let catalog = make_catalog();

        {
            let mut store = CustomServerStore::load_or_default(path.clone());
            store.add(make_server("internal-db", "Internal DB"), &catalog).unwrap();
            store.save().unwrap();
        }

        let reloaded = CustomServerStore::load(path).unwrap();
        assert_eq!(reloaded.list().len(), 1);
        let s = reloaded.get("internal-db").unwrap();
        assert_eq!(s.name, "Internal DB");
        assert_eq!(s.transport, Transport::Stdio);
        assert_eq!(s.connectivity, Connectivity::Local);
    }
}
