use std::sync::Arc;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};
use tracing::{error, info};

use plugmux_core::config;

use crate::engine::Engine;
use crate::events;

/// Starts watching the config file for external changes.
/// When the file is modified, reloads the config and emits a config_reloaded event.
pub fn start_config_watcher(
    app: AppHandle,
    engine: Arc<Engine>,
) -> Result<RecommendedWatcher, String> {
    let config_path = config::config_path();

    let app_handle = app.clone();
    let engine_clone = engine.clone();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) => {
                if matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_)
                ) {
                    info!("config file changed externally, reloading");
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
            }
            Err(e) => {
                error!(error = %e, "config watcher error");
            }
        }
    })
    .map_err(|e| format!("failed to create file watcher: {e}"))?;

    // Watch the parent directory (the file might not exist yet)
    if let Some(parent) = config_path.parent() {
        watcher
            .watch(parent, RecursiveMode::NonRecursive)
            .map_err(|e| format!("failed to watch config directory: {e}"))?;
        info!("watching config directory: {}", parent.display());
    }

    Ok(watcher)
}
