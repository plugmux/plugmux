# Phase 4: Agent Management — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move agent management into plugmux-core and build a comprehensive Agents page with auto-scan, enable/disable toggles, and setup wizard.

**Architecture:** New `agents/` module in plugmux-core following the catalog pattern (embedded JSON registry, inline tests). Agent detection/migration logic moves from Tauri commands to core. Frontend gets a redesigned Agents page replacing Dashboard.

**Tech Stack:** Rust (plugmux-core), Tauri v2, React 19, TypeScript, Tailwind CSS, shadcn/ui

**Spec:** `docs/superpowers/specs/2026-03-22-plugmux-phase4-agent-management-design.md`

---

## File Structure

### New Files

**Data:**
- `agents/agents.json` — bundled agent registry (25 agents, config paths per OS)

**Core (Rust):**
- `crates/plugmux-core/src/agents/mod.rs` — public API, re-exports
- `crates/plugmux-core/src/agents/registry.rs` — AgentRegistry, loads embedded agents.json
- `crates/plugmux-core/src/agents/detect.rs` — filesystem scanning, status detection
- `crates/plugmux-core/src/agents/migrate.rs` — surgical connect/disconnect, backup/restore
- `crates/plugmux-core/src/agents/state.rs` — AgentState persistence (agents_state.json)

**CLI:**
- `crates/plugmux-cli/src/commands/agents.rs` — agents subcommand

**Icons:**
- `assets/agent-icons/*.svg` — color + monochrome SVGs for all agents
- `crates/plugmux-app/src/assets/agent-icons/*.svg` — copies for Vite

**Frontend:**
- `crates/plugmux-app/src/pages/AgentsPage.tsx` — replaces DashboardPage
- `crates/plugmux-app/src/components/agents/AgentTable.tsx` — agent list with status dots, toggles
- `crates/plugmux-app/src/components/agents/AgentIcon.tsx` — icon with 2-letter fallback (replaces setup/AgentIcon.tsx)
- `crates/plugmux-app/src/components/agents/SetupDialog.tsx` — auto/manual choice → wizard (replaces setup/SetupDialog.tsx)
- `crates/plugmux-app/src/components/agents/AddAgentDialog.tsx` — registry + custom tabs
- `crates/plugmux-app/src/components/agents/DisableDialog.tsx` — disable vs disable+restore
- `crates/plugmux-app/src/hooks/useAgents.ts` — agent state hook

### Modified Files

- `crates/plugmux-core/src/lib.rs` — add `pub mod agents;`
- `crates/plugmux-core/Cargo.toml` — add `toml`, `chrono` deps
- `crates/plugmux-app/src-tauri/src/commands.rs` — replace agent commands with core wrappers
- `crates/plugmux-app/src-tauri/src/lib.rs` — register new commands
- `crates/plugmux-app/src/lib/commands.ts` — new TypeScript command bindings
- `crates/plugmux-app/src/components/layout/Sidebar.tsx` — rename Dashboard → Agents
- `crates/plugmux-app/src/App.tsx` — route agents page

### Deleted Files

- `crates/plugmux-app/src/pages/DashboardPage.tsx` — replaced by AgentsPage
- `crates/plugmux-app/src/components/setup/SetupDialog.tsx` — moved to agents/
- `crates/plugmux-app/src/components/setup/AgentIcon.tsx` — moved to agents/

---

## Task 1: Create agents.json registry data file

**Files:**
- Create: `agents/agents.json`

- [ ] **Step 1: Create the agents.json file with all 25 agents**

```json
{
  "version": 1,
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
    },
    {
      "id": "cursor",
      "name": "Cursor",
      "icon": "cursor",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/.cursor/mcp.json",
        "linux": "~/.cursor/mcp.json",
        "windows": "%USERPROFILE%\\.cursor\\mcp.json"
      }
    },
    {
      "id": "windsurf",
      "name": "Windsurf",
      "icon": "windsurf",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/.codeium/windsurf/mcp_config.json",
        "linux": "~/.codeium/windsurf/mcp_config.json",
        "windows": "%USERPROFILE%\\.codeium\\windsurf\\mcp_config.json"
      }
    },
    {
      "id": "codex",
      "name": "Codex",
      "icon": "codex",
      "config_format": "toml",
      "mcp_key": "mcp_servers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/.codex/config.toml",
        "linux": "~/.codex/config.toml",
        "windows": "%USERPROFILE%\\.codex\\config.toml"
      }
    },
    {
      "id": "gemini-cli",
      "name": "Gemini CLI",
      "icon": "gemini",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/.gemini/settings.json",
        "linux": "~/.gemini/settings.json",
        "windows": "%USERPROFILE%\\.gemini\\settings.json"
      }
    },
    {
      "id": "antigravity",
      "name": "Antigravity",
      "icon": "antigravity",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/.gemini/antigravity/mcp_config.json",
        "linux": "~/.gemini/antigravity/mcp_config.json",
        "windows": "%USERPROFILE%\\.gemini\\antigravity\\mcp_config.json"
      }
    },
    {
      "id": "vscode",
      "name": "VS Code",
      "icon": "copilot",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/.vscode/mcp.json",
        "linux": "~/.vscode/mcp.json",
        "windows": "%USERPROFILE%\\.vscode\\mcp.json"
      }
    },
    {
      "id": "copilot-cli",
      "name": "GitHub Copilot CLI",
      "icon": "githubcopilot",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/.copilot/mcp-config.json",
        "linux": "~/.copilot/mcp-config.json",
        "windows": "%USERPROFILE%\\.copilot\\mcp-config.json"
      }
    },
    {
      "id": "zed",
      "name": "Zed",
      "icon": null,
      "config_format": "json",
      "mcp_key": "context_servers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/.config/zed/settings.json",
        "linux": "~/.config/zed/settings.json",
        "windows": null
      }
    },
    {
      "id": "cline",
      "name": "Cline",
      "icon": "cline",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json",
        "linux": "~/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json",
        "windows": "%APPDATA%\\Code\\User\\globalStorage\\saoudrizwan.claude-dev\\settings\\cline_mcp_settings.json"
      }
    },
    {
      "id": "roocode",
      "name": "Roo Code",
      "icon": "roocode",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/Library/Application Support/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/mcp_settings.json",
        "linux": "~/.config/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/mcp_settings.json",
        "windows": "%APPDATA%\\Code\\User\\globalStorage\\rooveterinaryinc.roo-cline\\settings\\mcp_settings.json"
      }
    },
    {
      "id": "continue",
      "name": "Continue",
      "icon": null,
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/.continue/config.json",
        "linux": "~/.continue/config.json",
        "windows": "%USERPROFILE%\\.continue\\config.json"
      }
    },
    {
      "id": "goose",
      "name": "Goose",
      "icon": "goose",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/.config/goose/config.json",
        "linux": "~/.config/goose/config.json",
        "windows": "%APPDATA%\\goose\\config.json"
      }
    },
    {
      "id": "opencode",
      "name": "OpenCode",
      "icon": "opencode",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/.config/opencode/config.json",
        "linux": "~/.config/opencode/config.json",
        "windows": "%APPDATA%\\opencode\\config.json"
      }
    },
    {
      "id": "trae",
      "name": "Trae",
      "icon": "trae",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "auto",
      "config_paths": {
        "macos": "~/.trae/mcp.json",
        "linux": "~/.trae/mcp.json",
        "windows": "%USERPROFILE%\\.trae\\mcp.json"
      }
    },
    {
      "id": "cherrystudio",
      "name": "Cherry Studio",
      "icon": "cherrystudio",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "manual",
      "config_paths": null
    },
    {
      "id": "lmstudio",
      "name": "LM Studio",
      "icon": "lmstudio",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "manual",
      "config_paths": null
    },
    {
      "id": "openwebui",
      "name": "Open WebUI",
      "icon": "openwebui",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "manual",
      "config_paths": null
    },
    {
      "id": "n8n",
      "name": "n8n",
      "icon": "n8n",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "manual",
      "config_paths": null
    },
    {
      "id": "dify",
      "name": "Dify",
      "icon": "dify",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "manual",
      "config_paths": null
    },
    {
      "id": "coze",
      "name": "Coze",
      "icon": "coze",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "manual",
      "config_paths": null
    },
    {
      "id": "openhands",
      "name": "OpenHands",
      "icon": "openhands",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "manual",
      "config_paths": null
    },
    {
      "id": "poe",
      "name": "Poe",
      "icon": "poe",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "manual",
      "config_paths": null
    },
    {
      "id": "jetbrains",
      "name": "JetBrains",
      "icon": "junie",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "manual",
      "config_paths": null
    },
    {
      "id": "monica",
      "name": "Monica",
      "icon": "monica",
      "config_format": "json",
      "mcp_key": "mcpServers",
      "tier": "manual",
      "config_paths": null
    }
  ]
}
```

- [ ] **Step 2: Commit**

```bash
git add agents/agents.json
git commit -m "data: add agents.json registry with 25 MCP-compatible agents"
```

---

## Task 2: Download icon set from LobeHub

**Files:**
- Create: `assets/agent-icons/*.svg` (color + monochrome for all agents with icons)

- [ ] **Step 1: Download all available icons from lobehub/lobe-icons**

Download both monochrome and color variants for every agent that has a LobeHub icon. Use the GitHub API to fetch from `packages/static-svg/icons/`. Agents with icons: claudecode, cursor, windsurf, codex, gemini, antigravity, copilot, githubcopilot, cline, roocode, goose, opencode, trae, cherrystudio, lmstudio, openwebui, n8n, dify, coze, openhands, poe, junie, monica.

```bash
cd assets/agent-icons
# For each agent icon: curl both {name}.svg and {name}-color.svg
```

- [ ] **Step 2: Copy icons to app src for Vite**

```bash
cp -r assets/agent-icons crates/plugmux-app/src/assets/agent-icons
```

- [ ] **Step 3: Commit**

```bash
git add assets/agent-icons crates/plugmux-app/src/assets/agent-icons
git commit -m "assets: add color + monochrome icons for all registered agents"
```

---

## Task 3: Core — AgentRegistry (registry.rs)

**Files:**
- Create: `crates/plugmux-core/src/agents/mod.rs`
- Create: `crates/plugmux-core/src/agents/registry.rs`
- Modify: `crates/plugmux-core/src/lib.rs` — add `pub mod agents;`
- Modify: `crates/plugmux-core/Cargo.toml` — add `toml = "0.8"`, `chrono = "0.4"`

- [ ] **Step 1: Write failing test for registry loading**

In `registry.rs`, add a `#[cfg(test)]` module with test JSON and a test that `AgentRegistry::load()` parses it and returns the correct number of agents.

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p plugmux-core agents::registry
```
Expected: FAIL (module doesn't exist)

- [ ] **Step 3: Implement AgentRegistry structs and load()**

Define `AgentEntry`, `AgentData`, `AgentRegistry` structs with serde derives. Implement `load(json: &str)` and `load_bundled()` using `include_str!("../../../agents/agents.json")`. Add methods: `list_agents()`, `list_auto_agents()`, `get_agent(id)`, `resolve_config_path(agent)` (expand `~` via `dirs::home_dir()`).

- [ ] **Step 4: Add `pub mod agents;` to lib.rs and deps to Cargo.toml**

- [ ] **Step 5: Run test to verify it passes**

```bash
cargo test -p plugmux-core agents::registry
```
Expected: PASS

- [ ] **Step 6: Write tests for path resolution, auto filtering, get_agent**

- [ ] **Step 7: Run all tests**

```bash
cargo test -p plugmux-core
```
Expected: All PASS

- [ ] **Step 8: Commit**

```bash
git commit -m "feat(core): add agents registry module with bundled agents.json"
```

---

## Task 4: Core — AgentState (state.rs)

**Files:**
- Create: `crates/plugmux-core/src/agents/state.rs`
- Modify: `crates/plugmux-core/src/agents/mod.rs` — add module

- [ ] **Step 1: Write failing test for state load/save**

Test that `AgentState::load()` returns empty state when file doesn't exist, and that `save()` + `load()` round-trips correctly.

- [ ] **Step 2: Run test — expect FAIL**

- [ ] **Step 3: Implement AgentState**

Define `AgentStateEntry` (id, source, optional name/config_path/config_format/mcp_key) and `AgentState` (agents vec, dismissed_agents vec). Implement `load(dir)`, `save(dir)`, `add_agent()`, `dismiss_agent()`, `is_dismissed()`, `get_agent()`.

- [ ] **Step 4: Run test — expect PASS**

- [ ] **Step 5: Write tests for dismiss, add custom, is_dismissed**

- [ ] **Step 6: Run all tests — expect PASS**

- [ ] **Step 7: Commit**

```bash
git commit -m "feat(core): add agent state persistence (agents_state.json)"
```

---

## Task 5: Core — Agent detection (detect.rs)

**Files:**
- Create: `crates/plugmux-core/src/agents/detect.rs`
- Modify: `crates/plugmux-core/src/agents/mod.rs` — add module

- [ ] **Step 1: Write failing test for detect_agent status logic**

Create temp directories with mock config files. Test that:
- Missing file → Gray status
- File exists, no plugmux key → Gray
- File exists, only plugmux → Green
- File exists, plugmux + others → Yellow

- [ ] **Step 2: Run test — expect FAIL**

- [ ] **Step 3: Implement detect_agent and detect_all**

Define `AgentStatus` enum (Green, Yellow, Gray) and `DetectedAgent` struct. `detect_agent()` reads the config file, checks for plugmux key in the mcp section. `detect_all()` iterates auto-tier agents, merges with state, excludes dismissed.

Support both JSON (`mcpServers`) and TOML (`mcp_servers`) formats based on `config_format`.

- [ ] **Step 4: Run test — expect PASS**

- [ ] **Step 5: Write tests for TOML detection, detect_all with dismissed agents**

- [ ] **Step 6: Run all tests — expect PASS**

- [ ] **Step 7: Commit**

```bash
git commit -m "feat(core): add agent detection with green/yellow/gray status"
```

---

## Task 6: Core — Agent migration (migrate.rs)

**Files:**
- Create: `crates/plugmux-core/src/agents/migrate.rs`
- Modify: `crates/plugmux-core/src/agents/mod.rs` — add module

- [ ] **Step 1: Write failing test for connect_agent (JSON)**

Create a temp config file with existing mcpServers. Call `connect_agent()`. Assert:
- plugmux key added alongside existing entries
- Backup file created with original mcpServers
- Other config keys preserved

- [ ] **Step 2: Run test — expect FAIL**

- [ ] **Step 3: Implement connect_agent for JSON**

Read config, extract mcpServers, save backup (only if first time — check backup doesn't exist), insert `"plugmux": {"url": "..."}` into mcpServers, write back config preserving all other keys.

- [ ] **Step 4: Run test — expect PASS**

- [ ] **Step 5: Write test for disconnect_agent (surgical removal)**

Call `connect_agent()` then `disconnect_agent()`. Assert plugmux key removed, other MCPs remain, backup still exists.

- [ ] **Step 6: Implement disconnect_agent**

Read config, remove `"plugmux"` key from mcpServers, write back.

- [ ] **Step 7: Run test — expect PASS**

- [ ] **Step 8: Write test for disconnect_and_restore**

Call `connect_agent()` then `disconnect_and_restore()`. Assert mcpServers replaced with backup contents, backup file deleted.

- [ ] **Step 9: Implement disconnect_and_restore**

Read backup file, replace mcpServers with backup contents, remove plugmux key, write config, delete backup file.

- [ ] **Step 10: Write tests for TOML connect/disconnect (Codex)**

Same logic but for TOML format with `mcp_servers` key.

- [ ] **Step 11: Implement TOML support in connect/disconnect**

- [ ] **Step 12: Write test for get_backup_path**

- [ ] **Step 13: Run all tests — expect PASS**

```bash
cargo test -p plugmux-core agents::migrate
```

- [ ] **Step 14: Commit**

```bash
git commit -m "feat(core): add agent connect/disconnect/restore with backup"
```

---

## Task 7: Wire core module into Tauri commands

**Files:**
- Modify: `crates/plugmux-app/src-tauri/src/commands.rs` — replace agent code with core wrappers
- Modify: `crates/plugmux-app/src-tauri/src/lib.rs` — register new commands
- Modify: `crates/plugmux-app/src-tauri/Cargo.toml` — remove `toml`, `chrono` (now in core)

- [ ] **Step 1: Remove old agent detection/migration code from commands.rs**

Delete: `DetectedAgent`, `DiscoveredMcpServer`, `MigrateResult`, `agent_config_paths()`, `agent_has_plugmux_json/toml()`, `read_mcp_servers_json/toml()`, `detect_transport()`, `plugmux_mcp_entry()`, `mcp_backup_path()`, `migrate_json_config()`, `migrate_toml_config()`, `backup_filename()`, `home_dir()`, and all related `#[tauri::command]` functions.

- [ ] **Step 2: Add new thin-wrapper commands**

New commands calling into `plugmux_core::agents`:
- `get_agent_registry()` — returns all agents from registry
- `detect_agents()` — calls `detect_all()` from core
- `connect_agent(id)` — calls `migrate::connect_agent()` from core
- `disconnect_agent(id, restore: bool)` — calls disconnect or disconnect_and_restore
- `add_agent_from_registry(id, config_path)` — adds to state
- `add_custom_agent(name, config_path, format, mcp_key)` — adds to state
- `dismiss_agent(id)` — adds to dismissed list

- [ ] **Step 3: Register commands in lib.rs**

- [ ] **Step 4: Verify Rust compiles**

```bash
cargo check --manifest-path crates/plugmux-app/src-tauri/Cargo.toml
```

- [ ] **Step 5: Commit**

```bash
git commit -m "refactor(app): replace agent commands with plugmux-core wrappers"
```

---

## Task 8: Update TypeScript command bindings

**Files:**
- Modify: `crates/plugmux-app/src/lib/commands.ts`

- [ ] **Step 1: Update types and command functions**

Replace old `DetectedAgent`, `DiscoveredMcpServer`, `MigrateResult` with new types matching core:
- `AgentEntry` — registry agent
- `DetectedAgent` — with status: "green" | "yellow" | "gray"
- Add commands: `getAgentRegistry()`, `connectAgent(id)`, `disconnectAgent(id, restore)`, `addAgentFromRegistry(id, configPath)`, `addCustomAgent(...)`, `dismissAgent(id)`

- [ ] **Step 2: Commit**

```bash
git commit -m "refactor(app): update TypeScript agent command bindings"
```

---

## Task 9: Create useAgents hook

**Files:**
- Create: `crates/plugmux-app/src/hooks/useAgents.ts`

- [ ] **Step 1: Implement useAgents hook**

Hook that calls `detectAgents()` on mount and provides: `agents`, `loading`, `reload()`, `connect(id)`, `disconnect(id, restore)`, `dismiss(id)`. Re-fetches agent list after mutations.

- [ ] **Step 2: Commit**

```bash
git commit -m "feat(app): add useAgents hook"
```

---

## Task 10: Build AgentIcon component with 2-letter fallback

**Files:**
- Create: `crates/plugmux-app/src/components/agents/AgentIcon.tsx`
- Delete: `crates/plugmux-app/src/components/setup/AgentIcon.tsx`

- [ ] **Step 1: Implement AgentIcon**

Import all color SVGs. Map agent ID → icon src. If no icon found, render a colored circle with first 2 letters of agent name. Color derived from a simple hash of the agent name for consistency.

- [ ] **Step 2: Commit**

```bash
git commit -m "feat(app): add AgentIcon with 2-letter thumbnail fallback"
```

---

## Task 11: Build AgentsPage with agent table

**Files:**
- Create: `crates/plugmux-app/src/pages/AgentsPage.tsx`
- Create: `crates/plugmux-app/src/components/agents/AgentTable.tsx`
- Create: `crates/plugmux-app/src/components/agents/DisableDialog.tsx`
- Delete: `crates/plugmux-app/src/pages/DashboardPage.tsx`

- [ ] **Step 1: Implement AgentTable**

Table rows with: status dot (green/yellow/gray circle), AgentIcon, agent name, config path (truncated, muted), enable/disable toggle, delete button (trash icon).

Toggle ON → calls `connect(id)`. Toggle OFF → opens DisableDialog.

- [ ] **Step 2: Implement DisableDialog**

Confirmation dialog: "Disable plugmux for [Agent Name]". Two buttons:
- "Disable" — calls `disconnect(id, false)`
- "Disable & Restore" — calls `disconnect(id, true)` — only shown if backup exists
After confirm: shows "Restart [Agent Name] to apply changes."

- [ ] **Step 3: Implement AgentsPage**

Uses `useAgents` hook. Shows:
- Banner (if no connected agents) with "Setup" button
- "Add agent" button top-right
- AgentTable with all detected agents

- [ ] **Step 4: Commit**

```bash
git commit -m "feat(app): add AgentsPage with agent table and disable dialog"
```

---

## Task 12: Build SetupDialog (auto + manual paths)

**Files:**
- Create: `crates/plugmux-app/src/components/agents/SetupDialog.tsx`
- Delete: `crates/plugmux-app/src/components/setup/SetupDialog.tsx`

- [ ] **Step 1: Implement Step 1 — Choose method**

Two cards: "Auto Connect (Recommended)" and "Manual Setup (Advanced)" with descriptions.

- [ ] **Step 2: Implement Auto path (Step 2a)**

Scan agents → checkboxes (pre-selected installed agents) → "Connect" button → connect selected agents → show results.

- [ ] **Step 3: Implement Manual path (Step 2b)**

JSON snippet with copy button (dynamic port from config). Table of known agents with config file paths. "Validate" button that re-scans and shows green checkmarks.

- [ ] **Step 4: Commit**

```bash
git commit -m "feat(app): add setup dialog with auto and manual paths"
```

---

## Task 13: Build AddAgentDialog (registry + custom tabs)

**Files:**
- Create: `crates/plugmux-app/src/components/agents/AddAgentDialog.tsx`

- [ ] **Step 1: Implement Agents tab**

List of registered agents not already in user's list, with icons. Click one → prompt for config file path → add to state as gray.

- [ ] **Step 2: Implement Custom tab**

Form: name, config file path (with file picker), config format dropdown (JSON/TOML), MCP key field (default "mcpServers"). Add button.

- [ ] **Step 3: Commit**

```bash
git commit -m "feat(app): add AddAgentDialog with registry and custom tabs"
```

---

## Task 14: Wire up routing and sidebar

**Files:**
- Modify: `crates/plugmux-app/src/App.tsx` — replace dashboard route with agents
- Modify: `crates/plugmux-app/src/components/layout/Sidebar.tsx` — rename Dashboard → Agents

- [ ] **Step 1: Update Sidebar**

Change "Dashboard" to "Agents", icon from `LayoutDashboard` to `Cable`. Route to `"agents"` page.

- [ ] **Step 2: Update App.tsx**

Replace `DashboardPage` import with `AgentsPage`. Change default page from `"dashboard"` to `"agents"`. Update route case.

- [ ] **Step 3: Remove old files**

Delete `DashboardPage.tsx`, `setup/SetupDialog.tsx`, `setup/AgentIcon.tsx`.

- [ ] **Step 4: Verify app compiles and runs**

```bash
cd crates/plugmux-app && npm run tauri dev
```

- [ ] **Step 5: Commit**

```bash
git commit -m "feat(app): wire AgentsPage, rename Dashboard to Agents in sidebar"
```

---

## Task 15: CLI agents subcommand

**Files:**
- Create: `crates/plugmux-cli/src/commands/agents.rs`
- Modify: `crates/plugmux-cli/src/commands/mod.rs` — add `pub mod agents;`
- Modify: `crates/plugmux-cli/src/main.rs` — add Agents variant

- [ ] **Step 1: Implement AgentCommands enum**

Subcommands: `List`, `Connect { id: Option<String>, all: bool }`, `Disconnect { id: String, restore: bool }`, `Status`.

- [ ] **Step 2: Implement run() function**

- `List` → `AgentRegistry::load_bundled()` + `detect_all()`, print table
- `Connect` → `connect_agent()` per agent, print results
- `Disconnect` → `disconnect_agent()` or `disconnect_and_restore()`
- `Status` → same as list but focus on connected agents

- [ ] **Step 3: Register in main.rs**

Add `Agents { #[command(subcommand)] command: commands::agents::AgentCommands }` to Commands enum and match arm.

- [ ] **Step 4: Verify CLI compiles**

```bash
cargo build -p plugmux-cli
```

- [ ] **Step 5: Commit**

```bash
git commit -m "feat(cli): add agents subcommand (list, connect, disconnect, status)"
```

---

## Task 16: Final integration test and cleanup

- [ ] **Step 1: Run all tests**

```bash
cargo test --workspace
```

- [ ] **Step 2: Run the app and verify full flow**

1. Open app → Agents page shows auto-scanned agents
2. Click Setup → choose Auto → connect all → agents go green/yellow
3. Toggle an agent OFF → disable dialog → disable → goes gray
4. Click Add Agent → pick from registry → provide path → appears gray
5. Toggle it ON → goes green

- [ ] **Step 3: Final commit**

```bash
git commit -m "feat: complete Phase 4 Agent Management"
```
