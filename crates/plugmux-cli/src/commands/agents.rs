use std::collections::HashSet;

use clap::Subcommand;
use plugmux_core::agents::{
    AgentRegistry, AgentStatus, connect_agent, detect_all, disconnect_agent, disconnect_and_restore,
};
use plugmux_core::config;
use plugmux_core::db::Db;

#[derive(Subcommand)]
pub enum AgentCommands {
    /// List all detected agents with connection status
    List,
    /// Connect an agent to plugmux
    Connect {
        /// Agent ID to connect (or --all for all agents)
        id: Option<String>,
        /// Connect all detected agents
        #[arg(long)]
        all: bool,
    },
    /// Disconnect an agent from plugmux
    Disconnect {
        /// Agent ID to disconnect
        id: String,
        /// Also restore original MCP servers from backup
        #[arg(long)]
        restore: bool,
    },
    /// Show connection status for all agents
    Status,
}

pub fn run(cmd: &AgentCommands) -> Result<(), Box<dyn std::error::Error>> {
    let registry = AgentRegistry::load_bundled();
    let db = Db::open(&Db::default_path()).map_err(|e| format!("failed to open database: {e}"))?;

    match cmd {
        AgentCommands::List => {
            let agents = detect_all(&registry, &db, &HashSet::new());
            if agents.is_empty() {
                println!("No agents detected.");
                return Ok(());
            }
            println!("{:<20} {:<15} CONFIG PATH", "AGENT", "STATUS");
            println!("{}", "-".repeat(70));
            for agent in &agents {
                let status = match agent.status {
                    AgentStatus::Green => "active",
                    AgentStatus::Yellow => "configured",
                    AgentStatus::Gray => "disconnected",
                };
                let path = agent.config_path.as_deref().unwrap_or("-");
                println!("{:<20} {:<15} {}", agent.name, status, path);
            }
            Ok(())
        }
        AgentCommands::Connect { id, all } => {
            let cfg = config::load_or_default(&config::config_path());
            let port = cfg.port;

            if *all {
                let agents = detect_all(&registry, &db, &HashSet::new());
                for agent in agents.iter().filter(|a| a.installed) {
                    if let Some(entry) = registry.get_agent(&agent.id)
                        && let Some(path) = registry.resolve_config_path(entry)
                    {
                        match connect_agent(&path, &entry.config_format, &entry.mcp_key, port) {
                            Ok(_) => println!("Connected: {}", agent.name),
                            Err(e) => println!("Failed {}: {}", agent.name, e),
                        }
                    }
                }
            } else if let Some(agent_id) = id {
                if let Some(entry) = registry.get_agent(agent_id) {
                    if let Some(path) = registry.resolve_config_path(entry) {
                        connect_agent(&path, &entry.config_format, &entry.mcp_key, port)?;
                        println!("Connected: {}", entry.name);
                    } else {
                        println!("No config path for this agent on this OS");
                    }
                } else {
                    println!("Agent not found: {agent_id}");
                }
            } else {
                println!("Specify an agent ID or use --all");
            }
            Ok(())
        }
        AgentCommands::Disconnect { id, restore } => {
            if let Some(entry) = registry.get_agent(id) {
                if let Some(path) = registry.resolve_config_path(entry) {
                    if *restore {
                        disconnect_and_restore(&path, &entry.config_format, &entry.mcp_key)?;
                        println!("Disconnected and restored: {}", entry.name);
                    } else {
                        disconnect_agent(&path, &entry.config_format, &entry.mcp_key)?;
                        println!("Disconnected: {}", entry.name);
                    }
                } else {
                    println!("No config path for this agent on this OS");
                }
            } else {
                println!("Agent not found: {id}");
            }
            Ok(())
        }
        AgentCommands::Status => {
            let agents = detect_all(&registry, &db, &HashSet::new());
            let connected: Vec<_> = agents
                .iter()
                .filter(|a| matches!(a.status, AgentStatus::Green | AgentStatus::Yellow))
                .collect();
            if connected.is_empty() {
                println!("No agents connected to plugmux.");
                return Ok(());
            }
            for agent in connected {
                let status_str = match agent.status {
                    AgentStatus::Green => "active",
                    AgentStatus::Yellow => "configured",
                    AgentStatus::Gray => "not connected",
                };
                println!("{} ({})", agent.name, status_str);
            }
            Ok(())
        }
    }
}
