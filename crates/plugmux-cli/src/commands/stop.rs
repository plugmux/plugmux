pub async fn run(port: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    let port = port.unwrap_or(4242);
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
