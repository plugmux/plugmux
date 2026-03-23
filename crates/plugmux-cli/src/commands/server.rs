use clap::Subcommand;

use plugmux_core::catalog::CatalogRegistry;
use plugmux_core::config;
use plugmux_core::custom_servers::CustomServerStore;
use plugmux_core::environment;

#[derive(Subcommand)]
pub enum ServerCommands {
    /// Add a server to an environment
    Add {
        /// Server ID (must exist in catalog or custom servers)
        server_id: String,
        /// Environment to add the server to
        #[arg(long)]
        env: String,
    },
    /// Remove a server from an environment
    Remove {
        /// Server ID
        server_id: String,
        /// Environment to remove the server from
        #[arg(long)]
        env: String,
    },
    /// List servers in an environment
    List {
        /// Environment to list servers for
        #[arg(long)]
        env: String,
    },
}

pub fn run(cmd: &ServerCommands) -> Result<(), Box<dyn std::error::Error>> {
    let cfg_path = config::config_path();
    let mut cfg = config::load_or_default(&cfg_path);

    match cmd {
        ServerCommands::Add { server_id, env } => {
            // Validate that the server ID exists in catalog or custom servers
            let catalog = CatalogRegistry::load_bundled();
            let custom_path = config::config_dir().join("custom_servers.json");
            let custom_store = CustomServerStore::load_or_default(custom_path);

            if !catalog.has_server(server_id) && !custom_store.has(server_id) {
                return Err(format!(
                    "server '{server_id}' not found in catalog or custom servers. \
                     Use `plugmux catalog list` or `plugmux custom add` first."
                )
                .into());
            }

            environment::add_server(&mut cfg, env, server_id)?;
            config::save(&cfg_path, &cfg)?;
            println!("Added server '{server_id}' to environment '{env}'.");
        }

        ServerCommands::Remove { server_id, env } => {
            let removed = environment::remove_server(&mut cfg, env, server_id)?;
            if removed {
                config::save(&cfg_path, &cfg)?;
                println!("Removed server '{server_id}' from environment '{env}'.");
            } else {
                return Err(
                    format!("server '{server_id}' not found in environment '{env}'").into(),
                );
            }
        }

        ServerCommands::List { env } => {
            let server_ids = environment::get_server_ids(&cfg, env)
                .ok_or_else(|| format!("environment '{env}' not found"))?;

            if server_ids.is_empty() {
                println!("No servers in environment '{env}'.");
            } else {
                println!("Servers in environment '{env}':");
                for id in &server_ids {
                    println!("  {id}");
                }
            }
        }
    }

    Ok(())
}
