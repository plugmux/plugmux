use std::sync::Arc;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

use crate::engine::{Engine, EngineStatus};
use crate::events;

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let toggle = MenuItem::with_id(app, "toggle", "Stop", true, None::<&str>)?;
    let open = MenuItem::with_id(app, "open", "Open plugmux", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(app, &[&toggle, &separator, &open, &quit])?;

    // Load tray-specific icon (22x22 template image for macOS menu bar)
    let icon = {
        let icon_bytes = include_bytes!("../icons/tray-icon.png");
        let img = tauri::image::Image::from_bytes(icon_bytes)
            .expect("failed to load tray icon");
        img
    };

    TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "toggle" => {
                    let engine = app.state::<Arc<Engine>>();
                    let engine = engine.inner().clone();
                    let handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let status = engine.status.read().await.clone();
                        let result = if status == EngineStatus::Running {
                            engine.stop().await
                        } else {
                            engine.start().await
                        };
                        if let Err(e) = result {
                            tracing::error!(error = %e, "engine toggle failed");
                        }
                        let new_status = engine.status.read().await.clone();
                        let _ = handle.emit(
                            events::ENGINE_STATUS_CHANGED,
                            events::EngineStatusPayload {
                                status: new_status.as_str().to_string(),
                            },
                        );
                    });
                }
                "open" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    let engine = app.state::<Arc<Engine>>();
                    let engine = engine.inner().clone();
                    let handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = engine.stop().await;
                        handle.exit(0);
                    });
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
