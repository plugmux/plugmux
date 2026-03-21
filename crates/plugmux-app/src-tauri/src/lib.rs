pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running plugmux");
}
