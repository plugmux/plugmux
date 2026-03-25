use clap::Subcommand;

use plugmux_core::catalog::CatalogRegistry;
use plugmux_core::config;
use plugmux_core::custom_servers::CustomServerStore;
use plugmux_core::db::Db;
use plugmux_core::db::environments as db_env;

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
    let db = Db::open(&Db::default_path()).map_err(|e| format!("failed to open database: {e}"))?;

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

            db_env::add_server(&db, env, server_id)?;
            println!("Added server '{server_id}' to environment '{env}'.");
        }

        ServerCommands::Remove { server_id, env } => {
            db_env::remove_server(&db, env, server_id)?;
            println!("Removed server '{server_id}' from environment '{env}'.");
        }

        ServerCommands::List { env } => {
            let server_ids = db_env::get_server_ids(&db, env)
                .map_err(|e| format!("environment '{env}' not found: {e}"))?;

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
