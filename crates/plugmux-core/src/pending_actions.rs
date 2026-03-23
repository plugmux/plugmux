use std::collections::HashMap;
use std::time::{Duration, Instant};

const EXPIRY: Duration = Duration::from_secs(300); // 5 minutes

#[derive(Debug, Clone)]
pub struct PendingAction {
    pub env_id: String,
    pub server_id: String,
    pub action: String,
    pub created_at: Instant,
}

pub struct PendingActions {
    actions: HashMap<String, PendingAction>,
}

impl PendingActions {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    pub fn add(&mut self, env_id: &str, server_id: &str, action: &str) -> String {
        self.cleanup();
        let id = uuid::Uuid::new_v4().to_string();
        self.actions.insert(
            id.clone(),
            PendingAction {
                env_id: env_id.to_string(),
                server_id: server_id.to_string(),
                action: action.to_string(),
                created_at: Instant::now(),
            },
        );
        id
    }

    pub fn confirm(&mut self, action_id: &str) -> Option<PendingAction> {
        self.cleanup();
        self.actions.remove(action_id)
    }

    pub fn find_existing(&self, env_id: &str, server_id: &str, action: &str) -> Option<&str> {
        for (id, pa) in &self.actions {
            if pa.env_id == env_id
                && pa.server_id == server_id
                && pa.action == action
                && pa.created_at.elapsed() < EXPIRY
            {
                return Some(id.as_str());
            }
        }
        None
    }

    fn cleanup(&mut self) {
        self.actions
            .retain(|_, pa| pa.created_at.elapsed() < EXPIRY);
    }
}

impl Default for PendingActions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_confirm() {
        let mut pa = PendingActions::new();
        let id = pa.add("env1", "server1", "enable_server");
        assert!(pa.confirm(&id).is_some());
        assert!(pa.confirm(&id).is_none());
    }

    #[test]
    fn test_find_existing() {
        let mut pa = PendingActions::new();
        let id = pa.add("env1", "server1", "enable_server");
        let found = pa.find_existing("env1", "server1", "enable_server");
        assert_eq!(found, Some(id.as_str()));
        assert!(
            pa.find_existing("env1", "server1", "disable_server")
                .is_none()
        );
    }

    #[test]
    fn test_unknown_id_returns_none() {
        let mut pa = PendingActions::new();
        assert!(pa.confirm("nonexistent").is_none());
    }
}
