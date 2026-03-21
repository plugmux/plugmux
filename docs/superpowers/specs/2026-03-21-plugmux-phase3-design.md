# plugmux Phase 3 — Catalog & Community Design Spec

**Author:** Lasha Kvantaliani
**Date:** 2026-03-21
**Status:** Draft v2
**Parent spec:** `docs/superpowers/specs/2026-03-20-plugmux-design.md`

**Deviations from parent spec:**
- "Main" replaced with "Default" (just another environment)
- Per-environment permissions replaced with global permissions
- Catalog is bundled static JSON, not cloud-backed (cloud deferred to Phase 4)
- Server names from catalog are not user-editable (standardized for docs/prompts; parent spec allowed renaming)
- Usage stats and likes deferred (require cloud backend)

---

## 1. Goal

Introduce a bundled MCP server catalog, environment presets, and community contribution workflow. Simultaneously refactor the config model to be simpler: no inheritance, no overrides, string-based server references, global permissions, and "Default" replacing "Main."

---

## 2. Config Model Redesign

### Current Model (Phase 1/2)

Single `plugmux.json` with `main.servers` (full config objects), environments that inherit from Main, per-environment overrides, and per-environment permissions.

### New Model

Two user files at `~/.config/plugmux/`:

**`config.json`** — environments, permissions, port:

```json
{
  "port": 4242,
  "permissions": {
    "enable_server": "approve",
    "disable_server": "approve"
  },
  "environments": [
    {
      "id": "default",
      "name": "Default",
      "servers": ["figma", "context7"]
    },
    {
      "id": "my-saas-app",
      "name": "My SaaS App",
      "servers": ["figma", "shadcn", "postgres", "internal-db"]
    }
  ]
}
```

**`custom_servers.json`** — user-defined manual servers:

```json
{
  "version": 1,
  "servers": [
    {
      "id": "internal-db",
      "name": "Internal DB",
      "transport": "stdio",
      "command": "node",
      "args": ["./mcp-server.js"],
      "connectivity": "local"
    }
  ]
}
```

### Key Changes from Phase 1/2

- **"Main" becomes "Default"** — just an environment pinned at the top of the sidebar, cannot be deleted
- **No inheritance** — each environment is standalone. If two environments need the same server, both list it. Presets handle bootstrapping.
- **No overrides** — if you don't want a server in an environment, don't include it
- **No per-environment permissions** — permissions are global, controlling what any LLM can do through the gateway
- **Servers are string IDs** — environments reference servers by ID, resolved against the catalog then custom_servers
- **Server names are not user-editable for catalog servers** — standardized names enable consistent documentation, tutorials, and prompts
- **Endpoint URLs computed at runtime** — `http://localhost:{port}/env/{id}`, not stored in config
- **Port persisted in config** — `port` field in `config.json` replaces the in-memory-only port from Phase 2
- **Separate files** — `config.json` for environments/settings, `custom_servers.json` for user-defined servers

### Server ID Resolution

When resolving a server ID:
1. Look up in bundled `catalog/servers.json` → found? use catalog config
2. Not in catalog → look up in `custom_servers.json` → found? use custom config
3. Not in either → log warning, mark as unavailable (red dot), don't crash

**ID collision rule:** Custom server IDs must not collide with catalog IDs. `add_custom_server` validates this and returns an error if the ID already exists in the catalog. This prevents ambiguity — catalog entries are canonical and cannot be overridden by custom servers.

### Default Environment Bootstrapping

- On first launch (no `config.json` exists), plugmux creates `config.json` with a single "default" environment containing no servers.
- On every config load, if no environment with `id: "default"` exists, plugmux auto-creates it with an empty server list and logs a warning.
- `delete_environment("default")` always returns an error: "The default environment cannot be deleted."
- If the user manually deletes the default environment from the JSON file, the config watcher detects the change and re-creates it.

### Migration from Phase 1/2

On first launch after upgrade, if `~/.config/plugmux/plugmux.json` (old format) exists and `config.json` does not:

1. Read old `plugmux.json`
2. Create "default" environment with the servers from `main.servers`
3. For each old environment: keep its servers list, drop overrides and per-environment permissions
4. Any server that matches a catalog ID by command/url → convert to string reference
5. Any server that doesn't match catalog → move to `custom_servers.json`, reference by ID
6. Write `config.json` and `custom_servers.json`
7. Rename `plugmux.json` → `plugmux.json.backup`
8. Log: "Config migrated from Phase 2 format. Backup saved as plugmux.json.backup"

CLI alternative: `plugmux config migrate` runs the same logic manually.

---

## 3. Catalog Data Model

Bundled with the app binary. Updated via software releases and community GitHub PRs.

### File Structure

```
catalog/
├── servers.json      # curated MCP server entries
├── presets.json      # environment preset templates
├── icons/            # one SVG per server/preset
│   ├── figma.svg
│   ├── github.svg
│   └── ...
└── CONTRIBUTING.md   # community submission guide
```

### Bundling

The `catalog/` directory is embedded into both binaries at compile time:
- **Tauri app:** included via Tauri's resource system (`tauri.conf.json` resources)
- **CLI:** included via `include_str!` or `include_bytes!` macros for the JSON files; icons are not needed in CLI mode

This means the catalog ships inside the binary — no external files to manage at runtime.

### `catalog/servers.json`

```json
{
  "version": 1,
  "servers": [
    {
      "id": "figma",
      "name": "Figma",
      "description": "Read and inspect Figma designs",
      "icon": "figma.svg",
      "category": "design",
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "@anthropic/figma-mcp"],
      "connectivity": "online"
    },
    {
      "id": "context7",
      "name": "Context7",
      "description": "Up-to-date documentation and code examples for any library",
      "icon": "context7.svg",
      "category": "dev-tools",
      "transport": "http",
      "url": "https://context7.dev/mcp",
      "connectivity": "online"
    }
  ]
}
```

### `catalog/presets.json`

**Note:** Preset content will be defined after deep research into the current MCP server landscape. The structure is ready but entries will be populated based on what's trending and useful.

```json
{
  "version": 1,
  "presets": [
    {
      "id": "web-dev",
      "name": "Web Development",
      "description": "Frontend and full-stack web development",
      "icon": "web-dev.svg",
      "servers": ["figma", "shadcn", "context7", "browser-tools"]
    }
  ]
}
```

### Categories

`design`, `dev-tools`, `database`, `browser`, `ai`, `productivity`, `testing`, `infrastructure`, `marketing`, `content`

### Icons

- One SVG file per server in `catalog/icons/`
- Monochrome style, consistent sizing (24x24 viewbox)
- Sourced from official project brand assets (not copied from other apps)
- Referenced by filename in the catalog entry's `icon` field

---

## 4. Server Health States

Every server displays a health indicator dot, consistent across all UI surfaces.

Server instances are **shared globally** — if "figma" appears in both "default" and "my-saas-app" environments, they share the same running MCP server process. The `ServerManager` manages one instance per server ID. Health status is therefore global per server ID, not per environment.

### Health Status Enum

```rust
enum HealthStatus {
    Healthy,                           // green — connected, running
    Degraded { reason: String },       // yellow — auth required, config needed, partial
    Unavailable { reason: String },    // red — not found, crashed, unreachable
}
```

| State | Dot Color | Meaning | Action |
|-------|-----------|---------|--------|
| Healthy | Green | Connected, running | None |
| Degraded | Yellow | Needs attention — auth required, config needed, partially working | Show details, prompt user |
| Unavailable | Red | Server not found, crashed, unreachable, ID not in catalog or custom | Offer "Remove & Re-add" |

Health dots appear on:
- Server rows in environment pages
- Server cards in catalog (for installed servers)
- Sidebar environment badges (aggregate: worst status of any server in that environment)
- Tray icon (aggregate: worst status across all environments)

---

## 5. LLM-Facing Gateway Tools — Updated

The Phase 1/2 gateway tools `enable_server` and `disable_server` toggled an `enabled` flag on servers. In the new model there is no toggle — a server is either in an environment's server list or it isn't.

### Redefined Tools

| Tool | Phase 1/2 Behavior | Phase 3 Behavior |
|------|--------------------|------------------|
| `list_servers` | List enabled+healthy servers | List servers in this environment (healthy ones) |
| `get_tools` | Get tool schemas for a server | No change |
| `execute` | Proxy call to upstream server | No change |
| `enable_server(server_id)` | Toggle enabled flag | **Add server to this environment** (server must exist in catalog or custom_servers) |
| `disable_server(server_id)` | Toggle enabled flag | **Remove server from this environment** |
| `confirm_action(action_id)` | Confirm pending approval | No change |

### Global Permission Check

When an LLM calls `enable_server` or `disable_server` through an environment endpoint:

1. Read the global `permissions` from `config.json`
2. Check the permission level for that action
3. If **allow** → execute immediately (add/remove server from environment)
4. If **approve** → return `approval_required` with `action_id`. The `PendingAction` stores the action type, server_id, and env_id. LLM calls `confirm_action(action_id)` after getting user consent.
5. If **disable** → return error: "This action is not available"

The `PendingAction` struct:
```rust
struct PendingAction {
    id: String,
    action: String,          // "enable_server" or "disable_server"
    server_id: String,
    env_id: String,          // derived from the endpoint the LLM connected to
    created_at: Instant,     // expires after 5 minutes
}
```

---

## 6. UI Changes

### Sidebar — Updated

- "Main" renamed to "Default" — pinned at top, cannot be deleted
- All other navigation unchanged

### Catalog Page — New (Replacing Placeholder)

- **Search bar** at top — case-insensitive substring match on name + description
- **Category filter pills** below search — "All" + each category
- **Server cards** in responsive grid:
  - Server icon (from `catalog/icons/`)
  - Server name
  - Short description (2 lines max)
  - Category badge
  - Health dot (if already installed in any environment)
  - "Add" button → dropdown: "Add to Default" / "Add to My SaaS App" / etc.
- **Clicking a card** → expanded detail view:
  - Full description
  - Config preview (command/url, transport, connectivity)
  - List of environments where it's already installed
  - "Add to..." button

### Presets Page — Minimal

Show preset cards from `catalog/presets.json`:
- Preset icon, name, description
- List of included servers (with icons)
- "Create Environment" button → prompts for environment name, creates environment with those servers

### Environment Page — Simplified

- **Header:** environment name (editable) + computed endpoint URL with copy button
- **Server list** — flat, no inheritance/overrides split:
  - Server icon (from catalog or default icon for custom)
  - Server name (from catalog — not editable; from custom_servers — editable)
  - Description
  - Health dot (green/yellow/red)
  - Remove button
- **"Add Server" button** → navigates to Catalog page, or opens quick-add dialog for custom servers
- **Danger zone:** "Delete Environment" button (hidden for "default")

### Settings Page — Updated

- **Gateway:** port (persisted to `config.json` on change), engine status, start/stop
- **Permissions:** global permission dropdowns (allow/approve/disable) for each gateway action (`enable_server`, `disable_server`)
- **Startup:** autostart toggle
- **Appearance:** dark/light theme
- **Custom Servers:** list of user-defined servers from `custom_servers.json`, with add/edit/remove
- **About:** version, license

### UI Design Brief (For Designer)

**App type:** Developer desktop tool (Tauri, macOS/Linux). Menu bar tray icon + main window.

**Target user:** Developer-tool power users. People who use Raycast, Linear, TablePlus daily. Comfortable with technical tools but want clean, efficient interfaces.

**Window:** ~900x600 default, resizable. Fixed sidebar (~220px) + content area.

**Theme:** Dark by default, light toggle. High information density, minimal whitespace.

**Health indicators:** Colored dots (green/yellow/red) are the primary status language. They appear on every server row, in sidebar badges, and on the tray icon. Must be instantly scannable.

**Catalog page:** Grid of cards — the visual centerpiece of Phase 3. Each card shows an icon, name, description, category badge, and an "Add" button. Should feel like browsing an app store but denser and more developer-focused. Reference: Setapp, Homebrew Cask, VS Code extensions.

**Design references:**
| App | What to Study |
|-----|--------------|
| Docker Desktop | Sidebar nav, container list with status dots, tray icon states |
| Linear | Sidebar, dark theme, typography, information density |
| VS Code Extensions | Card grid with icons, search + category filters, install buttons |
| Raycast | Clean minimal UI, search-first, quick actions |
| Tailscale menu bar | Tray icon with status dot, simple dropdown |

**Design principles:**
- Information density over whitespace
- Status at a glance (health dots everywhere)
- Copy-friendly URLs and IDs
- Keyboard navigable
- No unnecessary animations — instant transitions
- Progressive disclosure (advanced options behind edit actions)

---

## 7. Tauri Commands

### Removed (Phase 1/2 commands no longer needed)

- `get_main_servers`, `add_main_server`, `remove_main_server`, `toggle_main_server` — no more "Main"
- `rename_server` — catalog names are standardized
- `toggle_env_override` — no more overrides
- Per-environment `get_permissions`/`set_permission`

### New Command Set

```rust
// Config
get_config() -> Config
get_port() -> u16
set_port(port: u16) -> Result<()>          // persists to config.json

// Permissions (global)
get_permissions() -> Permissions
set_permission(action: String, level: String) -> Result<()>

// Environments
list_environments() -> Vec<Environment>
create_environment(name: String) -> Result<Environment>
delete_environment(id: String) -> Result<()>          // errors on "default"
rename_environment(id: String, name: String) -> Result<()>

// Servers in environments
add_server_to_env(env_id: String, server_id: String) -> Result<()>
remove_server_from_env(env_id: String, server_id: String) -> Result<()>

// Custom servers
list_custom_servers() -> Vec<ServerConfig>
add_custom_server(config: ServerConfig) -> Result<()>  // validates ID not in catalog
update_custom_server(id: String, config: ServerConfig) -> Result<()>
remove_custom_server(id: String) -> Result<()>

// Catalog (read-only, bundled)
list_catalog_servers() -> Vec<CatalogEntry>
search_catalog(query: String, category: Option<String>) -> Vec<CatalogEntry>
get_catalog_entry(id: String) -> CatalogEntry

// Presets (read-only, bundled)
list_presets() -> Vec<Preset>
create_env_from_preset(preset_id: String, name: String) -> Result<Environment>

// Health
get_server_health(server_id: String) -> HealthStatus

// Migration
migrate_config() -> Result<()>             // manual trigger for Phase 2 migration
```

### Events

```
engine_status_changed    { status: "running" | "stopped" | "conflict" }
server_health_changed    { server_id, status: "healthy" | "degraded" | "unavailable" }
server_added             { server_id, env_id }        // always has env_id (no longer optional)
server_removed           { server_id, env_id }        // always has env_id (no longer optional)
environment_created      { env_id }
environment_deleted      { env_id }
config_reloaded          { }
```

---

## 8. CLI Updates

### Removed Commands

- `plugmux server add/remove/list/toggle` at Main level — replaced by environment-specific commands

### New Command Set

```bash
# Gateway
plugmux start [--port PORT]
plugmux stop
plugmux status

# Environments
plugmux env list
plugmux env create <name>
plugmux env create <name> --preset web-dev
plugmux env delete <id>

# Servers (always within an environment)
plugmux server add <server-id> --env <env-id>
plugmux server remove <server-id> --env <env-id>
plugmux server list --env <env-id>

# Custom servers
plugmux custom add --id my-tool --name "My Tool" --transport stdio --command "node" --args "./server.js"
plugmux custom edit <id> [--name "New Name"] [--command "new-cmd"]
plugmux custom remove <id>
plugmux custom list

# Catalog (read-only)
plugmux catalog search "figma"
plugmux catalog browse --category design
plugmux catalog list

# Config
plugmux config path
plugmux config show
plugmux config migrate                    # manual Phase 2 migration
```

---

## 9. Community Contribution Workflow

### Submission Process

1. Fork the plugmux repo on GitHub
2. Add server entry to `catalog/servers.json`
3. Add monochrome SVG icon to `catalog/icons/` (24x24 viewbox)
4. Open PR with title: `catalog: add <server-name>`

### CI Validation on PR

- JSON schema validation (required fields, valid transport/connectivity/category values)
- No duplicate server IDs
- Referenced icon file exists in `catalog/icons/`
- SVG format check (valid SVG, reasonable file size)

### CONTRIBUTING.md

Covers:
- Required fields and their format
- Icon guidelines (monochrome SVG, 24x24, consistent style)
- Available categories
- Example entry with explanation
- Review expectations and timeline

### Review Process

Maintainer verifies:
- MCP server is real and functional
- Config (command/url) is correct
- Description is accurate and concise
- Icon meets style guidelines
- Merge → ships with next release

---

## 10. Refactoring Impact

### plugmux-core — Deleted

- `resolve_environment()` merge/override logic in `environment.rs`
- `MainConfig` struct
- `ServerOverride` struct
- Per-environment `Permission` struct

### plugmux-core — Refactored

- `config.rs` → new `Config` struct (port, permissions, environments), loads `config.json`. Port now persisted to disk.
- `server.rs` → `ServerConfig` remains similar, used for custom_servers
- `gateway/tools.rs` → `enable_server`/`disable_server` now add/remove servers from environment. Permission check reads global permissions, ignores env_id.
- `manager.rs` → uses resolved configs from new resolver. One server instance per ID globally (shared across environments).
- `pending_actions.rs` → `PendingAction` stores action type, server_id, and env_id. Approval flow unchanged except permissions are global.

### plugmux-core — New

- `catalog.rs` → `CatalogRegistry` loads bundled `servers.json` at startup, provides lookup by ID, search by query/category
- `custom_servers.rs` → loads `~/.config/plugmux/custom_servers.json`, provides CRUD operations
- `resolver.rs` → resolves server ID to full config (catalog first, then custom_servers). Validates no ID collisions.
- `migration.rs` → migrates Phase 2 `plugmux.json` to new `config.json` + `custom_servers.json` format

### plugmux-app (Tauri)

- `commands.rs` → rewritten for new command set
- `engine.rs` → updated to use new config + resolver
- Config watcher updated to watch both `config.json` and `custom_servers.json`

### plugmux-app (React)

- `MainPage.tsx` → removed (Default is just an environment)
- `EnvironmentPage.tsx` → simplified, no inheritance/overrides
- `CatalogPage.tsx` → full implementation replacing placeholder
- `PresetsPage.tsx` → minimal implementation
- `Sidebar.tsx` → "Main" becomes "Default"
- `hooks/useConfig.ts` → updated for new config shape
- `lib/commands.ts` → updated for new command set
- `components/environments/InheritedServers.tsx` → deleted
- `components/environments/PermissionsPanel.tsx` → moved to Settings page

### plugmux-cli

- Command handlers updated for new config model and command structure
- New `plugmux config migrate` command

---

## 11. Not Included in Phase 3

- Cloud sync / accounts (Phase 4)
- GitHub OAuth (Phase 4)
- User-created presets (future)
- Usage stats / likes on catalog entries (requires cloud backend)
- Drag-and-drop server reordering
- Database (SurrealDB evaluation deferred to Phase 4)
- Full preset catalog (pending MCP server landscape research)

---

*plugmux Phase 3 Spec — Lasha Kvantaliani — March 2026 — Draft v2*
