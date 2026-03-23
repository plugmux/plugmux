use std::sync::Arc;

use crate::catalog::CatalogRegistry;
use crate::custom_servers::CustomServerStore;
use crate::server::{HealthStatus, ServerConfig};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum ServerSource {
    Catalog,
    Custom,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ResolvedServer {
    pub id: String,
    pub config: Option<ServerConfig>,
    pub source: ServerSource,
    pub health: HealthStatus,
}

// ---------------------------------------------------------------------------
// Resolver
// ---------------------------------------------------------------------------

pub struct ServerResolver {
    catalog: Arc<CatalogRegistry>,
    custom: Arc<std::sync::RwLock<CustomServerStore>>,
}

impl ServerResolver {
    pub fn new(
        catalog: Arc<CatalogRegistry>,
        custom: Arc<std::sync::RwLock<CustomServerStore>>,
    ) -> Self {
        Self { catalog, custom }
    }

    /// Resolve a single server ID to a `ResolvedServer`.
    ///
    /// Priority: Catalog > Custom > Unknown.
    pub fn resolve(&self, server_id: &str) -> ResolvedServer {
        // 1. Check catalog first.
        if let Some(entry) = self.catalog.get_server(server_id) {
            return ResolvedServer {
                id: server_id.to_string(),
                config: Some(CatalogRegistry::to_server_config(entry)),
                source: ServerSource::Catalog,
                health: HealthStatus::Healthy,
            };
        }

        // 2. Check custom servers.
        let custom = self.custom.read().unwrap();
        if let Some(config) = custom.get(server_id) {
            return ResolvedServer {
                id: server_id.to_string(),
                config: Some(config.clone()),
                source: ServerSource::Custom,
                health: HealthStatus::Healthy,
            };
        }

        // 3. Unknown.
        ResolvedServer {
            id: server_id.to_string(),
            config: None,
            source: ServerSource::Unknown,
            health: HealthStatus::Unavailable {
                reason: format!(
                    "Server '{}' not found in catalog or custom servers",
                    server_id
                ),
            },
        }
    }

    /// Resolve a slice of server IDs, returning one `ResolvedServer` per entry.
    pub fn resolve_all(&self, server_ids: &[String]) -> Vec<ResolvedServer> {
        server_ids.iter().map(|id| self.resolve(id)).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{Connectivity, Transport};

    // -----------------------------------------------------------------------
    // Fixtures
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
            },
            {
                "id": "github",
                "name": "GitHub",
                "description": "Interact with GitHub repositories",
                "icon": "github.svg",
                "category": "dev-tools",
                "transport": "stdio",
                "command": "npx",
                "args": ["-y", "@modelcontextprotocol/server-github"],
                "connectivity": "online"
            }
        ]
    }"#;

    const CATALOG_PRESETS_JSON: &str = r#"{"version": 1, "presets": []}"#;

    fn make_catalog() -> Arc<CatalogRegistry> {
        Arc::new(
            CatalogRegistry::load(CATALOG_SERVERS_JSON, CATALOG_PRESETS_JSON)
                .expect("test catalog JSON is valid"),
        )
    }

    fn make_custom_server(id: &str, name: &str) -> ServerConfig {
        ServerConfig {
            id: id.to_string(),
            name: name.to_string(),
            transport: Transport::Stdio,
            command: Some("node".to_string()),
            args: Some(vec!["./server.js".to_string()]),
            url: None,
            connectivity: Connectivity::Local,
            description: None,
        }
    }

    /// Returns a `ServerResolver` with `figma` and `github` in the catalog and
    /// `my-db` in the custom store.
    #[allow(dead_code)]
    fn make_resolver() -> ServerResolver {
        let catalog = make_catalog();
        let custom_store =
            CustomServerStore::load_or_default(std::path::PathBuf::from("/tmp/test_custom.json"));
        let custom = Arc::new(std::sync::RwLock::new(custom_store));
        ServerResolver::new(catalog, custom)
    }

    /// Helper that builds a resolver and pre-populates the custom store.
    fn make_resolver_with_custom(custom_ids: &[(&str, &str)]) -> ServerResolver {
        let catalog = make_catalog();

        // Use an in-memory-only store (non-existent path so load_or_default gives empty).
        let path = std::path::PathBuf::from(format!(
            "/tmp/plugmux_test_resolver_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        let mut store = CustomServerStore::load_or_default(path);

        for (id, name) in custom_ids {
            store
                .add(make_custom_server(id, name), &catalog)
                .expect("test custom server add should succeed");
        }

        let custom = Arc::new(std::sync::RwLock::new(store));
        ServerResolver::new(catalog, custom)
    }

    // -----------------------------------------------------------------------
    // resolve: catalog server
    // -----------------------------------------------------------------------

    #[test]
    fn test_resolve_catalog_server_returns_catalog_source() {
        let resolver = make_resolver_with_custom(&[]);
        let result = resolver.resolve("figma");

        assert_eq!(result.id, "figma");
        assert_eq!(result.source, ServerSource::Catalog);
        assert!(result.config.is_some());
        let config = result.config.unwrap();
        assert_eq!(config.id, "figma");
        assert_eq!(config.name, "Figma");
        assert_eq!(config.transport, Transport::Stdio);
        assert_eq!(result.health, HealthStatus::Healthy);
    }

    // -----------------------------------------------------------------------
    // resolve: custom server
    // -----------------------------------------------------------------------

    #[test]
    fn test_resolve_custom_server_returns_custom_source() {
        let resolver = make_resolver_with_custom(&[("my-db", "My Database")]);
        let result = resolver.resolve("my-db");

        assert_eq!(result.id, "my-db");
        assert_eq!(result.source, ServerSource::Custom);
        assert!(result.config.is_some());
        let config = result.config.unwrap();
        assert_eq!(config.id, "my-db");
        assert_eq!(config.name, "My Database");
        assert_eq!(result.health, HealthStatus::Healthy);
    }

    // -----------------------------------------------------------------------
    // resolve: unknown server
    // -----------------------------------------------------------------------

    #[test]
    fn test_resolve_unknown_server_returns_none_config_and_unavailable_health() {
        let resolver = make_resolver_with_custom(&[]);
        let result = resolver.resolve("does-not-exist");

        assert_eq!(result.id, "does-not-exist");
        assert_eq!(result.source, ServerSource::Unknown);
        assert!(result.config.is_none());
        match result.health {
            HealthStatus::Unavailable { reason } => {
                assert!(
                    reason.contains("does-not-exist"),
                    "reason should mention the server id, got: {reason}"
                );
            }
            other => panic!("expected Unavailable, got {:?}", other),
        }
    }

    // -----------------------------------------------------------------------
    // resolve_all: mixed IDs
    // -----------------------------------------------------------------------

    #[test]
    fn test_resolve_all_mixed_ids() {
        let resolver = make_resolver_with_custom(&[("custom-tool", "Custom Tool")]);

        let ids: Vec<String> = vec![
            "figma".to_string(),        // catalog
            "custom-tool".to_string(),  // custom
            "ghost-server".to_string(), // unknown
        ];
        let results = resolver.resolve_all(&ids);

        assert_eq!(results.len(), 3);

        let figma = &results[0];
        assert_eq!(figma.id, "figma");
        assert_eq!(figma.source, ServerSource::Catalog);
        assert!(figma.config.is_some());

        let custom = &results[1];
        assert_eq!(custom.id, "custom-tool");
        assert_eq!(custom.source, ServerSource::Custom);
        assert!(custom.config.is_some());

        let unknown = &results[2];
        assert_eq!(unknown.id, "ghost-server");
        assert_eq!(unknown.source, ServerSource::Unknown);
        assert!(unknown.config.is_none());
    }

    // -----------------------------------------------------------------------
    // resolve_all: empty slice
    // -----------------------------------------------------------------------

    #[test]
    fn test_resolve_all_empty_slice_returns_empty_vec() {
        let resolver = make_resolver_with_custom(&[]);
        let results = resolver.resolve_all(&[]);
        assert!(results.is_empty());
    }

    // -----------------------------------------------------------------------
    // catalog takes priority (collision can't normally happen due to add()
    // guard, but we verify the resolver's own ordering is correct)
    // -----------------------------------------------------------------------

    #[test]
    fn test_catalog_takes_priority_over_custom_when_ids_match() {
        // We can't use the normal `add()` path because it blocks catalog IDs.
        // Instead, build the custom store manually with a catalog ID injected
        // directly into the underlying map via a JSON round-trip.
        let catalog = make_catalog();

        // Build a raw JSON that bypasses the add() collision check.
        let raw_json = r#"{
            "version": 1,
            "servers": [
                {
                    "id": "figma",
                    "name": "Fake Figma Custom",
                    "transport": "stdio",
                    "command": "evil",
                    "connectivity": "local"
                }
            ]
        }"#;

        let path = {
            let dir = tempfile::tempdir().unwrap();
            let p = dir.keep().join("custom.json");
            std::fs::write(&p, raw_json).unwrap();
            p
        };

        let store = CustomServerStore::load(path).expect("raw json should parse");
        let custom = Arc::new(std::sync::RwLock::new(store));
        let resolver = ServerResolver::new(catalog, custom);

        let result = resolver.resolve("figma");

        // Must come from catalog, not custom.
        assert_eq!(result.source, ServerSource::Catalog);
        let config = result.config.unwrap();
        assert_eq!(
            config.name, "Figma",
            "catalog name should win over custom name"
        );
    }
}
