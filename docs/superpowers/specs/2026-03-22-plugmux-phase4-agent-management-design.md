# Phase 4: Agent Management — Design Spec

## Overview

Move agent detection, migration, and management from the Tauri app layer into plugmux-core as a first-class module. Build a comprehensive Agents page that replaces the Dashboard, supporting auto-scan, manual setup, enable/disable toggles, and a bundled registry of ~25 MCP-compatible tools.

## Goals

- Centralize agent management logic in plugmux-core (shared by app + CLI)
- Support ~15 auto-scanned dev tools and ~10 additional manual-add agents
- Provide both auto-connect and guided manual setup flows
- Enable per-agent enable/disable with surgical config manipulation
- Ship a bundled icon set (color + monochrome) for all registered agents

## Connect/Disconnect Semantics

**Connect** = add `"plugmux"` key to the agent's `mcpServers` section, preserving all existing MCP entries. A backup of the pre-connect mcpServers state is saved on first connect. After connect, existing MCPs remain — status is yellow if others exist, green if plugmux is the only one.

**Disconnect** = remove only the `"plugmux"` key from mcpServers. All other MCP entries remain untouched. Agent reverts to whatever MCPs it had alongside plugmux.

**Disconnect & Restore** = remove the `"plugmux"` key AND replace the current mcpServers with the backed-up pre-connect state. This reverts the agent to exactly how it was before plugmux was first added. Only available when a backup exists.

## Agent Registry

### Data Model

Bundled as `agents/agents.json` at workspace root (following the `catalog/servers.json` pattern), embedded via `include_str!`:

```json
{
  "agents": [
    {
      "id": "claude-code",
      "name": "Claude Code",
      "icon": "claudecode",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/.claude/settings.json",
        "linux": "~/.claude/settings.json",
        "windows": "%USERPROFILE%\\.claude\\settings.json"
      }
    }
  ]
}
```

Fields:
- `id` — unique identifier
- `name` — display name
- `icon` — filename stem in the icon set (e.g., `claudecode` → `claudecode.svg`, `claudecode-color.svg`). Null if no icon available.
- `config_format` — `json` or `toml`
- `mcp_key` — the key name for MCP servers in the config (`mcpServers` for JSON, `mcp_servers` for TOML)
- `tier` — `auto` (scanned on startup) or `manual` (available in "Add agent" registry)
- `config_paths` — per-OS paths. `~` expanded at runtime. May be null for manual-tier agents where the path varies or is GUI-configured.

### Registered Agents

**Tier: auto (auto-scan on startup, ~15):**

| # | ID | Name | Config Path (macOS) | Format |
|---|---|---|---|---|
| 1 | claude-code | Claude Code | ~/.claude/settings.json | json |
| 2 | cursor | Cursor | ~/.cursor/mcp.json | json |
| 3 | windsurf | Windsurf | ~/.codeium/windsurf/mcp_config.json | json |
| 4 | codex | Codex | ~/.codex/config.toml | toml |
| 5 | gemini-cli | Gemini CLI | ~/.gemini/settings.json | json |
| 6 | antigravity | Antigravity | ~/.gemini/antigravity/mcp_config.json | json |
| 7 | vscode | VS Code | ~/.vscode/mcp.json | json |
| 8 | copilot-cli | GitHub Copilot CLI | ~/.copilot/mcp-config.json | json |
| 9 | zed | Zed | ~/.config/zed/settings.json | json |
| 10 | cline | Cline | macOS: ~/Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json, Linux: ~/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json, Windows: %APPDATA%/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json | json |
| 11 | roocode | Roo Code | macOS: ~/Library/Application Support/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/mcp_settings.json, Linux: ~/.config/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/mcp_settings.json, Windows: %APPDATA%/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/mcp_settings.json | json |
| 12 | continue | Continue | ~/.continue/config.json | json |
| 13 | goose | Goose | ~/.config/goose/config.json | json |
| 14 | opencode | OpenCode | ~/.config/opencode/config.json | json |
| 15 | trae | Trae | ~/.trae/mcp.json | json |

**Tier: manual (add via registry, ~10):**

| # | ID | Name | Format | Notes |
|---|---|---|---|---|
| 16 | cherrystudio | Cherry Studio | json | Desktop app, config path varies |
| 17 | lmstudio | LM Studio | json | Desktop app, config path varies |
| 18 | openwebui | Open WebUI | json | Web-based, admin settings |
| 19 | n8n | n8n | json | Workflow automation |
| 20 | dify | Dify | json | AI app builder |
| 21 | coze | Coze | json | Bot builder |
| 22 | openhands | OpenHands | json | AI agent |
| 23 | poe | Poe | json | AI chat aggregator |
| 24 | jetbrains | JetBrains | json | ~/.config/github-copilot/mcp.json (via Copilot plugin) |
| 25 | monica | Monica | json | AI assistant |

Note: Manual-tier agents have no `config_paths` in the registry (or partial/uncertain paths). When a user adds one from the registry, they are prompted to provide the config file path. The registry provides icon and name pre-fill only.

LibreChat (YAML format) is excluded — YAML support is deferred. Can be added as a custom agent manually.

## Architecture

### Core Module: `plugmux-core/src/agents/`

```
agents/
├── mod.rs          — public API: AgentRegistry, AgentStatus, AgentState
├── registry.rs     — load/query bundled agents.json
├── detect.rs       — scan filesystem, determine installed/connected/status
├── migrate.rs      — surgical add/remove plugmux key, backup/restore
└── state.rs        — load/save agents_state.json (user's agent list + dismissed)
```

**`registry.rs`**
- `AgentRegistry::new()` — loads embedded agents.json
- `AgentRegistry::list_agents()` — all agents
- `AgentRegistry::list_auto_agents()` — tier=auto only
- `AgentRegistry::get_agent(id)` — lookup by ID
- `AgentRegistry::resolve_config_path(agent, os)` — expand ~ and env vars

**`detect.rs`**
- `detect_all(registry, state) -> Vec<DetectedAgent>` — scan auto-tier agents, merge with user state, exclude dismissed
- `detect_agent(agent) -> DetectedAgent` — check single agent
- `DetectedAgent` — id, name, installed (bool), status (green/yellow/gray), config_path
- Status logic:
  - Green: plugmux is the only entry in mcpServers
  - Yellow: plugmux present + other MCPs also present
  - Gray: agent installed but no plugmux key (or agent not installed)

**`migrate.rs`**
- `connect_agent(agent, port) -> Result<BackupPath>` — adds `"plugmux"` key to mcpServers alongside existing entries. If no backup exists yet, saves current mcpServers to backup file first. Returns the backup file path (or None if mcpServers was empty).
- `disconnect_agent(agent) -> Result<()>` — removes only the `"plugmux"` key from mcpServers, leaves all other entries intact.
- `disconnect_and_restore(agent) -> Result<()>` — removes `"plugmux"` key and replaces entire mcpServers with the contents of the backup file. Deletes the backup file after restore.
- `get_backup_path(agent) -> Option<PathBuf>` — returns path to backup file if it exists
- `BackupPath` = `Option<String>` (path to the backup file, or None if no MCPs were backed up)
- Backup file: `mcp_servers.backup_original_YYYY-MM-DD.json` in agent's config directory

**`state.rs`**
- `AgentState` struct — holds user's agent list and dismissed list
- `AgentState::load(config_dir) -> AgentState` — loads from `agents_state.json`, returns empty state if file doesn't exist
- `AgentState::save(config_dir) -> Result<()>` — persists to disk
- `AgentState::add_agent(entry)` — add auto/registry/custom agent
- `AgentState::dismiss_agent(id)` — add to dismissed list, remove from agents
- `AgentState::is_dismissed(id) -> bool` — check dismissed list
- Called by Engine on startup, updated on connect/disconnect/add/dismiss operations

### Dependencies

plugmux-core will need additional dependencies:
- `toml = "0.8"` — for Codex TOML config parsing
- `chrono = "0.4"` — for backup file date stamps

### User State: `agents_state.json`

Persisted in plugmux config directory (`~/.config/plugmux/` or platform equivalent):

```json
{
  "agents": [
    {
      "id": "claude-code",
      "source": "auto"
    },
    {
      "id": "jetbrains",
      "source": "registry",
      "config_path": "~/.config/github-copilot/mcp.json"
    },
    {
      "id": "my-custom-tool",
      "source": "custom",
      "name": "My Custom Tool",
      "config_path": "/path/to/config.json",
      "config_format": "json",
      "mcp_key": "mcpServers"
    }
  ],
  "dismissed_agents": ["poe", "monica"]
}
```

- `source`: `auto` (from scan), `registry` (added from registry), `custom` (user-defined)
- `config_path`: required for `registry` and `custom` sources (user provides it)
- `name`, `config_format`, `mcp_key`: only needed for `custom` source (registry agents get these from agents.json)
- `dismissed_agents`: agents the user explicitly deleted — auto-scan won't re-add them. Single source of truth for dismissal (no per-agent dismissed field).

### Tauri Commands (thin wrappers)

All existing agent commands in `commands.rs` replaced with calls to core:
- `detect_agents()` → `agents::detect_all()`
- `scan_agent_servers(ids)` → `agents::detect::scan_mcp_servers()`
- `migrate_agents(ids)` → `agents::migrate::connect_agent()` per agent
- New: `connect_agent(id)`, `disconnect_agent(id, restore: bool)`, `add_agent_from_registry(id, config_path)`, `add_custom_agent(name, config_path, format, mcp_key)`, `dismiss_agent(id)`, `get_agent_registry()`

### CLI Commands

New `agents` subcommand module in plugmux-cli:

```
plugmux agents list              — show all detected agents with status
plugmux agents connect <id>      — connect a specific agent
plugmux agents connect --all     — auto-connect all detected agents
plugmux agents disconnect <id>   — remove plugmux from agent (surgical)
plugmux agents disconnect <id> --restore  — remove + restore backup
plugmux agents status            — show connection status for all
```

## UI Design

### Agents Page (replaces Dashboard)

**Sidebar:** Rename "Dashboard" to "Agents". Change icon from `LayoutDashboard` to `Cable` or `Plug`.

**Layout:**
- Banner at top (only shown when no agents are connected): "Connect your code agents" with "Setup" button
- "Add agent" button top-right of table
- Table/list of agents

**Agent row:**
- Status dot (green / yellow / gray)
- Agent icon (color SVG, or 2-letter thumbnail if no icon)
- Agent name
- Config file path (truncated, muted)
- Enable/Disable toggle
- Delete button (trash icon)

### Setup Flow (from banner "Setup" button)

**Step 1 — Choose method (dialog):**

Two cards:

**Auto Connect (Recommended)**
> plugmux will find all installed agents, back up their configs, and connect them automatically.

**Manual Setup (Advanced)**
> Add plugmux to your agent configs yourself, with step-by-step guidance.

**Auto path → Step 2a:**
- Scan agents → show found agents with checkboxes (pre-selected) → "Connect" button
- After connection, agents appear in table with green/yellow dots

**Manual path → Step 2b:**
- JSON snippet with copy button (dynamically generated using current port):
  ```json
  {
    "plugmux": {
      "url": "http://localhost:{port}/env/default"
    }
  }
  ```
- Instruction: "Add this to the `mcpServers` section of your agent's config file, save, and restart the agent."
- Table of known agents with their config file paths for reference
- "Validate" button — re-scans and shows green checkmarks next to successfully configured agents

### Enable/Disable Toggle

**Toggle ON (gray → green/yellow):**
- Adds plugmux key to agent's mcpServers (preserving existing entries)
- Creates backup of current mcpServers if backup doesn't already exist
- Status dot updates

**Toggle OFF — confirmation dialog:**
- Title: "Disable plugmux for [Agent Name]"
- Description: "plugmux MCP will be removed from this agent's configuration."
- Two buttons:
  - **"Disable"** — removes plugmux key only, preserves other MCPs
  - **"Disable & Restore"** — removes plugmux key + restores mcpServers from backup to pre-connect state (only shown if backup file exists)
- After confirmation: "Restart [Agent Name] to apply changes."

### Delete Agent

- Removes agent from the list
- If currently enabled, runs disable flow first (same confirmation)
- Adds agent ID to `dismissed_agents` list (prevents auto-scan re-adding)
- Terminology: UI says "Delete", code uses `dismiss_agent` internally (delete from UI perspective, dismiss from persistence perspective since we need to remember not to re-add)

### Add Agent Dialog

**Two tabs:**

**Tab: Agents**
- Grid/list of registered agents not already in user's list
- Each shows icon + name
- Click one → prompted for config file path (with file picker) since manual-tier agents don't have known paths
- After providing path, agent added to table as gray (not connected)

**Tab: Custom**
- Name field (required)
- Config file path field (required) with file picker button
- Config format dropdown (JSON / TOML), defaults to JSON
- MCP key field, defaults to "mcpServers"
- "Add" button
- Agent appears with 2-letter thumbnail, gray status

## Icon Set

Download from [lobehub/lobe-icons](https://github.com/lobehub/lobe-icons) for all registered agents:
- Color variant: `{id}-color.svg` (used in agent rows)
- Monochrome variant: `{id}.svg` (available for alternative themes)

Stored in `assets/agent-icons/` at workspace root. Vite references these via `@/assets/agent-icons/` import alias or copies them into the app's src directory.

**2-letter thumbnail fallback:** For agents without icons (custom agents, some manual-tier without LobeHub icons), generate a colored circle with first two letters of agent name. Color derived from agent name hash for consistency.

## Migration from Current State

Current agent code in `crates/plugmux-app/src-tauri/src/commands.rs` (detect_agents, scan_agent_servers, migrate_agents, helper functions) moves to `plugmux-core/src/agents/`. The Tauri commands become thin wrappers. Frontend components (SetupDialog, AgentIcon, DashboardPage) get refactored to match the new design.

## Phase Ordering

- Phase 1: Core Gateway CLI (done)
- Phase 2: Tauri Desktop App (done)
- Phase 3: Catalog & Community (done)
- **Phase 4: Agent Management (this spec)**
- Phase 5: Cloud Sync & Distribution (was Phase 4, pushed)

## Out of Scope

- Cloud sync / SurrealDB (Phase 5)
- Per-environment agent connections (agents connect to default env for now)
- Agent health monitoring / live connection status
- YAML config format support (deferred until needed)
