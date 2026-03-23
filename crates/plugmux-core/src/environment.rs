use crate::config::{self, Config, ConfigError};

/// Get the list of server IDs for an environment.
/// Returns `None` if the environment does not exist.
pub fn get_server_ids(config: &Config, env_id: &str) -> Option<Vec<String>> {
    config::find_environment(config, env_id)
        .map(|env| env.servers.clone())
}

/// Add a server ID to an environment (if not already present).
pub fn add_server(config: &mut Config, env_id: &str, server_id: &str) -> Result<(), ConfigError> {
    let env = config::find_environment_mut(config, env_id)
        .ok_or(ConfigError::EnvironmentNotFound(env_id.to_string()))?;
    if !env.servers.contains(&server_id.to_string()) {
        env.servers.push(server_id.to_string());
    }
    Ok(())
}

/// Remove a server ID from an environment.
/// Returns `Ok(true)` if the server was present and removed, `Ok(false)` if it was not found.
pub fn remove_server(
    config: &mut Config,
    env_id: &str,
    server_id: &str,
) -> Result<bool, ConfigError> {
    let env = config::find_environment_mut(config, env_id)
        .ok_or(ConfigError::EnvironmentNotFound(env_id.to_string()))?;
    let before = env.servers.len();
    env.servers.retain(|s| s != server_id);
    Ok(env.servers.len() < before)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{add_environment, load_or_default};
    use std::path::PathBuf;

    /// Build a Config with a "global" env (empty) and a "work" env with two servers.
    fn config_with_envs() -> Config {
        // load_or_default on a nonexistent path returns a fresh default config
        let mut cfg = load_or_default(&PathBuf::from("/nonexistent/plugmux_test_config.json"));
        let env = add_environment(&mut cfg, "Work");
        env.servers.push("filesystem".to_string());
        env.servers.push("github".to_string());
        cfg
    }

    // -----------------------------------------------------------------------
    // get_server_ids
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_server_ids_returns_ids_for_environment() {
        let cfg = config_with_envs();
        let ids = get_server_ids(&cfg, "work").expect("work environment should exist");
        assert_eq!(ids, vec!["filesystem", "github"]);
    }

    #[test]
    fn test_get_server_ids_nonexistent_environment_returns_none() {
        let cfg = config_with_envs();
        assert!(get_server_ids(&cfg, "does-not-exist").is_none());
    }

    #[test]
    fn test_get_server_ids_global_environment_works() {
        let cfg = config_with_envs();
        let ids = get_server_ids(&cfg, "global").expect("global environment should exist");
        assert!(ids.is_empty());
    }

    // -----------------------------------------------------------------------
    // add_server
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_server_adds_id_to_environment() {
        let mut cfg = config_with_envs();
        add_server(&mut cfg, "work", "postgres").unwrap();
        let ids = get_server_ids(&cfg, "work").unwrap();
        assert!(ids.contains(&"postgres".to_string()));
    }

    #[test]
    fn test_add_server_does_not_duplicate_existing_id() {
        let mut cfg = config_with_envs();
        add_server(&mut cfg, "work", "filesystem").unwrap();
        let ids = get_server_ids(&cfg, "work").unwrap();
        assert_eq!(ids.iter().filter(|s| *s == "filesystem").count(), 1);
    }

    #[test]
    fn test_add_server_to_nonexistent_environment_returns_error() {
        let mut cfg = config_with_envs();
        let result = add_server(&mut cfg, "nope", "postgres");
        assert!(matches!(result, Err(ConfigError::EnvironmentNotFound(_))));
    }

    // -----------------------------------------------------------------------
    // remove_server
    // -----------------------------------------------------------------------

    #[test]
    fn test_remove_server_removes_id_and_returns_true() {
        let mut cfg = config_with_envs();
        let removed = remove_server(&mut cfg, "work", "github").unwrap();
        assert!(removed);
        let ids = get_server_ids(&cfg, "work").unwrap();
        assert!(!ids.contains(&"github".to_string()));
    }

    #[test]
    fn test_remove_server_missing_id_returns_false() {
        let mut cfg = config_with_envs();
        let removed = remove_server(&mut cfg, "work", "nonexistent").unwrap();
        assert!(!removed);
    }

    #[test]
    fn test_remove_server_from_nonexistent_environment_returns_error() {
        let mut cfg = config_with_envs();
        let result = remove_server(&mut cfg, "nope", "filesystem");
        assert!(matches!(result, Err(ConfigError::EnvironmentNotFound(_))));
    }
}
