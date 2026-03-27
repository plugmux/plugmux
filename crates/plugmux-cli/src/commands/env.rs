use clap::Subcommand;

use plugmux_core::catalog::CatalogRegistry;
use plugmux_core::config;
use plugmux_core::db::Db;
use plugmux_core::db::environments as db_env;
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
    let db = Db::open(&Db::default_path()).map_err(|e| format!("failed to open database: {e}"))?;
    let cfg = config::load_or_default(&config::config_path());

    match cmd {
        EnvCommands::List => {
            let envs = db_env::list_environments(&db);
            if envs.is_empty() {
                println!("No environments configured.");
                println!("Run `plugmux env create <name>` to get started.");
            } else {
                println!("Environments:");
                for env in &envs {
                    let server_count = db_env::get_server_ids(&db, &env.id)
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
            let existing = db_env::list_environments(&db);
            if existing.iter().any(|e| e.id == id) {
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

            db_env::add_environment(&db, &id, name)?;

            for server_id in &preset_servers {
                db_env::add_server(&db, &id, server_id)?;
            }

            println!("Created environment: {} ({})", name, id);
            println!("  Endpoint: http://localhost:{}/env/{id}", cfg.port);
        }

        EnvCommands::Delete { id } => {
            db_env::remove_environment(&db, id)?;
            println!("Deleted environment: {id}");
        }
    }

    Ok(())
}
