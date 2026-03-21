use clap::Subcommand;

use plugmux_core::config;
use plugmux_core::environment::resolve_named;
use plugmux_core::slug::slugify;

#[derive(Subcommand)]
pub enum EnvCommands {
    /// List all environments
    List,
    /// Create a new environment
    Create {
        /// Environment name
        name: String,
        /// Use a preset configuration (copies servers from another environment)
        #[arg(long)]
        preset: Option<String>,
    },
    /// Delete an environment
    Delete {
        /// Environment ID (slug)
        id: String,
    },
    /// Show the MCP endpoint URL for an environment
    Url {
        /// Environment ID (slug)
        id: String,
    },
}

pub fn run(cmd: &EnvCommands) -> Result<(), Box<dyn std::error::Error>> {
    let cfg_path = config::config_path();
    let mut cfg = config::load_or_default(&cfg_path)?;

    match cmd {
        EnvCommands::List => {
            if cfg.environments.is_empty() {
                println!("No environments configured.");
                println!("Run `plugmux env create <name>` to get started.");
            } else {
                println!("Environments:");
                for env in &cfg.environments {
                    let server_count = resolve_named(&cfg, &env.id)
                        .map(|s| s.len())
                        .unwrap_or(0);
                    println!(
                        "  {} ({}) - {} servers - {}",
                        env.name, env.id, server_count, env.endpoint
                    );
                }
            }
        }

        EnvCommands::Create { name, preset } => {
            let id = slugify(name);

            // Check for duplicates
            if cfg.environments.iter().any(|e| e.id == id) {
                return Err(format!("environment '{id}' already exists").into());
            }

            // If a preset is given, copy servers from the preset environment
            let preset_servers = if let Some(preset_id) = preset {
                let preset_env = cfg
                    .environments
                    .iter()
                    .find(|e| e.id == slugify(preset_id))
                    .ok_or_else(|| format!("preset environment '{preset_id}' not found"))?;
                preset_env.servers.clone()
            } else {
                Vec::new()
            };

            let env = config::add_environment(&mut cfg, name);
            env.servers = preset_servers;

            config::save(&cfg_path, &cfg)?;
            println!("Created environment: {} ({})", name, id);
            println!("  Endpoint: http://localhost:4242/env/{id}");
        }

        EnvCommands::Delete { id } => {
            if config::remove_environment(&mut cfg, id) {
                config::save(&cfg_path, &cfg)?;
                println!("Deleted environment: {id}");
            } else {
                return Err(format!("environment '{id}' not found").into());
            }
        }

        EnvCommands::Url { id } => {
            let env = config::find_environment(&cfg, id)
                .ok_or_else(|| format!("environment '{id}' not found"))?;
            println!("{}", env.endpoint);
        }
    }

    Ok(())
}
