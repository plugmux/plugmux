# plugmux Phase 2 — Tauri Desktop App Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri v2 desktop app that wraps `plugmux-core`, providing a tray icon with health status, a full management UI (servers, environments, settings), and an in-conversation permission approval system.

**Architecture:** The Tauri app embeds `plugmux-core` in-process. Rust Tauri commands expose engine operations to a React frontend via `invoke()`. State changes push to React via Tauri events. The app auto-starts the MCP gateway on launch and shows a tray icon with health status. shadcn/ui provides all UI components.

**Tech Stack:** Tauri v2, React 19, TypeScript, Vite, Tailwind CSS, shadcn/ui, tauri-plugin-autostart, `notify` crate (file watching)

**Spec:** `docs/superpowers/specs/2026-03-20-plugmux-phase2-design.md`

---

## File Structure

```
plugmux/
├── Cargo.toml                                  # Workspace root (add src-tauri member)
├── crates/
│   ├── plugmux-core/                           # Existing — shared library
│   │   └── src/
│   │       ├── lib.rs                          # Add: re-export new pending_actions module
│   │       └── pending_actions.rs              # NEW: approval flow + confirm_action storage
│   ├── plugmux-cli/                            # Existing — headless binary
│   └── plugmux-app/
│       ├── src-tauri/                          # Tauri Rust binary crate
│       │   ├── Cargo.toml
│       │   ├── build.rs
│       │   ├── tauri.conf.json
│       │   ├── capabilities/
│       │   │   └── default.json
│       │   ├── icons/                          # Generated from tray SVG
│       │   └── src/
│       │       ├── main.rs                     # Desktop entry → lib::run()
│       │       ├── lib.rs                      # Tauri builder setup, plugin registration
│       │       ├── engine.rs                   # Engine lifecycle (start/stop, wraps plugmux-core)
│       │       ├── commands.rs                 # All #[tauri::command] handlers
│       │       ├── tray.rs                     # Tray icon + menu + health dot
│       │       ├── events.rs                   # Event definitions + emit helpers
│       │       └── watcher.rs                  # Config file watcher (notify crate)
│       ├── src/                                # React frontend
│       │   ├── main.tsx                        # Entry point, renders <App />
│       │   ├── App.tsx                         # Router + Layout wrapper
│       │   ├── components/
│       │   │   ├── layout/
│       │   │   │   ├── Sidebar.tsx             # Fixed sidebar with nav items
│       │   │   │   └── Layout.tsx              # Sidebar + content area wrapper
│       │   │   ├── servers/
│       │   │   │   ├── ServerCard.tsx           # Server row: name, status, toggle
│       │   │   │   └── AddServerDialog.tsx      # Dialog for manual server config
│       │   │   ├── environments/
│       │   │   │   ├── InheritedServers.tsx     # "From Main" server list with overrides
│       │   │   │   ├── EnvironmentServers.tsx   # Env-specific server list
│       │   │   │   ├── PermissionsPanel.tsx     # Allow/Approve/Disable dropdowns
│       │   │   │   └── CreateEnvironmentDialog.tsx
│       │   │   └── ui/                         # shadcn/ui generated components
│       │   ├── pages/
│       │   │   ├── MainPage.tsx                # Base server management
│       │   │   ├── EnvironmentPage.tsx         # Per-environment view
│       │   │   ├── CatalogPage.tsx             # Placeholder
│       │   │   ├── PresetsPage.tsx             # Placeholder
│       │   │   └── SettingsPage.tsx            # Port, autostart, theme
│       │   ├── hooks/
│       │   │   ├── useEngine.ts                # Engine status state
│       │   │   ├── useConfig.ts                # Config CRUD via invoke
│       │   │   └── useEvents.ts                # Tauri event subscriptions
│       │   ├── lib/
│       │   │   └── commands.ts                 # Typed invoke() wrappers
│       │   └── styles/
│       │       └── globals.css                 # Tailwind base + shadcn theme vars
│       ├── index.html
│       ├── package.json
│       ├── tsconfig.json
│       ├── vite.config.ts
│       └── tailwind.config.ts
└── assets/
    └── icon-tray.svg                           # Existing tray icon
```

---

## Task 1: Scaffold Tauri v2 + React + TypeScript Project

**Files:**
- Create: `crates/plugmux-app/src-tauri/Cargo.toml`
- Create: `crates/plugmux-app/src-tauri/build.rs`
- Create: `crates/plugmux-app/src-tauri/tauri.conf.json`
- Create: `crates/plugmux-app/src-tauri/capabilities/default.json`
- Create: `crates/plugmux-app/src-tauri/src/main.rs`
- Create: `crates/plugmux-app/src-tauri/src/lib.rs`
- Create: `crates/plugmux-app/package.json`
- Create: `crates/plugmux-app/index.html`
- Create: `crates/plugmux-app/src/main.tsx`
- Create: `crates/plugmux-app/src/App.tsx`
- Create: `crates/plugmux-app/vite.config.ts`
- Create: `crates/plugmux-app/tsconfig.json`
- Modify: `plugmux/Cargo.toml` (add workspace member)

- [ ] **Step 1: Create the Tauri Rust crate**

`crates/plugmux-app/src-tauri/Cargo.toml`:
```toml
[package]
name = "plugmux-app"
version.workspace = true
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
plugmux-core = { path = "../../plugmux-core" }
tauri = { version = "2", features = ["tray-icon", "image-png"] }
tauri-plugin-autostart = "2"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
notify = "7"
uuid = { version = "1", features = ["v4"] }
```

`crates/plugmux-app/src-tauri/build.rs`:
```rust
fn main() {
    tauri_build::build()
}
```

`crates/plugmux-app/src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    plugmux_app::run()
}
```

`crates/plugmux-app/src-tauri/src/lib.rs`:
```rust
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running plugmux");
}
```

- [ ] **Step 2: Create tauri.conf.json**

`crates/plugmux-app/src-tauri/tauri.conf.json`:
```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/v2/crates/tauri-cli/schema.json",
  "productName": "plugmux",
  "version": "0.1.0",
  "identifier": "com.plugmux.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "plugmux",
        "width": 900,
        "height": 600,
        "resizable": true,
        "minWidth": 700,
        "minHeight": 400
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

- [ ] **Step 3: Create capabilities/default.json**

`crates/plugmux-app/src-tauri/capabilities/default.json`:
```json
{
  "identifier": "default",
  "description": "Default capabilities for plugmux",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "autostart:allow-enable",
    "autostart:allow-disable",
    "autostart:allow-is-enabled"
  ]
}
```

- [ ] **Step 4: Add workspace member**

In root `plugmux/Cargo.toml`, change:
```toml
members = ["crates/plugmux-core", "crates/plugmux-cli"]
```
to:
```toml
members = ["crates/plugmux-core", "crates/plugmux-cli", "crates/plugmux-app/src-tauri"]
```

- [ ] **Step 5: Create React frontend scaffold**

`crates/plugmux-app/package.json`:
```json
{
  "name": "plugmux-app",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-autostart": "^2",
    "react": "^19",
    "react-dom": "^19",
    "lucide-react": "^0.460",
    "clsx": "^2",
    "tailwind-merge": "^2"
  },
  "devDependencies": {
    "@types/react": "^19",
    "@types/react-dom": "^19",
    "@tauri-apps/cli": "^2",
    "@vitejs/plugin-react": "^4",
    "autoprefixer": "^10",
    "postcss": "^8",
    "tailwindcss": "^3",
    "typescript": "^5",
    "vite": "^6"
  }
}
```

`crates/plugmux-app/vite.config.ts`:
```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
});
```

`crates/plugmux-app/tsconfig.json`:
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src"]
}
```

`crates/plugmux-app/index.html`:
```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>plugmux</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

`crates/plugmux-app/src/main.tsx`:
```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/globals.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

`crates/plugmux-app/src/App.tsx`:
```tsx
function App() {
  return (
    <div className="h-screen bg-background text-foreground">
      <p className="p-4">plugmux is loading...</p>
    </div>
  );
}

export default App;
```

- [ ] **Step 6: Install npm dependencies and verify build**

Run:
```bash
cd crates/plugmux-app && npm install
```

- [ ] **Step 7: Verify Rust compilation**

Run:
```bash
cd plugmux && cargo check -p plugmux-app
```
Expected: compiles without errors.

- [ ] **Step 8: Commit**

```bash
git add crates/plugmux-app/ Cargo.toml
git commit -m "feat(phase2): scaffold Tauri v2 + React + TypeScript project"
```

---

## Task 2: Tailwind CSS + shadcn/ui Setup

**Files:**
- Create: `crates/plugmux-app/src/styles/globals.css`
- Create: `crates/plugmux-app/tailwind.config.ts`
- Create: `crates/plugmux-app/postcss.config.js`
- Create: `crates/plugmux-app/src/lib/utils.ts`
- Create: `crates/plugmux-app/components.json`

- [ ] **Step 1: Create Tailwind config**

`crates/plugmux-app/tailwind.config.ts`:
```typescript
import type { Config } from "tailwindcss";

const config: Config = {
  darkMode: "class",
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
    },
  },
  plugins: [],
};

export default config;
```

`crates/plugmux-app/postcss.config.js`:
```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
};
```

- [ ] **Step 2: Create globals.css with dark theme**

`crates/plugmux-app/src/styles/globals.css`:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 240 10% 3.9%;
    --card: 0 0% 100%;
    --card-foreground: 240 10% 3.9%;
    --popover: 0 0% 100%;
    --popover-foreground: 240 10% 3.9%;
    --primary: 240 5.9% 10%;
    --primary-foreground: 0 0% 98%;
    --secondary: 240 4.8% 95.9%;
    --secondary-foreground: 240 5.9% 10%;
    --muted: 240 4.8% 95.9%;
    --muted-foreground: 240 3.8% 46.1%;
    --accent: 240 4.8% 95.9%;
    --accent-foreground: 240 5.9% 10%;
    --destructive: 0 84.2% 60.2%;
    --destructive-foreground: 0 0% 98%;
    --border: 240 5.9% 90%;
    --input: 240 5.9% 90%;
    --ring: 240 5.9% 10%;
    --radius: 0.5rem;
  }

  .dark {
    --background: 240 10% 3.9%;
    --foreground: 0 0% 98%;
    --card: 240 10% 3.9%;
    --card-foreground: 0 0% 98%;
    --popover: 240 10% 3.9%;
    --popover-foreground: 0 0% 98%;
    --primary: 0 0% 98%;
    --primary-foreground: 240 5.9% 10%;
    --secondary: 240 3.7% 15.9%;
    --secondary-foreground: 0 0% 98%;
    --muted: 240 3.7% 15.9%;
    --muted-foreground: 240 5% 64.9%;
    --accent: 240 3.7% 15.9%;
    --accent-foreground: 0 0% 98%;
    --destructive: 0 62.8% 30.6%;
    --destructive-foreground: 0 0% 98%;
    --border: 240 3.7% 15.9%;
    --input: 240 3.7% 15.9%;
    --ring: 240 4.9% 83.9%;
  }
}

@layer base {
  * {
    @apply border-border;
  }
  body {
    @apply bg-background text-foreground;
  }
}
```

- [ ] **Step 3: Create utils.ts for shadcn**

`crates/plugmux-app/src/lib/utils.ts`:
```typescript
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
```

- [ ] **Step 4: Create components.json for shadcn CLI**

`crates/plugmux-app/components.json`:
```json
{
  "$schema": "https://ui.shadcn.com/schema.json",
  "style": "default",
  "rsc": false,
  "tsx": true,
  "tailwind": {
    "config": "tailwind.config.ts",
    "css": "src/styles/globals.css",
    "baseColor": "zinc",
    "cssVariables": true
  },
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/utils",
    "ui": "@/components/ui"
  }
}
```

- [ ] **Step 5: Install shadcn/ui components**

Run from `crates/plugmux-app/`:
```bash
npx shadcn@latest add button switch badge dialog dropdown-menu separator input label select card scroll-area tooltip
```

- [ ] **Step 6: Set dark mode on body in index.html**

Update `crates/plugmux-app/index.html` body tag:
```html
<body class="dark">
```

- [ ] **Step 7: Verify Tailwind + shadcn renders**

Update `crates/plugmux-app/src/App.tsx`:
```tsx
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

function App() {
  return (
    <div className="h-screen bg-background text-foreground flex items-center justify-center gap-4">
      <Button>plugmux</Button>
      <Badge variant="secondary">v0.1.0</Badge>
    </div>
  );
}

export default App;
```

Run:
```bash
cd crates/plugmux-app && npm run dev
```
Expected: dark background, styled button and badge visible at `http://localhost:1420`.

- [ ] **Step 8: Commit**

```bash
git add crates/plugmux-app/
git commit -m "feat(phase2): add Tailwind CSS + shadcn/ui with dark theme"
```

---

## Task 3: Engine Module (Rust)

**Files:**
- Create: `crates/plugmux-app/src-tauri/src/engine.rs`
- Create: `crates/plugmux-app/src-tauri/src/events.rs`
- Modify: `crates/plugmux-app/src-tauri/src/lib.rs`

- [ ] **Step 1: Create event definitions**

`crates/plugmux-app/src-tauri/src/events.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatusPayload {
    pub status: String, // "running", "stopped", "conflict"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHealthPayload {
    pub server_id: String,
    pub healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerChangedPayload {
    pub server_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerToggledPayload {
    pub server_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_id: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentChangedPayload {
    pub env_id: String,
}

// Event name constants
pub const ENGINE_STATUS_CHANGED: &str = "engine_status_changed";
pub const SERVER_HEALTH_CHANGED: &str = "server_health_changed";
pub const SERVER_ADDED: &str = "server_added";
pub const SERVER_REMOVED: &str = "server_removed";
pub const SERVER_TOGGLED: &str = "server_toggled";
pub const ENVIRONMENT_CREATED: &str = "environment_created";
pub const ENVIRONMENT_DELETED: &str = "environment_deleted";
pub const CONFIG_RELOADED: &str = "config_reloaded";
```

- [ ] **Step 2: Create engine module**

`crates/plugmux-app/src-tauri/src/engine.rs`:
```rust
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tracing::{error, info};

use plugmux_core::config::{self, PlugmuxConfig};
use plugmux_core::environment::resolve_named;
use plugmux_core::gateway::router;
use plugmux_core::health::start_health_checker;
use plugmux_core::manager::ServerManager;

/// Represents the current state of the engine.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineStatus {
    Stopped,
    Running,
    Conflict,
}

impl EngineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Running => "running",
            Self::Conflict => "conflict",
        }
    }
}

/// Holds all engine runtime state.
pub struct Engine {
    pub config: Arc<RwLock<PlugmuxConfig>>,
    pub manager: Arc<ServerManager>,
    pub status: Arc<RwLock<EngineStatus>>,
    pub port: Arc<RwLock<u16>>,
    shutdown_tx: Arc<RwLock<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl Engine {
    pub fn new() -> Self {
        let cfg = config::load_or_default(&config::config_path()).unwrap_or_default();
        Self {
            config: Arc::new(RwLock::new(cfg)),
            manager: Arc::new(ServerManager::new()),
            status: Arc::new(RwLock::new(EngineStatus::Stopped)),
            port: Arc::new(RwLock::new(4242)),
            shutdown_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the gateway: launch MCP servers and bind the HTTP port.
    pub async fn start(&self) -> Result<(), String> {
        let current = self.status.read().await.clone();
        if current == EngineStatus::Running {
            return Err("Engine is already running".to_string());
        }

        let port = *self.port.read().await;
        let cfg = self.config.read().await.clone();

        // Start all enabled Main servers
        for server in &cfg.main.servers {
            if server.enabled {
                if let Err(e) = self.manager.start_server(server.clone()).await {
                    error!(server_id = %server.id, error = %e, "failed to start main server");
                }
            }
        }

        // Start environment-specific servers
        for env in &cfg.environments {
            let resolved = resolve_named(&cfg, &env.id).unwrap_or_default();
            for rs in &resolved {
                if rs.source == plugmux_core::environment::ServerSource::Environment {
                    if let Err(e) = self.manager.start_server(rs.config.clone()).await {
                        error!(server_id = %rs.config.id, env = %env.id, error = %e, "failed to start env server");
                    }
                }
            }
        }

        // Start health checker
        let health_manager = self.manager.clone();
        tokio::spawn(start_health_checker(health_manager, Duration::from_secs(30)));

        // Start HTTP server
        let config = self.config.clone();
        let manager = self.manager.clone();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        let addr = format!("127.0.0.1:{port}");
        let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
            format!("Port {port} is already in use: {e}")
        })?;

        info!("plugmux gateway listening on http://{addr}");

        let router = router::build_router(config, manager);
        tokio::spawn(async move {
            let server = axum::serve(listener, router);
            tokio::select! {
                result = server => {
                    if let Err(e) = result {
                        error!(error = %e, "gateway server error");
                    }
                }
                _ = rx => {
                    info!("gateway server shutting down");
                }
            }
        });

        *self.shutdown_tx.write().await = Some(tx);
        *self.status.write().await = EngineStatus::Running;

        Ok(())
    }

    /// Stop the gateway: shut down all servers and release the port.
    pub async fn stop(&self) -> Result<(), String> {
        let current = self.status.read().await.clone();
        if current != EngineStatus::Running {
            return Err("Engine is not running".to_string());
        }

        // Signal HTTP server shutdown
        if let Some(tx) = self.shutdown_tx.write().await.take() {
            let _ = tx.send(());
        }

        // Shut down all MCP servers
        self.manager.shutdown_all().await;

        *self.status.write().await = EngineStatus::Stopped;
        info!("engine stopped");

        Ok(())
    }

    /// Reload config from disk.
    pub async fn reload_config(&self) -> Result<(), String> {
        let path = config::config_path();
        let new_cfg = config::load_or_default(&path).map_err(|e| e.to_string())?;
        *self.config.write().await = new_cfg;
        info!("config reloaded from disk");
        Ok(())
    }

    /// Save current in-memory config to disk.
    pub async fn save_config(&self) -> Result<(), String> {
        let path = config::config_path();
        let cfg = self.config.read().await;
        config::save(&path, &cfg).map_err(|e| e.to_string())
    }
}
```

- [ ] **Step 3: Wire engine into Tauri builder**

Update `crates/plugmux-app/src-tauri/src/lib.rs`:
```rust
mod engine;
mod events;

use engine::Engine;
use std::sync::Arc;

pub fn run() {
    tracing_subscriber::fmt::init();

    let engine = Arc::new(Engine::new());

    tauri::Builder::default()
        .manage(engine.clone())
        .setup(move |app| {
            let engine = engine.clone();
            let handle = app.handle().clone();

            // Auto-start engine
            tauri::async_runtime::spawn(async move {
                match engine.start().await {
                    Ok(()) => {
                        let _ = handle.emit(
                            events::ENGINE_STATUS_CHANGED,
                            events::EngineStatusPayload {
                                status: "running".to_string(),
                            },
                        );
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "failed to start engine");
                        let _ = handle.emit(
                            events::ENGINE_STATUS_CHANGED,
                            events::EngineStatusPayload {
                                status: "conflict".to_string(),
                            },
                        );
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running plugmux");
}
```

- [ ] **Step 4: Verify compilation**

Run:
```bash
cd plugmux && cargo check -p plugmux-app
```
Expected: compiles without errors.

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-app/src-tauri/src/
git commit -m "feat(phase2): add engine lifecycle and event definitions"
```

---

## Task 4: Tray Icon

**Files:**
- Create: `crates/plugmux-app/src-tauri/src/tray.rs`
- Modify: `crates/plugmux-app/src-tauri/src/lib.rs`

- [ ] **Step 1: Create tray module**

`crates/plugmux-app/src-tauri/src/tray.rs`:
```rust
use std::sync::Arc;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

use crate::engine::{Engine, EngineStatus};
use crate::events;

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let toggle = MenuItem::with_id(app, "toggle", "Stop", true, None::<&str>)?;
    let open = MenuItem::with_id(app, "open", "Open plugmux", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(app, &[&toggle, &separator, &open, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "toggle" => {
                    let engine = app.state::<Arc<Engine>>();
                    let engine = engine.inner().clone();
                    let handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let status = engine.status.read().await.clone();
                        let result = if status == EngineStatus::Running {
                            engine.stop().await
                        } else {
                            engine.start().await
                        };
                        if let Err(e) = result {
                            tracing::error!(error = %e, "engine toggle failed");
                        }
                        let new_status = engine.status.read().await.clone();
                        let _ = handle.emit(
                            events::ENGINE_STATUS_CHANGED,
                            events::EngineStatusPayload {
                                status: new_status.as_str().to_string(),
                            },
                        );
                    });
                }
                "open" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    let engine = app.state::<Arc<Engine>>();
                    let engine = engine.inner().clone();
                    let handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = engine.stop().await;
                        handle.exit(0);
                    });
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
```

- [ ] **Step 2: Wire tray into Tauri setup**

In `crates/plugmux-app/src-tauri/src/lib.rs`, add `mod tray;` at the top and add to the `.setup()` closure, before the engine start spawn:

```rust
tray::setup_tray(app.handle())?;
```

- [ ] **Step 3: Generate tray icons from SVG**

Run (uses Tauri icon generator):
```bash
cd crates/plugmux-app/src-tauri && npx @tauri-apps/cli icon ../../assets/icon-tray.svg
```

If that fails (SVG may not be supported), manually create a 32x32 and 128x128 PNG from the SVG using any tool, then run:
```bash
npx @tauri-apps/cli icon path/to/icon.png
```

- [ ] **Step 4: Verify tray appears**

Run:
```bash
cd crates/plugmux-app && npm run tauri dev
```
Expected: tray icon appears in macOS menu bar. Right-click shows Stop / Open / Quit menu.

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-app/src-tauri/
git commit -m "feat(phase2): add system tray with menu and health status"
```

---

## Task 5: Tauri Commands (Rust → React Bridge)

**Files:**
- Create: `crates/plugmux-app/src-tauri/src/commands.rs`
- Modify: `crates/plugmux-app/src-tauri/src/lib.rs`

- [ ] **Step 1: Create commands module**

`crates/plugmux-app/src-tauri/src/commands.rs`:
```rust
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use plugmux_core::config::{
    self, EnvironmentConfig, PlugmuxConfig, ServerOverride,
};
use plugmux_core::server::ServerConfig;
use plugmux_core::slug::slugify;

use crate::engine::{Engine, EngineStatus};
use crate::events;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub port: u16,
    pub autostart: bool,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentPermissions {
    pub enable_server: String,
    pub disable_server: String,
}

// ---------------------------------------------------------------------------
// Engine commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_engine_status(engine: State<'_, Arc<Engine>>) -> Result<String, String> {
    let status = engine.status.read().await;
    Ok(status.as_str().to_string())
}

#[tauri::command]
pub async fn start_engine(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
) -> Result<(), String> {
    engine.start().await?;
    let _ = app.emit(
        events::ENGINE_STATUS_CHANGED,
        events::EngineStatusPayload {
            status: "running".to_string(),
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn stop_engine(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
) -> Result<(), String> {
    engine.stop().await?;
    let _ = app.emit(
        events::ENGINE_STATUS_CHANGED,
        events::EngineStatusPayload {
            status: "stopped".to_string(),
        },
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Config commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_config(engine: State<'_, Arc<Engine>>) -> Result<PlugmuxConfig, String> {
    let cfg = engine.config.read().await;
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn get_main_servers(
    engine: State<'_, Arc<Engine>>,
) -> Result<Vec<ServerConfig>, String> {
    let cfg = engine.config.read().await;
    Ok(cfg.main.servers.clone())
}

#[tauri::command]
pub async fn add_main_server(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    config: ServerConfig,
) -> Result<(), String> {
    {
        let mut cfg = engine.config.write().await;
        cfg.main.servers.push(config.clone());
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::SERVER_ADDED,
        events::ServerChangedPayload {
            server_id: config.id,
            env_id: None,
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn remove_main_server(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    {
        let mut cfg = engine.config.write().await;
        cfg.main.servers.retain(|s| s.id != id);
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::SERVER_REMOVED,
        events::ServerChangedPayload {
            server_id: id,
            env_id: None,
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn toggle_main_server(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    let enabled;
    {
        let mut cfg = engine.config.write().await;
        let server = cfg
            .main
            .servers
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| format!("Server not found: {id}"))?;
        server.enabled = !server.enabled;
        enabled = server.enabled;
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::SERVER_TOGGLED,
        events::ServerToggledPayload {
            server_id: id,
            env_id: None,
            enabled,
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn rename_server(
    engine: State<'_, Arc<Engine>>,
    id: String,
    name: String,
) -> Result<(), String> {
    {
        let mut cfg = engine.config.write().await;
        // Search in main
        if let Some(server) = cfg.main.servers.iter_mut().find(|s| s.id == id) {
            server.name = name;
        } else {
            // Search in environments
            let mut found = false;
            for env in &mut cfg.environments {
                if let Some(server) = env.servers.iter_mut().find(|s| s.id == id) {
                    server.name = name.clone();
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(format!("Server not found: {id}"));
            }
        }
    }
    engine.save_config().await
}

// ---------------------------------------------------------------------------
// Environment commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn list_environments(
    engine: State<'_, Arc<Engine>>,
) -> Result<Vec<EnvironmentConfig>, String> {
    let cfg = engine.config.read().await;
    Ok(cfg.environments.clone())
}

#[tauri::command]
pub async fn create_environment(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    name: String,
) -> Result<EnvironmentConfig, String> {
    let env;
    {
        let mut cfg = engine.config.write().await;
        let port = *engine.port.read().await;
        let created = config::add_environment(&mut cfg, &name);
        // Override endpoint to match actual gateway route and port
        // Note: plugmux-core's add_environment uses port 3000 and path /{id} by default,
        // but the gateway router uses /env/{id}. This override ensures consistency.
        // Consider updating config::add_environment() to accept a port param in a future refactor.
        created.endpoint = format!("http://localhost:{port}/env/{}", created.id);
        env = created.clone();
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::ENVIRONMENT_CREATED,
        events::EnvironmentChangedPayload {
            env_id: env.id.clone(),
        },
    );
    Ok(env)
}

#[tauri::command]
pub async fn delete_environment(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    {
        let mut cfg = engine.config.write().await;
        if !config::remove_environment(&mut cfg, &id) {
            return Err(format!("Environment not found: {id}"));
        }
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::ENVIRONMENT_DELETED,
        events::EnvironmentChangedPayload { env_id: id },
    );
    Ok(())
}

#[tauri::command]
pub async fn rename_environment(
    engine: State<'_, Arc<Engine>>,
    id: String,
    name: String,
) -> Result<(), String> {
    {
        let mut cfg = engine.config.write().await;
        let env = cfg
            .environments
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or_else(|| format!("Environment not found: {id}"))?;
        env.name = name;
    }
    engine.save_config().await
}

#[tauri::command]
pub async fn add_env_server(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    env_id: String,
    config: ServerConfig,
) -> Result<(), String> {
    let server_id = config.id.clone();
    {
        let mut cfg = engine.config.write().await;
        let env = cfg
            .environments
            .iter_mut()
            .find(|e| e.id == env_id)
            .ok_or_else(|| format!("Environment not found: {env_id}"))?;
        env.servers.push(config);
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::SERVER_ADDED,
        events::ServerChangedPayload {
            server_id,
            env_id: Some(env_id),
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn remove_env_server(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    env_id: String,
    server_id: String,
) -> Result<(), String> {
    {
        let mut cfg = engine.config.write().await;
        let env = cfg
            .environments
            .iter_mut()
            .find(|e| e.id == env_id)
            .ok_or_else(|| format!("Environment not found: {env_id}"))?;
        env.servers.retain(|s| s.id != server_id);
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::SERVER_REMOVED,
        events::ServerChangedPayload {
            server_id,
            env_id: Some(env_id),
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn toggle_env_override(
    engine: State<'_, Arc<Engine>>,
    app: AppHandle,
    env_id: String,
    server_id: String,
) -> Result<(), String> {
    let enabled;
    {
        let mut cfg = engine.config.write().await;
        let env = cfg
            .environments
            .iter_mut()
            .find(|e| e.id == env_id)
            .ok_or_else(|| format!("Environment not found: {env_id}"))?;

        if let Some(ov) = env.overrides.iter_mut().find(|o| o.server_id == server_id) {
            let current = ov.enabled.unwrap_or(true);
            ov.enabled = Some(!current);
            enabled = !current;
        } else {
            env.overrides.push(ServerOverride {
                server_id: server_id.clone(),
                enabled: Some(false),
                url: None,
                permissions: None,
            });
            enabled = false;
        }
    }
    engine.save_config().await?;
    let _ = app.emit(
        events::SERVER_TOGGLED,
        events::ServerToggledPayload {
            server_id,
            env_id: Some(env_id),
            enabled,
        },
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Permission commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_permissions(
    engine: State<'_, Arc<Engine>>,
    env_id: String,
) -> Result<EnvironmentPermissions, String> {
    let cfg = engine.config.read().await;
    let env = cfg
        .environments
        .iter()
        .find(|e| e.id == env_id)
        .ok_or_else(|| format!("Environment not found: {env_id}"))?;

    // Derive permission levels from overrides
    // Default is "approve" if no explicit permission is set
    let mut enable_level = "approve".to_string();
    let mut disable_level = "approve".to_string();

    for ov in &env.overrides {
        if let Some(perm) = &ov.permissions {
            if perm.deny.as_ref().is_some_and(|d| d.iter().any(|a| a == "enable_server")) {
                enable_level = "disable".to_string();
            } else if perm.allow.as_ref().is_some_and(|a| a.iter().any(|x| x == "enable_server")) {
                enable_level = "allow".to_string();
            }
            if perm.deny.as_ref().is_some_and(|d| d.iter().any(|a| a == "disable_server")) {
                disable_level = "disable".to_string();
            } else if perm.allow.as_ref().is_some_and(|a| a.iter().any(|x| x == "disable_server")) {
                disable_level = "allow".to_string();
            }
        }
    }

    Ok(EnvironmentPermissions {
        enable_server: enable_level,
        disable_server: disable_level,
    })
}

#[tauri::command]
pub async fn set_permission(
    engine: State<'_, Arc<Engine>>,
    env_id: String,
    action: String,
    level: String,
) -> Result<(), String> {
    {
        let mut cfg = engine.config.write().await;
        let env = cfg
            .environments
            .iter_mut()
            .find(|e| e.id == env_id)
            .ok_or_else(|| format!("Environment not found: {env_id}"))?;

        // Find or create a "global" override entry for permissions
        // We use a special override with server_id "*" for environment-level permissions
        let ov = if let Some(ov) = env.overrides.iter_mut().find(|o| o.server_id == "*") {
            ov
        } else {
            env.overrides.push(ServerOverride {
                server_id: "*".to_string(),
                enabled: None,
                url: None,
                permissions: Some(plugmux_core::config::Permission {
                    allow: Some(vec![]),
                    deny: Some(vec![]),
                }),
            });
            env.overrides.last_mut().unwrap()
        };

        let perm = ov.permissions.get_or_insert(plugmux_core::config::Permission {
            allow: Some(vec![]),
            deny: Some(vec![]),
        });

        let allow = perm.allow.get_or_insert_with(Vec::new);
        let deny = perm.deny.get_or_insert_with(Vec::new);

        // Remove action from both lists first
        allow.retain(|a| a != &action);
        deny.retain(|a| a != &action);

        // Add to appropriate list
        match level.as_str() {
            "allow" => allow.push(action),
            "disable" => deny.push(action),
            "approve" => {} // default, not in either list
            _ => return Err(format!("Invalid permission level: {level}")),
        }
    }
    engine.save_config().await
}

// ---------------------------------------------------------------------------
// Settings commands
// Note: autostart is handled via @tauri-apps/plugin-autostart JS API.
// Theme is handled client-side via DOM class toggle.
// Only port needs a Rust command since it affects the engine.
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_port(engine: State<'_, Arc<Engine>>) -> Result<u16, String> {
    Ok(*engine.port.read().await)
}

#[tauri::command]
pub async fn set_port(engine: State<'_, Arc<Engine>>, port: u16) -> Result<(), String> {
    *engine.port.write().await = port;
    Ok(())
}
```

- [ ] **Step 2: Register commands in lib.rs**

Update `crates/plugmux-app/src-tauri/src/lib.rs` to add `mod commands;` and register all commands in the builder:

```rust
.invoke_handler(tauri::generate_handler![
    commands::get_engine_status,
    commands::start_engine,
    commands::stop_engine,
    commands::get_config,
    commands::get_main_servers,
    commands::add_main_server,
    commands::remove_main_server,
    commands::toggle_main_server,
    commands::rename_server,
    commands::list_environments,
    commands::create_environment,
    commands::delete_environment,
    commands::rename_environment,
    commands::add_env_server,
    commands::remove_env_server,
    commands::toggle_env_override,
    commands::get_permissions,
    commands::set_permission,
    commands::get_port,
    commands::set_port,
])
```

- [ ] **Step 3: Verify compilation**

Run:
```bash
cd plugmux && cargo check -p plugmux-app
```
Expected: compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add crates/plugmux-app/src-tauri/src/
git commit -m "feat(phase2): add Tauri commands for engine, config, environments, settings"
```

---

## Task 6: Config File Watcher

**Files:**
- Create: `crates/plugmux-app/src-tauri/src/watcher.rs`
- Modify: `crates/plugmux-app/src-tauri/src/lib.rs`

- [ ] **Step 1: Create watcher module**

`crates/plugmux-app/src-tauri/src/watcher.rs`:
```rust
use std::sync::Arc;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};
use tracing::{error, info};

use plugmux_core::config;

use crate::engine::Engine;
use crate::events;

/// Starts watching the config file for external changes.
/// When the file is modified, reloads the config and emits a config_reloaded event.
pub fn start_config_watcher(
    app: AppHandle,
    engine: Arc<Engine>,
) -> Result<RecommendedWatcher, String> {
    let config_path = config::config_path();

    let app_handle = app.clone();
    let engine_clone = engine.clone();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) => {
                if matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_)
                ) {
                    info!("config file changed externally, reloading");
                    let engine = engine_clone.clone();
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = engine.reload_config().await {
                            error!(error = %e, "failed to reload config");
                            return;
                        }
                        let _ = handle.emit(events::CONFIG_RELOADED, ());
                    });
                }
            }
            Err(e) => {
                error!(error = %e, "config watcher error");
            }
        }
    })
    .map_err(|e| format!("failed to create file watcher: {e}"))?;

    // Watch the parent directory (the file might not exist yet)
    if let Some(parent) = config_path.parent() {
        watcher
            .watch(parent, RecursiveMode::NonRecursive)
            .map_err(|e| format!("failed to watch config directory: {e}"))?;
        info!("watching config directory: {}", parent.display());
    }

    Ok(watcher)
}
```

- [ ] **Step 2: Wire watcher into Tauri setup**

In `crates/plugmux-app/src-tauri/src/lib.rs`, add `mod watcher;` and in the `.setup()` closure, after engine start, add:

```rust
// Start config file watcher (keep _watcher alive for the app lifetime)
let _watcher = watcher::start_config_watcher(
    app.handle().clone(),
    engine.clone(),
);
// Store watcher so it isn't dropped
app.manage(_watcher);
```

- [ ] **Step 3: Verify compilation**

Run:
```bash
cd plugmux && cargo check -p plugmux-app
```
Expected: compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add crates/plugmux-app/src-tauri/src/
git commit -m "feat(phase2): add config file watcher for external changes"
```

---

## Task 7: TypeScript Command Layer + Hooks

**Files:**
- Create: `crates/plugmux-app/src/lib/commands.ts`
- Create: `crates/plugmux-app/src/hooks/useEngine.ts`
- Create: `crates/plugmux-app/src/hooks/useConfig.ts`
- Create: `crates/plugmux-app/src/hooks/useEvents.ts`

- [ ] **Step 1: Create typed invoke wrappers**

`crates/plugmux-app/src/lib/commands.ts`:
```typescript
import { invoke } from "@tauri-apps/api/core";

// Types matching Rust structs
export interface ServerConfig {
  id: string;
  name: string;
  transport: "stdio" | "http";
  command?: string;
  args?: string[];
  url?: string;
  connectivity: "local" | "online";
  enabled: boolean;
  description?: string;
}

export interface ServerOverride {
  server_id: string;
  enabled?: boolean;
}

export interface EnvironmentConfig {
  id: string;
  name: string;
  endpoint: string;
  servers: ServerConfig[];
  overrides: ServerOverride[];
}

export interface PlugmuxConfig {
  main: { servers: ServerConfig[] };
  environments: EnvironmentConfig[];
}

// Engine
export const getEngineStatus = () => invoke<string>("get_engine_status");
export const startEngine = () => invoke<void>("start_engine");
export const stopEngine = () => invoke<void>("stop_engine");

// Config
export const getConfig = () => invoke<PlugmuxConfig>("get_config");
export const getMainServers = () => invoke<ServerConfig[]>("get_main_servers");
export const addMainServer = (config: ServerConfig) =>
  invoke<void>("add_main_server", { config });
export const removeMainServer = (id: string) =>
  invoke<void>("remove_main_server", { id });
export const toggleMainServer = (id: string) =>
  invoke<void>("toggle_main_server", { id });
export const renameServer = (id: string, name: string) =>
  invoke<void>("rename_server", { id, name });

// Environments
export const listEnvironments = () =>
  invoke<EnvironmentConfig[]>("list_environments");
export const createEnvironment = (name: string) =>
  invoke<EnvironmentConfig>("create_environment", { name });
export const deleteEnvironment = (id: string) =>
  invoke<void>("delete_environment", { id });
export const renameEnvironment = (id: string, name: string) =>
  invoke<void>("rename_environment", { id, name });
export const addEnvServer = (envId: string, config: ServerConfig) =>
  invoke<void>("add_env_server", { envId, config });
export const removeEnvServer = (envId: string, serverId: string) =>
  invoke<void>("remove_env_server", { envId, serverId });
export const toggleEnvOverride = (envId: string, serverId: string) =>
  invoke<void>("toggle_env_override", { envId, serverId });

// Settings
export const getPort = () => invoke<number>("get_port");
export const setPort = (port: number) => invoke<void>("set_port", { port });
```

- [ ] **Step 2: Create useEngine hook**

`crates/plugmux-app/src/hooks/useEngine.ts`:
```typescript
import { useState, useEffect, useCallback } from "react";
import { getEngineStatus, startEngine, stopEngine } from "@/lib/commands";
import { useEvents } from "./useEvents";

export function useEngine() {
  const [status, setStatus] = useState<"running" | "stopped" | "conflict">(
    "stopped",
  );

  useEffect(() => {
    getEngineStatus().then((s) =>
      setStatus(s as "running" | "stopped" | "conflict"),
    );
  }, []);

  useEvents("engine_status_changed", (payload: { status: string }) => {
    setStatus(payload.status as "running" | "stopped" | "conflict");
  });

  const toggle = useCallback(async () => {
    if (status === "running") {
      await stopEngine();
    } else {
      await startEngine();
    }
  }, [status]);

  return { status, toggle };
}
```

- [ ] **Step 3: Create useEvents hook**

`crates/plugmux-app/src/hooks/useEvents.ts`:
```typescript
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

export function useEvents<T>(eventName: string, callback: (payload: T) => void) {
  useEffect(() => {
    const unlisten = listen<T>(eventName, (event) => {
      callback(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [eventName, callback]);
}
```

- [ ] **Step 4: Create useConfig hook**

`crates/plugmux-app/src/hooks/useConfig.ts`:
```typescript
import { useState, useEffect, useCallback } from "react";
import {
  getConfig,
  type PlugmuxConfig,
  type ServerConfig,
  type EnvironmentConfig,
  addMainServer,
  removeMainServer,
  toggleMainServer,
  createEnvironment,
  deleteEnvironment,
  addEnvServer,
  removeEnvServer,
  toggleEnvOverride,
} from "@/lib/commands";
import { useEvents } from "./useEvents";

export function useConfig() {
  const [config, setConfig] = useState<PlugmuxConfig | null>(null);
  const [loading, setLoading] = useState(true);

  const reload = useCallback(async () => {
    const cfg = await getConfig();
    setConfig(cfg);
    setLoading(false);
  }, []);

  useEffect(() => {
    reload();
  }, [reload]);

  // Re-fetch on any server/environment change events
  useEvents("server_added", reload);
  useEvents("server_removed", reload);
  useEvents("server_toggled", reload);
  useEvents("environment_created", reload);
  useEvents("environment_deleted", reload);
  useEvents("config_reloaded", reload);

  return {
    config,
    loading,
    reload,
    addMainServer: async (server: ServerConfig) => {
      await addMainServer(server);
    },
    removeMainServer: async (id: string) => {
      await removeMainServer(id);
    },
    toggleMainServer: async (id: string) => {
      await toggleMainServer(id);
    },
    createEnvironment: async (name: string) => {
      return await createEnvironment(name);
    },
    deleteEnvironment: async (id: string) => {
      await deleteEnvironment(id);
    },
    addEnvServer: async (envId: string, server: ServerConfig) => {
      await addEnvServer(envId, server);
    },
    removeEnvServer: async (envId: string, serverId: string) => {
      await removeEnvServer(envId, serverId);
    },
    toggleEnvOverride: async (envId: string, serverId: string) => {
      await toggleEnvOverride(envId, serverId);
    },
  };
}
```

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-app/src/
git commit -m "feat(phase2): add TypeScript command wrappers and React hooks"
```

---

## Task 8: Layout + Sidebar + Routing

**Files:**
- Create: `crates/plugmux-app/src/components/layout/Sidebar.tsx`
- Create: `crates/plugmux-app/src/components/layout/Layout.tsx`
- Modify: `crates/plugmux-app/src/App.tsx`

- [ ] **Step 1: Create Sidebar component**

Use shadcn MCP to look up sidebar component patterns, then build `crates/plugmux-app/src/components/layout/Sidebar.tsx`:

```tsx
import { useConfig } from "@/hooks/useConfig";
import { useEngine } from "@/hooks/useEngine";
import { cn } from "@/lib/utils";
import {
  Server,
  Layers,
  BookOpen,
  LayoutTemplate,
  Settings,
  Plus,
  Circle,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

interface SidebarProps {
  activePage: string;
  onNavigate: (page: string) => void;
  onCreateEnvironment: () => void;
}

export function Sidebar({
  activePage,
  onNavigate,
  onCreateEnvironment,
}: SidebarProps) {
  const { config } = useConfig();
  const { status } = useEngine();

  const statusColor =
    status === "running"
      ? "text-green-500"
      : status === "conflict"
        ? "text-yellow-500"
        : "text-muted-foreground";

  return (
    <div className="w-[220px] h-full border-r border-border flex flex-col bg-card">
      {/* Header */}
      <div className="p-4 flex items-center gap-2">
        <Circle className={cn("h-2.5 w-2.5 fill-current", statusColor)} />
        <span className="font-semibold text-sm">plugmux</span>
      </div>

      {/* Main nav */}
      <nav className="flex-1 px-2 space-y-1">
        <NavItem
          icon={<Server className="h-4 w-4" />}
          label="Main"
          active={activePage === "main"}
          onClick={() => onNavigate("main")}
        />

        {/* Environments section */}
        <div className="pt-4">
          <span className="px-3 text-xs font-medium text-muted-foreground uppercase tracking-wider">
            Environments
          </span>
          <div className="mt-1 space-y-0.5">
            {config?.environments.map((env) => (
              <NavItem
                key={env.id}
                icon={<Layers className="h-4 w-4" />}
                label={env.name}
                badge={String(env.servers.length)}
                active={activePage === `env:${env.id}`}
                onClick={() => onNavigate(`env:${env.id}`)}
              />
            ))}
            <Button
              variant="ghost"
              size="sm"
              className="w-full justify-start text-muted-foreground"
              onClick={onCreateEnvironment}
            >
              <Plus className="h-4 w-4 mr-2" />
              New
            </Button>
          </div>
        </div>

        {/* Bottom nav */}
        <div className="pt-4 space-y-0.5">
          <NavItem
            icon={<BookOpen className="h-4 w-4" />}
            label="Catalog"
            active={activePage === "catalog"}
            onClick={() => onNavigate("catalog")}
          />
          <NavItem
            icon={<LayoutTemplate className="h-4 w-4" />}
            label="Presets"
            active={activePage === "presets"}
            onClick={() => onNavigate("presets")}
          />
        </div>
      </nav>

      {/* Settings at bottom */}
      <div className="p-2 border-t border-border">
        <NavItem
          icon={<Settings className="h-4 w-4" />}
          label="Settings"
          active={activePage === "settings"}
          onClick={() => onNavigate("settings")}
        />
      </div>
    </div>
  );
}

function NavItem({
  icon,
  label,
  badge,
  active,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  badge?: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "w-full flex items-center gap-2 px-3 py-1.5 rounded-md text-sm",
        active
          ? "bg-accent text-accent-foreground"
          : "text-muted-foreground hover:bg-accent/50 hover:text-foreground",
      )}
    >
      {icon}
      <span className="flex-1 text-left truncate">{label}</span>
      {badge && (
        <Badge variant="secondary" className="text-xs px-1.5 py-0">
          {badge}
        </Badge>
      )}
    </button>
  );
}
```

- [ ] **Step 2: Create Layout component**

`crates/plugmux-app/src/components/layout/Layout.tsx`:
```tsx
import { Sidebar } from "./Sidebar";

interface LayoutProps {
  activePage: string;
  onNavigate: (page: string) => void;
  onCreateEnvironment: () => void;
  children: React.ReactNode;
}

export function Layout({
  activePage,
  onNavigate,
  onCreateEnvironment,
  children,
}: LayoutProps) {
  return (
    <div className="h-screen flex">
      <Sidebar
        activePage={activePage}
        onNavigate={onNavigate}
        onCreateEnvironment={onCreateEnvironment}
      />
      <main className="flex-1 overflow-auto">{children}</main>
    </div>
  );
}
```

- [ ] **Step 3: Wire up App.tsx with routing**

`crates/plugmux-app/src/App.tsx`:
```tsx
import { useState, useCallback } from "react";
import { Layout } from "@/components/layout/Layout";
import { MainPage } from "@/pages/MainPage";
import { EnvironmentPage } from "@/pages/EnvironmentPage";
import { CatalogPage } from "@/pages/CatalogPage";
import { PresetsPage } from "@/pages/PresetsPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { CreateEnvironmentDialog } from "@/components/environments/CreateEnvironmentDialog";

function App() {
  const [activePage, setActivePage] = useState("main");
  const [showCreateEnv, setShowCreateEnv] = useState(false);

  const handleNavigate = useCallback((page: string) => {
    setActivePage(page);
  }, []);

  const renderPage = () => {
    if (activePage === "main") return <MainPage />;
    if (activePage === "catalog") return <CatalogPage />;
    if (activePage === "presets") return <PresetsPage />;
    if (activePage === "settings") return <SettingsPage />;
    if (activePage.startsWith("env:")) {
      const envId = activePage.slice(4);
      return <EnvironmentPage envId={envId} />;
    }
    return <MainPage />;
  };

  return (
    <Layout
      activePage={activePage}
      onNavigate={handleNavigate}
      onCreateEnvironment={() => setShowCreateEnv(true)}
    >
      {renderPage()}
      <CreateEnvironmentDialog
        open={showCreateEnv}
        onOpenChange={setShowCreateEnv}
        onCreated={(env) => {
          setShowCreateEnv(false);
          setActivePage(`env:${env.id}`);
        }}
      />
    </Layout>
  );
}

export default App;
```

- [ ] **Step 4: Create stub pages** (they will be implemented in Tasks 9-11)

Create minimal stubs for all pages so the app compiles:

`crates/plugmux-app/src/pages/MainPage.tsx`:
```tsx
export function MainPage() {
  return <div className="p-6"><h1 className="text-lg font-semibold">Main</h1></div>;
}
```

`crates/plugmux-app/src/pages/EnvironmentPage.tsx`:
```tsx
export function EnvironmentPage({ envId }: { envId: string }) {
  return <div className="p-6"><h1 className="text-lg font-semibold">Environment: {envId}</h1></div>;
}
```

`crates/plugmux-app/src/pages/CatalogPage.tsx`:
```tsx
export function CatalogPage() {
  return <div className="p-6"><h1 className="text-lg font-semibold">Catalog</h1><p className="text-muted-foreground mt-2">Coming soon — browse and install community MCP servers.</p></div>;
}
```

`crates/plugmux-app/src/pages/PresetsPage.tsx`:
```tsx
export function PresetsPage() {
  return <div className="p-6"><h1 className="text-lg font-semibold">Presets</h1><p className="text-muted-foreground mt-2">Coming soon — create environments from preset templates.</p></div>;
}
```

`crates/plugmux-app/src/pages/SettingsPage.tsx`:
```tsx
export function SettingsPage() {
  return <div className="p-6"><h1 className="text-lg font-semibold">Settings</h1></div>;
}
```

Create stub for `CreateEnvironmentDialog`:

`crates/plugmux-app/src/components/environments/CreateEnvironmentDialog.tsx`:
```tsx
import type { EnvironmentConfig } from "@/lib/commands";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: (env: EnvironmentConfig) => void;
}

export function CreateEnvironmentDialog({ open, onOpenChange, onCreated }: Props) {
  if (!open) return null;
  return null; // Will be implemented in Task 10
}
```

- [ ] **Step 5: Verify app renders with sidebar**

Run:
```bash
cd crates/plugmux-app && npm run tauri dev
```
Expected: window opens with sidebar showing Main, Environments, Catalog, Presets, Settings. Clicking items switches the content area.

- [ ] **Step 6: Commit**

```bash
git add crates/plugmux-app/src/
git commit -m "feat(phase2): add sidebar layout, page routing, and stub pages"
```

---

## Task 9: Main Page (Server Management)

**Files:**
- Create: `crates/plugmux-app/src/components/servers/ServerCard.tsx`
- Create: `crates/plugmux-app/src/components/servers/AddServerDialog.tsx`
- Modify: `crates/plugmux-app/src/pages/MainPage.tsx`

- [ ] **Step 1: Create ServerCard component**

`crates/plugmux-app/src/components/servers/ServerCard.tsx`:
```tsx
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Circle, Trash2 } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ServerConfig } from "@/lib/commands";

interface ServerCardProps {
  server: ServerConfig;
  onToggle: () => void;
  onRemove: () => void;
}

export function ServerCard({ server, onToggle, onRemove }: ServerCardProps) {
  return (
    <div
      className={cn(
        "flex items-center gap-3 px-4 py-3 rounded-lg border border-border",
        !server.enabled && "opacity-50",
      )}
    >
      {/* Health dot */}
      <Circle
        className={cn(
          "h-2.5 w-2.5 flex-shrink-0 fill-current",
          server.enabled ? "text-green-500" : "text-muted-foreground",
        )}
      />

      {/* Server info */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium truncate">{server.name}</span>
          <Badge
            variant="outline"
            className={cn(
              "text-xs",
              server.connectivity === "local"
                ? "text-green-500 border-green-500/30"
                : "text-blue-500 border-blue-500/30",
            )}
          >
            {server.connectivity}
          </Badge>
        </div>
        {server.description && (
          <p className="text-xs text-muted-foreground truncate mt-0.5">
            {server.description}
          </p>
        )}
      </div>

      {/* Actions */}
      <Switch checked={server.enabled} onCheckedChange={onToggle} />
      <Button
        variant="ghost"
        size="icon"
        className="h-8 w-8 text-muted-foreground hover:text-destructive"
        onClick={onRemove}
      >
        <Trash2 className="h-4 w-4" />
      </Button>
    </div>
  );
}
```

- [ ] **Step 2: Create AddServerDialog**

`crates/plugmux-app/src/components/servers/AddServerDialog.tsx`:
```tsx
import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { ServerConfig } from "@/lib/commands";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onAdd: (server: ServerConfig) => void;
}

export function AddServerDialog({ open, onOpenChange, onAdd }: Props) {
  const [id, setId] = useState("");
  const [name, setName] = useState("");
  const [transport, setTransport] = useState<"stdio" | "http">("stdio");
  const [command, setCommand] = useState("");
  const [url, setUrl] = useState("");
  const [connectivity, setConnectivity] = useState<"local" | "online">(
    "local",
  );

  const handleSubmit = () => {
    const server: ServerConfig = {
      id: id.trim(),
      name: name.trim() || id.trim(),
      transport,
      command: transport === "stdio" ? command.trim() || undefined : undefined,
      url: transport === "http" ? url.trim() || undefined : undefined,
      connectivity,
      enabled: true,
    };
    onAdd(server);
    // Reset form
    setId("");
    setName("");
    setCommand("");
    setUrl("");
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add Server</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="id">Server ID</Label>
            <Input
              id="id"
              placeholder="e.g. figma"
              value={id}
              onChange={(e) => setId(e.target.value)}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="name">Display Name</Label>
            <Input
              id="name"
              placeholder="e.g. Figma Design"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>
          <div className="space-y-2">
            <Label>Transport</Label>
            <Select
              value={transport}
              onValueChange={(v) => setTransport(v as "stdio" | "http")}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="stdio">stdio</SelectItem>
                <SelectItem value="http">HTTP</SelectItem>
              </SelectContent>
            </Select>
          </div>
          {transport === "stdio" ? (
            <div className="space-y-2">
              <Label htmlFor="command">Command</Label>
              <Input
                id="command"
                placeholder="e.g. npx -y @anthropic/figma-mcp"
                value={command}
                onChange={(e) => setCommand(e.target.value)}
              />
            </div>
          ) : (
            <div className="space-y-2">
              <Label htmlFor="url">URL</Label>
              <Input
                id="url"
                placeholder="e.g. https://context7.dev/mcp"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
              />
            </div>
          )}
          <div className="space-y-2">
            <Label>Connectivity</Label>
            <Select
              value={connectivity}
              onValueChange={(v) =>
                setConnectivity(v as "local" | "online")
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="local">Local</SelectItem>
                <SelectItem value="online">Online</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={!id.trim()}>
            Add Server
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

- [ ] **Step 3: Implement MainPage**

`crates/plugmux-app/src/pages/MainPage.tsx`:
```tsx
import { useState } from "react";
import { useConfig } from "@/hooks/useConfig";
import { ServerCard } from "@/components/servers/ServerCard";
import { AddServerDialog } from "@/components/servers/AddServerDialog";
import { Button } from "@/components/ui/button";
import { Plus } from "lucide-react";

export function MainPage() {
  const { config, addMainServer, removeMainServer, toggleMainServer } =
    useConfig();
  const [showAdd, setShowAdd] = useState(false);

  const servers = config?.main.servers ?? [];

  return (
    <div className="p-6 max-w-3xl">
      <div className="mb-6">
        <h1 className="text-lg font-semibold">Main</h1>
        <p className="text-sm text-muted-foreground mt-1">
          These servers are available in all environments.
        </p>
      </div>

      <div className="space-y-2">
        {servers.map((server) => (
          <ServerCard
            key={server.id}
            server={server}
            onToggle={() => toggleMainServer(server.id)}
            onRemove={() => removeMainServer(server.id)}
          />
        ))}

        {servers.length === 0 && (
          <p className="text-sm text-muted-foreground py-8 text-center">
            No servers configured. Add one to get started.
          </p>
        )}
      </div>

      <Button
        variant="outline"
        className="mt-4"
        onClick={() => setShowAdd(true)}
      >
        <Plus className="h-4 w-4 mr-2" />
        Add Server
      </Button>

      <AddServerDialog
        open={showAdd}
        onOpenChange={setShowAdd}
        onAdd={(server) => {
          addMainServer(server);
          setShowAdd(false);
        }}
      />
    </div>
  );
}
```

- [ ] **Step 4: Verify Main page renders with server cards**

Run:
```bash
cd crates/plugmux-app && npm run tauri dev
```
Expected: Main page shows server list (empty initially). "Add Server" button opens dialog. Adding a server shows a card with toggle and remove.

- [ ] **Step 5: Commit**

```bash
git add crates/plugmux-app/src/
git commit -m "feat(phase2): implement Main page with server cards, add/remove/toggle"
```

---

## Task 10: Environment Page

**Files:**
- Create: `crates/plugmux-app/src/components/environments/InheritedServers.tsx`
- Create: `crates/plugmux-app/src/components/environments/EnvironmentServers.tsx`
- Create: `crates/plugmux-app/src/components/environments/PermissionsPanel.tsx`
- Modify: `crates/plugmux-app/src/components/environments/CreateEnvironmentDialog.tsx`
- Modify: `crates/plugmux-app/src/pages/EnvironmentPage.tsx`

- [ ] **Step 1: Implement CreateEnvironmentDialog**

`crates/plugmux-app/src/components/environments/CreateEnvironmentDialog.tsx`:
```tsx
import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useConfig } from "@/hooks/useConfig";
import type { EnvironmentConfig } from "@/lib/commands";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: (env: EnvironmentConfig) => void;
}

export function CreateEnvironmentDialog({
  open,
  onOpenChange,
  onCreated,
}: Props) {
  const [name, setName] = useState("");
  const { createEnvironment } = useConfig();

  const handleCreate = async () => {
    if (!name.trim()) return;
    const env = await createEnvironment(name.trim());
    setName("");
    onCreated(env);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>New Environment</DialogTitle>
        </DialogHeader>
        <div className="space-y-2">
          <Label htmlFor="env-name">Name</Label>
          <Input
            id="env-name"
            placeholder="e.g. My SaaS App"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleCreate()}
          />
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleCreate} disabled={!name.trim()}>
            Create
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

- [ ] **Step 2: Create InheritedServers component**

`crates/plugmux-app/src/components/environments/InheritedServers.tsx`:
```tsx
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { Circle } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ServerConfig, ServerOverride } from "@/lib/commands";

interface Props {
  servers: ServerConfig[];
  overrides: ServerOverride[];
  onToggleOverride: (serverId: string) => void;
}

export function InheritedServers({ servers, overrides, onToggleOverride }: Props) {
  const isOverridden = (serverId: string) => {
    const override = overrides.find((o) => o.server_id === serverId);
    return override?.enabled === false;
  };

  if (servers.length === 0) return null;

  return (
    <div>
      <h2 className="text-sm font-medium text-muted-foreground mb-2">
        Inherited Servers
        <Badge variant="secondary" className="ml-2 text-xs">
          {servers.length}
        </Badge>
      </h2>
      <div className="space-y-2">
        {servers.map((server) => {
          const disabled = isOverridden(server.id);
          return (
            <div
              key={server.id}
              className={cn(
                "flex items-center gap-3 px-4 py-3 rounded-lg border border-border",
                disabled && "opacity-40",
              )}
            >
              <Circle
                className={cn(
                  "h-2.5 w-2.5 flex-shrink-0 fill-current",
                  disabled ? "text-muted-foreground" : "text-green-500",
                )}
              />
              <div className="flex-1 min-w-0">
                <span className={cn("text-sm font-medium", disabled && "line-through")}>
                  {server.name}
                </span>
              </div>
              <Badge variant="outline" className="text-xs text-muted-foreground">
                from Main
              </Badge>
              <Switch
                checked={!disabled}
                onCheckedChange={() => onToggleOverride(server.id)}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Create EnvironmentServers component**

`crates/plugmux-app/src/components/environments/EnvironmentServers.tsx`:
```tsx
import { ServerCard } from "@/components/servers/ServerCard";
import { Badge } from "@/components/ui/badge";
import type { ServerConfig } from "@/lib/commands";

interface Props {
  servers: ServerConfig[];
  onToggle: (serverId: string) => void;
  onRemove: (serverId: string) => void;
}

export function EnvironmentServers({ servers, onToggle, onRemove }: Props) {
  return (
    <div>
      <h2 className="text-sm font-medium text-muted-foreground mb-2">
        Environment Servers
        <Badge variant="secondary" className="ml-2 text-xs">
          {servers.length}
        </Badge>
      </h2>
      <div className="space-y-2">
        {servers.map((server) => (
          <ServerCard
            key={server.id}
            server={server}
            onToggle={() => onToggle(server.id)}
            onRemove={() => onRemove(server.id)}
          />
        ))}
        {servers.length === 0 && (
          <p className="text-sm text-muted-foreground py-4 text-center">
            No environment-specific servers.
          </p>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Create PermissionsPanel**

`crates/plugmux-app/src/components/environments/PermissionsPanel.tsx`:
```tsx
import { useState, useEffect } from "react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { ChevronDown, ChevronRight } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  envId: string;
}

const ACTIONS = [
  { id: "enable_server", label: "Enable Server" },
  { id: "disable_server", label: "Disable Server" },
];

export function PermissionsPanel({ envId }: Props) {
  const [expanded, setExpanded] = useState(false);
  const [permissions, setPermissions] = useState<Record<string, string>>({
    enable_server: "approve",
    disable_server: "approve",
  });

  useEffect(() => {
    invoke<{ enable_server: string; disable_server: string }>("get_permissions", { envId })
      .then((p) => setPermissions({ enable_server: p.enable_server, disable_server: p.disable_server }));
  }, [envId]);

  return (
    <div className="border border-border rounded-lg">
      <Button
        variant="ghost"
        className="w-full justify-start px-4 py-3"
        onClick={() => setExpanded(!expanded)}
      >
        {expanded ? (
          <ChevronDown className="h-4 w-4 mr-2" />
        ) : (
          <ChevronRight className="h-4 w-4 mr-2" />
        )}
        <span className="text-sm font-medium">Permissions</span>
      </Button>
      {expanded && (
        <div className="px-4 pb-4 space-y-3">
          {ACTIONS.map((action) => (
            <div key={action.id} className="flex items-center justify-between">
              <span className="text-sm">{action.label}</span>
              <Select
                value={permissions[action.id]}
                onValueChange={(v) => {
                  setPermissions((prev) => ({ ...prev, [action.id]: v }));
                  invoke("set_permission", { envId, action: action.id, level: v });
                }}
              >
                <SelectTrigger className="w-32">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="allow">Allow</SelectItem>
                  <SelectItem value="approve">Approve</SelectItem>
                  <SelectItem value="disable">Disable</SelectItem>
                </SelectContent>
              </Select>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 5: Implement EnvironmentPage**

`crates/plugmux-app/src/pages/EnvironmentPage.tsx`:
```tsx
import { useState } from "react";
import { useConfig } from "@/hooks/useConfig";
import { InheritedServers } from "@/components/environments/InheritedServers";
import { EnvironmentServers } from "@/components/environments/EnvironmentServers";
import { PermissionsPanel } from "@/components/environments/PermissionsPanel";
import { AddServerDialog } from "@/components/servers/AddServerDialog";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Separator } from "@/components/ui/separator";
import { Plus, Copy, Check, Trash2 } from "lucide-react";

export function EnvironmentPage({ envId }: { envId: string }) {
  const {
    config,
    toggleEnvOverride,
    addEnvServer,
    removeEnvServer,
    deleteEnvironment,
  } = useConfig();
  const [showAdd, setShowAdd] = useState(false);
  const [copied, setCopied] = useState(false);

  const env = config?.environments.find((e) => e.id === envId);
  if (!env) {
    return (
      <div className="p-6">
        <p className="text-muted-foreground">Environment not found.</p>
      </div>
    );
  }

  const mainServers = config?.main.servers ?? [];

  const handleCopy = () => {
    navigator.clipboard.writeText(env.endpoint);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="p-6 max-w-3xl">
      {/* Header */}
      <div className="mb-6">
        <h1 className="text-lg font-semibold">{env.name}</h1>
        <div className="flex items-center gap-2 mt-1">
          <code className="text-xs text-muted-foreground bg-muted px-2 py-1 rounded">
            {env.endpoint}
          </code>
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6"
                  onClick={handleCopy}
                >
                  {copied ? (
                    <Check className="h-3 w-3" />
                  ) : (
                    <Copy className="h-3 w-3" />
                  )}
                </Button>
              </TooltipTrigger>
              <TooltipContent>Copy endpoint URL</TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      </div>

      {/* Inherited servers from Main */}
      <InheritedServers
        servers={mainServers}
        overrides={env.overrides}
        onToggleOverride={(serverId) => toggleEnvOverride(envId, serverId)}
      />

      <Separator className="my-6" />

      {/* Environment-specific servers */}
      <EnvironmentServers
        servers={env.servers}
        onToggle={(serverId) => toggleEnvOverride(envId, serverId)}
        onRemove={(serverId) => removeEnvServer(envId, serverId)}
      />

      <Button
        variant="outline"
        className="mt-4"
        onClick={() => setShowAdd(true)}
      >
        <Plus className="h-4 w-4 mr-2" />
        Add Server
      </Button>

      <Separator className="my-6" />

      {/* Permissions */}
      <PermissionsPanel envId={envId} />

      {/* Danger zone */}
      <Separator className="my-6" />
      <div className="border border-destructive/30 rounded-lg p-4">
        <h3 className="text-sm font-medium text-destructive mb-2">
          Danger Zone
        </h3>
        <Button
          variant="destructive"
          size="sm"
          onClick={() => {
            if (confirm(`Delete environment "${env.name}"?`)) {
              deleteEnvironment(envId);
            }
          }}
        >
          <Trash2 className="h-4 w-4 mr-2" />
          Delete Environment
        </Button>
      </div>

      <AddServerDialog
        open={showAdd}
        onOpenChange={setShowAdd}
        onAdd={(server) => {
          addEnvServer(envId, server);
          setShowAdd(false);
        }}
      />
    </div>
  );
}
```

- [ ] **Step 6: Verify environment page**

Run:
```bash
cd crates/plugmux-app && npm run tauri dev
```
Expected: creating an environment navigates to its page. Shows inherited servers from Main with override toggles, environment-specific server list, permissions panel (collapsible), endpoint URL with copy, and delete button.

- [ ] **Step 7: Commit**

```bash
git add crates/plugmux-app/src/
git commit -m "feat(phase2): implement Environment page with inherited servers, permissions, and CRUD"
```

---

## Task 11: Settings Page

**Files:**
- Modify: `crates/plugmux-app/src/pages/SettingsPage.tsx`

- [ ] **Step 1: Implement SettingsPage**

`crates/plugmux-app/src/pages/SettingsPage.tsx`:
```tsx
import { useState, useEffect } from "react";
import { useEngine } from "@/hooks/useEngine";
import { getPort, setPort as setPortCmd } from "@/lib/commands";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";

export function SettingsPage() {
  const { status, toggle } = useEngine();
  const [port, setPort] = useState(4242);
  const [autostart, setAutostart] = useState(true);
  const [darkMode, setDarkMode] = useState(true);

  useEffect(() => {
    getPort().then(setPort);
    isEnabled().then(setAutostart);
  }, []);

  const handlePortChange = async (value: string) => {
    const num = parseInt(value, 10);
    if (!isNaN(num) && num > 0 && num < 65536) {
      setPort(num);
      await setPortCmd(num);
    }
  };

  const handleAutostartChange = async (checked: boolean) => {
    setAutostart(checked);
    if (checked) {
      await enable();
    } else {
      await disable();
    }
  };

  const handleThemeChange = (checked: boolean) => {
    setDarkMode(checked);
    document.documentElement.classList.toggle("dark", checked);
  };

  return (
    <div className="p-6 max-w-xl">
      <h1 className="text-lg font-semibold mb-6">Settings</h1>

      {/* Gateway */}
      <section className="space-y-4">
        <h2 className="text-sm font-medium">Gateway</h2>
        <div className="flex items-center justify-between">
          <div>
            <Label>Status</Label>
            <div className="flex items-center gap-2 mt-1">
              <Badge
                variant={status === "running" ? "default" : "secondary"}
              >
                {status}
              </Badge>
            </div>
          </div>
          <Button variant="outline" size="sm" onClick={toggle}>
            {status === "running" ? "Stop" : "Start"}
          </Button>
        </div>
        <div className="space-y-2">
          <Label htmlFor="port">Port</Label>
          <Input
            id="port"
            type="number"
            value={port}
            onChange={(e) => handlePortChange(e.target.value)}
            className="w-32"
          />
        </div>
      </section>

      <Separator className="my-6" />

      {/* Startup */}
      <section className="space-y-4">
        <h2 className="text-sm font-medium">Startup</h2>
        <div className="flex items-center justify-between">
          <Label htmlFor="autostart">Launch on login</Label>
          <Switch
            id="autostart"
            checked={autostart}
            onCheckedChange={handleAutostartChange}
          />
        </div>
      </section>

      <Separator className="my-6" />

      {/* Appearance */}
      <section className="space-y-4">
        <h2 className="text-sm font-medium">Appearance</h2>
        <div className="flex items-center justify-between">
          <Label htmlFor="dark-mode">Dark mode</Label>
          <Switch
            id="dark-mode"
            checked={darkMode}
            onCheckedChange={handleThemeChange}
          />
        </div>
      </section>

      <Separator className="my-6" />

      {/* About */}
      <section className="space-y-2">
        <h2 className="text-sm font-medium">About</h2>
        <p className="text-sm text-muted-foreground">plugmux v0.1.0</p>
        <p className="text-sm text-muted-foreground">MIT License</p>
      </section>
    </div>
  );
}
```

- [ ] **Step 2: Register autostart plugin in Rust**

In `crates/plugmux-app/src-tauri/src/lib.rs`, add autostart plugin to the builder chain (before `.setup()`):

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_autostart::init(
        tauri_plugin_autostart::MacosLauncher::LaunchAgent,
        None,
    ))
    // ... rest of builder
```

- [ ] **Step 3: Verify Settings page**

Run:
```bash
cd crates/plugmux-app && npm run tauri dev
```
Expected: Settings page shows gateway status with start/stop, port config, autostart toggle, dark/light theme toggle, and about info.

- [ ] **Step 4: Commit**

```bash
git add crates/plugmux-app/src/
git commit -m "feat(phase2): implement Settings page with port, autostart, theme, and engine toggle"
```

---

## Task 12: Pending Actions + confirm_action Tool

**Files:**
- Create: `crates/plugmux-core/src/pending_actions.rs`
- Modify: `crates/plugmux-core/src/lib.rs`
- Modify: `crates/plugmux-core/src/gateway/tools.rs`
- Modify: `crates/plugmux-core/src/gateway/router.rs`

- [ ] **Step 1: Write failing test for PendingActions**

Create `crates/plugmux-core/src/pending_actions.rs`:
```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde_json::Value;

const EXPIRY: Duration = Duration::from_secs(300); // 5 minutes

#[derive(Debug, Clone)]
pub struct PendingAction {
    pub env_id: String,
    pub server_id: String,
    pub action: String,
    pub created_at: Instant,
}

pub struct PendingActions {
    actions: HashMap<String, PendingAction>,
}

impl PendingActions {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    /// Store a pending action and return its ID.
    pub fn add(&mut self, env_id: &str, server_id: &str, action: &str) -> String {
        // Clean expired entries
        self.cleanup();

        let id = uuid::Uuid::new_v4().to_string();
        self.actions.insert(
            id.clone(),
            PendingAction {
                env_id: env_id.to_string(),
                server_id: server_id.to_string(),
                action: action.to_string(),
                created_at: Instant::now(),
            },
        );
        id
    }

    /// Confirm and remove a pending action. Returns the action if found and not expired.
    pub fn confirm(&mut self, action_id: &str) -> Option<PendingAction> {
        self.cleanup();
        self.actions.remove(action_id)
    }

    /// Find an existing pending action for the same env + server + action (for retry idempotency).
    pub fn find_existing(&self, env_id: &str, server_id: &str, action: &str) -> Option<&str> {
        for (id, pa) in &self.actions {
            if pa.env_id == env_id
                && pa.server_id == server_id
                && pa.action == action
                && pa.created_at.elapsed() < EXPIRY
            {
                return Some(id.as_str());
            }
        }
        None
    }

    fn cleanup(&mut self) {
        self.actions.retain(|_, pa| pa.created_at.elapsed() < EXPIRY);
    }
}

impl Default for PendingActions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_confirm() {
        let mut pa = PendingActions::new();
        let id = pa.add("env1", "server1", "enable_server");
        assert!(pa.confirm(&id).is_some());
        // Second confirm should return None
        assert!(pa.confirm(&id).is_none());
    }

    #[test]
    fn test_find_existing() {
        let mut pa = PendingActions::new();
        let id = pa.add("env1", "server1", "enable_server");
        let found = pa.find_existing("env1", "server1", "enable_server");
        assert_eq!(found, Some(id.as_str()));
        assert!(pa.find_existing("env1", "server1", "disable_server").is_none());
    }

    #[test]
    fn test_unknown_id_returns_none() {
        let mut pa = PendingActions::new();
        assert!(pa.confirm("nonexistent").is_none());
    }
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run:
```bash
cd plugmux && cargo test -p plugmux-core pending_actions
```
Expected: 3 tests pass.

- [ ] **Step 3: Add module to lib.rs**

Add to `crates/plugmux-core/src/lib.rs`:
```rust
pub mod pending_actions;
```

- [ ] **Step 4: Add uuid dependency to plugmux-core**

In `crates/plugmux-core/Cargo.toml`, add:
```toml
uuid = { version = "1", features = ["v4"] }
```

- [ ] **Step 5: Update GatewayTools to use PendingActions**

In `crates/plugmux-core/src/gateway/tools.rs`:

Add a `pending` field to `GatewayTools`:
```rust
use tokio::sync::Mutex;
use crate::pending_actions::PendingActions;

pub struct GatewayTools {
    pub config: Arc<RwLock<PlugmuxConfig>>,
    pub manager: Arc<ServerManager>,
    pub pending: Mutex<PendingActions>,
}
```

Update `GatewayTools::new()`:
```rust
pub fn new(config: Arc<RwLock<PlugmuxConfig>>, manager: Arc<ServerManager>) -> Self {
    Self {
        config,
        manager,
        pending: Mutex::new(PendingActions::new()),
    }
}
```

Update `check_permission` to return approval_required with action_id instead of a plain error:
```rust
async fn check_permission(
    &self,
    env_id: &str,
    server_id: &str,
    action: &str,
) -> Result<(), ProxyError> {
    let level = self.resolve_permission(env_id, server_id, action).await;
    match level {
        PermissionLevel::Allow => Ok(()),
        PermissionLevel::Approve => {
            let mut pending = self.pending.lock().await;
            // Return existing pending action if retry
            let action_id = if let Some(existing) = pending.find_existing(env_id, server_id, action) {
                existing.to_string()
            } else {
                pending.add(env_id, server_id, action)
            };
            Err(ProxyError::ApprovalRequired {
                action_id,
                message: format!(
                    "Action '{action}' on server '{server_id}' requires approval. Please confirm with the user."
                ),
            })
        }
        PermissionLevel::Deny => Err(ProxyError::ToolCallFailed(format!(
            "action '{action}' on server '{server_id}' is disabled"
        ))),
    }
}
```

Add a `confirm_action` method to `GatewayTools`:
```rust
pub async fn confirm_action(&self, action_id: &str) -> Result<(), ProxyError> {
    let mut pending = self.pending.lock().await;
    let action = pending.confirm(action_id).ok_or_else(|| {
        ProxyError::ToolCallFailed(
            "action expired or not found — please retry the original action".to_string(),
        )
    })?;

    // Drop pending lock before executing
    drop(pending);

    // Execute the confirmed action
    match action.action.as_str() {
        "enable_server" => {
            // Directly modify config (skip permission check this time)
            let mut cfg = self.config.write().await;
            if let Some(env) = cfg.environments.iter_mut().find(|e| e.id == action.env_id) {
                if let Some(ov) = env.overrides.iter_mut().find(|o| o.server_id == action.server_id) {
                    ov.enabled = Some(true);
                } else {
                    env.overrides.push(crate::config::ServerOverride {
                        server_id: action.server_id,
                        enabled: Some(true),
                        url: None,
                        permissions: None,
                    });
                }
            }
            Ok(())
        }
        "disable_server" => {
            let mut cfg = self.config.write().await;
            if let Some(env) = cfg.environments.iter_mut().find(|e| e.id == action.env_id) {
                if let Some(ov) = env.overrides.iter_mut().find(|o| o.server_id == action.server_id) {
                    ov.enabled = Some(false);
                } else {
                    env.overrides.push(crate::config::ServerOverride {
                        server_id: action.server_id,
                        enabled: Some(false),
                        url: None,
                        permissions: None,
                    });
                }
            }
            Ok(())
        }
        _ => Err(ProxyError::ToolCallFailed(format!(
            "unknown action: {}",
            action.action
        ))),
    }
}
```

- [ ] **Step 6: Add ApprovalRequired variant to ProxyError**

In `crates/plugmux-core/src/proxy/mod.rs`, add to the `ProxyError` enum:
```rust
#[error("approval required: {message}")]
ApprovalRequired {
    action_id: String,
    message: String,
},
```

- [ ] **Step 7: Add confirm_action to router**

In `crates/plugmux-core/src/gateway/router.rs`:

Add `confirm_action` to `handle_tools_list()` tools array:
```json
{
    "name": "confirm_action",
    "description": "Confirm a pending action that requires user approval",
    "inputSchema": {
        "type": "object",
        "properties": {
            "action_id": {
                "type": "string",
                "description": "The action ID returned by the approval_required response"
            }
        },
        "required": ["action_id"]
    }
}
```

Add the match arm in `handle_tools_call()`:
```rust
"confirm_action" => {
    let action_id = args
        .get("action_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing 'action_id' argument".to_string())?;

    tools
        .confirm_action(action_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(wrap_content("action confirmed and executed"))
}
```

Update the error handling in `handle_jsonrpc` to return `approval_required` as a structured tool result (not an error):
```rust
Err(err) => {
    // Check if this is an approval-required response
    if let ProxyError::ApprovalRequired { action_id, message } = &err {
        return (
            StatusCode::OK,
            Json(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string(&serde_json::json!({
                            "status": "approval_required",
                            "action_id": action_id,
                            "message": message,
                        })).unwrap(),
                    }]
                },
            })),
        );
    }
    // ... existing error handling
}
```

- [ ] **Step 7b: Refactor dispatch/handler to propagate ProxyError**

The existing `dispatch()` and `handle_tools_call()` return `Result<Value, String>`. They need to return `Result<Value, ProxyError>` so that `ApprovalRequired` can be caught in `handle_jsonrpc`.

In `crates/plugmux-core/src/gateway/router.rs`:

1. Change `dispatch` signature:
```rust
async fn dispatch(
    tools: &GatewayTools,
    env_id: &str,
    method: &str,
    params: &Value,
) -> Result<Value, ProxyError> {
```

2. Change `handle_tools_call` signature:
```rust
async fn handle_tools_call(
    tools: &GatewayTools,
    env_id: &str,
    params: &Value,
) -> Result<Value, ProxyError> {
```

3. Replace all `.map_err(|e| e.to_string())?` calls with plain `?` since they already return `ProxyError`.

4. Replace string error returns like `Err("missing 'name'...".to_string())` with `Err(ProxyError::Transport("missing 'name'...".to_string()))`.

5. Update `handle_jsonrpc` error handling to convert `ProxyError` to JSON-RPC:
```rust
let result = dispatch(&state.tools, &env_id, method, &params).await;

match result {
    Ok(value) => (
        StatusCode::OK,
        Json(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": value,
        })),
    ),
    Err(ProxyError::ApprovalRequired { action_id, message }) => (
        StatusCode::OK,
        Json(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string(&json!({
                        "status": "approval_required",
                        "action_id": action_id,
                        "message": message,
                    })).unwrap(),
                }]
            },
        })),
    ),
    Err(err) => {
        error!(method = %method, env = %env_id, error = %err, "JSON-RPC error");
        (
            StatusCode::OK,
            Json(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32603,
                    "message": err.to_string(),
                },
            })),
        )
    }
}
```

- [ ] **Step 8: Run all tests**

Run:
```bash
cd plugmux && cargo test
```
Expected: all existing tests pass, plus the 3 new pending_actions tests.

- [ ] **Step 9: Commit**

```bash
git add crates/plugmux-core/
git commit -m "feat(phase2): add confirm_action tool and pending approval system"
```

---

## Task 13: Window Close Behavior (Hide to Tray)

**Files:**
- Modify: `crates/plugmux-app/src-tauri/src/lib.rs`

- [ ] **Step 1: Add close-requested handler**

In `crates/plugmux-app/src-tauri/src/lib.rs`, inside the `.setup()` closure, after tray setup:

```rust
// Hide window on close instead of quitting (tray keeps running)
if let Some(window) = app.get_webview_window("main") {
    let w = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = w.hide();
        }
    });
}
```

- [ ] **Step 2: Verify behavior**

Run the app, close the window. Tray icon should remain. Click "Open plugmux" from tray → window re-appears.

- [ ] **Step 3: Commit**

```bash
git add crates/plugmux-app/src-tauri/src/lib.rs
git commit -m "feat(phase2): hide window to tray on close instead of quitting"
```

---

## Task 14: Final Integration + Autostart Default

**Files:**
- Modify: `crates/plugmux-app/src-tauri/src/lib.rs`

- [ ] **Step 1: Enable autostart by default on first run**

In `crates/plugmux-app/src-tauri/src/lib.rs`, inside `.setup()`, after autostart plugin init:

```rust
// Enable autostart by default on first launch
let handle_autostart = app.handle().clone();
tauri::async_runtime::spawn(async move {
    if let Ok(plugin) = handle_autostart.autostart() {
        if !plugin.is_enabled().unwrap_or(false) {
            let _ = plugin.enable();
        }
    }
});
```

Note: the autostart check may need to use the JS API instead. If the Rust API isn't available, skip this step and handle it from the React Settings page on first render.

- [ ] **Step 2: Full integration test**

Run:
```bash
cd crates/plugmux-app && npm run tauri dev
```

Verify:
1. App launches with tray icon
2. Engine starts automatically (check `http://localhost:4242/health` returns OK)
3. Sidebar shows Main, Environments, Catalog, Presets, Settings
4. Main page: can add/remove/toggle servers
5. Create environment: dialog works, navigates to environment page
6. Environment page: shows inherited servers, can override, has permissions panel, copy URL works
7. Settings: port change, autostart toggle, dark/light theme
8. Close window → hides to tray, engine keeps running
9. Tray right-click → menu shows, Open/Stop/Quit work
10. Tray left-click → window re-opens

- [ ] **Step 3: Run cargo clippy**

```bash
cd plugmux && cargo clippy -p plugmux-app -- -D warnings
```
Fix any warnings.

- [ ] **Step 4: Final commit**

```bash
git add .
git commit -m "feat(phase2): complete Tauri desktop app with tray, UI, and autostart"
```

---

## Summary

| Task | What it builds | Key files |
|------|---------------|-----------|
| 1 | Project scaffold | Cargo.toml, package.json, vite, tsconfig |
| 2 | Tailwind + shadcn/ui | globals.css, tailwind config, components.json |
| 3 | Engine module | engine.rs, events.rs |
| 4 | Tray icon | tray.rs |
| 5 | Tauri commands | commands.rs |
| 6 | Config file watcher | watcher.rs |
| 7 | TypeScript layer | commands.ts, hooks |
| 8 | Layout + routing | Sidebar, Layout, App.tsx |
| 9 | Main page | ServerCard, AddServerDialog, MainPage |
| 10 | Environment page | InheritedServers, PermissionsPanel, EnvironmentPage |
| 11 | Settings page | SettingsPage |
| 12 | Permission system | pending_actions.rs, confirm_action tool |
| 13 | Window close behavior | Hide to tray |
| 14 | Integration + autostart | Final wiring |
