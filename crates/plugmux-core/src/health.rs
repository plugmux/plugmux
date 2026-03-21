//! Background health checker — periodically probes each managed server
//! and updates its health status.

use std::sync::Arc;
use std::time::Duration;

use tracing::{debug, warn};

use crate::manager::ServerManager;

/// Starts a background loop that periodically checks the health of all managed servers.
///
/// This function never returns — it is intended to be spawned as a Tokio task:
///
/// ```ignore
/// tokio::spawn(start_health_checker(manager.clone(), Duration::from_secs(30)));
/// ```
///
/// On each tick the checker iterates over all servers, calls `client.health_check()`,
/// and updates the health flag via `manager.set_health()`. Servers that were previously
/// online but now fail the check are marked unhealthy.
pub async fn start_health_checker(manager: Arc<ServerManager>, interval: Duration) {
    loop {
        tokio::time::sleep(interval).await;

        let servers = manager.list_servers().await;
        for (id, _was_healthy) in &servers {
            let healthy = check_server_health(&manager, id).await;
            manager.set_health(id, healthy).await;

            if !healthy {
                warn!(server_id = %id, "server health check failed — marked unhealthy");
            } else {
                debug!(server_id = %id, "server health check passed");
            }
        }
    }
}

/// Perform the actual health check by acquiring a read lock on the manager
/// and delegating to the client's `health_check()` method.
///
/// Returns `false` if the server is not found or the check fails.
async fn check_server_health(manager: &ServerManager, server_id: &str) -> bool {
    // We use list_tools as a lightweight probe — but the real check is
    // delegated to the client's health_check() method which checks if
    // the underlying transport is still alive.
    //
    // We cannot call manager.servers directly (private), so we rely on
    // the public API. The health_check on the client is called via
    // a dedicated method we expose on ServerManager.
    manager.check_health(server_id).await
}
