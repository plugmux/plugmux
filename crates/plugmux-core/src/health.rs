//! Background health checker — periodically probes each managed server
//! and updates its health status.

use std::sync::Arc;
use std::time::Duration;

use tracing::{debug, warn};

use crate::manager::ServerManager;
use crate::server::HealthStatus;

/// Starts a background loop that periodically checks the health of all managed servers.
///
/// This function never returns — it is intended to be spawned as a Tokio task:
///
/// ```ignore
/// tokio::spawn(start_health_checker(manager.clone(), Duration::from_secs(30)));
/// ```
///
/// On each tick the checker iterates over all servers, calls `client.health_check()`,
/// and updates the health status via `manager.set_health()`.
pub async fn start_health_checker(manager: Arc<ServerManager>, interval: Duration) {
    loop {
        tokio::time::sleep(interval).await;

        let servers = manager.list_servers().await;
        for (id, _prev_health) in &servers {
            let status = check_server_health(&manager, id).await;
            let is_healthy = matches!(status, HealthStatus::Healthy);
            manager.set_health(id, status).await;

            if !is_healthy {
                warn!(server_id = %id, "server health check failed — marked unavailable");
            } else {
                debug!(server_id = %id, "server health check passed");
            }
        }
    }
}

/// Perform the actual health check by delegating to the client's `health_check()` method.
///
/// Returns `HealthStatus::Healthy` if the check passes, otherwise
/// `HealthStatus::Unavailable` with a reason.
async fn check_server_health(manager: &ServerManager, server_id: &str) -> HealthStatus {
    if manager.check_health(server_id).await {
        HealthStatus::Healthy
    } else {
        HealthStatus::Unavailable {
            reason: "Health check failed".into(),
        }
    }
}
