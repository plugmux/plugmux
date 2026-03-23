use clap::Subcommand;

use plugmux_core::catalog::CatalogRegistry;

#[derive(Subcommand)]
pub enum CatalogCommands {
    /// List all available servers in the catalog
    List,
    /// Search the catalog for servers
    Search {
        /// Search query (matches name and description)
        query: String,
    },
    /// Browse servers by category
    Browse {
        /// Category to filter by
        #[arg(long)]
        category: String,
    },
}

pub fn run(cmd: &CatalogCommands) -> Result<(), Box<dyn std::error::Error>> {
    let catalog = CatalogRegistry::load_bundled();

    match cmd {
        CatalogCommands::List => {
            let servers = catalog.list_servers();
            if servers.is_empty() {
                println!("No servers in catalog.");
            } else {
                println!("Catalog servers ({} available):", servers.len());
                for s in servers {
                    println!(
                        "  {} ({}) [{}] - {}",
                        s.name, s.id, s.category, s.description
                    );
                }
            }

            println!();

            let presets = catalog.list_presets();
            if !presets.is_empty() {
                println!("Presets:");
                for p in presets {
                    println!(
                        "  {} ({}) - {} ({} servers)",
                        p.name,
                        p.id,
                        p.description,
                        p.servers.len()
                    );
                }
            }
        }

        CatalogCommands::Search { query } => {
            let results = catalog.search(query, None);
            if results.is_empty() {
                println!("No servers matching '{query}'.");
            } else {
                println!("Search results for '{query}':");
                for s in &results {
                    println!(
                        "  {} ({}) [{}] - {}",
                        s.name, s.id, s.category, s.description
                    );
                }
            }
        }

        CatalogCommands::Browse { category } => {
            let results = catalog.search("", Some(category));
            if results.is_empty() {
                println!("No servers in category '{category}'.");
            } else {
                println!("Servers in category '{category}':");
                for s in &results {
                    println!("  {} ({}) - {}", s.name, s.id, s.description);
                }
            }
        }
    }

    Ok(())
}
