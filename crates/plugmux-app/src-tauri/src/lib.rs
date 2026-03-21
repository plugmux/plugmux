mod commands;
mod engine;
mod events;
mod tray;

use engine::Engine;
use std::sync::Arc;
use tauri::Emitter;

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
        .setup(move |app| {
            tray::setup_tray(app.handle())?;

            let engine = engine.clone();
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

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running plugmux");
}
