use clap::Subcommand;

use plugmux_core::config;
use plugmux_core::environment::resolve_named;
use plugmux_core::server::{Connectivity, ServerConfig, Transport};
use plugmux_core::slug::slugify;

#[derive(Subcommand)]
pub enum ServerCommands {
    /// Add a server
    Add {
        /// Server ID (will be slugified)
        id: String,
        /// Human-readable name
        #[arg(long)]
        name: String,
        /// Transport type: stdio or http
        #[arg(long)]
        transport: String,
        /// Command to run (for stdio transport)
        #[arg(long)]
        command: Option<String>,
        /// URL (for http transport)
        #[arg(long)]
        url: Option<String>,
        /// Add to a specific environment instead of main
        #[arg(long)]
        env: Option<String>,
        /// Connectivity: local or online
        #[arg(long, default_value = "local")]
        connectivity: String,
    },
    /// Remove a server
    Remove {
        /// Server ID
        id: String,
        /// Remove from a specific environment instead of main
        #[arg(long)]
        env: Option<String>,
    },
    /// List servers
    List {
        /// Show resolved servers for an environment (inherited + local)
        #[arg(long)]
        env: Option<String>,
    },
    /// Enable or disable a server
    Toggle {
        /// Server ID
        id: String,
        /// Toggle in a specific environment (uses overrides for Main servers)
        #[arg(long)]
        env: Option<String>,
    },
    /// Rename a server
    Rename {
        /// Server ID
        id: String,
        /// New human-readable name
        #[arg(long)]
        name: String,
    },
}

pub fn run(cmd: &ServerCommands) -> Result<(), Box<dyn std::error::Error>> {
    let cfg_path = config::config_path();
    let mut cfg = config::load_or_default(&cfg_path)?;

    match cmd {
        ServerCommands::Add {
            id,
            name,
            transport,
            command,
            url,
            env,
            connectivity,
        } => {
            let transport = match transport.as_str() {
                "stdio" => Transport::Stdio,
                "http" => Transport::Http,
                other => return Err(format!("unknown transport: {other} (use 'stdio' or 'http')").into()),
            };

            let connectivity = match connectivity.as_str() {
                "local" => Connectivity::Local,
                "online" => Connectivity::Online,
                other => return Err(format!("unknown connectivity: {other} (use 'local' or 'online')").into()),
            };

            let server = ServerConfig {
                id: slugify(id),
                name: name.clone(),
                transport,
                command: command.clone(),
                args: None,
                url: url.clone(),
                connectivity,
                enabled: true,
                description: None,
            };

            if let Some(env_id) = env {
                let env_cfg = cfg
                    .environments
                    .iter_mut()
                    .find(|e| e.id == *env_id)
                    .ok_or_else(|| format!("environment '{env_id}' not found"))?;
                env_cfg.servers.push(server);
            } else {
                cfg.main.servers.push(server);
            }

            config::save(&cfg_path, &cfg)?;
            println!("Added server: {} ({})", name, slugify(id));
        }

        ServerCommands::Remove { id, env } => {
            let removed = if let Some(env_id) = env {
                let env_cfg = cfg
                    .environments
                    .iter_mut()
                    .find(|e| e.id == *env_id)
                    .ok_or_else(|| format!("environment '{env_id}' not found"))?;
                let before = env_cfg.servers.len();
                env_cfg.servers.retain(|s| s.id != *id);
                env_cfg.servers.len() < before
            } else {
                let before = cfg.main.servers.len();
                cfg.main.servers.retain(|s| s.id != *id);
                cfg.main.servers.len() < before
            };

            if removed {
                config::save(&cfg_path, &cfg)?;
                println!("Removed server: {id}");
            } else {
                return Err(format!("server '{id}' not found").into());
            }
        }

        ServerCommands::List { env } => {
            if let Some(env_id) = env {
                // Show resolved servers for the environment
                let resolved = resolve_named(&cfg, env_id)
                    .ok_or_else(|| format!("environment '{env_id}' not found"))?;

                if resolved.is_empty() {
                    println!("No servers in environment '{env_id}'.");
                } else {
                    println!("Servers in environment '{env_id}':");
                    for rs in &resolved {
                        let source = match rs.source {
                            plugmux_core::environment::ServerSource::Main => "main",
                            plugmux_core::environment::ServerSource::Environment => "env",
                        };
                        let transport = match rs.config.transport {
                            Transport::Stdio => "stdio",
                            Transport::Http => "http",
                        };
                        println!(
                            "  {} ({}) [{}] [{}] {}",
                            rs.config.name,
                            rs.config.id,
                            source,
                            transport,
                            if rs.config.enabled { "enabled" } else { "disabled" },
                        );
                    }
                }
            } else {
                // Show main servers
                if cfg.main.servers.is_empty() {
                    println!("No servers in main config.");
                } else {
                    println!("Main servers:");
                    for s in &cfg.main.servers {
                        let transport = match s.transport {
                            Transport::Stdio => "stdio",
                            Transport::Http => "http",
                        };
                        println!(
                            "  {} ({}) [{}] {}",
                            s.name,
                            s.id,
                            transport,
                            if s.enabled { "enabled" } else { "disabled" },
                        );
                    }
                }
            }
        }

        ServerCommands::Toggle { id, env } => {
            if let Some(env_id) = env {
                // For env-scoped toggle: if the server is in main, use overrides.
                // If the server is env-local, toggle directly.
                let env_cfg = cfg
                    .environments
                    .iter_mut()
                    .find(|e| e.id == *env_id)
                    .ok_or_else(|| format!("environment '{env_id}' not found"))?;

                // Check if it's an env-local server
                if let Some(s) = env_cfg.servers.iter_mut().find(|s| s.id == *id) {
                    s.enabled = !s.enabled;
                    let state = if s.enabled { "enabled" } else { "disabled" };
                    println!("Server '{id}' is now {state} in environment '{env_id}'.");
                } else {
                    // It must be a main server — toggle via override
                    let main_server_exists = cfg.main.servers.iter().any(|s| s.id == *id);
                    if !main_server_exists {
                        return Err(format!("server '{id}' not found").into());
                    }

                    // Re-borrow env_cfg after checking main
                    let env_cfg = cfg
                        .environments
                        .iter_mut()
                        .find(|e| e.id == *env_id)
                        .unwrap();

                    if let Some(ov) = env_cfg.overrides.iter_mut().find(|o| o.server_id == *id) {
                        let current = ov.enabled.unwrap_or(true);
                        ov.enabled = Some(!current);
                        let state = if !current { "enabled" } else { "disabled" };
                        println!("Server '{id}' override is now {state} in environment '{env_id}'.");
                    } else {
                        // No override exists — the server is currently enabled (from main),
                        // so add an override to disable it.
                        env_cfg.overrides.push(config::ServerOverride {
                            server_id: id.clone(),
                            enabled: Some(false),
                            url: None,
                            permissions: None,
                        });
                        println!("Server '{id}' is now disabled in environment '{env_id}' (override added).");
                    }
                }
            } else {
                // Toggle in main
                let s = cfg
                    .main
                    .servers
                    .iter_mut()
                    .find(|s| s.id == *id)
                    .ok_or_else(|| format!("server '{id}' not found in main config"))?;
                s.enabled = !s.enabled;
                let state = if s.enabled { "enabled" } else { "disabled" };
                println!("Server '{id}' is now {state}.");
            }

            config::save(&cfg_path, &cfg)?;
        }

        ServerCommands::Rename { id, name } => {
            // Search main first, then all environments
            let mut found = false;

            if let Some(s) = cfg.main.servers.iter_mut().find(|s| s.id == *id) {
                s.name = name.clone();
                found = true;
            }

            for env in &mut cfg.environments {
                if let Some(s) = env.servers.iter_mut().find(|s| s.id == *id) {
                    s.name = name.clone();
                    found = true;
                }
            }

            if found {
                config::save(&cfg_path, &cfg)?;
                println!("Renamed server '{id}' to '{name}'.");
            } else {
                return Err(format!("server '{id}' not found").into());
            }
        }
    }

    Ok(())
}
