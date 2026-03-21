use clap::Subcommand;
use std::path::PathBuf;

use plugmux_core::config;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show the config file path
    Path,
    /// Export the config to a file
    Export {
        /// Output file path
        path: PathBuf,
    },
    /// Import a config from a file
    Import {
        /// Input file path
        path: PathBuf,
    },
}

pub fn run(cmd: &ConfigCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        ConfigCommands::Path => {
            println!("{}", config::config_path().display());
        }

        ConfigCommands::Export { path } => {
            let cfg_path = config::config_path();
            let cfg = config::load_or_default(&cfg_path)?;
            config::save(path, &cfg)?;
            println!("Config exported to: {}", path.display());
        }

        ConfigCommands::Import { path } => {
            let cfg = config::load(path)?;
            let cfg_path = config::config_path();
            config::save(&cfg_path, &cfg)?;
            println!("Config imported from: {}", path.display());
        }
    }

    Ok(())
}
