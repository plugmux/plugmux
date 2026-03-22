use plugmux_core::config;

pub async fn run(port: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load_or_default(&config::config_path());
    let port = port.unwrap_or(cfg.port);
    let url = format!("http://127.0.0.1:{port}/health");

    match reqwest::get(&url).await {
        Ok(_) => {
            println!("plugmux gateway is running on port {port}.");
            println!("To stop it, terminate the process (Ctrl+C or kill the process).");
        }
        Err(_) => {
            println!("No plugmux gateway found on port {port}.");
        }
    }

    Ok(())
}
