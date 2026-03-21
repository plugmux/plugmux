use std::fs;
use std::path::PathBuf;

use crate::catalog::CatalogRegistry;
use crate::config::{self, Config, ConfigError, Environment};
use crate::custom_servers::CustomServersData;
use crate::server::ServerConfig;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns true when a Phase-2 `plugmux.json` exists and the new `config.json`
/// does not — i.e. a migration is needed.
pub fn needs_migration() -> bool {
    let old = config::config_dir().join("plugmux.json");
    let new = config::config_path();
    old.exists() && !new.exists()
}

/// Migrates Phase-2 `plugmux.json` → Phase-3 `config.json` + `custom_servers.json`.
///
/// Steps:
/// 1. Parse old `plugmux.json` as a `serde_json::Value`.
/// 2. Build a "default" environment from `main.servers`.
/// 3. Build additional environments from `environments[*].servers`.
/// 4. For every server, try to match it to a catalog entry by `command` or `url`.
///    - Matched  → use the catalog ID (no custom server entry needed).
///    - Unmatched → add to `custom_servers.json` and use the old server ID.
/// 5. Write `config.json` and `custom_servers.json`.
/// 6. Rename `plugmux.json` → `plugmux.json.backup`.
pub fn migrate(catalog: &CatalogRegistry) -> Result<(), ConfigError> {
    let old_path = config::config_dir().join("plugmux.json");
    let config_path = config::config_path();
    let custom_path = config::config_dir().join("custom_servers.json");

    let old_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&old_path)?)?;

    let mut custom_servers: Vec<ServerConfig> = Vec::new();
    let mut environments: Vec<Environment> = Vec::new();

    // 1. main.servers → "default" environment
    let main_server_ids = process_servers(
        &old_json["main"]["servers"],
        catalog,
        &mut custom_servers,
    );
    environments.push(Environment {
        id: "default".to_string(),
        name: "Default".to_string(),
        servers: main_server_ids,
    });

    // 2. Each old environment → its own environment entry
    if let Some(envs) = old_json["environments"].as_array() {
        for env in envs {
            let id = env["id"].as_str().unwrap_or("unnamed").to_string();
            let name = env["name"].as_str().unwrap_or(&id).to_string();
            let server_ids = process_servers(
                &env["servers"],
                catalog,
                &mut custom_servers,
            );
            environments.push(Environment {
                id,
                name,
                servers: server_ids,
            });
        }
    }

    // 3. Write config.json
    let cfg = Config {
        port: 4242,
        permissions: Default::default(),
        environments,
    };
    config::save(&config_path, &cfg)?;

    // 4. Write custom_servers.json
    let custom_data = CustomServersData {
        version: 1,
        servers: custom_servers,
    };
    let json = serde_json::to_string_pretty(&custom_data)?;
    fs::write(&custom_path, json)?;

    // 5. Rename old file to backup
    let backup_path = config::config_dir().join("plugmux.json.backup");
    fs::rename(&old_path, &backup_path)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Try to match a server JSON object against the catalog by `command` or `url`.
/// Returns the catalog ID on a hit, `None` otherwise.
fn match_catalog(server: &serde_json::Value, catalog: &CatalogRegistry) -> Option<String> {
    for entry in catalog.list_servers() {
        // Match by command (stdio servers)
        if let (Some(cmd), Some(entry_cmd)) =
            (server["command"].as_str(), entry.command.as_deref())
        {
            if cmd == entry_cmd {
                return Some(entry.id.clone());
            }
        }
        // Match by url (HTTP servers)
        if let (Some(url), Some(entry_url)) =
            (server["url"].as_str(), entry.url.as_deref())
        {
            if url == entry_url {
                return Some(entry.id.clone());
            }
        }
    }
    None
}

/// Processes a JSON array of server objects.
///
/// Each server is either matched to a catalog entry (by command/url) or added
/// to `custom_servers` and referenced by its old ID.  Duplicate IDs are
/// silently de-duplicated.  Returns the ordered list of server ID strings for
/// the environment.
fn process_servers(
    servers_val: &serde_json::Value,
    catalog: &CatalogRegistry,
    custom_servers: &mut Vec<ServerConfig>,
) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();

    let Some(servers) = servers_val.as_array() else {
        return ids;
    };

    for server in servers {
        if let Some(catalog_id) = match_catalog(server, catalog) {
            if !ids.contains(&catalog_id) {
                ids.push(catalog_id);
            }
        } else {
            // Fall back to the old ID
            let id = server["id"].as_str().unwrap_or("unknown").to_string();

            // Deserialise into ServerConfig; unknown fields (e.g. `enabled`)
            // are silently ignored because ServerConfig has no
            // `#[serde(deny_unknown_fields)]`.
            if let Ok(config) = serde_json::from_value::<ServerConfig>(server.clone()) {
                if !custom_servers.iter().any(|s| s.id == config.id) {
                    custom_servers.push(config);
                }
            }

            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }

    ids
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::CatalogRegistry;
    use tempfile::TempDir;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Minimal catalog with one stdio server (`npx` command) and one http server.
    const SERVERS_JSON: &str = r#"{
        "version": 1,
        "servers": [
            {
                "id": "filesystem",
                "name": "Filesystem",
                "description": "Access the local filesystem",
                "icon": "fs.svg",
                "category": "local",
                "transport": "stdio",
                "command": "npx",
                "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
                "connectivity": "local"
            },
            {
                "id": "context7",
                "name": "Context7",
                "description": "Up-to-date docs",
                "icon": "ctx.svg",
                "category": "dev-tools",
                "transport": "http",
                "url": "https://mcp.context7.com/mcp",
                "connectivity": "online"
            }
        ]
    }"#;

    const PRESETS_JSON: &str = r#"{"version": 1, "presets": []}"#;

    fn make_catalog() -> CatalogRegistry {
        CatalogRegistry::load(SERVERS_JSON, PRESETS_JSON).expect("test catalog valid")
    }

    /// Write `plugmux.json` into `dir`, return the dir path.
    fn write_old_config(dir: &TempDir, content: &str) -> PathBuf {
        let path = dir.path().join("plugmux.json");
        fs::write(&path, content).unwrap();
        dir.path().to_path_buf()
    }

    // -----------------------------------------------------------------------
    // needs_migration
    // -----------------------------------------------------------------------

    #[test]
    fn test_needs_migration_true_when_old_exists_new_does_not() {
        let dir = TempDir::new().unwrap();
        let old_path = dir.path().join("plugmux.json");
        fs::write(&old_path, "{}").unwrap();
        // new config.json does NOT exist

        let result = needs_migration_in(dir.path(), dir.path());
        assert!(result, "should need migration when plugmux.json exists and config.json does not");
    }

    #[test]
    fn test_needs_migration_false_when_new_already_exists() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("plugmux.json"), "{}").unwrap();
        fs::write(dir.path().join("config.json"), "{}").unwrap();

        let result = needs_migration_in(dir.path(), dir.path());
        assert!(!result, "should not need migration when config.json already exists");
    }

    #[test]
    fn test_needs_migration_false_when_old_does_not_exist() {
        let dir = TempDir::new().unwrap();
        // neither file exists

        let result = needs_migration_in(dir.path(), dir.path());
        assert!(!result, "should not need migration when plugmux.json does not exist");
    }

    // -----------------------------------------------------------------------
    // migrate: main.servers → default environment
    // -----------------------------------------------------------------------

    #[test]
    fn test_migrate_main_servers_become_default_environment() {
        let dir = TempDir::new().unwrap();
        let old_json = r#"{
            "main": {
                "servers": [
                    {
                        "id": "my-custom",
                        "name": "My Custom",
                        "transport": "stdio",
                        "command": "node",
                        "args": ["./server.js"],
                        "connectivity": "local",
                        "enabled": true
                    }
                ]
            },
            "environments": []
        }"#;
        write_old_config(&dir, old_json);

        let catalog = make_catalog();
        migrate_in(dir.path(), &catalog).unwrap();

        let cfg_path = dir.path().join("config.json");
        let cfg: Config = serde_json::from_str(&fs::read_to_string(&cfg_path).unwrap()).unwrap();

        assert_eq!(cfg.environments.len(), 1);
        let default_env = cfg.environments.iter().find(|e| e.id == "default").unwrap();
        assert!(
            default_env.servers.contains(&"my-custom".to_string()),
            "default env should contain the main server id"
        );
    }

    // -----------------------------------------------------------------------
    // migrate: environments → separate environment entries
    // -----------------------------------------------------------------------

    #[test]
    fn test_migrate_environments_become_separate_entries() {
        let dir = TempDir::new().unwrap();
        let old_json = r#"{
            "main": { "servers": [] },
            "environments": [
                {
                    "id": "my-project",
                    "name": "My Project",
                    "endpoint": "http://localhost:4242/env/my-project",
                    "servers": [
                        {
                            "id": "custom-api",
                            "name": "Custom API",
                            "transport": "http",
                            "url": "https://api.example.com/mcp",
                            "connectivity": "online",
                            "enabled": true
                        }
                    ],
                    "overrides": []
                }
            ]
        }"#;
        write_old_config(&dir, old_json);

        let catalog = make_catalog();
        migrate_in(dir.path(), &catalog).unwrap();

        let cfg_path = dir.path().join("config.json");
        let cfg: Config = serde_json::from_str(&fs::read_to_string(&cfg_path).unwrap()).unwrap();

        // Should have default + my-project
        assert_eq!(cfg.environments.len(), 2);
        let proj_env = cfg.environments.iter().find(|e| e.id == "my-project").unwrap();
        assert_eq!(proj_env.name, "My Project");
        assert!(proj_env.servers.contains(&"custom-api".to_string()));
    }

    // -----------------------------------------------------------------------
    // migrate: servers matching catalog → catalog ID references
    // -----------------------------------------------------------------------

    #[test]
    fn test_migrate_catalog_matched_server_uses_catalog_id() {
        let dir = TempDir::new().unwrap();
        // "npx" matches the catalog "filesystem" entry
        let old_json = r#"{
            "main": {
                "servers": [
                    {
                        "id": "filesystem",
                        "name": "Filesystem",
                        "transport": "stdio",
                        "command": "npx",
                        "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
                        "connectivity": "local",
                        "enabled": true
                    }
                ]
            },
            "environments": []
        }"#;
        write_old_config(&dir, old_json);

        let catalog = make_catalog();
        migrate_in(dir.path(), &catalog).unwrap();

        let cfg_path = dir.path().join("config.json");
        let cfg: Config = serde_json::from_str(&fs::read_to_string(&cfg_path).unwrap()).unwrap();

        let default_env = cfg.environments.iter().find(|e| e.id == "default").unwrap();
        assert!(
            default_env.servers.contains(&"filesystem".to_string()),
            "catalog-matched server should use catalog id 'filesystem'"
        );

        // Nothing should end up in custom_servers.json
        let custom_path = dir.path().join("custom_servers.json");
        let custom: CustomServersData =
            serde_json::from_str(&fs::read_to_string(&custom_path).unwrap()).unwrap();
        assert!(
            custom.servers.is_empty(),
            "catalog-matched server must not appear in custom_servers"
        );
    }

    // -----------------------------------------------------------------------
    // migrate: servers matching catalog by URL → catalog ID references
    // -----------------------------------------------------------------------

    #[test]
    fn test_migrate_catalog_matched_server_by_url_uses_catalog_id() {
        let dir = TempDir::new().unwrap();
        // "https://mcp.context7.com/mcp" matches the catalog "context7" entry
        let old_json = r#"{
            "main": {
                "servers": [
                    {
                        "id": "context7",
                        "name": "Context7",
                        "transport": "http",
                        "url": "https://mcp.context7.com/mcp",
                        "connectivity": "online",
                        "enabled": true
                    }
                ]
            },
            "environments": []
        }"#;
        write_old_config(&dir, old_json);

        let catalog = make_catalog();
        migrate_in(dir.path(), &catalog).unwrap();

        let cfg_path = dir.path().join("config.json");
        let cfg: Config = serde_json::from_str(&fs::read_to_string(&cfg_path).unwrap()).unwrap();

        let default_env = cfg.environments.iter().find(|e| e.id == "default").unwrap();
        assert!(default_env.servers.contains(&"context7".to_string()));

        let custom_path = dir.path().join("custom_servers.json");
        let custom: CustomServersData =
            serde_json::from_str(&fs::read_to_string(&custom_path).unwrap()).unwrap();
        assert!(custom.servers.is_empty(), "url-matched server must not appear in custom_servers");
    }

    // -----------------------------------------------------------------------
    // migrate: servers NOT matching catalog → moved to custom_servers.json
    // -----------------------------------------------------------------------

    #[test]
    fn test_migrate_unmatched_server_goes_to_custom_servers() {
        let dir = TempDir::new().unwrap();
        let old_json = r#"{
            "main": {
                "servers": [
                    {
                        "id": "my-internal-api",
                        "name": "Internal API",
                        "transport": "http",
                        "url": "https://internal.corp/mcp",
                        "connectivity": "online",
                        "enabled": false
                    }
                ]
            },
            "environments": []
        }"#;
        write_old_config(&dir, old_json);

        let catalog = make_catalog();
        migrate_in(dir.path(), &catalog).unwrap();

        let custom_path = dir.path().join("custom_servers.json");
        let custom: CustomServersData =
            serde_json::from_str(&fs::read_to_string(&custom_path).unwrap()).unwrap();

        assert_eq!(custom.servers.len(), 1, "unmatched server should be in custom_servers");
        assert_eq!(custom.servers[0].id, "my-internal-api");
        assert_eq!(custom.servers[0].name, "Internal API");

        // The environment should reference the old id
        let cfg_path = dir.path().join("config.json");
        let cfg: Config = serde_json::from_str(&fs::read_to_string(&cfg_path).unwrap()).unwrap();
        let default_env = cfg.environments.iter().find(|e| e.id == "default").unwrap();
        assert!(default_env.servers.contains(&"my-internal-api".to_string()));
    }

    // -----------------------------------------------------------------------
    // migrate: old file renamed to .backup after migration
    // -----------------------------------------------------------------------

    #[test]
    fn test_old_file_renamed_to_backup_after_migration() {
        let dir = TempDir::new().unwrap();
        let old_json = r#"{"main": {"servers": []}, "environments": []}"#;
        write_old_config(&dir, old_json);

        let catalog = make_catalog();
        migrate_in(dir.path(), &catalog).unwrap();

        assert!(
            !dir.path().join("plugmux.json").exists(),
            "original plugmux.json should be removed"
        );
        assert!(
            dir.path().join("plugmux.json.backup").exists(),
            "plugmux.json.backup should exist after migration"
        );
    }

    // -----------------------------------------------------------------------
    // migrate: environments get server IDs from their own servers array
    // -----------------------------------------------------------------------

    #[test]
    fn test_environment_servers_come_from_their_own_array_not_main() {
        let dir = TempDir::new().unwrap();
        let old_json = r#"{
            "main": {
                "servers": [
                    {
                        "id": "main-only",
                        "name": "Main Only",
                        "transport": "stdio",
                        "command": "main-binary",
                        "connectivity": "local",
                        "enabled": true
                    }
                ]
            },
            "environments": [
                {
                    "id": "work",
                    "name": "Work",
                    "servers": [
                        {
                            "id": "work-only",
                            "name": "Work Only",
                            "transport": "stdio",
                            "command": "work-binary",
                            "connectivity": "local",
                            "enabled": true
                        }
                    ],
                    "overrides": []
                }
            ]
        }"#;
        write_old_config(&dir, old_json);

        let catalog = make_catalog();
        migrate_in(dir.path(), &catalog).unwrap();

        let cfg_path = dir.path().join("config.json");
        let cfg: Config = serde_json::from_str(&fs::read_to_string(&cfg_path).unwrap()).unwrap();

        let default_env = cfg.environments.iter().find(|e| e.id == "default").unwrap();
        let work_env = cfg.environments.iter().find(|e| e.id == "work").unwrap();

        // default only has main-only
        assert!(default_env.servers.contains(&"main-only".to_string()));
        assert!(!default_env.servers.contains(&"work-only".to_string()));

        // work only has work-only
        assert!(work_env.servers.contains(&"work-only".to_string()));
        assert!(!work_env.servers.contains(&"main-only".to_string()));
    }

    // -----------------------------------------------------------------------
    // migrate: enabled field on old servers is silently ignored
    // -----------------------------------------------------------------------

    #[test]
    fn test_old_enabled_field_silently_ignored_during_deserialization() {
        let dir = TempDir::new().unwrap();
        let old_json = r#"{
            "main": {
                "servers": [
                    {
                        "id": "my-server",
                        "name": "My Server",
                        "transport": "stdio",
                        "command": "my-cmd",
                        "connectivity": "local",
                        "enabled": true
                    }
                ]
            },
            "environments": []
        }"#;
        write_old_config(&dir, old_json);

        let catalog = make_catalog();
        // Must not error out due to `enabled` field
        let result = migrate_in(dir.path(), &catalog);
        assert!(result.is_ok(), "migration should succeed even with `enabled` field present");

        let custom_path = dir.path().join("custom_servers.json");
        let custom: CustomServersData =
            serde_json::from_str(&fs::read_to_string(&custom_path).unwrap()).unwrap();
        // Verify the server was parsed and stored without `enabled`
        assert_eq!(custom.servers.len(), 1);
        assert_eq!(custom.servers[0].id, "my-server");
        let serialized = serde_json::to_string(&custom.servers[0]).unwrap();
        assert!(!serialized.contains("enabled"), "serialized ServerConfig must not contain `enabled`");
    }

    // -----------------------------------------------------------------------
    // migrate: duplicate server IDs are de-duplicated across environments
    // -----------------------------------------------------------------------

    #[test]
    fn test_duplicate_server_ids_are_deduplicated() {
        let dir = TempDir::new().unwrap();
        // Two servers with the same command ("npx") both match catalog "filesystem"
        let old_json = r#"{
            "main": {
                "servers": [
                    {
                        "id": "fs1",
                        "name": "FS 1",
                        "transport": "stdio",
                        "command": "npx",
                        "connectivity": "local",
                        "enabled": true
                    },
                    {
                        "id": "fs2",
                        "name": "FS 2",
                        "transport": "stdio",
                        "command": "npx",
                        "connectivity": "local",
                        "enabled": true
                    }
                ]
            },
            "environments": []
        }"#;
        write_old_config(&dir, old_json);

        let catalog = make_catalog();
        migrate_in(dir.path(), &catalog).unwrap();

        let cfg_path = dir.path().join("config.json");
        let cfg: Config = serde_json::from_str(&fs::read_to_string(&cfg_path).unwrap()).unwrap();

        let default_env = cfg.environments.iter().find(|e| e.id == "default").unwrap();
        // Both match "filesystem" in the catalog — only one reference should appear
        let fs_count = default_env.servers.iter().filter(|s| *s == "filesystem").count();
        assert_eq!(fs_count, 1, "deduplicated: filesystem should appear only once");
    }

    // -----------------------------------------------------------------------
    // Helpers to run migration against a temp directory (not ~/.config/plugmux)
    // -----------------------------------------------------------------------

    /// Mirror of `needs_migration()` but operates on arbitrary base directories.
    fn needs_migration_in(old_dir: &std::path::Path, new_dir: &std::path::Path) -> bool {
        old_dir.join("plugmux.json").exists() && !new_dir.join("config.json").exists()
    }

    /// Runs migration logic against `base_dir` instead of `~/.config/plugmux`.
    fn migrate_in(
        base_dir: &std::path::Path,
        catalog: &CatalogRegistry,
    ) -> Result<(), ConfigError> {
        let old_path = base_dir.join("plugmux.json");
        let config_path = base_dir.join("config.json");
        let custom_path = base_dir.join("custom_servers.json");

        let old_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&old_path)?)?;

        let mut custom_servers: Vec<ServerConfig> = Vec::new();
        let mut environments: Vec<Environment> = Vec::new();

        let main_server_ids = process_servers(
            &old_json["main"]["servers"],
            catalog,
            &mut custom_servers,
        );
        environments.push(Environment {
            id: "default".to_string(),
            name: "Default".to_string(),
            servers: main_server_ids,
        });

        if let Some(envs) = old_json["environments"].as_array() {
            for env in envs {
                let id = env["id"].as_str().unwrap_or("unnamed").to_string();
                let name = env["name"].as_str().unwrap_or(&id).to_string();
                let server_ids = process_servers(
                    &env["servers"],
                    catalog,
                    &mut custom_servers,
                );
                environments.push(Environment {
                    id,
                    name,
                    servers: server_ids,
                });
            }
        }

        let cfg = Config {
            port: 4242,
            permissions: Default::default(),
            environments,
        };
        config::save(&config_path, &cfg)?;

        let custom_data = CustomServersData {
            version: 1,
            servers: custom_servers,
        };
        fs::write(&custom_path, serde_json::to_string_pretty(&custom_data)?)?;

        let backup_path = base_dir.join("plugmux.json.backup");
        fs::rename(&old_path, &backup_path)?;

        Ok(())
    }
}
