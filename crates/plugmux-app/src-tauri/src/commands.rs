use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use plugmux_core::agents::{
    AgentEntry, AgentRegistry, AgentSource, AgentStateEntry, ConfigFormat, DetectedAgent,
};
use plugmux_core::api_client::{
    AuthUser, CatalogResponse, CollectionsResponse, HealthResponse, RemoteCatalogServer,
    RemoteCollection,
};
use plugmux_core::catalog::{CatalogEntry, Preset};
use plugmux_core::config::{Config, PermissionLevel, Permissions};
use plugmux_core::db::{
    self, environments as db_envs,
    logs::{self, LogEntry},
};
use plugmux_core::environment;
use plugmux_core::migration;
use plugmux_core::server::{HealthStatus, ServerConfig};

use crate::engine::Engine;
use crate::events;

/// Serializable environment for Tauri commands (mirrors `db::environments::EnvironmentRow`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub servers: Vec<String>,
}

// ---------------------------------------------------------------------------
// Engine commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_engine_status(engine: State<'_, Arc<Engine>>) -> Result<String, String> {
    let status = engine.status.read().await;
    Ok(status.as_str().to_string())
}

#[tauri::command]
pub async fn start_engine(engine: State<'_, Arc<Engine>>, app: AppHandle) -> Result<(), String> {
    engine.start().await?;
    let _ = app.emit(
        events::ENGINE_STATUS_CHANGED,
        events::EngineStatusPayload {
            status: "running".to_string(),
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn stop_engine(engine: State<'_, Arc<Engine>>, app: AppHandle) -> Result<(), String> {
    engine.stop().await?;
    let _ = app.emit(
        events::ENGINE_STATUS_CHANGED,
        events::EngineStatusPayload {
            status: "stopped".to_string(),
        },
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Config commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_config(engine: State<'_, Arc<Engine>>) -> Result<Config, String> {
    let cfg = engine.config.read().await;
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn get_port(engine: State<'_, Arc<Engine>>) -> Result<u16, String> {
    Ok(*engine.port.read().await)
}

#[tauri::command]
pub async fn set_port(engine: State<'_, Arc<Engine>>, port: u16) -> Result<(), String> {
    {
        let mut cfg = engine.config.write().await;
        cfg.port = port;
    }
    *engine.port.write().await = port;
    engine.save_config().await
}

// ---------------------------------------------------------------------------
// Permission commands (global)
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_permissions(engine: State<'_, Arc<Engine>>) -> Result<Permissions, String> {
    let cfg = engine.config.read().await;
    Ok(cfg.permissions.clone())
}

#[tauri::command]
pub async fn set_permission(
    engine: State<'_, Arc<Engine>>,
    action: String,
    level: String,
) -> Result<(), String> {
    let perm_level = match level.as_str() {
        "allow" => PermissionLevel::Allow,
        "approve" => PermissionLevel::Approve,
        "disable" => PermissionLevel::Disable,
        _ => return Err(format!("Invalid permission level: {level}")),
    };

    {
        let mut cfg = engine.config.write().await;
        match action.as_str() {
            "enable_server" => cfg.permissions.enable_server = perm_level,
            "disable_server" => cfg.permissions.disable_server = perm_level,
            _ => return Err(format!("Unknown permission action: {action}")),
        }
    }
    engine.save_config().await
}

// ---------------------------------------------------------------------------
// Environment commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn list_environments(engine: State<'_, Arc<Engine>>) -> Result<Vec<Environment>, String> {
    let rows = db_envs::list_environments(&engine.db);
    let mut envs = Vec::with_capacity(rows.len());
    for row in rows {
        let servers = db_envs::get_server_ids(&engine.db, &row.id).unwrap_or_default();
        envs.push(Environment {
            id: row.id,
            name: row.name,
            servers,
        });
    }
    Ok(envs)
}

#[tauri::command]
pub async fn create_environment(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    name: String,
) -> Result<Environment, String> {
    let id = plugmux_core::slug::slugify(&name);
    db_envs::add_environment(&engine.db, &id, &name)?;
    let _ = app.emit(
        events::ENVIRONMENT_CREATED,
        events::EnvironmentChangedPayload { env_id: id.clone() },
    );
    Ok(Environment {
        id,
        name,
        servers: vec![],
    })
}

#[tauri::command]
pub async fn delete_environment(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    db_envs::remove_environment(&engine.db, &id)?;
    let _ = app.emit(
        events::ENVIRONMENT_DELETED,
        events::EnvironmentChangedPayload { env_id: id },
    );
    Ok(())
}

#[tauri::command]
pub async fn rename_environment(
    engine: State<'_, Arc<Engine>>,
    id: String,
    name: String,
) -> Result<(), String> {
    let conn = engine.db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE environments SET name = ?1 WHERE id = ?2",
        rusqlite::params![name, id],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Servers in environments
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn add_server_to_env(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    env_id: String,
    server_id: String,
) -> Result<(), String> {
    environment::add_server(&engine.db, &env_id, &server_id)?;
    let _ = app.emit(
        events::SERVER_ADDED,
        events::ServerChangedPayload { server_id, env_id },
    );
    Ok(())
}

#[tauri::command]
pub async fn remove_server_from_env(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    env_id: String,
    server_id: String,
) -> Result<(), String> {
    environment::remove_server(&engine.db, &env_id, &server_id)?;
    let _ = app.emit(
        events::SERVER_REMOVED,
        events::ServerChangedPayload { server_id, env_id },
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Custom servers
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn list_custom_servers(
    engine: State<'_, Arc<Engine>>,
) -> Result<Vec<ServerConfig>, String> {
    let lock = engine.custom_servers.read().map_err(|e| e.to_string())?;
    Ok(lock.list().into_iter().cloned().collect())
}

#[tauri::command]
pub async fn add_custom_server(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    config: ServerConfig,
) -> Result<(), String> {
    let server_id = config.id.clone();
    {
        let mut lock = engine.custom_servers.write().map_err(|e| e.to_string())?;
        lock.add(config, &engine.catalog)
            .map_err(|e| e.to_string())?;
    }
    engine.save_custom_servers()?;
    let _ = app.emit(
        events::CUSTOM_SERVER_ADDED,
        events::CustomServerChangedPayload { server_id },
    );
    Ok(())
}

#[tauri::command]
pub async fn update_custom_server(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    id: String,
    config: ServerConfig,
) -> Result<(), String> {
    {
        let mut lock = engine.custom_servers.write().map_err(|e| e.to_string())?;
        lock.update(&id, config).map_err(|e| e.to_string())?;
    }
    engine.save_custom_servers()?;
    let _ = app.emit(
        events::CUSTOM_SERVER_UPDATED,
        events::CustomServerChangedPayload { server_id: id },
    );
    Ok(())
}

#[tauri::command]
pub async fn remove_custom_server(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    {
        let mut lock = engine.custom_servers.write().map_err(|e| e.to_string())?;
        if !lock.remove(&id) {
            return Err(format!("Custom server not found: {id}"));
        }
    }
    engine.save_custom_servers()?;
    let _ = app.emit(
        events::CUSTOM_SERVER_REMOVED,
        events::CustomServerChangedPayload { server_id: id },
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Catalog (read-only, bundled)
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn list_catalog_servers(
    engine: State<'_, Arc<Engine>>,
) -> Result<Vec<CatalogEntry>, String> {
    Ok(engine.catalog.list_servers().to_vec())
}

#[tauri::command]
pub async fn search_catalog(
    engine: State<'_, Arc<Engine>>,
    query: String,
    category: Option<String>,
) -> Result<Vec<CatalogEntry>, String> {
    let results = engine.catalog.search(&query, category.as_deref());
    Ok(results.into_iter().cloned().collect())
}

#[tauri::command]
pub async fn get_catalog_entry(
    engine: State<'_, Arc<Engine>>,
    id: String,
) -> Result<CatalogEntry, String> {
    engine
        .catalog
        .get_server(&id)
        .cloned()
        .ok_or_else(|| format!("Catalog entry not found: {id}"))
}

// ---------------------------------------------------------------------------
// Presets (read-only, bundled)
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn list_presets(engine: State<'_, Arc<Engine>>) -> Result<Vec<Preset>, String> {
    Ok(engine.catalog.list_presets().to_vec())
}

#[tauri::command]
pub async fn create_env_from_preset(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    preset_id: String,
    name: String,
) -> Result<Environment, String> {
    let preset = engine
        .catalog
        .get_preset(&preset_id)
        .ok_or_else(|| format!("Preset not found: {preset_id}"))?
        .clone();

    let id = plugmux_core::slug::slugify(&name);
    db_envs::add_environment(&engine.db, &id, &name)?;
    for server_id in &preset.servers {
        db_envs::add_server(&engine.db, &id, server_id)?;
    }

    let _ = app.emit(
        events::ENVIRONMENT_CREATED,
        events::EnvironmentChangedPayload { env_id: id.clone() },
    );
    Ok(Environment {
        id,
        name,
        servers: preset.servers,
    })
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_server_health(
    engine: State<'_, Arc<Engine>>,
    server_id: String,
) -> Result<HealthStatus, String> {
    Ok(engine
        .manager
        .get_health(&server_id)
        .await
        .unwrap_or(HealthStatus::Unavailable {
            reason: "Server not running".to_string(),
        }))
}

// ---------------------------------------------------------------------------
// Agent commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_agent_registry() -> Result<Vec<AgentEntry>, String> {
    let registry = AgentRegistry::load_bundled();
    Ok(registry.list_agents().into_iter().cloned().collect())
}

#[tauri::command]
pub async fn detect_agents(engine: State<'_, Arc<Engine>>) -> Result<Vec<DetectedAgent>, String> {
    let registry = AgentRegistry::load_bundled();
    let active = engine
        .active_agents
        .read()
        .map_err(|e| e.to_string())?
        .clone();
    Ok(plugmux_core::agents::detect_all(
        &registry, &engine.db, &active,
    ))
}

#[tauri::command]
pub async fn connect_agent_cmd(
    engine: State<'_, Arc<Engine>>,
    agent_id: String,
) -> Result<Option<String>, String> {
    let registry = AgentRegistry::load_bundled();
    let port = *engine.port.read().await;

    let (config_path, config_format, mcp_key) =
        resolve_agent_config(&registry, &engine.db, &agent_id)?;

    let result = plugmux_core::agents::connect_agent(&config_path, &config_format, &mcp_key, port)?;

    Ok(result.map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn disconnect_agent_cmd(
    engine: State<'_, Arc<Engine>>,
    agent_id: String,
    restore: bool,
) -> Result<(), String> {
    let registry = AgentRegistry::load_bundled();

    let (config_path, config_format, mcp_key) =
        resolve_agent_config(&registry, &engine.db, &agent_id)?;

    if restore {
        plugmux_core::agents::disconnect_and_restore(&config_path, &config_format, &mcp_key)
    } else {
        plugmux_core::agents::disconnect_agent(&config_path, &config_format, &mcp_key)
    }
}

#[tauri::command]
pub async fn has_agent_backup(
    engine: State<'_, Arc<Engine>>,
    agent_id: String,
) -> Result<bool, String> {
    let registry = AgentRegistry::load_bundled();
    let (config_path, _, _) = resolve_agent_config(&registry, &engine.db, &agent_id)?;
    Ok(plugmux_core::agents::get_backup_path(&config_path).is_some())
}

#[tauri::command]
pub async fn add_agent_from_registry(
    engine: State<'_, Arc<Engine>>,
    agent_id: String,
    config_path: String,
) -> Result<(), String> {
    db::agents::add_agent(
        &engine.db,
        &AgentStateEntry {
            id: agent_id,
            source: AgentSource::Registry,
            name: None,
            icon: None,
            config_path: Some(config_path),
            config_format: None,
            mcp_key: None,
        },
    )
}

#[tauri::command]
pub async fn add_custom_agent(
    engine: State<'_, Arc<Engine>>,
    name: String,
    config_path: String,
    config_format: String,
    mcp_key: String,
) -> Result<(), String> {
    let id = plugmux_core::slug::slugify(&name);
    // Normalize format string
    let fmt = match config_format.as_str() {
        "toml" => "toml",
        _ => "json",
    };
    db::agents::add_agent(
        &engine.db,
        &AgentStateEntry {
            id,
            source: AgentSource::Custom,
            name: Some(name),
            icon: None,
            config_path: Some(config_path),
            config_format: Some(fmt.to_string()),
            mcp_key: Some(mcp_key),
        },
    )
}

#[tauri::command]
pub async fn dismiss_agent(engine: State<'_, Arc<Engine>>, agent_id: String) -> Result<(), String> {
    db::agents::dismiss_agent(&engine.db, &agent_id)
}

/// Resolves agent config details from registry or db.
fn resolve_agent_config(
    registry: &AgentRegistry,
    db: &Arc<db::Db>,
    agent_id: &str,
) -> Result<(std::path::PathBuf, ConfigFormat, String), String> {
    // Try registry first
    if let Some(entry) = registry.get_agent(agent_id) {
        let path = registry
            .resolve_config_path(entry)
            .ok_or_else(|| format!("No config path for agent on this OS: {agent_id}"))?;
        return Ok((path, entry.config_format.clone(), entry.mcp_key.clone()));
    }

    // Try db (custom/registry agents)
    if let Some(state_entry) = db::agents::get_agent(db, agent_id) {
        let path_str = state_entry
            .config_path
            .as_ref()
            .ok_or_else(|| format!("No config path for agent: {agent_id}"))?;
        let path = std::path::PathBuf::from(
            path_str.replace(
                "~",
                dirs::home_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .as_ref(),
            ),
        );
        let format = match state_entry.config_format.as_deref() {
            Some("toml") => ConfigFormat::Toml,
            _ => ConfigFormat::Json,
        };
        let mcp_key = state_entry
            .mcp_key
            .clone()
            .unwrap_or_else(|| "mcpServers".to_string());
        return Ok((path, format, mcp_key));
    }

    Err(format!("Agent not found: {agent_id}"))
}

// ---------------------------------------------------------------------------
// Migration
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn migrate_config(engine: State<'_, Arc<Engine>>) -> Result<(), String> {
    if !migration::needs_migration() {
        return Err("No migration needed".to_string());
    }
    migration::migrate(&engine.catalog, &engine.db).map_err(|e| e.to_string())?;
    engine.reload_config().await?;
    engine.reload_custom_servers()
}

// ---------------------------------------------------------------------------
// Logs
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_recent_logs(
    engine: State<'_, Arc<Engine>>,
    limit: Option<usize>,
) -> Result<Vec<LogEntry>, String> {
    logs::read_recent_logs(&engine.db, limit.unwrap_or(100))
        .map_err(|e| format!("failed to read logs: {e}"))
}

// ─── Cloud API commands ───

#[tauri::command]
pub async fn api_health(engine: State<'_, Arc<Engine>>) -> Result<HealthResponse, String> {
    let client = engine.api_client.read().await;
    client.health().await
}

#[tauri::command]
pub async fn api_list_servers(
    engine: State<'_, Arc<Engine>>,
    limit: Option<u32>,
    cursor: Option<String>,
    search: Option<String>,
    category: Option<String>,
) -> Result<CatalogResponse, String> {
    let client = engine.api_client.read().await;
    client
        .list_servers(
            limit,
            cursor.as_deref(),
            search.as_deref(),
            category.as_deref(),
        )
        .await
}

#[tauri::command]
pub async fn api_get_server(
    engine: State<'_, Arc<Engine>>,
    id: String,
) -> Result<RemoteCatalogServer, String> {
    let client = engine.api_client.read().await;
    client.get_server(&id).await
}

#[tauri::command]
pub async fn api_list_collections(
    engine: State<'_, Arc<Engine>>,
) -> Result<CollectionsResponse, String> {
    let client = engine.api_client.read().await;
    client.list_collections().await
}

#[tauri::command]
pub async fn api_get_collection(
    engine: State<'_, Arc<Engine>>,
    id: String,
) -> Result<RemoteCollection, String> {
    let client = engine.api_client.read().await;
    client.get_collection(&id).await
}

#[tauri::command]
pub async fn api_get_auth_url(engine: State<'_, Arc<Engine>>) -> Result<String, String> {
    let client = engine.api_client.read().await;
    Ok(client.github_auth_url())
}

#[tauri::command]
pub async fn api_set_token(engine: State<'_, Arc<Engine>>, token: String) -> Result<(), String> {
    let mut client = engine.api_client.write().await;
    client.set_token(token);
    Ok(())
}

#[tauri::command]
pub async fn api_get_profile(engine: State<'_, Arc<Engine>>) -> Result<AuthUser, String> {
    let client = engine.api_client.read().await;
    client.get_profile().await
}

#[tauri::command]
pub async fn api_get_base_url(engine: State<'_, Arc<Engine>>) -> Result<String, String> {
    let client = engine.api_client.read().await;
    Ok(client.base_url().to_string())
}
