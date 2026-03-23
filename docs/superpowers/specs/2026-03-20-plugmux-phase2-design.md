# plugmux Phase 2 — Tauri Desktop App Design Spec

**Author:** Lasha Kvantaliani
**Date:** 2026-03-20
**Status:** Draft v1
**Parent spec:** `docs/superpowers/specs/2026-03-20-plugmux-design.md`

---

## 1. Goal

Wrap `plugmux-core` in a Tauri v2 desktop app with a tray icon and full management UI. The app becomes the primary way users interact with plugmux — starting the gateway, managing servers, configuring environments, and monitoring health.

---

## 2. Architecture

### Engine Embedding

The Tauri app embeds `plugmux-core` directly in-process. No separate daemon, no IPC, no Unix sockets.

```
plugmux-core (library)
     ↑              ↑
plugmux-cli      plugmux-app (Tauri)
  (headless)       (desktop)
```

- **Tauri app** launches → engine starts automatically → gateway listens on configured port (default 4242)
- **CLI** is a separate binary for headless environments (servers, CI). Embeds its own engine independently.
- They share the config file (`~/.config/plugmux/plugmux.json`) but do not communicate with each other.
- If one is already running and the other attempts to start, it gets a polite error: "plugmux is already running on port 4242."

### Why No IPC

The Tauri app calls `plugmux-core` Rust functions directly — zero serialization, zero network overhead. The CLI is for headless servers where there's no GUI. There's no practical scenario where both need to run simultaneously on the same machine.

Config-only CLI commands (`plugmux env list`, `plugmux server add`) read/write the JSON file directly and don't need a running engine.

### Config File Concurrency

If the Tauri app is running (engine active) and a CLI command modifies the config file on disk, the in-memory config must stay in sync. The Tauri app watches the config file using the `notify` crate. On external file change:

1. Reload config from disk
2. Diff against in-memory state
3. Apply changes to the running engine (start/stop servers as needed)
4. Emit events to update the React UI

### Tauri ↔ React Communication

```
React UI  ←→  Tauri Commands (Rust)  ←→  plugmux-core Engine
               ↓
          Tauri Event System → React (state updates)
```

- **Tauri commands:** React calls Rust functions via `invoke()`. Each UI action (toggle server, create environment) maps to a Tauri command that calls `plugmux-core`.
- **Tauri events:** Engine state changes (health updates, server toggled) emit Tauri events. React subscribes and re-renders.

---

## 3. Tech Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Desktop shell | Tauri v2 | Rust backend + web frontend, native tray, ~10MB binary, cross-platform |
| UI framework | React + TypeScript | Bundled in Tauri webview |
| Styling | Tailwind CSS | Utility-first, fast iteration |
| Components | shadcn/ui | High-quality, customizable, dark theme ready |
| Autostart | tauri-plugin-autostart | Launch on system login, all platforms |
| State management | React context + Tauri events | Simple, no external state library needed |

---

## 4. Crate Structure

```
plugmux/
├── Cargo.toml                          # Workspace root (add plugmux-app)
├── crates/
│   ├── plugmux-core/                   # Existing — shared library
│   ├── plugmux-cli/                    # Existing — headless binary
│   └── plugmux-app/
│       ├── src-tauri/                  # Tauri Rust crate (workspace member)
│       │   ├── Cargo.toml              # Binary crate, depends on plugmux-core
│       │   ├── src/
│       │   │   ├── main.rs             # Tauri entry point
│       │   │   ├── commands.rs         # Tauri command handlers (invoke from React)
│       │   │   ├── engine.rs           # Engine lifecycle (start/stop, wraps plugmux-core)
│       │   │   ├── tray.rs             # Tray icon setup, menu, health dot
│       │   │   └── events.rs           # Event bridge (engine events → Tauri events)
│       │   ├── tauri.conf.json
│       │   ├── icons/                  # App icons generated from tray SVG
│       │   └── capabilities/
│       │       └── default.json        # Tauri v2 permissions
│       ├── src/                        # React frontend
│       │   ├── App.tsx                 # Root component, router
│       │   ├── main.tsx                # Entry point
│       │   ├── components/
│       │   │   ├── layout/
│       │   │   │   ├── Sidebar.tsx     # Fixed sidebar navigation
│       │   │   │   └── Layout.tsx      # Sidebar + content wrapper
│       │   │   ├── servers/
│       │   │   │   ├── ServerCard.tsx  # Server row: icon, name, status, toggle
│       │   │   │   └── AddServerDialog.tsx
│       │   │   ├── environments/
│       │   │   │   ├── InheritedServers.tsx   # "From Main" section
│       │   │   │   ├── EnvironmentServers.tsx # Environment-specific servers
│       │   │   │   ├── PermissionsPanel.tsx   # Allow/Approve/Disable dropdowns
│       │   │   │   └── CreateEnvironmentDialog.tsx
│       │   │   └── shared/
│       │   │       ├── CopyButton.tsx  # One-click URL/text copy
│       │   │       └── StatusDot.tsx   # Health indicator dot
│       │   ├── pages/
│       │   │   ├── MainPage.tsx        # Base server management
│       │   │   ├── EnvironmentPage.tsx # Per-environment view
│       │   │   ├── CatalogPage.tsx     # Placeholder for Phase 3
│       │   │   ├── PresetsPage.tsx     # Placeholder for Phase 3
│       │   │   └── SettingsPage.tsx    # Port, autostart, appearance
│       │   ├── hooks/
│       │   │   ├── useEngine.ts        # Engine state (running/stopped)
│       │   │   ├── useConfig.ts        # Config state, CRUD operations via invoke
│       │   │   └── useEvents.ts        # Subscribe to Tauri events
│       │   ├── lib/
│       │   │   └── commands.ts         # Typed wrappers around Tauri invoke()
│       │   └── styles/
│       │       └── globals.css         # Tailwind base + shadcn theme
│       ├── index.html
│       ├── package.json
│       ├── tsconfig.json
│       ├── tailwind.config.ts
│       └── vite.config.ts
└── assets/
    └── icon-tray.svg                   # Tray icon source
```

---

## 5. Engine Lifecycle

### Startup Sequence

1. Tauri app launches (auto-start on login by default)
2. `engine::start()` is called:
   - Load config from `~/.config/plugmux/plugmux.json`
   - Attempt to bind TCP port (default 4242)
   - If port is busy → set engine state to "conflict", show conflict banner in UI with the occupied port number and a "Change Port" button that opens an inline port picker. After changing, engine retries automatically.
   - If port is free → start axum server, start all enabled servers
3. Tray icon appears with health status dot
4. Main window opens (or stays hidden if launched via autostart — tray only)

### Stop/Start Toggle

- UI has a toggle (like Tailscale) to stop/start the engine
- Stop: gracefully shut down all MCP servers, release port, update tray icon to grey
- Start: re-bind port, re-launch servers, update tray icon

### Shutdown

- Quit from tray menu → engine stops, app exits
- Close window → window hides, tray icon stays, engine keeps running

---

## 6. Tray Icon

### Icon

Monochrome SVG (`assets/icon-tray.svg`): three lines converging through a diamond node to one output line. Used as a macOS template image (adapts to light/dark menu bar).

### Health Status Dot

Small colored dot overlaid on the tray icon (like Docker Desktop, ESET):
- **Green** — all servers healthy
- **Yellow** — some servers degraded or unreachable
- **Grey** — engine stopped or offline mode

### Right-Click Menu

```
┌──────────────────────────────────┐
│  ● plugmux — Running            │  ← status + toggle (click to stop/start)
│──────────────────────────────────│
│  Environments:                  │
│    my-saas-app    📋            │  ← click to copy URL
│    rust-embedded  📋            │
│──────────────────────────────────│
│  Open plugmux                   │  ← open main window
│  Settings                       │
│──────────────────────────────────│
│  Quit                           │
└──────────────────────────────────┘
```

### Left-Click

Opens/focuses the main window.

---

## 7. Main Window

**Dimensions:** ~900x600 default, resizable.
**Theme:** Dark by default, light toggle in Settings.

### Sidebar (fixed, ~220px)

```
┌──────────────────┐
│  [icon] plugmux  │
│──────────────────│
│  Main            │
│──────────────────│
│  ENVIRONMENTS    │
│    my-saas-app   │  ← badge: server count
│    rust-embedded │
│  + New           │
│──────────────────│
│  Catalog         │  ← Phase 3 content
│  Presets         │  ← Phase 3 content
│──────────────────│
│  Settings        │
└──────────────────┘
```

Active item highlighted. Environment list shows server count badge.

---

## 8. Pages

### Main Page

Header: "Main" with subtitle "These servers are available in all environments."

Server list — each row:
- Server name (editable display name)
- Short description (one line, if provided)
- Connectivity badge: "local" (green) or "online" (blue)
- Health dot: green (running), red (error), grey (stopped)
- Enable/disable toggle switch

Actions:
- "Add Server" button → dialog with manual config (id, name, transport, command/url, connectivity)
- Remove server (icon button or context menu)

### Environment Page

Header: environment name (editable) + endpoint URL with copy button.

**From Main section:**
- Header: "Inherited Servers" with count
- Same server row format as Main
- Toggle here overrides the inherited server for this environment only
- Disabled overrides shown dimmed

**Environment Servers section:**
- Header: "Environment Servers" with count
- Servers specific to this environment
- Add/remove controls

**Permissions panel (collapsible):**
- Table with action name + dropdown (Allow / Approve / Disable)
- Actions: `enable_server`, `disable_server`

**Danger zone:** "Delete Environment" button with confirmation dialog.

### Catalog Page (Placeholder)

"Coming Soon" state with brief description: "Browse and install community MCP servers." Visually polished placeholder — not a blank page.

### Presets Page (Placeholder)

"Coming Soon" state with brief description: "Create environments from preset templates." Same treatment.

### Settings Page

- **Gateway:** Port number (default 4242), engine status indicator
- **Startup:** Auto-start on login toggle (default: on)
- **Appearance:** Dark / Light theme toggle
- **Config:** Config file path display, "Open in editor" button
- **About:** Version, license, links

---

## 9. Permission System

### Design Change from Parent Spec

The parent spec (Section 4) described approval as "user confirms via tray notification." This phase replaces that with in-conversation confirmation because: (a) the LLM agent already has a human-in-the-loop approval flow, (b) the user is already in the agent's conversation when the action is triggered, and (c) it removes OS notification complexity and works identically across all platforms and agents.

### How It Works

Each environment has permission settings for LLM-triggered actions. Three levels:

| Level | Behavior |
|-------|----------|
| **Allow** | Execute immediately, return result |
| **Approve** | Return `approval_required` response with `action_id`. LLM asks user for confirmation in its own conversation, then calls `confirm_action(action_id)` |
| **Disable** | Return error: "This action is not available in this environment" |

### Approval Flow

```
1. LLM calls enable_server("figma") via MCP
2. plugmux checks environment permissions → "Approve"
3. plugmux returns:
   {
     "status": "approval_required",
     "action_id": "a1b2c3",
     "message": "Enabling server 'figma' requires approval. Please confirm with the user."
   }
4. LLM reads response, asks user: "Should I enable the Figma server?"
5. User confirms
6. LLM calls confirm_action("a1b2c3")
7. plugmux executes the pending action, returns result
```

This is agent-agnostic — works with Claude, Cursor, Codex, any MCP client. No OS notifications, no modals. The approval happens in the agent's conversation where the user is already working.

### Fallback for Non-Cooperative LLMs

If an LLM ignores the `approval_required` response and retries the same action without confirming, plugmux returns the same `approval_required` response with the same `action_id`. It never auto-approves. The action stays pending until explicitly confirmed or it expires.

### New Gateway Tool

This phase extends the parent spec's 5-tool set with a 6th tool: `confirm_action`. The full set:

| Tool | Purpose |
|------|---------|
| `list_servers` | List healthy servers in environment |
| `get_tools` | Get tool schemas for a server |
| `execute` | Call a tool on a server |
| `enable_server` | Enable a server (may require approval) |
| `disable_server` | Disable a server (may require approval) |
| `confirm_action` | Confirm a pending approval |

### Pending Action Storage

Pending actions are held in an in-memory `HashMap<String, PendingAction>` inside the engine. They expire after 5 minutes and are lost on engine restart. Calling `confirm_action` with an expired or unknown `action_id` returns an error: `"action expired or not found — please retry the original action."`

---

## 10. Tauri Commands (Rust → React Bridge)

```rust
// Engine
get_engine_status() -> EngineStatus
start_engine() -> Result<()>
stop_engine() -> Result<()>

// Config
get_config() -> PlugmuxConfig
get_main_servers() -> Vec<ServerConfig>
add_main_server(config: ServerConfig) -> Result<()>
remove_main_server(id: String) -> Result<()>
toggle_main_server(id: String) -> Result<()>
rename_server(id: String, name: String) -> Result<()>

// Environments
list_environments() -> Vec<EnvironmentConfig>
create_environment(name: String) -> Result<EnvironmentConfig>
delete_environment(id: String) -> Result<()>
rename_environment(id: String, name: String) -> Result<()>
add_env_server(env_id: String, config: ServerConfig) -> Result<()>
remove_env_server(env_id: String, server_id: String) -> Result<()>
toggle_env_override(env_id: String, server_id: String) -> Result<()>

// Permissions
get_permissions(env_id: String) -> Permissions
set_permission(env_id: String, action: String, level: PermissionLevel) -> Result<()>

// Settings
get_settings() -> AppSettings
set_port(port: u16) -> Result<()>
set_autostart(enabled: bool) -> Result<()>
set_theme(theme: Theme) -> Result<()>
```

---

## 11. Tauri Events (Rust → React Push)

```
engine_status_changed      { status: "running" | "stopped" | "conflict" }
server_health_changed      { server_id, healthy: bool }
server_added               { server_id, env_id? }
server_removed             { server_id, env_id? }
server_toggled             { server_id, env_id?, enabled: bool }
environment_created        { env_id }
environment_deleted        { env_id }
config_reloaded            { }  // external file change detected, full reload
```

React subscribes via `useEvents()` hook. UI re-renders on each event.

---

## 12. Design Principles

- **Information density over whitespace** — show more at once, developers expect it
- **Status at a glance** — every server shows health, every environment shows count, tray shows overall
- **Copy-friendly** — endpoint URLs, server IDs, all one-click copyable
- **Keyboard navigable** — power users shouldn't need the mouse
- **Fast transitions** — no animations. Instant page switches. Toggles respond immediately.
- **Progressive disclosure** — permissions panel collapsed by default, advanced config behind edit actions
- **shadcn/ui heavily** — use shadcn components for every UI element: buttons, toggles, cards, dialogs, dropdowns, sidebar, badges

---

## 13. UI References

| App | What to Study | Why |
|-----|--------------|-----|
| **Docker Desktop** | Container list with status toggles, tray icon with dot | Closest model — managing running services |
| **Linear** | Sidebar nav, dark theme, typography | Best-in-class dev tool UI |
| **Tailscale menu bar** | Tray with status, start/stop toggle | Engine lifecycle UX |

---

## 14. Not Included in Phase 2

- Drag-and-drop server reordering
- Catalog content (placeholder UI only)
- Preset content (placeholder UI only)
- Cloud sync / accounts
- Unix socket IPC between CLI and Tauri
- Server installation from catalog
- `install_server` permission (deferred to Phase 3 with catalog functionality)

---

*plugmux Phase 2 Spec — Lasha Kvantaliani — March 2026 — Draft v1*
