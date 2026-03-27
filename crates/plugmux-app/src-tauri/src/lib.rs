mod commands;
mod engine;
mod events;
mod tray;
mod watcher;

use engine::Engine;
use std::sync::Arc;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn run() {
    tracing_subscriber::fmt::init();

    let engine = Arc::new(Engine::new());

    tauri::Builder::default()
        .manage(engine.clone())
        .invoke_handler(tauri::generate_handler![
            // Engine
            commands::get_engine_status,
            commands::start_engine,
            commands::stop_engine,
            // Config
            commands::get_config,
            commands::get_port,
            commands::set_port,
            // Permissions (global)
            commands::get_permissions,
            commands::set_permission,
            // Environments
            commands::list_environments,
            commands::create_environment,
            commands::delete_environment,
            commands::rename_environment,
            // Servers in environments
            commands::add_server_to_env,
            commands::remove_server_from_env,
            // Custom servers
            commands::list_custom_servers,
            commands::add_custom_server,
            commands::update_custom_server,
            commands::remove_custom_server,
            // Catalog (read-only, bundled)
            commands::list_catalog_servers,
            commands::search_catalog,
            commands::get_catalog_entry,
            // Presets (read-only, bundled)
            commands::list_presets,
            commands::create_env_from_preset,
            // Health
            commands::get_server_health,
            // Migration
            commands::migrate_config,
            // Agents
            commands::get_agent_registry,
            commands::detect_agents,
            commands::connect_agent_cmd,
            commands::disconnect_agent_cmd,
            commands::has_agent_backup,
            commands::add_agent_from_registry,
            commands::add_custom_agent,
            commands::dismiss_agent,
            // Logs
            commands::get_recent_logs,
        ])
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(move |app| {
            // Create main window programmatically so we can set traffic light position
            #[allow(unused_mut)]
            let mut builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("plugmux")
                .inner_size(900.0, 600.0)
                .min_inner_size(700.0, 400.0)
                .resizable(true);

            #[cfg(target_os = "macos")]
            {
                builder = builder
                    .hidden_title(true)
                    .title_bar_style(tauri::TitleBarStyle::Overlay)
                    .traffic_light_position(tauri::LogicalPosition::new(12.0, 34.5));
            }

            let window = builder.build()?;

            tray::setup_tray(app.handle())?;

            let engine = engine.clone();
            let engine_for_watcher = engine.clone();
            let handle = app.handle().clone();

            // Wire gateway callback: track active agents + emit UI events
            {
                let active_agents = engine.active_agents.clone();
                let db_ref = engine.db.clone();
                let handle_cb = handle.clone();
                let cb: plugmux_core::gateway::OnRequest = Arc::new(move |event| {
                    // Emit log event to frontend
                    let _ = handle_cb.emit(
                        events::LOG_ADDED,
                        events::LogAddedPayload {
                            agent_id: event.agent_id.clone(),
                            method: event.method,
                            env_id: event.env_id,
                            duration_ms: event.duration_ms,
                            error: event.error,
                        },
                    );

                    // Track active agents
                    if let Some(ref id) = event.agent_id {
                        let is_new = {
                            let mut set = active_agents.write().unwrap();
                            set.insert(id.clone())
                        };
                        // Persist to DB
                        if is_new {
                            if let Err(e) =
                                plugmux_core::db::active_agents::mark_active(&db_ref, id)
                            {
                                tracing::warn!(error = %e, "failed to persist active agent");
                            }
                        }
                        let _ = handle_cb.emit(
                            events::AGENT_ACTIVITY,
                            events::AgentActivityPayload {
                                agent_id: id.clone(),
                                is_new,
                            },
                        );
                    }
                });

                let engine_for_cb = engine.clone();
                tauri::async_runtime::spawn(async move {
                    engine_for_cb.set_on_request(cb).await;
                });
            }

            // Auto-start engine (slight delay to ensure callback is wired first)
            tauri::async_runtime::spawn(async move {
                tokio::task::yield_now().await;
                match engine.start().await {
                    Ok(()) => {
                        let _ = handle.emit(
                            events::ENGINE_STATUS_CHANGED,
                            events::EngineStatusPayload {
                                status: "running".to_string(),
                            },
                        );
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "failed to start engine");
                        let _ = handle.emit(
                            events::ENGINE_STATUS_CHANGED,
                            events::EngineStatusPayload {
                                status: "conflict".to_string(),
                            },
                        );
                    }
                }
            });

            // Hide window on close instead of quitting (tray keeps running)
            {
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w.hide();
                    }
                });
            }

            // Start config file watcher (keep watcher alive for app lifetime)
            match watcher::start_config_watcher(app.handle().clone(), engine_for_watcher) {
                Ok(w) => {
                    app.manage(w);
                }
                Err(e) => {
                    tracing::warn!(error = %e, "config watcher not available");
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running plugmux");
}
