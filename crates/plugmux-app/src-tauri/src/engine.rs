use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tracing::{error, info};

use plugmux_core::config::{self, PlugmuxConfig};
use plugmux_core::environment::resolve_named;
use plugmux_core::gateway::router;
use plugmux_core::health::start_health_checker;
use plugmux_core::manager::ServerManager;

/// Represents the current state of the engine.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineStatus {
    Stopped,
    Running,
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
    pub config: Arc<RwLock<PlugmuxConfig>>,
    pub manager: Arc<ServerManager>,
    pub status: Arc<RwLock<EngineStatus>>,
    pub port: Arc<RwLock<u16>>,
    shutdown_tx: Arc<RwLock<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl Engine {
    pub fn new() -> Self {
        let cfg = config::load_or_default(&config::config_path()).unwrap_or_default();
        Self {
            config: Arc::new(RwLock::new(cfg)),
            manager: Arc::new(ServerManager::new()),
            status: Arc::new(RwLock::new(EngineStatus::Stopped)),
            port: Arc::new(RwLock::new(4242)),
            shutdown_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the gateway: launch MCP servers and bind the HTTP port.
    pub async fn start(&self) -> Result<(), String> {
        let current = self.status.read().await.clone();
        if current == EngineStatus::Running {
            return Err("Engine is already running".to_string());
        }

        let port = *self.port.read().await;
        let cfg = self.config.read().await.clone();

        // Start all enabled Main servers
        for server in &cfg.main.servers {
            if server.enabled {
                if let Err(e) = self.manager.start_server(server.clone()).await {
                    error!(server_id = %server.id, error = %e, "failed to start main server");
                }
            }
        }

        // Start environment-specific servers
        for env in &cfg.environments {
            let resolved = resolve_named(&cfg, &env.id).unwrap_or_default();
            for rs in &resolved {
                if rs.source == plugmux_core::environment::ServerSource::Environment {
                    if let Err(e) = self.manager.start_server(rs.config.clone()).await {
                        error!(server_id = %rs.config.id, env = %env.id, error = %e, "failed to start env server");
                    }
                }
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

        let router = router::build_router(config, manager);
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
        let new_cfg = config::load_or_default(&path).map_err(|e| e.to_string())?;
        *self.config.write().await = new_cfg;
        info!("config reloaded from disk");
        Ok(())
    }

    /// Save current in-memory config to disk.
    pub async fn save_config(&self) -> Result<(), String> {
        let path = config::config_path();
        let cfg = self.config.read().await;
        config::save(&path, &cfg).map_err(|e| e.to_string())
    }
}
