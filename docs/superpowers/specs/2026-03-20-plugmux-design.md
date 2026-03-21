# plugmux — Design Spec

**Author:** Lasha Kvantaliani
**Date:** 2026-03-20
**Status:** Draft v1

---

## 1. What is plugmux?

plugmux is a local gateway that sits between AI coding agents (Claude, Cursor, Codex) and MCP servers. Instead of configuring every MCP server separately for every agent on every machine, you install plugmux once. It manages all your MCP servers, organizes them into environments per project, and gives each environment a single URL you point your agent at.

### The Problem

Today, using MCP servers with AI agents is painful:

1. **Restart hell** — every time you add or change an MCP server, you restart your agent
2. **Config file juggling** — managing servers through JSON files or terminal commands
3. **Context window bloat** — too many MCP servers dump thousands of tokens of tool schemas before the agent does anything useful
4. **No cross-machine story** — setting up the same MCP servers on another machine means starting from scratch
5. **Per-agent configuration** — must install and configure servers separately for Claude, Cursor, Codex

### The Solution

plugmux is one install that solves all five:

- **Hot-reload** — add/remove servers without restarting your agent
- **Visual management** — tray app with full UI, no JSON editing
- **Scoped environments** — agent only sees servers relevant to the current project
- **Cloud sync** — one-click to replicate your setup on another machine
- **Agent-agnostic** — any agent that speaks MCP connects to plugmux's URL

---

## 2. Core Concepts

### Server

A configured MCP server. Each has:
- **id** — unique internal identifier
- **name** — user-customizable display name (shown in UI and to LLMs)
- **transport** — `stdio` or `http+sse`
- **connectivity** — `local` or `online`
- **enabled/disabled** toggle

### Main

The base layer. Servers added here are inherited by every environment. This is where you put your essential, always-available servers (GitHub, Figma, Context7, etc.).

### Environment

A named workspace that extends Main. Each environment:
- Inherits all Main servers (can disable inherited ones per-environment)
- Has additional servers specific to this context
- Gets a unique endpoint URL using a slugified name (e.g., `http://localhost:4242/env/my-saas-app`)
- Has its own permission settings

You point your agent at an environment URL. One URL per project.

### Preset

A one-click template to bootstrap an environment. Examples: "Web Dev", "Backend", "Data Science". Creates an environment pre-loaded with relevant servers. User customizes after creation.

### Permissions (per environment)

Each management action has three states:
- **Allow** — LLM does it freely
- **Approve** — LLM requests it, user confirms via tray notification
- **Disable** — not available to LLM

---

## 3. Architecture

### Two Binaries, One Codebase

```
plugmux (monorepo)
├── crates/
│   ├── plugmux-core/        # Library crate — gateway logic
│   │   ├── proxy/           # MCP proxy router (stdio + HTTP+SSE)
│   │   ├── registry/        # Server registry, config management
│   │   ├── environments/    # Environment manager, endpoint routing
│   │   └── mcp_server/      # Gateway MCP server (tools exposed to LLMs)
│   │
│   ├── plugmux-cli/         # Binary crate — headless CLI
│   │   └── main.rs          # Starts gateway, no UI
│   │
│   └── plugmux-app/         # Tauri app — tray + web UI (Phase 2)
│       ├── src-tauri/       # Rust: tray icon, system integration, wraps plugmux-core
│       └── src/             # React: full management UI
│
├── web/                     # Next.js + shadcn/ui + Fumadocs (Phase 5)
│   ├── app/                 # Landing page + marketing
│   └── content/docs/        # MDX documentation
│
├── api/                     # Cloudflare Workers backend (Phase 3)
│
├── Cargo.toml               # Rust workspace root
└── README.md
```

**plugmux-core** is the shared library. Both CLI and Tauri app use it — no logic duplication.

### Runtime Architecture

```
Agent (Claude / Cursor / Codex)
    │
    │  connects to environment URL
    │  http://localhost:4242/env/my-saas-app
    │
    ▼
┌─── plugmux ─────────────────────────┐
│                                      │
│  Environment Router                  │
│    │                                 │
│  MCP Server (per-environment)        │
│    tools: list_servers, get_tools,   │
│    execute, enable_server,           │
│    disable_server                    │
│    │                                 │
│  Proxy Router                        │
│    │         │         │             │
└────┼─────────┼─────────┼────────────┘
     │         │         │
   figma    shadcn    browser
  (stdio)   (http)    (stdio)
```

Each environment gets its own HTTP endpoint. The agent connects to one URL and sees only the servers enabled in that environment.

### Health & Connectivity

- Each server is tagged `local` or `online`
- plugmux pings servers on startup and periodically
- Online servers that are unreachable are temporarily excluded from `list_servers()` — the LLM never sees them
- Tray icon reflects overall status: green (all healthy), yellow (some degraded), grey (offline mode)

### Cloud Backend (Cloudflare Workers)

Separate service, not part of the local binary:
- **Registry API** — community-submitted servers (name, icon, description, config, usage stats, likes)
- **Sync API** — push/pull user config for cross-machine sync
- **Auth** — GitHub OAuth

The local app works fully offline. Cloud is optional.

---

## 4. LLM-Facing Tools

Each environment endpoint exposes an MCP server with these tools:

### `list_servers()`

Returns all currently available (enabled + healthy) servers in this environment.

```json
{
  "servers": [
    { "id": "figma", "name": "Design", "description": "Read and inspect Figma designs", "status": "ready", "tools_count": 12 },
    { "id": "shadcn", "name": "shadcn/ui", "description": "Search and add shadcn components", "status": "ready", "tools_count": 7 }
  ],
  "environment": "my-saas-app",
  "total_servers": 2,
  "total_tools": 19
}
```

~200 tokens for 10 servers. LLM calls this first to orient.

### `get_tools(server_id)`

Returns full tool schemas for a specific server.

```json
{
  "server": "figma",
  "tools": [
    { "name": "get_design_context", "description": "...", "inputSchema": { ... } },
    { "name": "get_screenshot", "description": "...", "inputSchema": { ... } }
  ]
}
```

Only loads schemas the LLM actually needs.

### `execute(server_id, tool_name, args)`

Proxies the call to the upstream MCP server. Transparent pass-through.

```json
execute("figma", "get_screenshot", { "fileKey": "abc123", "nodeId": "1:2" })
→ { "imageUrl": "..." }
```

### `enable_server(server_id)` / `disable_server(server_id)`

Toggle a server within this environment. Respects the environment's permission settings (Allow/Approve/Disable).

### Admin Operations (Separate from Environments)

Environment management, permissions, and server installation are **not** exposed inside environment endpoints. They are accessed via:
- Tray UI (primary)
- CLI commands
- Optionally a dedicated admin endpoint (future)

This prevents confusion — the LLM working in "my-saas-app" doesn't see "delete rust-embedded environment."

---

## 5. Config File

Single file at `~/.config/plugmux/plugmux.json`. Human-readable, git-syncable.

```json
{
  "version": 1,
  "main": {
    "servers": [
      {
        "id": "figma",
        "name": "Design",
        "transport": "stdio",
        "command": "npx",
        "args": ["-y", "@anthropic/figma-mcp"],
        "connectivity": "online",
        "enabled": true
      },
      {
        "id": "context7",
        "name": "Context7",
        "transport": "http",
        "url": "https://context7.dev/mcp",
        "connectivity": "online",
        "enabled": true
      }
    ]
  },
  "environments": [
    {
      "id": "my-saas-app",
      "name": "My SaaS App",
      "endpoint": "http://localhost:4242/env/my-saas-app",
      "servers": [
        {
          "id": "shadcn",
          "name": "shadcn/ui",
          "transport": "stdio",
          "command": "npx",
          "args": ["-y", "@shadcn/mcp"],
          "connectivity": "online",
          "enabled": true
        }
      ],
      "overrides": {
        "browser-tools": { "enabled": false }
      },
      "permissions": {
        "enable_server": "approve",
        "disable_server": "approve"
      }
    }
  ],
  "cloud": {
    "sync_enabled": false,
    "account": null
  }
}
```

---

## 6. CLI Interface

```bash
# Gateway
plugmux start                          # Start gateway (foreground)
plugmux start --daemon                 # Start as background service
plugmux stop                           # Stop gateway
plugmux status                         # Show health of all servers

# Environments
plugmux env list                       # List all environments with URLs
plugmux env create my-project          # Create empty environment
plugmux env create my-project --preset web-dev
plugmux env delete my-project
plugmux env url my-project             # Print endpoint URL

# Servers (in Main)
plugmux server add figma --transport stdio --command "npx -y @anthropic/figma-mcp"
plugmux server add figma --name "Design"
plugmux server remove figma
plugmux server list
plugmux server toggle figma
plugmux server rename figma --name "My Design Tool"

# Servers (in environment)
plugmux server add shadcn --env my-project --transport stdio --command "npx -y @shadcn/mcp"
plugmux server toggle figma --env my-project
plugmux server list --env my-project

# Catalog
plugmux catalog search "figma"
plugmux catalog browse --category design
plugmux catalog install figma
plugmux catalog install figma --env my-project

# Config
plugmux config path
plugmux config export
plugmux config import ./plugmux.json

# Cloud sync (future)
plugmux login
plugmux sync push
plugmux sync pull
```

---

## 7. UI Design Brief

### For the Designer — What is plugmux?

plugmux is a desktop app for developers who use AI coding assistants (like Claude, Cursor, or GitHub Copilot). These AI assistants connect to external tools called "MCP servers" — think of them as plugins that give the AI new abilities (access to Figma designs, GitHub repos, databases, etc.).

The problem: managing these plugins is a mess. You have to edit config files, restart your AI assistant every time you add one, and set them up separately on each machine. If you have too many, they slow down your AI.

plugmux solves this. It's a single app that manages all your plugins in one place. You organize them into "environments" (one per project), and your AI assistant connects to one URL instead of dozens of individual plugins.

**Target user:** Developer-tool power users. People who are comfortable with technical tools but want a clean, efficient interface. Think someone who uses Raycast, Linear, or TablePlus daily.

### Tray Icon

- Lives in macOS menu bar / Linux system tray
- Small, monochrome icon (fits macOS menu bar style)
- **Status indicators:**
  - Green dot — all servers healthy
  - Yellow dot — some servers degraded or unreachable
  - Grey dot — offline mode (no internet, local servers still work)
- **Left-click** → opens main window
- **Right-click** → quick context menu:
  - Active environment name + URL (click to copy)
  - List of environments (click to switch/copy URL)
  - Divider
  - "Open plugmux" → main window
  - "Quit"

### Main Window

**Dimensions:** ~900x600 default, resizable. Tauri webview.

**Layout:** Fixed sidebar (left, ~220px) + content area (right).

**Theme:** Dark theme default, light theme option. The audience is developers — dark is expected.

#### Sidebar

- **plugmux logo** at top (small, minimal)
- **Navigation items** (icon + label):
  - **Main** — base server configuration (always first)
  - **Environments** section header
    - List of environments, each showing name + active server count badge
    - "+ New Environment" button at bottom of list
  - **Catalog** — browse community servers
  - **Presets** — environment templates
  - **Settings** — account, sync, appearance

#### Main Page (Base Servers)

This is the "home base" — servers here are inherited by every environment.

- **Header:** "Main" with subtitle "These servers are available in all environments"
- **Server list** (card or row format, each showing):
  - Server icon (from registry, or default icon)
  - Server name (user-editable display name)
  - Short description (one line)
  - Connectivity badge: "local" (green) or "online" (blue)
  - Health indicator: green dot (running), red dot (error), grey dot (stopped)
  - Enable/disable toggle switch
- **"Add Server" button** → opens catalog or manual config dialog
- Servers should be reorderable (drag to sort)

#### Environment Page

- **Header:** Environment name (editable) + unique endpoint URL with copy button
- **Inherited Servers section:**
  - Header: "From Main" with count
  - Same server row format as Main page
  - Toggle here acts as an override (disable/re-enable an inherited server for this env only)
  - Visually distinguish overridden (disabled) servers — e.g., dimmed row, strikethrough
- **Additional Servers section:**
  - Header: "Environment Servers" with count
  - Servers added specifically to this environment
  - Same row format + ability to remove
  - "Add Server" button
- **Permissions panel** (collapsible):
  - Table with action name + dropdown (Allow / Approve / Disable)
  - Actions: enable_server, disable_server, install_server
- **Danger zone** at bottom: "Delete Environment" button (with confirmation)

#### Catalog Page

- **Search bar** at top (simple text filtering by name, description, tags)
- **Category filter pills** below search: "All", "Design", "Database", "Dev Tools", "Browser", "AI", "Productivity", "Testing"
- **Server cards** in a responsive grid (3-4 columns):
  - Server icon (prominent)
  - Server name
  - Short description (2 lines max)
  - Category tag
  - Usage count ("12K installs")
  - Like count / heart button
  - "Add" button → dropdown: "Add to Main" / "Add to [environment name]..."
- **Clicking a card** → expanded detail view:
  - Full description
  - Config preview (what command/URL will be used)
  - Screenshots or documentation link if available
  - "Add to..." button

#### Presets Page

- **Grid of preset cards:**
  - Preset name ("Web Dev", "Backend", "Data Science", etc.)
  - Icon/illustration
  - List of included servers (with icons)
  - "Create Environment" button → names the new environment, populates servers
- **"Create Custom Preset"** button → pick servers from catalog/existing config, name it, save

#### Settings Page

- **Account:** Login with GitHub, account status, logout
- **Cloud Sync:** Enable/disable, last sync time, "Sync Now" button
- **Appearance:** Dark/light theme toggle
- **General:** Port number (default 4242), startup on login toggle, config file location

### Design Principles

- **Information density over whitespace** — developers prefer seeing more at once. Don't spread 5 items across a full screen.
- **Status at a glance** — every server shows health state. Every environment shows active server count. The tray icon shows overall status.
- **Copy-friendly** — endpoint URLs, server configs, IDs — all one-click copyable.
- **Keyboard navigable** — power users shouldn't need the mouse for common actions.
- **Fast transitions** — no unnecessary animations. Instant page switches. Toggles respond immediately.
- **Progressive disclosure** — permissions panel is collapsed by default. Advanced config (transport, command args) is behind an "edit" action, not shown in the list view.

### UI References

| App | What to Study | Why |
|-----|--------------|-----|
| **Docker Desktop** | Sidebar nav, container list with status toggles, tray icon states | Closest mental model — managing running services with on/off toggles |
| **Raycast** | Clean minimal UI, search-first design, quick actions | The "one-click" speed and polish we want |
| **Linear** | Sidebar navigation, list/detail views, dark theme, typography | Best-in-class dev tool UI |
| **TablePlus** | Connection management, sidebar with groups | Managing multiple connections (like environments) |
| **Setapp / Homebrew Cask** | App catalog with categories, install buttons | The catalog/discovery experience |
| **Tailscale menu bar** | Tray icon with status, simple dropdown, device list | Lightweight tray presence, network/status display |

---

## 8. Tech Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Gateway core | Rust | Single binary, fast, no runtime deps |
| Desktop app | Tauri v2 | Rust backend + web frontend, ~10MB binary, native tray |
| UI framework | React + TypeScript | Bundled in Tauri webview |
| Config | JSON file | Human-readable, git-syncable, no database needed |
| Cloud backend | Cloudflare Workers | Free tier, Rust/WASM support, global edge |
| Auth | GitHub OAuth | Developer audience, no password management |
| Registry storage | Cloudflare D1 / KV | Serverless database for catalog, usage stats, likes |

---

## 9. Roadmap

### Phase 1 — Core Gateway (CLI only)

- Rust workspace: `plugmux-core` + `plugmux-cli`
- Config file (`plugmux.json`): Main servers, environments, overrides
- MCP proxy router (stdio + HTTP+SSE transport)
- Environment endpoints with unique URLs
- LLM tools: `list_servers`, `get_tools`, `execute`, `enable_server`, `disable_server`
- Server health checks, connectivity tagging (local/online), offline exclusion
- CLI commands: start, stop, status, env, server, config

### Phase 2 — Tauri Desktop App

- `plugmux-app` crate wrapping `plugmux-core`
- Tray icon with health status indicators
- Main window: sidebar nav, server management, environment management
- Permission system (Allow/Approve/Disable) with tray notifications
- Custom server naming
- Enable/disable toggles, environment overrides

### Phase 3 — Catalog & Community

- Cloudflare Workers backend: registry API, search
- Curated initial set of ~30-50 quality servers with icons and descriptions
- In-app catalog browsing with category filters and text search
- Presets (Web Dev, Backend, Data Science, etc.)
- GitHub-based community submissions for new servers
- Usage stats + likes

### Phase 4 — Cloud Sync & Distribution

- GitHub OAuth accounts
- Config sync (push/pull across machines)
- Config export/import for manual/git-based sync
- Evaluate **SurrealDB** as embedded database — Rust-native, document storage, built-in sync/replication capabilities, can run embedded (no separate server). Good candidate for multi-device config merge.
- Homebrew formula
- macOS `.dmg`, Linux `.deb` / `.AppImage`
- Landing page at plugmux.com

### Phase 5 — Website, Docs & Landing Page

- Landing page at plugmux.com
- Built with Next.js + shadcn/ui + 21st.dev components
- Documentation powered by Fumadocs (MDX, built into Next.js, shadcn-compatible)
- Design using ui-ux-pro-max skill for polished developer-facing aesthetic
- Hero section with ASCII art banner or animated terminal demo
- Feature breakdown (environments, one-click presets, agent-agnostic)
- Pain-point-driven messaging (from founder's 5 pain points)
- Download section (macOS, Linux, Homebrew, CLI)
- Community section (GitHub, submit servers, presets)
- Pricing section (free for individuals, teams coming soon)
- Blog/changelog for updates
- SEO-optimized for "MCP gateway", "MCP server manager", "AI coding tools"

### Future (v2+)

- Teams / shared environments (paid tier)
- Admin MCP endpoint for LLM-driven environment management
- Custom preset sharing between users
- Webhook notifications for server health changes

---

## 10. Distribution

| Channel | Format | Target |
|---------|--------|--------|
| macOS app | `.dmg` via Tauri | Drag to Applications, tray icon |
| Homebrew | `brew install plugmux` | CLI-only or full app |
| Linux | `.deb` / `.AppImage` | CLI or desktop |
| CLI-only | Single Rust binary | Headless servers, CI, Linux terminals |

---

## 11. Monetization (Future)

- **Free tier:** Individuals, unlimited machines, full functionality
- **Paid tier (future):** Teams — shared environments, shared configs, access control, team catalog
- **Revenue wedge:** Cloud sync creates natural account system; teams are the upsell

---

*plugmux Design Spec — Lasha Kvantaliani — March 2026 — Draft v1*
