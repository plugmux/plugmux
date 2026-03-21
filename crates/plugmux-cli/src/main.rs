mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "plugmux", version, about = "MCP gateway -- one URL, all your servers")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the plugmux gateway
    Start {
        /// Port to listen on (default: 4242)
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Check if the gateway is running and show stop instructions
    Stop {
        /// Port to check (default: 4242)
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Show gateway status
    Status {
        /// Port to check (default: 4242)
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Manage environments
    Env {
        #[command(subcommand)]
        command: commands::env::EnvCommands,
    },
    /// Manage servers
    Server {
        #[command(subcommand)]
        command: commands::server::ServerCommands,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: commands::config::ConfigCommands,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Start { port } => commands::start::run(*port).await,
        Commands::Stop { port } => commands::stop::run(*port).await,
        Commands::Status { port } => commands::status::run(*port).await,
        Commands::Env { command } => commands::env::run(command).map_err(Into::into),
        Commands::Server { command } => commands::server::run(command).map_err(Into::into),
        Commands::Config { command } => commands::config::run(command).map_err(Into::into),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
