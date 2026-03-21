pub async fn run(port: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    let port = port.unwrap_or(4242);
    let url = format!("http://127.0.0.1:{port}/health");

    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => {
            println!("plugmux gateway is running on port {port}.");
            let body = resp.text().await.unwrap_or_default();
            println!("  Health: {body}");
        }
        Ok(resp) => {
            println!("plugmux gateway responded with status {}.", resp.status());
        }
        Err(_) => {
            println!("plugmux gateway is not running (port {port}).");
        }
    }

    Ok(())
}
