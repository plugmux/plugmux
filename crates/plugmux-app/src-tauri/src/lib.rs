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
        .setup(move |app| {
            // Create main window programmatically so we can set traffic light position
            let mut builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("plugmux")
                .inner_size(900.0, 600.0)
                .min_inner_size(700.0, 400.0)
                .resizable(true)
                .hidden_title(true)
                .title_bar_style(tauri::TitleBarStyle::Overlay);

            #[cfg(target_os = "macos")]
            {
                builder = builder.traffic_light_position(tauri::LogicalPosition::new(12.0, 27.0));
            }

            let window = builder.build()?;

            tray::setup_tray(app.handle())?;

            let engine = engine.clone();
            let engine_for_watcher = engine.clone();
            let handle = app.handle().clone();

            // Auto-start engine
            tauri::async_runtime::spawn(async move {
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
