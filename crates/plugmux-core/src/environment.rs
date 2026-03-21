use crate::config::{EnvironmentConfig, PlugmuxConfig};
use crate::server::ServerConfig;

/// Where a resolved server originated.
#[derive(Debug, Clone, PartialEq)]
pub enum ServerSource {
    Main,
    Environment,
}

/// A server that has been fully resolved for an environment.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedServer {
    pub config: ServerConfig,
    pub source: ServerSource,
}

/// Resolves the merged server list for `env_option` against the top-level `main` config.
///
/// Algorithm:
/// 1. Start with all enabled servers from `main`, tagged as `Main`.
/// 2. If an environment is provided:
///    a. Append environment-specific servers tagged as `Environment`.
///    b. Apply per-server overrides (currently: `enabled` field).
/// 3. Filter out any server where `enabled == false`.
pub fn resolve_environment(
    main: &PlugmuxConfig,
    env_option: Option<&EnvironmentConfig>,
) -> Vec<ResolvedServer> {
    // Build a working list starting from main servers.
    let mut servers: Vec<ResolvedServer> = main
        .main
        .servers
        .iter()
        .map(|s| ResolvedServer {
            config: s.clone(),
            source: ServerSource::Main,
        })
        .collect();

    if let Some(env) = env_option {
        // Append environment-specific servers.
        for s in &env.servers {
            servers.push(ResolvedServer {
                config: s.clone(),
                source: ServerSource::Environment,
            });
        }

        // Apply overrides by server id.
        for ov in &env.overrides {
            if let Some(rs) = servers.iter_mut().find(|rs| rs.config.id == ov.server_id) {
                if let Some(enabled) = ov.enabled {
                    rs.config.enabled = enabled;
                }
                if let Some(ref url) = ov.url {
                    rs.config.url = Some(url.clone());
                }
            }
        }
    }

    // Filter disabled servers.
    servers.retain(|rs| rs.config.enabled);
    servers
}

/// Resolves servers for the main (no-environment) context.
pub fn resolve_main(config: &PlugmuxConfig) -> Vec<ResolvedServer> {
    resolve_environment(config, None)
}

/// Resolves servers for a named environment. Returns `None` if the environment is not found.
pub fn resolve_named(config: &PlugmuxConfig, env_id: &str) -> Option<Vec<ResolvedServer>> {
    let env = config.environments.iter().find(|e| e.id == env_id)?;
    Some(resolve_environment(config, Some(env)))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{EnvironmentConfig, MainConfig, PlugmuxConfig, ServerOverride};
    use crate::server::{Connectivity, ServerConfig, Transport};

    fn make_server(id: &str, enabled: bool) -> ServerConfig {
        ServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            transport: Transport::Stdio,
            command: Some("npx".to_string()),
            args: None,
            url: None,
            connectivity: Connectivity::Local,
            enabled,
            description: None,
        }
    }

    fn make_config(servers: Vec<ServerConfig>) -> PlugmuxConfig {
        PlugmuxConfig {
            main: MainConfig { servers },
            environments: vec![],
        }
    }

    #[test]
    fn test_resolve_main_only() {
        let cfg = make_config(vec![make_server("a", true), make_server("b", true)]);
        let resolved = resolve_main(&cfg);
        assert_eq!(resolved.len(), 2);
        assert!(resolved.iter().all(|r| r.source == ServerSource::Main));
    }

    #[test]
    fn test_resolve_excludes_disabled_main() {
        let cfg = make_config(vec![
            make_server("a", true),
            make_server("b", false), // disabled
        ]);
        let resolved = resolve_main(&cfg);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].config.id, "a");
    }

    #[test]
    fn test_resolve_env_inherits_main() {
        let mut cfg = make_config(vec![make_server("main-s", true)]);
        cfg.environments.push(EnvironmentConfig {
            id: "dev".to_string(),
            name: "Dev".to_string(),
            endpoint: "http://localhost:3000/dev".to_string(),
            servers: vec![make_server("env-s", true)],
            overrides: vec![],
        });

        let resolved = resolve_named(&cfg, "dev").unwrap();
        assert_eq!(resolved.len(), 2);

        let sources: Vec<&ServerSource> = resolved.iter().map(|r| &r.source).collect();
        assert!(sources.contains(&&ServerSource::Main));
        assert!(sources.contains(&&ServerSource::Environment));
    }

    #[test]
    fn test_resolve_env_override_disables_main_server() {
        let mut cfg = make_config(vec![make_server("shared", true)]);
        cfg.environments.push(EnvironmentConfig {
            id: "staging".to_string(),
            name: "Staging".to_string(),
            endpoint: "http://localhost:3000/staging".to_string(),
            servers: vec![],
            overrides: vec![ServerOverride {
                server_id: "shared".to_string(),
                enabled: Some(false),
                url: None,
                permissions: None,
            }],
        });

        let resolved = resolve_named(&cfg, "staging").unwrap();
        // The "shared" server is disabled by the override, so nothing should remain.
        assert!(resolved.is_empty());
    }

    #[test]
    fn test_resolve_named_not_found() {
        let cfg = make_config(vec![]);
        assert!(resolve_named(&cfg, "nonexistent").is_none());
    }
}
