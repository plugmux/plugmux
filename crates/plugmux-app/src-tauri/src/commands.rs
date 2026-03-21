use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use plugmux_core::config::{self, EnvironmentConfig, PlugmuxConfig, ServerOverride};
use plugmux_core::server::ServerConfig;

use crate::engine::Engine;
use crate::events;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentPermissions {
    pub enable_server: String,
    pub disable_server: String,
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
pub async fn get_config(engine: State<'_, Arc<Engine>>) -> Result<PlugmuxConfig, String> {
    let cfg = engine.config.read().await;
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn get_main_servers(engine: State<'_, Arc<Engine>>) -> Result<Vec<ServerConfig>, String> {
    let cfg = engine.config.read().await;
    Ok(cfg.main.servers.clone())
}

#[tauri::command]
pub async fn add_main_server(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    config: ServerConfig,
) -> Result<(), String> {
    {
        let mut cfg = engine.config.write().await;
        cfg.main.servers.push(config.clone());
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::SERVER_ADDED,
        events::ServerChangedPayload {
            server_id: config.id,
            env_id: None,
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn remove_main_server(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    {
        let mut cfg = engine.config.write().await;
        cfg.main.servers.retain(|s| s.id != id);
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::SERVER_REMOVED,
        events::ServerChangedPayload {
            server_id: id,
            env_id: None,
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn toggle_main_server(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    let enabled;
    {
        let mut cfg = engine.config.write().await;
        let server = cfg
            .main
            .servers
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| format!("Server not found: {id}"))?;
        server.enabled = !server.enabled;
        enabled = server.enabled;
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::SERVER_TOGGLED,
        events::ServerToggledPayload {
            server_id: id,
            env_id: None,
            enabled,
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn rename_server(
    engine: State<'_, Arc<Engine>>,
    id: String,
    name: String,
) -> Result<(), String> {
    {
        let mut cfg = engine.config.write().await;
        if let Some(server) = cfg.main.servers.iter_mut().find(|s| s.id == id) {
            server.name = name;
        } else {
            let mut found = false;
            for env in &mut cfg.environments {
                if let Some(server) = env.servers.iter_mut().find(|s| s.id == id) {
                    server.name = name.clone();
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(format!("Server not found: {id}"));
            }
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
) -> Result<Vec<EnvironmentConfig>, String> {
    let cfg = engine.config.read().await;
    Ok(cfg.environments.clone())
}

#[tauri::command]
pub async fn create_environment(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    name: String,
) -> Result<EnvironmentConfig, String> {
    let env;
    {
        let mut cfg = engine.config.write().await;
        let port = *engine.port.read().await;
        let created = config::add_environment(&mut cfg, &name);
        created.endpoint = format!("http://localhost:{port}/env/{}", created.id);
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
        if !config::remove_environment(&mut cfg, &id) {
            return Err(format!("Environment not found: {id}"));
        }
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
        let env = cfg
            .environments
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or_else(|| format!("Environment not found: {id}"))?;
        env.name = name;
    }
    engine.save_config().await
}

#[tauri::command]
pub async fn add_env_server(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    env_id: String,
    config: ServerConfig,
) -> Result<(), String> {
    let server_id = config.id.clone();
    {
        let mut cfg = engine.config.write().await;
        let env = cfg
            .environments
            .iter_mut()
            .find(|e| e.id == env_id)
            .ok_or_else(|| format!("Environment not found: {env_id}"))?;
        env.servers.push(config);
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::SERVER_ADDED,
        events::ServerChangedPayload {
            server_id,
            env_id: Some(env_id),
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn remove_env_server(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    env_id: String,
    server_id: String,
) -> Result<(), String> {
    {
        let mut cfg = engine.config.write().await;
        let env = cfg
            .environments
            .iter_mut()
            .find(|e| e.id == env_id)
            .ok_or_else(|| format!("Environment not found: {env_id}"))?;
        env.servers.retain(|s| s.id != server_id);
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::SERVER_REMOVED,
        events::ServerChangedPayload {
            server_id,
            env_id: Some(env_id),
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn toggle_env_override(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    env_id: String,
    server_id: String,
) -> Result<(), String> {
    let enabled;
    {
        let mut cfg = engine.config.write().await;
        let env = cfg
            .environments
            .iter_mut()
            .find(|e| e.id == env_id)
            .ok_or_else(|| format!("Environment not found: {env_id}"))?;

        if let Some(ov) = env.overrides.iter_mut().find(|o| o.server_id == server_id) {
            let current = ov.enabled.unwrap_or(true);
            ov.enabled = Some(!current);
            enabled = !current;
        } else {
            env.overrides.push(ServerOverride {
                server_id: server_id.clone(),
                enabled: Some(false),
                url: None,
                permissions: None,
            });
            enabled = false;
        }
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::SERVER_TOGGLED,
        events::ServerToggledPayload {
            server_id,
            env_id: Some(env_id),
            enabled,
        },
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Permission commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_permissions(
    engine: State<'_, Arc<Engine>>,
    env_id: String,
) -> Result<EnvironmentPermissions, String> {
    let cfg = engine.config.read().await;
    let env = cfg
        .environments
        .iter()
        .find(|e| e.id == env_id)
        .ok_or_else(|| format!("Environment not found: {env_id}"))?;

    let mut enable_level = "approve".to_string();
    let mut disable_level = "approve".to_string();

    for ov in &env.overrides {
        if let Some(perm) = &ov.permissions {
            if perm
                .deny
                .as_ref()
                .is_some_and(|d| d.iter().any(|a| a == "enable_server"))
            {
                enable_level = "disable".to_string();
            } else if perm
                .allow
                .as_ref()
                .is_some_and(|a| a.iter().any(|x| x == "enable_server"))
            {
                enable_level = "allow".to_string();
            }

            if perm
                .deny
                .as_ref()
                .is_some_and(|d| d.iter().any(|a| a == "disable_server"))
            {
                disable_level = "disable".to_string();
            } else if perm
                .allow
                .as_ref()
                .is_some_and(|a| a.iter().any(|x| x == "disable_server"))
            {
                disable_level = "allow".to_string();
            }
        }
    }

    Ok(EnvironmentPermissions {
        enable_server: enable_level,
        disable_server: disable_level,
    })
}

#[tauri::command]
pub async fn set_permission(
    engine: State<'_, Arc<Engine>>,
    env_id: String,
    action: String,
    level: String,
) -> Result<(), String> {
    {
        let mut cfg = engine.config.write().await;
        let env = cfg
            .environments
            .iter_mut()
            .find(|e| e.id == env_id)
            .ok_or_else(|| format!("Environment not found: {env_id}"))?;

        let ov = if let Some(ov) = env.overrides.iter_mut().find(|o| o.server_id == "*") {
            ov
        } else {
            env.overrides.push(ServerOverride {
                server_id: "*".to_string(),
                enabled: None,
                url: None,
                permissions: Some(plugmux_core::config::Permission {
                    allow: Some(vec![]),
                    deny: Some(vec![]),
                }),
            });
            env.overrides.last_mut().unwrap()
        };

        let perm = ov.permissions.get_or_insert(plugmux_core::config::Permission {
            allow: Some(vec![]),
            deny: Some(vec![]),
        });

        let allow = perm.allow.get_or_insert_with(Vec::new);
        let deny = perm.deny.get_or_insert_with(Vec::new);

        allow.retain(|a| a != &action);
        deny.retain(|a| a != &action);

        match level.as_str() {
            "allow" => allow.push(action),
            "disable" => deny.push(action),
            "approve" => {} // Neither allow nor deny — requires approval
            _ => return Err(format!("Invalid permission level: {level}")),
        }
    }
    engine.save_config().await
}

// ---------------------------------------------------------------------------
// Settings commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_port(engine: State<'_, Arc<Engine>>) -> Result<u16, String> {
    Ok(*engine.port.read().await)
}

#[tauri::command]
pub async fn set_port(engine: State<'_, Arc<Engine>>, port: u16) -> Result<(), String> {
    *engine.port.write().await = port;
    Ok(())
}
