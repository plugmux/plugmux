use clap::Subcommand;

use plugmux_core::catalog::CatalogRegistry;
use plugmux_core::config;
use plugmux_core::environment;
use plugmux_core::slug::slugify;

#[derive(Subcommand)]
pub enum EnvCommands {
    /// List all environments
    List,
    /// Create a new environment
    Create {
        /// Environment name
        name: String,
        /// Use a preset configuration (adds preset servers to the new environment)
        #[arg(long)]
        preset: Option<String>,
    },
    /// Delete an environment
    Delete {
        /// Environment ID (slug)
        id: String,
    },
}

pub fn run(cmd: &EnvCommands) -> Result<(), Box<dyn std::error::Error>> {
    let cfg_path = config::config_path();
    let mut cfg = config::load_or_default(&cfg_path);

    match cmd {
        EnvCommands::List => {
            if cfg.environments.is_empty() {
                println!("No environments configured.");
                println!("Run `plugmux env create <name>` to get started.");
            } else {
                println!("Environments:");
                for env in &cfg.environments {
                    let server_count = environment::get_server_ids(&cfg, &env.id)
                        .map(|s| s.len())
                        .unwrap_or(0);
                    println!(
                        "  {} ({}) - {} servers - http://localhost:{}/env/{}",
                        env.name, env.id, server_count, cfg.port, env.id
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

            // If a preset is given, look it up from the catalog
            let preset_servers = if let Some(preset_id) = preset {
                let catalog = CatalogRegistry::load_bundled();
                let preset_entry = catalog
                    .get_preset(preset_id)
                    .ok_or_else(|| format!("preset '{preset_id}' not found"))?;
                preset_entry.servers.clone()
            } else {
                Vec::new()
            };

            let env = config::add_environment(&mut cfg, name);
            env.servers = preset_servers;

            config::save(&cfg_path, &cfg)?;
            println!("Created environment: {} ({})", name, id);
            println!("  Endpoint: http://localhost:{}/env/{id}", cfg.port);
        }

        EnvCommands::Delete { id } => {
            config::remove_environment(&mut cfg, id)?;
            config::save(&cfg_path, &cfg)?;
            println!("Deleted environment: {id}");
        }
    }

    Ok(())
}
