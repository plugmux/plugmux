use std::sync::Arc;

use tokio::sync::RwLock;

use plugmux_core::catalog::CatalogRegistry;
use plugmux_core::config;
use plugmux_core::custom_servers::CustomServerStore;
use plugmux_core::db::Db;
use plugmux_core::db::environments as db_env;
use plugmux_core::gateway::router;
use plugmux_core::manager::ServerManager;
use plugmux_core::migration;
use plugmux_core::resolver::ServerResolver;

const BANNER: &str = r#"
           __
    ____  / /_  ______ _____ ___  __  ___  __
   / __ \/ / / / / __ `/ __ `__ \/ / / / |/_/
  / /_/ / / /_/ / /_/ / / / / / / /_/ />  <
 / .___/_/\__,_/\__, /_/ /_/ /_/\__,_/_/|_|
/_/            /____/
"#;

pub async fn run(port: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Open database and check for migration
    let catalog = CatalogRegistry::load_bundled();
    let db = Db::open(&Db::default_path())
        .map_err(|e| format!("failed to open database: {e}"))?;
    if migration::needs_migration() {
        println!("  Migrating config from Phase 2 to Phase 3...");
        migration::migrate(&catalog, &db)?;
        println!("  Migration complete.");
    }

    // 2. Load config (for port only)
    let cfg = config::load_or_default(&config::config_path());
    let port = port.unwrap_or(cfg.port);

    // 3. Load custom servers
    let custom_path = config::config_dir().join("custom_servers.json");
    let custom_store = CustomServerStore::load_or_default(custom_path);

    // 4. Create resolver
    let catalog = Arc::new(catalog);
    let custom = Arc::new(std::sync::RwLock::new(custom_store));
    let resolver = ServerResolver::new(Arc::clone(&catalog), Arc::clone(&custom));

    // 5. Start servers for each environment
    let manager = Arc::new(ServerManager::new());
    let envs = db_env::list_environments(&db);

    for env in &envs {
        let server_ids = db_env::get_server_ids(&db, &env.id).unwrap_or_default();
        let resolved = resolver.resolve_all(&server_ids);
        for rs in &resolved {
            if let Some(server_config) = &rs.config {
                // Only start if not already running (avoid duplicate starts across envs)
                if !manager.is_healthy(&rs.id).await
                    && let Err(e) = manager.start_server(server_config.clone()).await
                {
                    eprintln!(
                        "  [warn] failed to start server '{}' for env '{}': {}",
                        rs.id, env.id, e
                    );
                }
            } else {
                eprintln!(
                    "  [warn] server '{}' in env '{}' not found in catalog or custom servers",
                    rs.id, env.id
                );
            }
        }
    }

    // 6. Print banner and environment URLs
    println!("{BANNER}");
    println!("  plugmux v{}", env!("CARGO_PKG_VERSION"));
    println!("  gateway: http://127.0.0.1:{port}");
    println!("  health:  http://127.0.0.1:{port}/health");
    println!();

    if envs.is_empty() {
        println!("  No environments configured.");
        println!("  Run `plugmux env create <name>` to get started.");
    } else {
        println!("  Environments:");
        for env in &envs {
            println!("    {} -> http://127.0.0.1:{port}/env/{}", env.name, env.id);
        }
    }
    println!();

    // 7. Start axum server
    let config = Arc::new(RwLock::new(cfg));
    router::start_server(config, manager, port, Some(db)).await?;

    Ok(())
}
