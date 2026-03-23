use std::path::{Path, PathBuf};

use super::ConfigFormat;

const BACKUP_PREFIX: &str = "mcp_servers.backup_original_";

/// Connect an agent to plugmux by adding the plugmux entry to its MCP config.
/// Preserves existing MCP entries. Creates backup of pre-connect state on first connect.
/// Returns the backup file path if a backup was created.
pub fn connect_agent(
    config_path: &Path,
    config_format: &ConfigFormat,
    mcp_key: &str,
    port: u16,
) -> Result<Option<PathBuf>, String> {
    match config_format {
        ConfigFormat::Json => connect_json(config_path, mcp_key, port),
        ConfigFormat::Toml => connect_toml(config_path, mcp_key, port),
    }
}

/// Disconnect by removing only the "plugmux" key. Leaves other MCPs intact.
pub fn disconnect_agent(
    config_path: &Path,
    config_format: &ConfigFormat,
    mcp_key: &str,
) -> Result<(), String> {
    match config_format {
        ConfigFormat::Json => disconnect_json(config_path, mcp_key),
        ConfigFormat::Toml => disconnect_toml(config_path, mcp_key),
    }
}

/// Disconnect and restore: remove plugmux + replace mcpServers with backup contents.
/// Deletes backup file after successful restore.
pub fn disconnect_and_restore(
    config_path: &Path,
    config_format: &ConfigFormat,
    mcp_key: &str,
) -> Result<(), String> {
    let backup_path = get_backup_path(config_path)
        .ok_or_else(|| "No backup file found for this agent config".to_string())?;

    let backup_content = std::fs::read_to_string(&backup_path)
        .map_err(|e| format!("Failed to read backup file: {e}"))?;

    let backup_mcp: serde_json::Value = serde_json::from_str(&backup_content)
        .map_err(|e| format!("Failed to parse backup file: {e}"))?;

    match config_format {
        ConfigFormat::Json => restore_json(config_path, mcp_key, &backup_mcp)?,
        ConfigFormat::Toml => restore_toml(config_path, mcp_key, &backup_mcp)?,
    }

    std::fs::remove_file(&backup_path).map_err(|e| format!("Failed to delete backup file: {e}"))?;

    Ok(())
}

/// Check if a backup file exists for this agent config.
pub fn get_backup_path(config_path: &Path) -> Option<PathBuf> {
    let parent = config_path.parent()?;

    let dir = std::fs::read_dir(parent).ok()?;
    for entry in dir.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with(BACKUP_PREFIX) && name_str.ends_with(".json") {
            return Some(entry.path());
        }
    }

    None
}

// ---------------------------------------------------------------------------
// JSON helpers
// ---------------------------------------------------------------------------

fn connect_json(config_path: &Path, mcp_key: &str, port: u16) -> Result<Option<PathBuf>, String> {
    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {e}"))?;
    }

    // Read or create empty config
    let content = if config_path.exists() {
        std::fs::read_to_string(config_path).map_err(|e| format!("Failed to read config: {e}"))?
    } else {
        "{}".to_string()
    };

    let mut root: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config JSON: {e}"))?;

    // Create backup on first connect
    let mut backup_created = None;
    if get_backup_path(config_path).is_none() {
        let current_mcp = root.get(mcp_key).cloned().unwrap_or(serde_json::json!({}));
        let backup_path = make_backup_path(config_path);
        let backup_json = serde_json::to_string_pretty(&current_mcp)
            .map_err(|e| format!("Failed to serialize backup: {e}"))?;
        std::fs::write(&backup_path, backup_json)
            .map_err(|e| format!("Failed to write backup file: {e}"))?;
        backup_created = Some(backup_path);
    }

    // Get or create mcpServers object
    if root.get(mcp_key).is_none() {
        root[mcp_key] = serde_json::json!({});
    }

    // Insert plugmux entry
    let url = crate::config::global_url(port);
    root[mcp_key]["plugmux"] = serde_json::json!({ "type": "http", "url": url });

    // Write back pretty-printed
    let output = serde_json::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    std::fs::write(config_path, output).map_err(|e| format!("Failed to write config: {e}"))?;

    Ok(backup_created)
}

fn disconnect_json(config_path: &Path, mcp_key: &str) -> Result<(), String> {
    let content =
        std::fs::read_to_string(config_path).map_err(|e| format!("Failed to read config: {e}"))?;

    let mut root: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config JSON: {e}"))?;

    if let Some(serde_json::Value::Object(map)) = root.get_mut(mcp_key) {
        map.remove("plugmux");
    }

    let output = serde_json::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    std::fs::write(config_path, output).map_err(|e| format!("Failed to write config: {e}"))?;

    Ok(())
}

fn restore_json(
    config_path: &Path,
    mcp_key: &str,
    backup_mcp: &serde_json::Value,
) -> Result<(), String> {
    let content =
        std::fs::read_to_string(config_path).map_err(|e| format!("Failed to read config: {e}"))?;

    let mut root: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config JSON: {e}"))?;

    // Replace mcpServers with backup contents
    root[mcp_key] = backup_mcp.clone();

    // Remove plugmux if still present in the restored data
    if let Some(serde_json::Value::Object(map)) = root.get_mut(mcp_key) {
        map.remove("plugmux");
    }

    let output = serde_json::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    std::fs::write(config_path, output).map_err(|e| format!("Failed to write config: {e}"))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// TOML helpers
// ---------------------------------------------------------------------------

fn connect_toml(config_path: &Path, mcp_key: &str, port: u16) -> Result<Option<PathBuf>, String> {
    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {e}"))?;
    }

    // Read or create empty config
    let content = if config_path.exists() {
        std::fs::read_to_string(config_path).map_err(|e| format!("Failed to read config: {e}"))?
    } else {
        String::new()
    };

    let mut root: toml::Value = content
        .parse()
        .map_err(|e| format!("Failed to parse config TOML: {e}"))?;

    // Create backup on first connect — serialize current mcp_servers table as JSON
    let mut backup_created = None;
    if get_backup_path(config_path).is_none() {
        let current_mcp = root
            .get(mcp_key)
            .cloned()
            .unwrap_or(toml::Value::Table(toml::map::Map::new()));
        let json_value = toml_value_to_json(&current_mcp);
        let backup_path = make_backup_path(config_path);
        let backup_json = serde_json::to_string_pretty(&json_value)
            .map_err(|e| format!("Failed to serialize backup: {e}"))?;
        std::fs::write(&backup_path, backup_json)
            .map_err(|e| format!("Failed to write backup file: {e}"))?;
        backup_created = Some(backup_path);
    }

    // Get or create mcp_servers table
    let root_table = root
        .as_table_mut()
        .ok_or("Config root is not a TOML table")?;
    if !root_table.contains_key(mcp_key) {
        root_table.insert(
            mcp_key.to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
    }

    // Insert plugmux entry
    let url = crate::config::global_url(port);
    let mut plugmux_table = toml::map::Map::new();
    plugmux_table.insert("url".to_string(), toml::Value::String(url));

    if let Some(toml::Value::Table(mcp_table)) = root_table.get_mut(mcp_key) {
        mcp_table.insert("plugmux".to_string(), toml::Value::Table(plugmux_table));
    }

    // Write back
    let output = toml::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize config TOML: {e}"))?;
    std::fs::write(config_path, output).map_err(|e| format!("Failed to write config: {e}"))?;

    Ok(backup_created)
}

fn disconnect_toml(config_path: &Path, mcp_key: &str) -> Result<(), String> {
    let content =
        std::fs::read_to_string(config_path).map_err(|e| format!("Failed to read config: {e}"))?;

    let mut root: toml::Value = content
        .parse()
        .map_err(|e| format!("Failed to parse config TOML: {e}"))?;

    if let Some(toml::Value::Table(mcp_table)) = root.get_mut(mcp_key) {
        mcp_table.remove("plugmux");
    }

    let output = toml::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize config TOML: {e}"))?;
    std::fs::write(config_path, output).map_err(|e| format!("Failed to write config: {e}"))?;

    Ok(())
}

fn restore_toml(
    config_path: &Path,
    mcp_key: &str,
    backup_mcp: &serde_json::Value,
) -> Result<(), String> {
    let content =
        std::fs::read_to_string(config_path).map_err(|e| format!("Failed to read config: {e}"))?;

    let mut root: toml::Value = content
        .parse()
        .map_err(|e| format!("Failed to parse config TOML: {e}"))?;

    // Convert backup JSON back to TOML value
    let mut restored_toml = json_value_to_toml(backup_mcp);

    // Remove plugmux if present in restored data
    if let toml::Value::Table(ref mut table) = restored_toml {
        table.remove("plugmux");
    }

    // Replace mcp_servers with restored contents
    let root_table = root
        .as_table_mut()
        .ok_or("Config root is not a TOML table")?;
    root_table.insert(mcp_key.to_string(), restored_toml);

    let output = toml::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize config TOML: {e}"))?;
    std::fs::write(config_path, output).map_err(|e| format!("Failed to write config: {e}"))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

fn toml_value_to_json(value: &toml::Value) -> serde_json::Value {
    match value {
        toml::Value::String(s) => serde_json::Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::json!(*i),
        toml::Value::Float(f) => serde_json::json!(*f),
        toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(toml_value_to_json).collect())
        }
        toml::Value::Table(table) => {
            let map: serde_json::Map<String, serde_json::Value> = table
                .iter()
                .map(|(k, v)| (k.clone(), toml_value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
    }
}

fn json_value_to_toml(value: &serde_json::Value) -> toml::Value {
    match value {
        serde_json::Value::Null => toml::Value::String("null".to_string()),
        serde_json::Value::Bool(b) => toml::Value::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                toml::Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                toml::Value::Float(f)
            } else {
                toml::Value::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => toml::Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            toml::Value::Array(arr.iter().map(json_value_to_toml).collect())
        }
        serde_json::Value::Object(map) => {
            let mut table = toml::map::Map::new();
            for (k, v) in map {
                table.insert(k.clone(), json_value_to_toml(v));
            }
            toml::Value::Table(table)
        }
    }
}

fn make_backup_path(config_path: &Path) -> PathBuf {
    let parent = config_path.parent().unwrap_or(Path::new("."));
    let date = chrono::Local::now().format("%Y-%m-%d");
    parent.join(format!("{BACKUP_PREFIX}{date}.json"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // 1. connect_agent JSON — adds plugmux alongside existing MCPs, creates backup
    #[test]
    fn test_connect_json_adds_plugmux_and_creates_backup() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.json");
        std::fs::write(&config, r#"{"mcpServers": {"github": {"command": "gh"}}}"#).unwrap();

        let result = connect_agent(&config, &ConfigFormat::Json, "mcpServers", 4242);
        assert!(result.is_ok());
        let backup = result.unwrap();
        assert!(
            backup.is_some(),
            "backup should be created on first connect"
        );
        assert!(backup.unwrap().exists(), "backup file should exist on disk");

        // Verify config contents
        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config).unwrap()).unwrap();
        let mcp = content.get("mcpServers").unwrap().as_object().unwrap();
        assert!(mcp.contains_key("plugmux"), "plugmux should be present");
        assert!(
            mcp.contains_key("github"),
            "existing MCP should be preserved"
        );
        assert_eq!(mcp["plugmux"]["url"], "http://localhost:4242/env/global");
    }

    // 2. connect_agent JSON — backup only created on first connect
    #[test]
    fn test_connect_json_backup_not_overwritten() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.json");
        std::fs::write(&config, r#"{"mcpServers": {"github": {"command": "gh"}}}"#).unwrap();

        // First connect — creates backup
        let first = connect_agent(&config, &ConfigFormat::Json, "mcpServers", 4242).unwrap();
        assert!(first.is_some());
        let backup_path = first.unwrap();
        let backup_content = std::fs::read_to_string(&backup_path).unwrap();

        // Second connect — should NOT create new backup
        let second = connect_agent(&config, &ConfigFormat::Json, "mcpServers", 5555).unwrap();
        assert!(second.is_none(), "backup should not be created again");

        // Backup content should be unchanged (still the original pre-connect state)
        let backup_content_after = std::fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, backup_content_after);
    }

    // 3. connect_agent JSON — creates file if not exists
    #[test]
    fn test_connect_json_creates_file_if_missing() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("subdir").join("config.json");
        assert!(!config.exists());

        let result = connect_agent(&config, &ConfigFormat::Json, "mcpServers", 4242);
        assert!(result.is_ok());
        assert!(config.exists(), "config file should be created");

        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config).unwrap()).unwrap();
        let mcp = content.get("mcpServers").unwrap().as_object().unwrap();
        assert!(mcp.contains_key("plugmux"));
    }

    // 4. connect_agent JSON — preserves non-MCP config keys
    #[test]
    fn test_connect_json_preserves_other_keys() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.json");
        std::fs::write(
            &config,
            r#"{"theme": "dark", "mcpServers": {}, "version": 3}"#,
        )
        .unwrap();

        connect_agent(&config, &ConfigFormat::Json, "mcpServers", 4242).unwrap();

        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config).unwrap()).unwrap();
        assert_eq!(content["theme"], "dark");
        assert_eq!(content["version"], 3);
        assert!(content["mcpServers"]["plugmux"].is_object());
    }

    // 5. disconnect_agent JSON — removes only plugmux, keeps others
    #[test]
    fn test_disconnect_json_removes_only_plugmux() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.json");
        std::fs::write(
            &config,
            r#"{"mcpServers": {"plugmux": {"type": "http", "url": "http://localhost:4242/env/global"}, "github": {"command": "gh"}}}"#,
        )
        .unwrap();

        disconnect_agent(&config, &ConfigFormat::Json, "mcpServers").unwrap();

        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config).unwrap()).unwrap();
        let mcp = content.get("mcpServers").unwrap().as_object().unwrap();
        assert!(!mcp.contains_key("plugmux"), "plugmux should be removed");
        assert!(mcp.contains_key("github"), "other MCPs should remain");
    }

    // 6. disconnect_agent JSON — no error if plugmux doesn't exist
    #[test]
    fn test_disconnect_json_no_error_if_no_plugmux() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.json");
        std::fs::write(&config, r#"{"mcpServers": {"github": {"command": "gh"}}}"#).unwrap();

        let result = disconnect_agent(&config, &ConfigFormat::Json, "mcpServers");
        assert!(result.is_ok());
    }

    // 7. disconnect_and_restore JSON — restores from backup, deletes backup
    #[test]
    fn test_disconnect_and_restore_json() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.json");
        std::fs::write(
            &config,
            r#"{"mcpServers": {"github": {"command": "gh"}, "slack": {"command": "slack-mcp"}}}"#,
        )
        .unwrap();

        // Connect (creates backup with github + slack)
        let backup = connect_agent(&config, &ConfigFormat::Json, "mcpServers", 4242)
            .unwrap()
            .unwrap();
        assert!(backup.exists());

        // Verify plugmux is now in config
        let mid: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config).unwrap()).unwrap();
        assert!(mid["mcpServers"]["plugmux"].is_object());

        // Disconnect and restore
        disconnect_and_restore(&config, &ConfigFormat::Json, "mcpServers").unwrap();

        // Verify plugmux is gone and originals are restored
        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config).unwrap()).unwrap();
        let mcp = content.get("mcpServers").unwrap().as_object().unwrap();
        assert!(!mcp.contains_key("plugmux"), "plugmux should be removed");
        assert!(mcp.contains_key("github"), "github should be restored");
        assert!(mcp.contains_key("slack"), "slack should be restored");

        // Backup should be deleted
        assert!(!backup.exists(), "backup should be deleted after restore");
    }

    // 8. disconnect_and_restore — error if no backup
    #[test]
    fn test_disconnect_and_restore_error_if_no_backup() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.json");
        std::fs::write(
            &config,
            r#"{"mcpServers": {"plugmux": {"type": "http", "url": "http://localhost:4242/env/global"}}}"#,
        )
        .unwrap();

        let result = disconnect_and_restore(&config, &ConfigFormat::Json, "mcpServers");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No backup file found"));
    }

    // 9. get_backup_path — returns Some when backup exists, None otherwise
    #[test]
    fn test_get_backup_path() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.json");
        std::fs::write(&config, "{}").unwrap();

        assert!(get_backup_path(&config).is_none());

        // Create a backup file
        let backup = tmp
            .path()
            .join("mcp_servers.backup_original_2026-01-15.json");
        std::fs::write(&backup, "{}").unwrap();

        let found = get_backup_path(&config);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), backup);
    }

    // 10. connect_agent TOML — adds plugmux to mcp_servers table
    #[test]
    fn test_connect_toml_adds_plugmux() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.toml");
        std::fs::write(&config, "[mcp_servers.github]\ncommand = \"gh\"\n").unwrap();

        let result = connect_agent(&config, &ConfigFormat::Toml, "mcp_servers", 4242);
        assert!(result.is_ok());
        let backup = result.unwrap();
        assert!(backup.is_some(), "backup should be created");

        let content: toml::Value = std::fs::read_to_string(&config).unwrap().parse().unwrap();
        let mcp = content.get("mcp_servers").unwrap().as_table().unwrap();
        assert!(mcp.contains_key("plugmux"), "plugmux should be added");
        assert!(
            mcp.contains_key("github"),
            "existing entry should be preserved"
        );
        assert_eq!(
            mcp["plugmux"]["url"].as_str().unwrap(),
            "http://localhost:4242/env/global"
        );

        // Verify backup is JSON (not TOML)
        let backup_path = backup.unwrap();
        let backup_content = std::fs::read_to_string(&backup_path).unwrap();
        let backup_json: serde_json::Value = serde_json::from_str(&backup_content).unwrap();
        assert!(
            backup_json.get("github").is_some(),
            "backup should contain original entries as JSON"
        );
    }

    // 11. disconnect_agent TOML — removes plugmux from mcp_servers
    #[test]
    fn test_disconnect_toml_removes_plugmux() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.toml");
        std::fs::write(
            &config,
            "[mcp_servers.plugmux]\nurl = \"http://localhost:4242/env/global\"\n\n[mcp_servers.github]\ncommand = \"gh\"\n",
        )
        .unwrap();

        disconnect_agent(&config, &ConfigFormat::Toml, "mcp_servers").unwrap();

        let content: toml::Value = std::fs::read_to_string(&config).unwrap().parse().unwrap();
        let mcp = content.get("mcp_servers").unwrap().as_table().unwrap();
        assert!(!mcp.contains_key("plugmux"), "plugmux should be removed");
        assert!(mcp.contains_key("github"), "other entries should remain");
    }
}
