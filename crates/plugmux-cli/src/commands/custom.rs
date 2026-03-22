use clap::Subcommand;

use plugmux_core::catalog::CatalogRegistry;
use plugmux_core::config;
use plugmux_core::custom_servers::CustomServerStore;
use plugmux_core::server::{Connectivity, ServerConfig, Transport};
use plugmux_core::slug::slugify;

#[derive(Subcommand)]
pub enum CustomCommands {
    /// Add a custom server
    Add {
        /// Server ID (will be slugified)
        #[arg(long)]
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
        /// Arguments for the command
        #[arg(long, num_args = 1..)]
        args: Option<Vec<String>>,
        /// URL (for http transport)
        #[arg(long)]
        url: Option<String>,
    },
    /// Edit a custom server
    Edit {
        /// Server ID
        id: String,
        /// New human-readable name
        #[arg(long)]
        name: Option<String>,
        /// New command
        #[arg(long)]
        command: Option<String>,
    },
    /// Remove a custom server
    Remove {
        /// Server ID
        id: String,
    },
    /// List all custom servers
    List,
}

pub fn run(cmd: &CustomCommands) -> Result<(), Box<dyn std::error::Error>> {
    let custom_path = config::config_dir().join("custom_servers.json");
    let mut store = CustomServerStore::load_or_default(custom_path);
    let catalog = CatalogRegistry::load_bundled();

    match cmd {
        CustomCommands::Add {
            id,
            name,
            transport,
            command,
            args,
            url,
        } => {
            let transport = match transport.as_str() {
                "stdio" => Transport::Stdio,
                "http" => Transport::Http,
                other => {
                    return Err(
                        format!("unknown transport: {other} (use 'stdio' or 'http')").into(),
                    )
                }
            };

            let server = ServerConfig {
                id: slugify(id),
                name: name.clone(),
                transport,
                command: command.clone(),
                args: args.clone(),
                url: url.clone(),
                connectivity: Connectivity::Local,
                description: None,
            };

            store.add(server, &catalog)?;
            store.save()?;
            println!("Added custom server: {} ({})", name, slugify(id));
        }

        CustomCommands::Edit { id, name, command } => {
            let existing = store
                .get(id)
                .ok_or_else(|| format!("custom server '{id}' not found"))?
                .clone();

            let updated = ServerConfig {
                id: existing.id.clone(),
                name: name.clone().unwrap_or(existing.name),
                transport: existing.transport,
                command: command.clone().or(existing.command),
                args: existing.args,
                url: existing.url,
                connectivity: existing.connectivity,
                description: existing.description,
            };

            store.update(id, updated)?;
            store.save()?;
            println!("Updated custom server: {id}");
        }

        CustomCommands::Remove { id } => {
            if store.remove(id) {
                store.save()?;
                println!("Removed custom server: {id}");
            } else {
                return Err(format!("custom server '{id}' not found").into());
            }
        }

        CustomCommands::List => {
            let servers = store.list();
            if servers.is_empty() {
                println!("No custom servers configured.");
                println!("Use `plugmux custom add` to add one.");
            } else {
                println!("Custom servers:");
                for s in &servers {
                    let transport = match s.transport {
                        Transport::Stdio => "stdio",
                        Transport::Http => "http",
                    };
                    println!("  {} ({}) [{}]", s.name, s.id, transport);
                }
            }
        }
    }

    Ok(())
}
