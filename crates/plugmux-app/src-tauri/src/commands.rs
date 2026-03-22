use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use plugmux_core::catalog::{CatalogEntry, Preset};
use plugmux_core::config::{self, Config, Environment, Permissions, PermissionLevel};
use plugmux_core::environment;
use plugmux_core::migration;
use plugmux_core::server::{HealthStatus, ServerConfig};

use crate::engine::Engine;
use crate::events;

// ---------------------------------------------------------------------------
// Engine commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_engine_status(engine: State<'_, Arc<Engine>>) -> Result<String, String> {
    let status = engine.status.read().await;
    Ok(status.as_str().to_string())
}

#[tauri::command]
pub async fn start_engine(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
) -> Result<(), String> {
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
pub async fn stop_engine(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
) -> Result<(), String> {
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
pub async fn list_environments(
    engine: State<'_, Arc<Engine>>,
) -> Result<Vec<Environment>, String> {
    let cfg = engine.config.read().await;
    Ok(cfg.environments.clone())
}

#[tauri::command]
pub async fn create_environment(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    name: String,
) -> Result<Environment, String> {
    let env;
    {
        let mut cfg = engine.config.write().await;
        let created = config::add_environment(&mut cfg, &name);
        env = created.clone();
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::ENVIRONMENT_CREATED,
        events::EnvironmentChangedPayload {
            env_id: env.id.clone(),
        },
    );
    Ok(env)
}

#[tauri::command]
pub async fn delete_environment(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    {
        let mut cfg = engine.config.write().await;
        config::remove_environment(&mut cfg, &id).map_err(|e| e.to_string())?;
    }
    engine.save_config().await?;
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
    {
        let mut cfg = engine.config.write().await;
        let env = config::find_environment_mut(&mut cfg, &id)
            .ok_or_else(|| format!("Environment not found: {id}"))?;
        env.name = name;
    }
    engine.save_config().await
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
    {
        let mut cfg = engine.config.write().await;
        environment::add_server(&mut cfg, &env_id, &server_id)
            .map_err(|e| e.to_string())?;
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::SERVER_ADDED,
        events::ServerChangedPayload {
            server_id,
            env_id,
        },
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
    {
        let mut cfg = engine.config.write().await;
        environment::remove_server(&mut cfg, &env_id, &server_id)
            .map_err(|e| e.to_string())?;
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::SERVER_REMOVED,
        events::ServerChangedPayload {
            server_id,
            env_id,
        },
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Custom servers
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn list_custom_servers(engine: State<'_, Arc<Engine>>) -> Result<Vec<ServerConfig>, String> {
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
        lock.add(config, &engine.catalog).map_err(|e| e.to_string())?;
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
pub async fn list_catalog_servers(engine: State<'_, Arc<Engine>>) -> Result<Vec<CatalogEntry>, String> {
    Ok(engine.catalog.list_servers().to_vec())
}

#[tauri::command]
pub async fn search_catalog(
    engine: State<'_, Arc<Engine>>,
    query: String,
    category: Option<String>,
) -> Result<Vec<CatalogEntry>, String> {
    let results = engine
        .catalog
        .search(&query, category.as_deref());
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

    let env;
    {
        let mut cfg = engine.config.write().await;
        let created = config::add_environment(&mut cfg, &name);
        created.servers = preset.servers;
        env = created.clone();
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::ENVIRONMENT_CREATED,
        events::EnvironmentChangedPayload {
            env_id: env.id.clone(),
        },
    );
    Ok(env)
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
// Migration
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn migrate_config(engine: State<'_, Arc<Engine>>) -> Result<(), String> {
    if !migration::needs_migration() {
        return Err("No migration needed".to_string());
    }
    migration::migrate(&engine.catalog).map_err(|e| e.to_string())?;
    engine.reload_config().await?;
    engine.reload_custom_servers()
}
