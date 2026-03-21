use std::sync::Arc;

use tokio::sync::RwLock;

use plugmux_core::config;
use plugmux_core::environment::resolve_named;
use plugmux_core::gateway::router;
use plugmux_core::manager::ServerManager;

const BANNER: &str = r#"
           __
    ____  / /_  ______ _____ ___  __  ___  __
   / __ \/ / / / / __ `/ __ `__ \/ / / / |/_/
  / /_/ / / /_/ / /_/ / / / / / / /_/ />  <
 / .___/_/\__,_/\__, /_/ /_/ /_/\__,_/_/|_|
/_/            /____/
"#;

/// Default gateway port.
const DEFAULT_PORT: u16 = 4242;

pub async fn run(port: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    let port = port.unwrap_or(DEFAULT_PORT);

    // 1. Load config
    let cfg_path = config::config_path();
    let cfg = config::load_or_default(&cfg_path)?;

    let manager = Arc::new(ServerManager::new());

    // 2. Start all enabled Main servers
    for server in &cfg.main.servers {
        if server.enabled {
            if let Err(e) = manager.start_server(server.clone()).await {
                eprintln!("  [warn] failed to start main server '{}': {}", server.id, e);
            }
        }
    }

    // 3. Start all enabled environment-specific servers
    for env in &cfg.environments {
        let resolved = resolve_named(&cfg, &env.id).unwrap_or_default();
        for rs in &resolved {
            // Only start env-specific servers (main servers already started above)
            if rs.source == plugmux_core::environment::ServerSource::Environment {
                if let Err(e) = manager.start_server(rs.config.clone()).await {
                    eprintln!(
                        "  [warn] failed to start server '{}' for env '{}': {}",
                        rs.config.id, env.id, e
                    );
                }
            }
        }
    }

    // 4. Print banner and environment URLs
    println!("{BANNER}");
    println!("  plugmux v{}", env!("CARGO_PKG_VERSION"));
    println!("  gateway: http://127.0.0.1:{port}");
    println!("  health:  http://127.0.0.1:{port}/health");
    println!();

    if cfg.environments.is_empty() {
        println!("  No environments configured.");
        println!("  Run `plugmux env create <name>` to get started.");
    } else {
        println!("  Environments:");
        for env in &cfg.environments {
            println!("    {} -> http://127.0.0.1:{port}/env/{}", env.name, env.id);
        }
    }
    println!();

    // 5. Start axum server
    let config = Arc::new(RwLock::new(cfg));
    router::start_server(config, manager, port).await?;

    Ok(())
}
