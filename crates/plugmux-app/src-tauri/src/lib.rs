mod commands;
mod engine;
mod events;
mod tray;
mod watcher;

use engine::Engine;
use std::sync::Arc;
use tauri::{Emitter, Manager};

pub fn run() {
    tracing_subscriber::fmt::init();

    let engine = Arc::new(Engine::new());

    tauri::Builder::default()
        .manage(engine.clone())
        .invoke_handler(tauri::generate_handler![
            commands::get_engine_status,
            commands::start_engine,
            commands::stop_engine,
            commands::get_config,
            commands::get_main_servers,
            commands::add_main_server,
            commands::remove_main_server,
            commands::toggle_main_server,
            commands::rename_server,
            commands::list_environments,
            commands::create_environment,
            commands::delete_environment,
            commands::rename_environment,
            commands::add_env_server,
            commands::remove_env_server,
            commands::toggle_env_override,
            commands::get_permissions,
            commands::set_permission,
            commands::get_port,
            commands::set_port,
        ])
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(move |app| {
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

            // Start config file watcher (keep watcher alive for app lifetime)
            match watcher::start_config_watcher(app.handle().clone(), engine_for_watcher) {
                Ok(w) => { app.manage(w); }
                Err(e) => { tracing::warn!(error = %e, "config watcher not available"); }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running plugmux");
}
