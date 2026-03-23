use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tracing::{error, info, warn};

use plugmux_core::catalog::CatalogRegistry;
use plugmux_core::config::{self, Config};
use plugmux_core::custom_servers::CustomServerStore;
use plugmux_core::gateway::router;
use plugmux_core::health::start_health_checker;
use plugmux_core::manager::ServerManager;
use plugmux_core::migration;
use plugmux_core::resolver::ServerResolver;

/// Represents the current state of the engine.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineStatus {
    Stopped,
    Running,
    #[allow(dead_code)] // Set when port bind fails
    Conflict,
}

impl EngineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Running => "running",
            Self::Conflict => "conflict",
        }
    }
}

/// Holds all engine runtime state.
pub struct Engine {
    pub config: Arc<RwLock<Config>>,
    pub catalog: Arc<CatalogRegistry>,
    pub custom_servers: Arc<std::sync::RwLock<CustomServerStore>>,
    pub resolver: Arc<ServerResolver>,
    pub manager: Arc<ServerManager>,
    pub status: Arc<RwLock<EngineStatus>>,
    pub port: Arc<RwLock<u16>>,
    pub shutdown_tx: Arc<RwLock<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl Engine {
    pub fn new() -> Self {
        // Load catalog (bundled)
        let catalog = Arc::new(CatalogRegistry::load_bundled());

        // Check migration
        if migration::needs_migration() {
            info!("Phase-2 config detected, running migration");
            if let Err(e) = migration::migrate(&catalog) {
                warn!(error = %e, "migration failed, starting with defaults");
            }
        }

        // Load config
        let cfg = config::load_or_default(&config::config_path());
        let port = cfg.port;

        // Load custom servers
        let custom_path = config::config_dir().join("custom_servers.json");
        let custom_store = CustomServerStore::load_or_default(custom_path);
        let custom_servers = Arc::new(std::sync::RwLock::new(custom_store));

        // Create resolver
        let resolver = Arc::new(ServerResolver::new(catalog.clone(), custom_servers.clone()));

        Self {
            config: Arc::new(RwLock::new(cfg)),
            catalog,
            custom_servers,
            resolver,
            manager: Arc::new(ServerManager::new()),
            status: Arc::new(RwLock::new(EngineStatus::Stopped)),
            port: Arc::new(RwLock::new(port)),
            shutdown_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the gateway: resolve server IDs from ALL environments, start unique
    /// servers, and bind the HTTP port.
    pub async fn start(&self) -> Result<(), String> {
        let current = self.status.read().await.clone();
        if current == EngineStatus::Running {
            return Err("Engine is already running".to_string());
        }

        let port = *self.port.read().await;
        let cfg = self.config.read().await.clone();

        // Collect all unique server IDs across all environments
        let mut seen_ids = HashSet::new();
        for env in &cfg.environments {
            for server_id in &env.servers {
                seen_ids.insert(server_id.clone());
            }
        }

        // Resolve and start each unique server
        for server_id in &seen_ids {
            let resolved = self.resolver.resolve(server_id);
            if let Some(server_config) = resolved.config {
                if let Err(e) = self.manager.start_server(server_config).await {
                    error!(server_id = %server_id, error = %e, "failed to start server");
                }
            } else {
                warn!(server_id = %server_id, "server not found in catalog or custom servers");
            }
        }

        // Start health checker
        let health_manager = self.manager.clone();
        tokio::spawn(start_health_checker(health_manager, Duration::from_secs(30)));

        // Start HTTP server
        let config = self.config.clone();
        let manager = self.manager.clone();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        let addr = format!("127.0.0.1:{port}");
        let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
            format!("Port {port} is already in use: {e}")
        })?;

        info!("plugmux gateway listening on http://{addr}");

        let router = router::build_router(config, manager, None);
        tokio::spawn(async move {
            let server = axum::serve(listener, router);
            tokio::select! {
                result = server => {
                    if let Err(e) = result {
                        error!(error = %e, "gateway server error");
                    }
                }
                _ = rx => {
                    info!("gateway server shutting down");
                }
            }
        });

        *self.shutdown_tx.write().await = Some(tx);
        *self.status.write().await = EngineStatus::Running;

        Ok(())
    }

    /// Stop the gateway: shut down all servers and release the port.
    pub async fn stop(&self) -> Result<(), String> {
        let current = self.status.read().await.clone();
        if current != EngineStatus::Running {
            return Err("Engine is not running".to_string());
        }

        // Signal HTTP server shutdown
        if let Some(tx) = self.shutdown_tx.write().await.take() {
            let _ = tx.send(());
        }

        // Shut down all MCP servers
        self.manager.shutdown_all().await;

        *self.status.write().await = EngineStatus::Stopped;
        info!("engine stopped");

        Ok(())
    }

    /// Reload config from disk.
    pub async fn reload_config(&self) -> Result<(), String> {
        let path = config::config_path();
        let new_cfg = config::load(&path).map_err(|e| e.to_string())?;
        *self.port.write().await = new_cfg.port;
        *self.config.write().await = new_cfg;
        info!("config reloaded from disk");
        Ok(())
    }

    /// Reload custom servers from disk.
    pub fn reload_custom_servers(&self) -> Result<(), String> {
        let custom_path = config::config_dir().join("custom_servers.json");
        let store = CustomServerStore::load_or_default(custom_path);
        let mut lock = self.custom_servers.write().map_err(|e| e.to_string())?;
        *lock = store;
        info!("custom servers reloaded from disk");
        Ok(())
    }

    /// Save current in-memory config to disk.
    pub async fn save_config(&self) -> Result<(), String> {
        let path = config::config_path();
        let cfg = self.config.read().await;
        config::save(&path, &cfg).map_err(|e| e.to_string())
    }

    /// Save custom servers to disk.
    pub fn save_custom_servers(&self) -> Result<(), String> {
        let lock = self.custom_servers.read().map_err(|e| e.to_string())?;
        lock.save().map_err(|e| e.to_string())
    }
}
