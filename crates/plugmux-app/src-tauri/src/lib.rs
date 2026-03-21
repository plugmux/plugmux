mod engine;
mod events;

use engine::Engine;
use std::sync::Arc;
use tauri::Emitter;

pub fn run() {
    tracing_subscriber::fmt::init();

    let engine = Arc::new(Engine::new());

    tauri::Builder::default()
        .manage(engine.clone())
        .setup(move |app| {
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
