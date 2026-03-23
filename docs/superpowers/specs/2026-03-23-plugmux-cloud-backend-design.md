# plugmux Cloud Backend — Design Spec

**Author:** Lasha Kvantaliani
**Date:** 2026-03-23
**Status:** Draft v2 (post-review)

---

## 1. Overview

A Cloudflare-hosted backend that serves plugmux's curated MCP server catalog, handles user authentication, and syncs user configuration across devices. The plugmux desktop app keeps redb for offline-capable local storage and communicates with the backend via REST API.

### Goals

- Serve a curated, quality-first MCP server catalog (~100-150 servers)
- Authenticate users via GitHub OAuth (Google OAuth planned for later)
- Sync user config (environments, servers, favorites, custom servers) across devices without data loss
- Serve SVG icons efficiently via R2 with client-side caching
- Keep the backend private (separate repo from the open-source app)

### Non-Goals

- Mirroring Smithery's full 7,300+ server catalog — quality over quantity
- Real-time sync (WebSockets, live updates) — periodic pull is sufficient
- Environment presets — deferred
- Billing/payments — free for now
- Syncing environment variables / secrets — sensitive, machine-specific, deferred

### Prerequisites

- Extract `plugmux-types` crate from `plugmux-core` (shared types: `CatalogEntry`, `Transport`, `Connectivity`, sync action types). Must be lightweight and WASM-compatible.

---

## 2. Catalog Strategy

### Curation Model

plugmux maintains its own curated server registry. Servers are hand-picked for quality:

- **Official/first-party MCP servers** (Anthropic, GitHub, Figma, etc.) — always included
- **High-quality community servers** — reviewed and approved manually
- **Community servers** exist but are flagged and visually de-emphasized in the UI

Initial metadata (SVGs, descriptions) can be sourced from Smithery, but plugmux owns and maintains the data going forward. Smithery is a data source for bootstrapping, not a runtime dependency.

### Online/Offline Behavior

When online, the app fetches the catalog from the API and caches it in redb. When offline, the app uses the cached catalog. The bundled `catalog/servers.json` serves as a fallback if no cached catalog exists (fresh install, no network).

### Catalog Entry Schema (D1)

```sql
CREATE TABLE catalog_servers (
    id              TEXT PRIMARY KEY,       -- "figma", "github"
    name            TEXT NOT NULL,          -- "Figma"
    description     TEXT NOT NULL,          -- short description
    icon_key        TEXT,                   -- R2 object key "figma.svg"
    icon_hash       TEXT,                   -- hash for cache busting
    categories      TEXT NOT NULL,          -- JSON array: ["design", "dev-tools"]
    transport       TEXT NOT NULL,          -- "stdio" | "http"
    command         TEXT,                   -- for stdio servers
    args            TEXT,                   -- JSON array, for stdio servers
    url             TEXT,                   -- for http servers
    connectivity    TEXT NOT NULL,          -- "local" | "online"
    official        INTEGER NOT NULL,       -- 1 = first-party, 0 = community
    tool_count      INTEGER,               -- nullable, updated by reconciler
    security_score  TEXT,                   -- nullable, updated by reconciler
    added_at        TEXT NOT NULL,          -- ISO 8601
    updated_at      TEXT NOT NULL           -- ISO 8601
);
```

Notes:
- `categories` is a JSON array (multi-category support), matching the frontend TypeScript type.
- `args` is a JSON array stored as TEXT. API validates JSON on read/write.
- `tool_count` and `security_score` are maintained by a manual reconciliation script (see Section 8).

---

## 3. User & Device Model

### Users (D1)

```sql
CREATE TABLE users (
    id              TEXT PRIMARY KEY,       -- UUID
    github_id       TEXT UNIQUE NOT NULL,   -- GitHub user ID
    github_username TEXT NOT NULL,
    email           TEXT,                   -- nullable, from GitHub profile
    created_at      TEXT NOT NULL,
    last_sync_at    TEXT
);
```

### Devices (D1)

```sql
CREATE TABLE devices (
    id              TEXT PRIMARY KEY,       -- UUID, generated on first app launch
    user_id         TEXT NOT NULL REFERENCES users(id),
    name            TEXT NOT NULL,          -- auto-detected from OS hostname
    last_seen_at    TEXT NOT NULL
);
```

### Favorites (D1)

```sql
CREATE TABLE favorites (
    user_id         TEXT NOT NULL REFERENCES users(id),
    server_id       TEXT NOT NULL,
    added_at        TEXT NOT NULL,
    PRIMARY KEY (user_id, server_id)
);
```

Materialized from the change log. Kept as a table for fast reads — the change log is the source of truth, but this table is rebuilt/updated as changes are applied.

### Device Identity Flow

1. First app launch — generate UUID, store in redb
2. User logs in with GitHub — device registers to their account
3. Device name auto-detected from hostname, user can rename in settings
4. App works fully offline without login — device ID exists locally regardless

---

## 4. Sync Model

### Problem

User adds servers on Machine A, different servers on Machine B. Simple push/pull overwrites data.

### Solution: Change-Log Based Merge

Instead of syncing the whole config as a blob, sync individual operations. Each change is an append-only log entry. Devices pull changes since their last sync and merge — no overwrites.

### Changes Table (D1)

```sql
CREATE TABLE changes (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         TEXT NOT NULL REFERENCES users(id),
    device_id       TEXT NOT NULL REFERENCES devices(id),
    timestamp       TEXT NOT NULL,          -- ISO 8601, when change happened on device
    action          TEXT NOT NULL,          -- see action enum below
    payload         TEXT NOT NULL           -- JSON blob
);

CREATE INDEX idx_changes_user_id ON changes(user_id, id);
```

### Sync Cursor

Sync uses the auto-increment `id` column, not timestamps. This avoids clock skew issues between devices. Each device stores its `last_seen_change_id` in redb.

### Actions

| Action | Payload |
|--------|---------|
| `add_server_to_env` | `{env: "backend", server_id: "redis"}` |
| `remove_server_from_env` | `{env: "backend", server_id: "redis"}` |
| `create_env` | `{name: "backend", servers: ["postgres", "redis"]}` |
| `delete_env` | `{env: "backend"}` |
| `update_env` | `{env: "backend", field: "<field>", value: "<value>"}` |
| `add_favorite` | `{server_id: "figma"}` |
| `remove_favorite` | `{server_id: "figma"}` |
| `add_custom_server` | `{id: "my-tool", name: "...", transport: "stdio", ...}` |
| `update_custom_server` | `{id: "my-tool", field: "<field>", value: "..."}` |
| `remove_custom_server` | `{id: "my-tool"}` |

`update_env` supported fields: `name`.

`update_custom_server` supported fields: `name`, `description`, `command`, `args`, `url`, `enabled`.

**Custom server path caveat:** stdio servers with absolute paths in `command` or `args` are machine-specific. When syncing custom servers, the app should warn the user if a synced server has a path that doesn't exist on the current machine. The server config syncs as metadata — the user may need to adjust paths locally.

### Conflict Resolution Rules

- **Add + Add** — both apply (set union). No conflict. `create_env` is idempotent — if environment already exists, merge servers.
- **Delete + Add** — last-write-wins (by change `id` order). If an add arrives after a delete, it resurrects the entity. No silent data loss.
- **Rename on two devices** — last-write-wins by change `id`.
- **Same server removed on both** — idempotent, no issue.

### Offline Queue

Pending changes are stored in redb under a `sync_queue` key. Changes survive app restarts. On next sync, queued changes are pushed before pulling.

### Change Log Compaction

To prevent unbounded growth:
- After a successful `GET /sync/snapshot`, the server may delete changes older than 90 days for that user.
- `GET /sync/pull` returns at most 1,000 changes. If more exist, the client should call `/sync/snapshot` instead for a full rebuild.

### Sync Flow

1. Device comes online, calls `GET /sync/pull?since_id=<last_seen_change_id>`
2. Server returns all changes for this user with `id > since_id` (excluding changes from the requesting device)
3. Device applies changes to local redb
4. Device pushes its queued local changes via `POST /sync/push`
5. Device updates its `last_seen_change_id` in redb
6. New device setup: `GET /sync/snapshot` returns full materialized config

### Snapshot Response Format

```json
{
  "environments": [
    {
      "name": "backend",
      "servers": ["postgres", "redis"],
      "overrides": {}
    }
  ],
  "favorites": ["figma", "github"],
  "custom_servers": [
    {
      "id": "my-tool",
      "name": "My Tool",
      "transport": "stdio",
      "command": "node",
      "args": ["/path/to/server.js"],
      "connectivity": "local"
    }
  ],
  "last_change_id": 4523
}
```

---

## 5. Authentication

### GitHub OAuth (Launch)

Standard OAuth 2.0 redirect flow with CSRF protection:

1. App opens `GET /auth/github` — Worker generates a random `state` parameter, stores it in a short-lived KV entry, redirects to GitHub authorization with `state`
2. User authorizes — GitHub redirects to callback with code and `state`
3. `GET /auth/github/callback` — Worker verifies `state` matches KV entry, exchanges code for access token, creates/finds user in D1, returns a session token (JWT)
4. App stores JWT in redb, includes as Bearer token in protected requests

### Google OAuth (Future)

Same pattern. Deferred until company/project registration is complete. Add a `google_id` column to users and a second OAuth flow.

### Sessions

- Stateless JWT with 30-day expiration, signed with a Cloudflare Worker Secret
- Logout is client-side only (delete JWT from redb) — no server-side session table needed
- If expired, user re-authenticates via OAuth
- No silent refresh — token is valid until expiry

### Required Cloudflare Secrets

- `GITHUB_CLIENT_ID` — GitHub OAuth app client ID
- `GITHUB_CLIENT_SECRET` — GitHub OAuth app client secret
- `JWT_SECRET` — signing key for JWT tokens
- `ADMIN_SECRET` — API key for admin/reconciler endpoints

---

## 6. Icon Loading

### Strategy

Icons are not bundled in the app binary. They are served from R2 and cached locally.

1. App fetches catalog from API — each entry includes `icon_key` and `icon_hash`
2. App checks local cache at `~/.config/plugmux/icons/`
3. If icon missing or hash differs — fetch from `GET /icons/:id.svg`, save locally
4. If offline or loading — show fallback (initial letter + color hash, already built)
5. R2 egress is free and unlimited

### R2 Structure

```
plugmux-icons/
├── figma.svg
├── github.svg
├── postgres.svg
└── ...
```

---

## 7. API Architecture

### Stack

| Component | Technology |
|-----------|-----------|
| Compute | Cloudflare Workers (Rust/WASM preferred, TypeScript fallback) |
| Database | Cloudflare D1 (SQLite) |
| Object Storage | Cloudflare R2 |
| Auth | GitHub OAuth, built into Worker |
| Domain | Attached to existing Cloudflare account |

**Worker language note:** Rust/WASM is preferred to match the codebase. However, Cloudflare free tier has a 1 MB compressed Worker size limit. If the Rust/WASM binary exceeds this, the Worker should be written in TypeScript instead — the shared types from `plugmux-types` can be mirrored as TypeScript interfaces. This is a pragmatic fallback, not a failure.

### Endpoints

**Public (no auth):**

```
GET  /v1/catalog/servers              — list, search, filter (paginated: ?limit=&cursor=)
GET  /v1/catalog/servers/:id          — single server detail
GET  /v1/icons/:id.svg                — SVG from R2
GET  /v1/health                       — status check
```

**Auth:**

```
GET  /v1/auth/github                  — initiate OAuth redirect
GET  /v1/auth/github/callback         — exchange code, return JWT
```

**Protected (Bearer token):**

```
POST /v1/devices/register             — register device to user account
GET  /v1/sync/pull?since_id=<id>      — pull changes since last sync
POST /v1/sync/push                    — push change log entries
GET  /v1/sync/snapshot                — full config for new device setup
GET  /v1/user/profile                 — account info
DELETE /v1/user/account               — delete account (requires re-auth)
```

**Admin (admin secret header):**

```
PUT  /v1/admin/catalog/servers/:id    — create/update catalog entry
DELETE /v1/admin/catalog/servers/:id  — remove catalog entry
POST /v1/admin/icons/:id.svg          — upload icon to R2
```

### Free Tier Capacity

| Resource | Free Limit | Expected Usage |
|----------|-----------|----------------|
| D1 | 500 MB per DB | Few MB (catalog + users + changes) |
| R2 | 10 GB storage, 10M reads/mo | ~3 MB of SVGs |
| R2 egress | Unlimited, free | Icon serving |
| Workers | 100K requests/day | Catalog + sync calls |

Easily within free tier for the foreseeable future.

---

## 8. Reconciliation Script

A local desktop script (Python or Rust) run manually (~monthly) to keep catalog metadata fresh.

**What it does:**

1. Connects to each catalog server (stdio: spawn process, http: connect to URL)
2. Calls `tools/list` to get current tool count
3. Optionally runs MCP-Scan (Invariant open-source) for security analysis
4. Diffs results against current D1 data via the admin API
5. Uploads changes to `PUT /v1/admin/catalog/servers/:id`
6. Logs any servers that failed to connect or changed significantly

**Not a Cloudflare service** — runs on any machine with the required runtimes (Node, Python, etc.). Can run on sweenkserver, local laptop, or anywhere.

**Future consideration:** Could be automated as a Coolify container on sweenkserver with a cron schedule, or triggered manually when reviewing new servers to add.

---

## 9. Repository Structure

```
~/Development/lasharela/
├── plugmux/                    (public repo)
│   ├── crates/
│   │   ├── plugmux-core/      — gateway, redb, sync client
│   │   ├── plugmux-types/     — shared types (catalog, sync actions) [NEW]
│   │   ├── plugmux-cli/       — CLI binary
│   │   └── plugmux-app/       — Tauri desktop app
│   └── catalog/                — bundled fallback catalog data
│
└── plugmux-api/                (private repo)
    ├── src/
    │   ├── routes/             — catalog, auth, sync, devices, admin
    │   ├── models/             — D1 queries, types
    │   ├── auth/               — GitHub OAuth flow
    │   └── lib.rs              — Worker entrypoint
    ├── migrations/             — D1 schema migrations
    ├── icons/                  — source SVGs (deployed to R2)
    ├── reconciler/             — local script for metadata updates
    ├── wrangler.toml           — Cloudflare Worker config
    └── Cargo.toml              — depends on plugmux-types via git
```

### Shared Types

`plugmux-types` is a lightweight crate in the public repo containing only struct definitions shared between client and API. It must:

- Be `no_std`-compatible or at minimum compile to `wasm32-unknown-unknown`
- Only depend on `serde` and `serde_json`
- Not pull in any of `plugmux-core`'s heavy dependencies (tokio, axum, rmcp, etc.)

Types to extract from `plugmux-core`:
- `CatalogEntry` — server catalog schema (updated with `icon_key`, `icon_hash`, `categories`, `official`, `tool_count`, `security_score`, `added_at`, `updated_at` fields)
- `Transport` — enum: stdio, http
- `Connectivity` — enum: local, online
- `SyncAction` — change log action enum
- `ChangeEntry` — change log entry struct
- `DeviceInfo` — device registration struct
- `SyncSnapshot` — snapshot response struct

The API depends on it via git:

```toml
plugmux-types = { git = "https://github.com/lasharela/plugmux", path = "crates/plugmux-types" }
```

If the API is written in TypeScript (see Worker language note), these types are mirrored as TypeScript interfaces in the API repo.

---

## 10. Security Considerations

- All API communication over HTTPS (Cloudflare handles TLS)
- Stateless JWT tokens for session auth, stored in redb (not localStorage or cookies)
- GitHub OAuth tokens never stored — only used during exchange, then discarded
- OAuth flow uses `state` parameter for CSRF protection
- Admin endpoints require `ADMIN_SECRET` header — separate from user auth
- `DELETE /user/account` requires re-authentication (fresh OAuth flow)
- User data deletion removes all data: user record, devices, changes, favorites (GDPR-friendly)
- No user credentials stored — auth delegated entirely to GitHub
- API versioned (`/v1/`) to allow breaking changes without stranding old app versions

---

## 11. Future Additions (Not in Initial Build)

- **Google OAuth** — when company registration is complete
- **Environment presets** — curated templates (Web Dev, Backend, etc.)
- **Automated reconciliation** — cron-based, on sweenkserver or as Cloudflare Cron Trigger for http-only servers
- **Usage analytics** — install counts, popular servers (anonymized)
- **Config export/import** — manual JSON backup/restore as alternative to cloud sync
- **Admin dashboard** — web UI for managing catalog entries (currently done via API/script)
- **Environment variable sync** — securely syncing server credentials (requires encryption layer)

---

*plugmux Cloud Backend Design Spec — Lasha Kvantaliani — March 2026 — Draft v2*
