use std::sync::Arc;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};
use tracing::{error, info};

use plugmux_core::config;

use crate::engine::Engine;
use crate::events;

/// Starts watching the config directory for changes to either `config.json`
/// or `custom_servers.json`. When files are modified, reloads the relevant
/// data and emits a config_reloaded event.
pub fn start_config_watcher(
    app: AppHandle,
    engine: Arc<Engine>,
) -> Result<RecommendedWatcher, String> {
    let config_path = config::config_path();
    let custom_path = config::config_dir().join("custom_servers.json");

    let app_handle = app.clone();
    let engine_clone = engine.clone();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) => {
                if !matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_)
                ) {
                    return;
                }

                let changed_config = event.paths.iter().any(|p| p == &config_path);
                let changed_custom = event.paths.iter().any(|p| p == &custom_path);

                if changed_config {
                    info!("config.json changed externally, reloading");
                    let engine = engine_clone.clone();
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = engine.reload_config().await {
                            error!(error = %e, "failed to reload config");
                            return;
                        }
                        let _ = handle.emit(events::CONFIG_RELOADED, ());
                    });
                }

                if changed_custom {
                    info!("custom_servers.json changed externally, reloading");
                    let engine = engine_clone.clone();
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = engine.reload_custom_servers() {
                            error!(error = %e, "failed to reload custom servers");
                            return;
                        }
                        let _ = handle.emit(events::CONFIG_RELOADED, ());
                    });
                }
            }
            Err(e) => {
                error!(error = %e, "config watcher error");
            }
        }
    })
    .map_err(|e| format!("failed to create file watcher: {e}"))?;

    // Watch the config directory (handles both config.json and custom_servers.json)
    let dir = config::config_dir();
    // Ensure the directory exists so the watcher can attach
    let _ = std::fs::create_dir_all(&dir);
    watcher
        .watch(&dir, RecursiveMode::NonRecursive)
        .map_err(|e| format!("failed to watch config directory: {e}"))?;
    info!("watching config directory: {}", dir.display());

    Ok(watcher)
}
