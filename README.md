# plugmux

[![CI](https://github.com/plugmux/plugmux/actions/workflows/ci.yml/badge.svg)](https://github.com/plugmux/plugmux/actions/workflows/ci.yml)
[![Release](https://github.com/plugmux/plugmux/actions/workflows/release.yml/badge.svg)](https://github.com/plugmux/plugmux/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Tauri v2](https://img.shields.io/badge/tauri-v2-blue.svg)](https://v2.tauri.app)

**One URL, all your MCP servers.**

plugmux is a local gateway that aggregates multiple [Model Context Protocol (MCP)](https://modelcontextprotocol.io) servers behind a single HTTP endpoint. Instead of configuring each AI agent with a dozen separate MCP server connections, point it at one plugmux URL and let the agent discover and use tools across all your servers dynamically.

## Quick Start

```bash
# Build from source
cargo build --release

# Add a server to Main (shared across all environments)
plugmux server add --name "Filesystem" --command npx -- -y @modelcontextprotocol/server-filesystem /tmp

# Create an environment for your project
plugmux env create my-project

# Start the gateway
plugmux start

# Configure your AI agent to use:
#   http://127.0.0.1:4242/env/my-project
```

Your agent now has access to 5 gateway tools: `list_servers`, `get_tools`, `execute`, `enable_server`, and `disable_server`. It can discover available MCP servers, browse their tools, and call them -- all through a single endpoint.

## How It Works

```
  AI Agent
     |
     |  POST /env/my-project  (JSON-RPC)
     v
 +--------+
 |plugmux |----> MCP Server A (stdio)
 |gateway |----> MCP Server B (stdio)
 |        |----> MCP Server C (HTTP)
 +--------+
```

The agent sends standard MCP JSON-RPC requests to plugmux. plugmux exposes five meta-tools that let the agent navigate and use the underlying servers:

1. **list_servers** -- see all healthy servers in the environment
2. **get_tools** -- list tools from a specific server
3. **execute** -- call a tool on a specific server
4. **enable_server** / **disable_server** -- toggle servers at runtime

## Concepts

### Main

The **Main** section holds servers that are shared across every environment. When you add a server with `plugmux server add`, it goes into Main by default.

### Environments

An **Environment** is a named configuration scope (e.g. "work", "personal", "my-project"). Each environment:

- Inherits all Main servers
- Can add its own environment-specific servers
- Can override Main servers (disable them, change URLs, set permissions)
- Gets its own HTTP endpoint: `/env/{environment-id}`

### Transports

plugmux supports two MCP transport types:

- **stdio** -- spawns the server as a child process (local tools like filesystem, git)
- **http** -- connects to a remote MCP server over HTTP Streamable transport

## CLI Reference

```
plugmux start [--port PORT]        Start the gateway (default port: 4242)
plugmux stop  [--port PORT]        Show stop instructions
plugmux status [--port PORT]       Show gateway status

plugmux env create <name>          Create a new environment
plugmux env list                   List all environments
plugmux env delete <id>            Delete an environment

plugmux server add [options]       Add a server to Main
plugmux server list                List Main servers
plugmux server remove <id>         Remove a server from Main

plugmux config path                Show config file path
plugmux config show                Print current configuration
plugmux config edit                Open config in $EDITOR
```

## Configuration

plugmux stores its configuration at `~/.config/plugmux/plugmux.json`. You can edit it directly or use the CLI commands.

```json
{
  "main": {
    "servers": [
      {
        "id": "filesystem",
        "name": "Filesystem",
        "transport": "stdio",
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
        "connectivity": "local",
        "enabled": true
      }
    ]
  },
  "environments": [
    {
      "id": "my-project",
      "name": "My Project",
      "endpoint": "http://localhost:4242/env/my-project",
      "servers": [],
      "overrides": []
    }
  ]
}
```

## Development

```bash
# Run all tests (18 unit + 1 integration)
cargo test

# Check for lint warnings
cargo clippy --workspace --all-targets

# Build in release mode
cargo build --release
```

## License

MIT
