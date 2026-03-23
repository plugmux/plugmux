# MCP Registries & Configuration Research

> Date: 2026-03-22

---

## Table of Contents

- [1. Global vs Local MCP Config Override](#1-global-vs-local-mcp-config-override)
- [2. Standard MCP Config Format](#2-standard-mcp-config-format)
- [3. Config File Locations by OS](#3-config-file-locations-by-os)
- [4. MCP Registries](#4-mcp-registries)
  - [4.1 Official MCP Registry (Anthropic)](#41-official-mcp-registry-anthropic)
  - [4.2 Smithery](#42-smithery)
  - [4.3 Comparison](#43-comparison)
- [5. MCP Server Manager Tools](#5-mcp-server-manager-tools)

---

## 1. Global vs Local MCP Config Override

| Tool | Global Config | Local/Project Override | Can Override? |
|---|---|---|---|
| **Claude Code** | `~/.claude/settings.json` | `.claude/settings.json` or `.claude/settings.local.json` | **Yes** — same key in project-level wins |
| **Claude Desktop** | Single config file | No project-level concept | **No** — one flat config |
| **Cursor** | User-level `mcp.json` | `.cursor/mcp.json` in workspace | **Yes** — workspace overrides user |
| **Windsurf** | `~/.codeium/windsurf/mcp_config.json` | `.windsurf/mcp.json` in workspace | **Yes** — workspace overrides global |
| **Codex (OpenAI)** | `~/.codex/config.json` | `codex.json` in project root | **Yes** — project overrides global |
| **ChatGPT Desktop** | No native MCP support yet | — | **N/A** |

For Claude Code, Cursor, Windsurf, and Codex you can define the same server name at both levels and the local one wins. This lets you point `my-server` to `localhost:3000` in dev and a production URL globally.

---

## 2. Standard MCP Config Format

The config shape is largely standardized across tools. Two transport types:

### Stdio (local process)

```json
{
  "mcpServers": {
    "my-server": {
      "command": "npx",
      "args": ["-y", "@company/mcp-server"],
      "env": {
        "API_KEY": "sk-..."
      }
    }
  }
}
```

### Remote (SSE / Streamable HTTP)

```json
{
  "mcpServers": {
    "my-server": {
      "url": "https://mcp.example.com/sse",
      "headers": {
        "Authorization": "Bearer ..."
      }
    }
  }
}
```

This `mcpServers` shape is the **de facto standard** — Claude Desktop established it, and most tools adopted it. Minor variations exist (Cursor adds `"disabled"` flag, Claude Code nests it under `"mcpServers"` in its settings structure), but the core schema is the same.

### Standard config form inputs

- **Name** (key)
- **Transport**: stdio vs remote
- If stdio: `command`, `args[]`, `env{}`
- If remote: `url`, `headers{}`

---

## 3. Config File Locations by OS

| Tool | macOS | Windows | Linux |
|---|---|---|---|
| **Claude Desktop** | `~/Library/Application Support/Claude/claude_desktop_config.json` | `%APPDATA%\Claude\claude_desktop_config.json` | `~/.config/Claude/claude_desktop_config.json` |
| **Claude Code** | `~/.claude/settings.json` | `%USERPROFILE%\.claude\settings.json` | `~/.claude/settings.json` |
| **Cursor** | `~/Library/Application Support/Cursor/User/globalStorage/cursor.mcp/mcp.json` | `%APPDATA%\Cursor\User\globalStorage\cursor.mcp\mcp.json` | `~/.config/Cursor/User/globalStorage/cursor.mcp/mcp.json` |
| **Windsurf** | `~/.codeium/windsurf/mcp_config.json` | `%USERPROFILE%\.codeium\windsurf\mcp_config.json` | `~/.codeium/windsurf/mcp_config.json` |
| **Codex** | `~/.codex/config.json` | `%USERPROFILE%\.codex\config.json` | `~/.codex/config.json` |

These are the defaults — users can change them, but if they haven't touched anything, these are where configs live.

---

## 4. MCP Registries

### 4.1 Official MCP Registry (Anthropic)

- **URL:** https://registry.modelcontextprotocol.io
- **GitHub:** https://github.com/modelcontextprotocol/registry
- **Launched:** September 2025 (preview)
- **Backed by:** Anthropic, GitHub, Microsoft, PulseMCP
- **Auth to read:** None required
- **Naming:** Reverse-DNS (`io.github.user/server`, `com.example/server`) with verified ownership

#### API Endpoints

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/v0.1/servers` | List/search servers |
| `GET` | `/v0.1/servers/{name}/versions` | All versions of a server |
| `GET` | `/v0.1/servers/{name}/versions/{version}` | Single version detail (`latest` supported) |
| `POST` | `/v0.1/publish` | Publish a server (auth required) |
| `PATCH` | `/v0.1/servers/{name}/versions/{version}/status` | Update version status |
| `PATCH` | `/v0.1/servers/{name}/status` | Update all versions status |
| `POST` | `/v0.1/validate` | Validate server.json without publishing |

#### Query params for `GET /servers`

- `limit` — items per page (default 30, max 100)
- `cursor` — pagination cursor
- `search` — case-insensitive substring on server name
- `version` — `latest` or exact version
- `updated_since` — RFC3339 timestamp filter
- `include_deleted` — boolean (default false)

#### Real response — maximally populated server

```json
{
  "server": {
    "$schema": "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json",
    "name": "ai.agenttrust/mcp-server",
    "description": "Identity, trust, and A2A orchestration for autonomous AI agents.",
    "title": "AgentTrust — Identity & Trust for A2A Agents",
    "version": "1.1.1",
    "websiteUrl": "https://agenttrust.ai",
    "repository": {
      "url": "https://github.com/agenttrust/mcp-server",
      "source": "github",
      "id": "891584339",
      "subfolder": "packages/..."
    },
    "icons": [
      {
        "src": "https://agenttrust.ai/icon.png",
        "sizes": ["96x96"],
        "mimeType": "image/png"
      }
    ],
    "packages": [
      {
        "registryType": "npm",
        "identifier": "@agenttrust/mcp-server",
        "version": "1.1.1",
        "transport": {
          "type": "stdio",
          "url": "...",
          "headers": [{ "name": "...", "value": "...", "isSecret": true }]
        },
        "environmentVariables": [
          {
            "name": "AGENTTRUST_API_KEY",
            "description": "Your API key from https://agenttrust.ai",
            "isRequired": true,
            "isSecret": true,
            "default": "...",
            "choices": ["a", "b"],
            "format": "string",
            "placeholder": "sk-..."
          }
        ],
        "packageArguments": [
          {
            "name": "--port",
            "description": "...",
            "type": "positional",
            "format": "number",
            "isRequired": false,
            "isSecret": false,
            "isRepeated": false,
            "choices": [],
            "default": "3000",
            "placeholder": "3000"
          }
        ],
        "runtimeArguments": [
          { "name": "--yes", "value": "" }
        ],
        "fileSha256": "..."
      }
    ],
    "remotes": [
      {
        "type": "streamable-http",
        "url": "https://mcp.example.com/mcp",
        "headers": [
          { "name": "Authorization", "value": "Bearer {api_key}", "isRequired": true, "isSecret": true }
        ],
        "variables": {}
      }
    ]
  },
  "_meta": {
    "io.modelcontextprotocol.registry/official": {
      "status": "active",
      "statusChangedAt": "2026-03-06T11:23:10.721165Z",
      "publishedAt": "2026-03-06T11:23:10.721165Z",
      "updatedAt": "2026-03-06T11:23:10.721165Z",
      "statusMessage": "...",
      "isLatest": true
    }
  }
}
```

#### List response wrapper

```json
{
  "servers": [],
  "metadata": {
    "count": 10,
    "nextCursor": "ai.agenttrust/mcp-server:1.1.1"
  }
}
```

#### All possible `server` fields (from OpenAPI spec)

| Field | Type | Required | Description |
|---|---|---|---|
| `$schema` | string | yes | JSON Schema URI |
| `name` | string | yes | Reverse-DNS identifier |
| `description` | string | yes | Max 100 chars |
| `title` | string | no | Display name |
| `version` | string | yes | Semver recommended |
| `websiteUrl` | string | no | Project website |
| `repository` | object | no | `url`, `source`, `id`, `subfolder` |
| `icons` | array | no | `src`, `sizes[]`, `mimeType` |
| `packages` | array | no | Install config (see below) |
| `remotes` | array | no | Remote endpoints |
| `_meta` | object | no | Vendor extensions |

#### Package fields

| Field | Type | Description |
|---|---|---|
| `registryType` | enum | `npm`, `pypi`, `oci`, `nuget`, `mcpb` |
| `identifier` | string | Package name or URL |
| `version` | string | Exact version (no ranges) |
| `transport` | object | `type`, `url`, `headers[]` |
| `environmentVariables` | array | `name`, `description`, `isRequired`, `isSecret`, `default`, `choices`, `format`, `placeholder` |
| `packageArguments` | array | `name`, `description`, `type` (positional/named), `format`, `isRequired`, `isSecret`, `isRepeated`, `choices`, `default`, `placeholder`, `value`, `variables` |
| `runtimeArguments` | array | `name`, `value` |
| `fileSha256` | string | Integrity hash |

---

### 4.2 Smithery

- **URL:** https://smithery.ai
- **GitHub:** https://github.com/smithery-ai
- **Status:** Independent, third-party registry (not official)
- **Auth to read:** API key required (from smithery.ai/account/api-keys)
- **Naming:** Flat slug (`exa`, `owner/slug`)

#### API Endpoints

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/servers` | Search/browse servers |
| `GET` | `/servers/{qualifiedName}` | Single server detail |

#### Query params for `GET /servers`

| Parameter | Type | Description |
|---|---|---|
| `q` | string | Full-text and semantic search |
| `page` | integer | Page number (1-indexed) |
| `pageSize` | integer | Results per page (default 10, max 100) |
| `topK` | integer | Candidate results before pagination (10-500) |
| `fields` | string | Comma-separated field filter |
| `ids` | array (UUID) | Filter by specific server IDs |
| `qualifiedName` | string | Exact match (deprecated) |
| `namespace` | string | Filter by namespace |
| `remote` | enum | Filter by remote status |
| `isDeployed` | enum | Filter by hosting status |
| `verified` | enum | Only verified servers |
| `ownerId` | string | Filter by owner |
| `repoOwner` | string | Filter by GitHub repo owner |
| `repoName` | string | Filter by GitHub repo name |
| `seed` | integer | Deterministic pagination seed |

#### Real response — server list

```json
{
  "servers": [
    {
      "id": "e4867c0f-3a35-4905-87d7-f74d0af2428f",
      "qualifiedName": "googledrive",
      "namespace": "googledrive",
      "slug": "",
      "displayName": "Google Drive",
      "description": "Google Drive is a cloud storage solution...",
      "iconUrl": "https://logos.composio.dev/api/googledrive",
      "verified": true,
      "useCount": 4231,
      "remote": true,
      "isDeployed": true,
      "createdAt": "2025-11-19T07:26:50.147Z",
      "homepage": "https://smithery.ai/servers/googledrive",
      "owner": "e99a8a47-0746-4982-aa0a-724bfcb87f88",
      "score": 0.014
    }
  ],
  "pagination": {
    "currentPage": 1,
    "pageSize": 1,
    "totalPages": 102,
    "totalCount": 102
  }
}
```

#### Real response — server detail (`/servers/exa`)

```json
{
  "qualifiedName": "exa",
  "displayName": "Exa Search",
  "description": "Fast, intelligent web search and web crawling...",
  "iconUrl": "https://api.smithery.ai/servers/exa/icon",
  "remote": true,
  "deploymentUrl": "https://exa.run.tools",
  "connections": [
    {
      "type": "http",
      "deploymentUrl": "https://exa.run.tools",
      "configSchema": {}
    }
  ],
  "security": null,
  "tools": [
    {
      "name": "web_search_exa",
      "description": "Search the web using Exa AI...",
      "inputSchema": {
        "type": "object",
        "properties": {
          "query": { "type": "string", "description": "Websearch query" },
          "numResults": { "type": "number", "description": "Number of results (default: 8)" }
        }
      }
    },
    {
      "name": "company_research_exa",
      "description": "Research companies using Exa AI...",
      "inputSchema": {
        "type": "object",
        "properties": {
          "companyName": { "type": "string" },
          "numResults": { "type": "number" }
        }
      }
    },
    {
      "name": "get_code_context_exa",
      "description": "Search and get relevant context for programming tasks...",
      "inputSchema": {
        "type": "object",
        "properties": {
          "query": { "type": "string" },
          "tokensNum": { "type": "number", "default": 5000, "minimum": 1000, "maximum": 50000 }
        }
      }
    }
  ],
  "resources": [
    {
      "name": "tools_list",
      "uri": "exa://tools/list",
      "description": "List of available Exa tools",
      "mimeType": "application/json"
    }
  ],
  "prompts": [
    {
      "name": "web_search_help",
      "description": "Get help with web search using Exa",
      "arguments": []
    },
    {
      "name": "code_search_help",
      "description": "Get help finding code examples and documentation",
      "arguments": []
    }
  ],
  "eventTopics": null
}
```

#### All possible fields — list item

| Field | Type | Description |
|---|---|---|
| `id` | UUID | Unique server ID |
| `qualifiedName` | string | `namespace/slug` format |
| `namespace` | string | Owning namespace |
| `slug` | string | Server slug |
| `displayName` | string | Human-readable name |
| `description` | string | Server description |
| `iconUrl` | string | Icon URL |
| `verified` | boolean | Verified status |
| `useCount` | integer | Installation/usage count |
| `remote` | boolean | Has remote endpoint |
| `isDeployed` | boolean | Hosted on Smithery infra |
| `createdAt` | ISO 8601 | Creation timestamp |
| `homepage` | string | Smithery page URL |
| `owner` | string | Owner user ID |
| `score` | float | Search relevance score |

#### All possible fields — detail

| Field | Type | Description |
|---|---|---|
| `qualifiedName` | string | Server identifier |
| `displayName` | string | Display name |
| `description` | string | Full description |
| `iconUrl` | string | Icon URL |
| `remote` | boolean | Has remote endpoint |
| `deploymentUrl` | string | Hosted URL |
| `connections` | array | `type`, `deploymentUrl`, `configSchema` |
| `security` | object/null | `scanPassed` boolean |
| `tools` | array | `name`, `description`, `inputSchema` |
| `resources` | array | `name`, `uri`, `description`, `mimeType` |
| `prompts` | array | `name`, `description`, `arguments[]` |
| `eventTopics` | array/null | Event topics |

---

### 4.3 Comparison

| Aspect | Official (Anthropic) | Smithery |
|---|---|---|
| **ID format** | Reverse-DNS (`io.github.user/server`) | Flat slug (`exa`, `owner/slug`) |
| **Versioning** | First-class (multiple versions, `latest`) | No versioning |
| **Install info** | `packages[]` with registry type, env vars, args | `connections[]` with configSchema |
| **Tool introspection** | Not included | Full `tools[]` with inputSchema |
| **Resources/Prompts** | Not included | Included |
| **Icons** | `icons[]` array with sizes/mimeType | Single `iconUrl` string |
| **Popularity** | No usage stats | `useCount` |
| **Search** | Simple substring on name | Semantic + full-text |
| **Pagination** | Cursor-based | Page-based |
| **Auth to read** | None | API key required |
| **Hosting** | No (registry only) | Yes (`isDeployed`, `deploymentUrl`) |
| **Security scan** | Not exposed | `security.scanPassed` |
| **Status lifecycle** | `active` / `deprecated` / `deleted` | `verified` boolean |

**Key takeaway:** The official registry is focused on **distribution metadata** (how to install and run), while Smithery is focused on **runtime metadata** (what tools/resources the server exposes) plus managed hosting.

---

## 5. MCP Server Manager Tools

| Tool | Platform | Source | Notes |
|---|---|---|---|
| [MCP Dock (App Store)](https://apps.apple.com/us/app/mcp-dock/id6748305262) | macOS | Proprietary (simpleswift) | Native SwiftUI, card-based dashboard, v0.1.1 |
| [MCP Dock (OldJii)](https://github.com/OldJii/mcp-dock) | Cross-platform | Proprietary | Aggregates Official + Smithery, one-click install, Skills Store |
| [MCP Dockmaster](https://github.com/dcSpark/mcp-dockmaster) | Mac/Win/Linux | Open source | Desktop app + CLI + library |
| [MyMCP](https://www.josh.ing/mymcp) | macOS | Open source | Supports Claude/Cursor/VS Code |
| [MCP Server Manager](https://github.com/vlazic/mcp-server-manager) | Cross-platform | Open source | Go binary + HTMX web UI |

---

## Sources

- [Official MCP Registry](https://registry.modelcontextprotocol.io/)
- [Official Registry OpenAPI Spec](https://registry.modelcontextprotocol.io/openapi.yaml)
- [Official Registry GitHub](https://github.com/modelcontextprotocol/registry)
- [Official Registry API Docs](https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/api/official-registry-api.md)
- [Generic Registry API Spec](https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/api/generic-registry-api.md)
- [MCP Registry Blog Post](https://blog.modelcontextprotocol.io/posts/2025-09-08-mcp-registry-preview/)
- [Smithery](https://smithery.ai/)
- [Smithery AI Overview (WorkOS)](https://workos.com/blog/smithery-ai)
- [MCP Dock (OldJii) GitHub](https://github.com/OldJii/mcp-dock)
