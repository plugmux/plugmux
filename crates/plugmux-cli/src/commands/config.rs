use clap::Subcommand;

use plugmux_core::catalog::CatalogRegistry;
use plugmux_core::config;
use plugmux_core::db::Db;
use plugmux_core::migration;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show the config file path
    Path,
    /// Show the current configuration
    Show,
    /// Migrate from Phase 2 config format to Phase 3
    Migrate,
}

pub fn run(cmd: &ConfigCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        ConfigCommands::Path => {
            println!("{}", config::config_path().display());
        }

        ConfigCommands::Show => {
            let cfg_path = config::config_path();
            let cfg = config::load_or_default(&cfg_path);
            let json = serde_json::to_string_pretty(&cfg)?;
            println!("{json}");
        }

        ConfigCommands::Migrate => {
            if migration::needs_migration() {
                let catalog = CatalogRegistry::load_bundled();
                let db = Db::open(&Db::default_path())
                    .map_err(|e| format!("failed to open database: {e}"))?;
                migration::migrate(&catalog, &db)?;
                println!("Migration complete.");
                println!("  Old config backed up to: plugmux.json.backup");
                println!("  New config: {}", config::config_path().display());
            } else {
                println!("No migration needed.");
            }
        }
    }

    Ok(())
}
