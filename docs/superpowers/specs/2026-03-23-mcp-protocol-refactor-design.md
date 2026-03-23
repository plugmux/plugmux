# Plugmux MCP Protocol Refactor — Design Spec

## Summary

Refactor plugmux from a meta-tool gateway (5 custom tools with indirection) to a transparent MCP multiplexer that proxies all MCP primitives — tools, resources, prompts, apps, sampling, elicitation, and roots — from backend servers through a single endpoint per environment.

## Goals

1. Flatten all backend server tools into a standard `tools/list` response (namespaced)
2. Proxy resources, prompts, and apps from backend servers
3. Relay sampling and elicitation (server-to-agent) and roots (agent-to-server)
4. Rename "default" environment to "global"
5. Separate plugmux management (global URL) from pure proxy (project URLs)
6. Log all request/response traffic to an embedded database
7. Detect and track agents through the gateway

## Non-Goals (this phase)

- Favorites / server detection from agent configs
- Environment inheritance
- Config/catalog migration to embedded DB (see Future Work)
- CLI updates (follows after core is stable)
- UI updates for log filtering

---

## Architecture

Three layers in `plugmux-core`:

```
Agent (Claude Code, Cursor, etc.)
  |
  v
+----------------------------------+
|  Gateway Layer                   |
|  - HTTP router (Axum)            |
|  - Logging middleware            |
|  - Agent detection               |
|  - Notifications dispatch        |
+---------------+------------------+
                | dispatch by env
        +-------+--------+
        v                v
+------------+   +---------------+
| Plugmux    |   | Proxy Layer   |
| Layer      |   |               |
|            |   | - Aggregate   |
| Own tools  |   |   tools       |
| Own res.   |   | - Aggregate   |
| Server     |   |   resources   |
| mgmt       |   | - Aggregate   |
|            |   |   prompts     |
|            |   | - Namespace   |
|            |   | - Route calls |
|            |   | - Relay       |
|            |   |   sampling/   |
|            |   |   elicitation/|
|            |   |   roots       |
+------------+   +---------------+
                       |
               +-------+-------+
               v       v       v
            [figma] [github]  [fs]
            backend servers
```

### URL Model

| URL | Purpose | What agent sees |
|-----|---------|-----------------|
| `/env/global` | Plugmux management | `plugmux__*` tools + `plugmux://` resources |
| `/env/{project}` | Pure proxy | Flattened tools/resources/prompts from backend servers |

- `/env/global` is added to the agent's **global** config (e.g., `~/.claude.json`)
- `/env/{project}` is added **per project** (e.g., `.mcp.json` in project root)
- The agent sees both URLs simultaneously
- Best practice: do NOT add MCP servers to the global environment — only plugmux management lives there. All MCP servers go into project environments.

### Routing Rule

On `/env/global`:
- All methods → Plugmux layer (own tools, own resources)

On `/env/{project}`:
- `tools/call` → parse `{server_id}__` prefix → strip prefix → forward to backend
- `resources/read` → parse URI scheme → forward to backend
- `prompts/get` → parse `{server_id}__` prefix → strip prefix → forward to backend
- `sampling/createMessage` (from backend) → relay to agent
- `elicitation/create` (from backend) → relay to agent
- `notifications/roots/updated` (from agent) → broadcast to all backends

---

## Layer 1: Proxy Layer

**Location:** `plugmux-core/src/proxy_layer/` (new module)

Separate from existing `proxy/` which handles individual client connections. The proxy layer operates at the environment level — aggregating across all backend servers in an environment.

### Tool Aggregation & Routing

**`tools/list`** — for each running backend server in the environment:
1. Call `list_tools()` on the backend
2. Prefix each tool name: `{server_id}__{original_name}`
3. Prefix the description with `[{server_name}]` for clarity
4. Preserve `inputSchema`, `outputSchema`, and `annotations` as-is (including app UI metadata)
5. Merge all into one flat list

**`tools/call`** — parse the tool name:
1. Split on first `__` → `server_id` + `original_tool_name`
2. Look up backend server by `server_id`
3. Forward call with original tool name and arguments
4. Return response as-is

### Resource Aggregation & Routing

**`resources/list`** — for each backend server:
1. Call `list_resources()` on the backend
2. Prefix URIs: `{server_id}://{original_path}` (or preserve if already namespaced)
3. Merge into flat list

**`resources/read`** — parse the URI:
1. Extract server_id from URI prefix
2. Forward to backend with original URI
3. Return response as-is

**`resources/subscribe`** — forward subscription to the appropriate backend, relay `notifications/resources/updated` back to agent.

### Prompt Aggregation & Routing

Same pattern as tools:
- `prompts/list` → aggregate, prefix with `{server_id}__`
- `prompts/get` → parse prefix, forward to backend

### App Proxying

Apps are tools with `annotations.ui` metadata. The proxy layer handles them automatically through tool aggregation — no special logic needed. The `annotations.ui.resourceUri` values are rewritten to route through plugmux so the client fetches app HTML via plugmux.

### Pass-Through Relay

**Sampling** (server → agent direction):
- Backend server sends `sampling/createMessage` to plugmux
- Plugmux relays to the connected agent
- Agent responds → plugmux relays back to backend

**Elicitation** (server → agent → user direction):
- Backend server sends `elicitation/create` to plugmux
- Plugmux relays to agent → agent shows UI to user
- User responds → plugmux relays back to backend

**Roots** (agent → server direction):
- Agent sends `notifications/roots/updated` or roots during `initialize`
- Plugmux broadcasts to all backend servers in the environment
- Plugmux also stores roots locally for agent/session tracking

### McpClient Trait Extension

```rust
// Existing
async fn list_tools(&self) -> Result<Vec<ToolInfo>, ProxyError>;
async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ProxyError>;
async fn health_check(&self) -> bool;
async fn shutdown(&mut self) -> Result<(), ProxyError>;

// New
async fn list_resources(&self) -> Result<Vec<ResourceInfo>, ProxyError>;
async fn read_resource(&self, uri: &str) -> Result<Value, ProxyError>;
async fn subscribe_resource(&self, uri: &str) -> Result<(), ProxyError>;
async fn list_prompts(&self) -> Result<Vec<PromptInfo>, ProxyError>;
async fn get_prompt(&self, name: &str, args: Value) -> Result<Value, ProxyError>;
async fn send_roots(&self, roots: Value) -> Result<(), ProxyError>;
```

### New Data Types

```rust
pub struct ResourceInfo {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

pub struct PromptInfo {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
}

pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}
```

---

## Layer 2: Plugmux Layer

**Location:** `plugmux-core/src/plugmux_layer/` (new module, replaces `gateway/tools.rs`)

Handles plugmux's own MCP interface — only served on `/env/global`.

### Plugmux Tools

| Tool | Description |
|------|-------------|
| `plugmux__list_servers` | List all servers (catalog + custom) with health status |
| `plugmux__enable_server` | Add a server to an environment |
| `plugmux__disable_server` | Remove a server from an environment |
| `plugmux__list_environments` | List all environments |
| `plugmux__server_status` | Detailed status of a specific server |
| `plugmux__confirm_action` | Confirm a pending approval |

### Plugmux Resources

| URI | Description |
|-----|-------------|
| `plugmux://servers` | All servers with health, tool count, connection status |
| `plugmux://environments` | All environments with their server lists |
| `plugmux://agents` | Connected/detected agents and their status |
| `plugmux://logs/recent` | Recent activity from the log database |

---

## Layer 3: Gateway Layer

**Location:** `plugmux-core/src/gateway/` (refactored)

### Router Changes

The router becomes a thin dispatcher:

```rust
async fn dispatch(env_id: &str, method: &str, params: &Value) -> Result<Value, ProxyError> {
    match method {
        "initialize" => handle_initialize(env_id),
        "notifications/initialized" => Ok(Value::Null),
        "ping" => Ok(json!({})),

        // For /env/global → plugmux layer
        // For /env/{project} → proxy layer
        "tools/list" => route_tools_list(env_id),
        "tools/call" => route_tools_call(env_id, params),
        "resources/list" => route_resources_list(env_id),
        "resources/read" => route_resources_read(env_id, params),
        "resources/subscribe" => route_resources_subscribe(env_id, params),
        "prompts/list" => route_prompts_list(env_id),
        "prompts/get" => route_prompts_get(env_id, params),
        "notifications/roots/updated" => handle_roots_updated(env_id, params),
        _ => Err(ProxyError::Transport(format!("unknown method: {method}"))),
    }
}
```

Each `route_*` function checks if `env_id == "global"` → plugmux layer, else → proxy layer.

### Initialize Response

Updated to advertise full capabilities:

```json
{
    "protocolVersion": "2025-03-26",
    "capabilities": {
        "tools": { "listChanged": true },
        "resources": { "subscribe": true, "listChanged": true },
        "prompts": { "listChanged": true }
    },
    "serverInfo": {
        "name": "plugmux",
        "version": "0.2.0"
    }
}
```

### Logging Middleware

Every request/response is logged to the embedded database:

```rust
struct LogEntry {
    id: String,                    // UUID
    timestamp: DateTime<Utc>,
    env_id: String,
    method: String,
    params: Value,                 // request params
    result: Option<Value>,         // response (truncated for large payloads)
    error: Option<String>,         // error message if failed
    duration_ms: u64,
    agent: Option<AgentInfo>,      // detected agent
    roots: Option<Vec<String>>,    // workspace roots if available
}

struct AgentInfo {
    user_agent: Option<String>,    // from HTTP headers
    agent_id: Option<String>,      // detected agent type (claude-code, cursor, etc.)
    session_id: String,            // per-connection session
}
```

### Agent Detection & Status Updates

- Parse `User-Agent` header and known agent patterns
- On first request from a manually configured agent → update its status from "unknown" (yellow) to "active" (green) in agent config
- Store agent info per session for log correlation

### Notifications

When servers are enabled/disabled:
- Gateway sends `notifications/tools/list_changed` to all active connections on affected environments
- Requires tracking active SSE/WebSocket connections per environment (connection registry)

---

## Config Changes

### Rename default → global

```rust
// config.rs
pub fn ensure_global(config: &mut Config) {
    // If "default" exists, migrate to "global"
    if let Some(env) = config.environments.iter_mut().find(|e| e.id == "default") {
        env.id = "global".to_string();
        env.name = "Global".to_string();
    }
    // Ensure global exists
    if !config.environments.iter().any(|e| e.id == "global") {
        config.environments.insert(0, Environment {
            id: "global".to_string(),
            name: "Global".to_string(),
            servers: Vec::new(),
        });
    }
}

// Error type
CannotDeleteDefault → CannotDeleteGlobal
```

### Plugmux always present

The `plugmux` server is not stored in config — it's injected at runtime. When `tools/list` is called on `/env/global`, plugmux tools are always included regardless of what's in `config.environments[global].servers`.

---

## Embedded Database

**Crate:** `redb` (pure Rust, embedded, zero-config, ACID) or `sled` — to be evaluated during implementation.

**Location:** `~/.config/plugmux/plugmux.db`

**Tables (this phase):**
- `logs` — request/response log entries (keyed by UUID, indexed by timestamp + env_id)

**Future phases (noted, not implemented):**
- `config` — migrate config.json into DB
- `catalog` — migrate catalog into DB
- `agents` — migrate agent state into DB
- `sync` — server sync metadata

> Note: The embedded DB is introduced in this phase for logging. Future phases should use this same DB instance for config/catalog/agent storage migration, leveraging its sync capabilities.

---

## Namespacing Convention

| Primitive | Pattern | Example |
|-----------|---------|---------|
| Tools | `{server_id}__{tool_name}` | `figma__get_screenshot` |
| Resources | `{server_id}://{path}` | `figma://designs/recent` |
| Prompts | `{server_id}__{prompt_name}` | `figma__design-to-code` |
| Apps | Same as tools (apps are tools with UI annotations) | `figma__dashboard` |

Double underscore `__` is chosen because:
- Single `_` is common in tool names (`get_screenshot`)
- `/` conflicts with URI paths
- `::` conflicts with Rust syntax and some JSON parsers
- `__` is unambiguous and easy to split on first occurrence

---

## Module Structure (after refactor)

```
plugmux-core/src/
  config.rs              (updated: default → global)
  manager.rs             (updated: McpClient trait extensions)
  server.rs              (unchanged)
  environment.rs         (unchanged)

  proxy/                 (existing — individual client connections)
    mod.rs               (updated: new trait methods)
    stdio.rs             (updated: implement new methods)
    http_sse.rs          (updated: implement new methods)

  proxy_layer/           (NEW — environment-level aggregation)
    mod.rs               (ProxyLayer struct)
    tools.rs             (aggregate + route tools)
    resources.rs         (aggregate + route resources)
    prompts.rs           (aggregate + route prompts)
    relay.rs             (sampling, elicitation, roots)

  plugmux_layer/         (NEW — replaces gateway/tools.rs)
    mod.rs               (PlugmuxLayer struct)
    tools.rs             (plugmux__* tool implementations)
    resources.rs         (plugmux:// resource implementations)

  gateway/               (refactored)
    router.rs            (thin dispatcher, env-aware routing)
    logging.rs           (NEW — middleware, LogEntry, DB writes)
    agent_detect.rs      (NEW — agent detection from headers)
    connections.rs       (NEW — active connection registry for notifications)

  db/                    (NEW — embedded database)
    mod.rs               (DB initialization, shared handle)
    logs.rs              (log entry CRUD)

  catalog.rs             (unchanged)
  custom_servers.rs      (unchanged)
  agents/                (updated: status tracking from gateway)
```

---

## Testing Strategy

- **Proxy layer:** Unit tests with mock McpClient implementations — verify namespacing, routing, aggregation
- **Plugmux layer:** Unit tests similar to existing gateway/tools.rs tests
- **Gateway:** Integration tests — send JSON-RPC requests, verify routing to correct layer
- **Logging:** Verify log entries written to DB for each request
- **Agent detection:** Unit tests for User-Agent parsing patterns

---

## Migration Path

1. Build proxy layer and plugmux layer as new modules
2. Update McpClient trait and implementations (stdio, http_sse)
3. Refactor router to dispatch to new layers
4. Add logging middleware and embedded DB
5. Add agent detection
6. Remove old `gateway/tools.rs` (replaced by plugmux_layer)
7. Update tests
8. CLI updates follow in a separate phase
