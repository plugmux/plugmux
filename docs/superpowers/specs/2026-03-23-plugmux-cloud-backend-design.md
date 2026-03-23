# plugmux Cloud Backend — Design Spec

**Author:** Lasha Kvantaliani
**Date:** 2026-03-23
**Status:** Draft v1

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

---

## 2. Catalog Strategy

### Curation Model

plugmux maintains its own curated server registry. Servers are hand-picked for quality:

- **Official/first-party MCP servers** (Anthropic, GitHub, Figma, etc.) — always included
- **High-quality community servers** — reviewed and approved manually
- **Community servers** exist but are flagged and visually de-emphasized in the UI

Initial metadata (SVGs, descriptions) can be sourced from Smithery, but plugmux owns and maintains the data going forward. Smithery is a data source for bootstrapping, not a runtime dependency.

### Catalog Entry Schema (D1)

```sql
CREATE TABLE catalog_servers (
    id              TEXT PRIMARY KEY,       -- "figma", "github"
    name            TEXT NOT NULL,          -- "Figma"
    description     TEXT NOT NULL,          -- short description
    icon_key        TEXT,                   -- R2 object key "figma.svg"
    icon_hash       TEXT,                   -- hash for cache busting
    category        TEXT NOT NULL,          -- "design", "dev-tools", "database"
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

`tool_count` and `security_score` are maintained by a manual reconciliation script (see Section 8).

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

**Device identity flow:**

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

CREATE INDEX idx_changes_user_time ON changes(user_id, timestamp);
```

### Actions

| Action | Payload |
|--------|---------|
| `add_server_to_env` | `{env: "backend", server_id: "redis"}` |
| `remove_server_from_env` | `{env: "backend", server_id: "redis"}` |
| `create_env` | `{name: "backend", servers: ["postgres", "redis"]}` |
| `delete_env` | `{env: "backend"}` |
| `update_env` | `{env: "backend", field: "name", value: "Backend API"}` |
| `add_favorite` | `{server_id: "figma"}` |
| `remove_favorite` | `{server_id: "figma"}` |
| `add_custom_server` | `{id: "my-tool", name: "...", transport: "stdio", ...}` |
| `update_custom_server` | `{id: "my-tool", field: "name", value: "..."}` |
| `remove_custom_server` | `{id: "my-tool"}` |

### Conflict Resolution Rules

- **Add + Add** — both apply (set union). No conflict.
- **Delete + Add** — delete wins.
- **Rename on two devices** — last-write-wins by timestamp.
- **Same server removed on both** — idempotent, no issue.

### Sync Flow

1. Device comes online, calls `GET /sync/pull?since=<last_sync_timestamp>`
2. Server returns all changes for this user since that timestamp (excluding changes from the requesting device)
3. Device applies changes to local redb
4. Device pushes its queued local changes via `POST /sync/push`
5. New device setup: `GET /sync/snapshot` returns full materialized config

---

## 5. Authentication

### GitHub OAuth (Launch)

Standard OAuth 2.0 redirect flow:

1. App opens `GET /auth/github` — Worker redirects to GitHub authorization
2. User authorizes — GitHub redirects to callback with code
3. `GET /auth/github/callback` — Worker exchanges code for access token, creates/finds user in D1, returns a session token (JWT)
4. App stores JWT in redb, includes as Bearer token in protected requests

### Google OAuth (Future)

Same pattern. Deferred until company/project registration is complete. The auth module should be designed to support multiple providers — just add a `google_id` column to users and a second OAuth flow.

### Sessions

- JWT tokens with expiration (e.g., 30 days)
- Refresh on each sync call
- No refresh token complexity — if expired, user re-authenticates

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
| Compute | Cloudflare Workers (Rust/WASM) |
| Database | Cloudflare D1 (SQLite) |
| Object Storage | Cloudflare R2 |
| Auth | GitHub OAuth, built into Worker |
| Domain | Attached to existing Cloudflare account |

### Endpoints

**Public (no auth):**

```
GET  /catalog/servers                  — list, search, filter by category
GET  /catalog/servers/:id              — single server detail
GET  /icons/:id.svg                    — SVG from R2
```

**Auth:**

```
GET  /auth/github                      — initiate OAuth redirect
GET  /auth/github/callback             — exchange code, return JWT
POST /auth/logout                      — invalidate session
```

**Protected (Bearer token):**

```
POST /devices/register                 — register device to user account
GET  /sync/pull?since=<timestamp>      — pull changes since last sync
POST /sync/push                        — push change log entries
GET  /sync/snapshot                    — full config for new device setup
GET  /user/profile                     — account info
DELETE /user/account                   — delete account and all data
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
4. Diffs results against current D1 data via the API
5. Uploads changes to `PUT /catalog/servers/:id` (admin-only endpoint)
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
│   │   ├── plugmux-types/     — shared types (catalog, sync actions)
│   │   ├── plugmux-cli/       — CLI binary
│   │   └── plugmux-app/       — Tauri desktop app
│   └── catalog/                — existing local catalog data
│
└── plugmux-api/                (private repo)
    ├── src/
    │   ├── routes/             — catalog, auth, sync, devices
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

`plugmux-types` is a lightweight crate in the public repo containing only struct definitions shared between client and API:

- `CatalogEntry` — server catalog schema
- `SyncAction` — change log action enum
- `ChangeEntry` — change log entry
- `DeviceInfo` — device registration

The API depends on it via git:

```toml
plugmux-types = { git = "https://github.com/lasharela/plugmux", path = "crates/plugmux-types" }
```

---

## 10. Security Considerations

- All API communication over HTTPS (Cloudflare handles TLS)
- JWT tokens for session auth, stored in redb (not localStorage)
- GitHub OAuth tokens never stored — only used during exchange, then discarded
- Admin endpoints for catalog updates require a separate admin secret
- User data deletion via `DELETE /user/account` removes all data (GDPR-friendly)
- No user credentials stored — auth delegated to GitHub entirely

---

## 11. Future Additions (Not in Initial Build)

- **Google OAuth** — when company registration is complete
- **Environment presets** — curated templates (Web Dev, Backend, etc.)
- **Automated reconciliation** — cron-based, on sweenkserver or as Cloudflare Cron Trigger for http-only servers
- **Usage analytics** — install counts, popular servers (anonymized)
- **Config export/import** — manual JSON backup/restore as alternative to cloud sync
- **Admin dashboard** — web UI for managing catalog entries (currently done via API/script)

---

*plugmux Cloud Backend Design Spec — Lasha Kvantaliani — March 2026 — Draft v1*
