# plugmux-api — Cloudflare Worker Backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Cloudflare Worker API that serves the curated MCP server catalog, handles GitHub OAuth, and provides sync endpoints for multi-device config merging.

**Architecture:** A single Cloudflare Worker (Rust/WASM via `worker-rs`) with route-based handlers. D1 for relational data, R2 for SVG icons. Stateless JWT auth. All endpoints versioned under `/v1/`.

**Tech Stack:** Rust, `worker` crate (Cloudflare Workers Rust SDK), D1 (SQLite), R2, `serde`/`serde_json`, `jsonwebtoken`, `uuid`

**Spec:** `docs/superpowers/specs/2026-03-23-plugmux-cloud-backend-design.md`

---

## File Map

```
plugmux-api/
├── Cargo.toml
├── wrangler.toml
├── migrations/
│   └── 0001_initial.sql
├── icons/                          — source SVGs for R2 deployment
│   └── (copied from plugmux/catalog/icons/)
├── seed/
│   └── catalog.sql                 — initial catalog INSERT statements
├── reconciler/
│   └── reconcile.py                — manual reconciliation script
├── src/
│   ├── lib.rs                      — Worker entrypoint, router setup
│   ├── router.rs                   — route definitions, dispatch
│   ├── error.rs                    — ApiError type, response helpers
│   ├── auth/
│   │   ├── mod.rs                  — re-exports
│   │   ├── github.rs               — GitHub OAuth redirect + callback
│   │   ├── jwt.rs                  — JWT sign/verify, claims struct
│   │   └── middleware.rs           — auth extraction from Bearer header
│   ├── routes/
│   │   ├── mod.rs                  — re-exports
│   │   ├── catalog.rs              — GET /v1/catalog/servers, GET /v1/catalog/servers/:id
│   │   ├── icons.rs                — GET /v1/icons/:id.svg
│   │   ├── health.rs               — GET /v1/health
│   │   ├── sync.rs                 — GET /v1/sync/pull, POST /v1/sync/push, GET /v1/sync/snapshot
│   │   ├── devices.rs              — POST /v1/devices/register
│   │   ├── user.rs                 — GET /v1/user/profile, DELETE /v1/user/account
│   │   └── admin.rs                — PUT /v1/admin/catalog/servers/:id
│   └── models/
│       ├── mod.rs                  — re-exports
│       ├── catalog.rs              — CatalogServer struct, D1 queries
│       ├── user.rs                 — User struct, D1 queries
│       ├── device.rs               — Device struct, D1 queries
│       ├── change.rs               — Change struct, D1 queries
│       └── favorite.rs             — Favorite helpers, D1 queries
└── README.md
```

---

### Task 1: Scaffold the Cloudflare Worker project

**Files:**
- Create: `plugmux-api/Cargo.toml`
- Create: `plugmux-api/wrangler.toml`
- Create: `plugmux-api/src/lib.rs`
- Create: `plugmux-api/.gitignore`

- [ ] **Step 1: Create the repo directory**

```bash
mkdir -p ~/Development/lasharela/plugmux-api/src
cd ~/Development/lasharela/plugmux-api
git init
```

- [ ] **Step 2: Create Cargo.toml**

```toml
[package]
name = "plugmux-api"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
worker = { version = "0.5", features = ["d1", "r2"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "js"] }
getrandom = { version = "0.2", features = ["js"] }
jsonwebtoken = "9"
chrono = { version = "0.4", features = ["wasmbind"] }
```

Note: `edition = "2021"` (not 2024) because `worker` crate may not support 2024 edition yet. The `js` feature on `uuid`/`getrandom` is required for WASM random number generation.

- [ ] **Step 3: Create wrangler.toml**

```toml
name = "plugmux-api"
main = "build/worker/shim.mjs"
compatibility_date = "2024-12-01"

[build]
command = "cargo install worker-build && worker-build --release"

[[d1_databases]]
binding = "DB"
database_name = "plugmux-db"
database_id = "<will be set after creation>"

[[r2_buckets]]
binding = "ICONS"
bucket_name = "plugmux-icons"

[vars]
ENVIRONMENT = "production"
```

- [ ] **Step 4: Create minimal lib.rs**

```rust
use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Response::ok("plugmux-api is running")
}
```

- [ ] **Step 5: Create .gitignore**

```
/target
/build
/node_modules
.wrangler
```

- [ ] **Step 6: Verify it builds**

```bash
npx wrangler dev
```

Expected: Worker starts locally, responds with "plugmux-api is running"

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "chore: scaffold Cloudflare Worker project"
```

---

### Task 2: D1 schema migration

**Files:**
- Create: `plugmux-api/migrations/0001_initial.sql`

- [ ] **Step 1: Create D1 database**

```bash
npx wrangler d1 create plugmux-db
```

Copy the `database_id` output into `wrangler.toml`.

- [ ] **Step 2: Write the initial migration**

```sql
-- migrations/0001_initial.sql

CREATE TABLE catalog_servers (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL,
    icon_key        TEXT,
    icon_hash       TEXT,
    categories      TEXT NOT NULL DEFAULT '[]',
    transport       TEXT NOT NULL,
    command         TEXT,
    args            TEXT,
    url             TEXT,
    connectivity    TEXT NOT NULL,
    official        INTEGER NOT NULL DEFAULT 0,
    tool_count      INTEGER,
    security_score  TEXT,
    added_at        TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE users (
    id              TEXT PRIMARY KEY,
    github_id       TEXT UNIQUE NOT NULL,
    github_username TEXT NOT NULL,
    email           TEXT,
    created_at      TEXT NOT NULL,
    last_sync_at    TEXT
);

CREATE TABLE devices (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL,
    name            TEXT NOT NULL,
    last_seen_at    TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE favorites (
    user_id         TEXT NOT NULL,
    server_id       TEXT NOT NULL,
    added_at        TEXT NOT NULL,
    PRIMARY KEY (user_id, server_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE changes (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         TEXT NOT NULL,
    device_id       TEXT NOT NULL,
    timestamp       TEXT NOT NULL,
    action          TEXT NOT NULL,
    payload         TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (device_id) REFERENCES devices(id)
);

CREATE INDEX idx_changes_user_id ON changes(user_id, id);
```

- [ ] **Step 3: Apply migration locally**

```bash
npx wrangler d1 migrations apply plugmux-db --local
```

Expected: Migration applied successfully.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add D1 schema migration"
```

---

### Task 3: Error handling and router skeleton

**Files:**
- Create: `plugmux-api/src/error.rs`
- Create: `plugmux-api/src/router.rs`
- Modify: `plugmux-api/src/lib.rs`

- [ ] **Step 1: Create error.rs**

```rust
use worker::*;

pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Unauthorized,
    Forbidden,
    Internal(String),
}

impl ApiError {
    pub fn into_response(self) -> Result<Response> {
        match self {
            ApiError::NotFound(msg) => Response::error(msg, 404),
            ApiError::BadRequest(msg) => Response::error(msg, 400),
            ApiError::Unauthorized => Response::error("Unauthorized", 401),
            ApiError::Forbidden => Response::error("Forbidden", 403),
            ApiError::Internal(msg) => Response::error(msg, 500),
        }
    }
}

impl From<worker::Error> for ApiError {
    fn from(e: worker::Error) -> Self {
        ApiError::Internal(e.to_string())
    }
}
```

- [ ] **Step 2: Create router.rs**

```rust
use worker::*;

pub async fn handle(req: Request, env: Env) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();
    let method = req.method();

    // CORS headers for all responses
    let cors = |mut resp: Response| -> Result<Response> {
        let headers = resp.headers_mut();
        headers.set("Access-Control-Allow-Origin", "*")?;
        headers.set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")?;
        headers.set("Access-Control-Allow-Headers", "Content-Type, Authorization")?;
        Ok(resp)
    };

    // Preflight
    if method == Method::Options {
        return cors(Response::empty()?);
    }

    let resp = match (method, path.as_str()) {
        (Method::Get, "/v1/health") => Response::ok("ok"),

        _ => Response::error("Not Found", 404),
    };

    cors(resp?)
}
```

- [ ] **Step 3: Update lib.rs**

```rust
use worker::*;

mod error;
mod router;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    router::handle(req, env).await
}
```

- [ ] **Step 4: Test locally**

```bash
npx wrangler dev
curl http://localhost:8787/v1/health
```

Expected: `ok` with 200 status and CORS headers.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add error handling and router skeleton with health endpoint"
```

---

### Task 4: Catalog models and public endpoints

**Files:**
- Create: `plugmux-api/src/models/mod.rs`
- Create: `plugmux-api/src/models/catalog.rs`
- Create: `plugmux-api/src/routes/mod.rs`
- Create: `plugmux-api/src/routes/catalog.rs`
- Modify: `plugmux-api/src/router.rs`
- Modify: `plugmux-api/src/lib.rs`

- [ ] **Step 1: Create models/mod.rs**

```rust
pub mod catalog;
```

- [ ] **Step 2: Create models/catalog.rs**

```rust
use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogServer {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_key: Option<String>,
    pub icon_hash: Option<String>,
    pub categories: Vec<String>,
    pub transport: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub connectivity: String,
    pub official: bool,
    pub tool_count: Option<i32>,
    pub security_score: Option<String>,
    pub added_at: String,
    pub updated_at: String,
}

impl CatalogServer {
    pub fn from_row(row: &worker::d1::Row) -> Result<Self> {
        let categories_str: String = row.get("categories")?;
        let categories: Vec<String> = serde_json::from_str(&categories_str)
            .unwrap_or_default();

        let args_str: Option<String> = row.get("args")?;
        let args: Option<Vec<String>> = args_str
            .and_then(|s| serde_json::from_str(&s).ok());

        Ok(CatalogServer {
            id: row.get("id")?,
            name: row.get("name")?,
            description: row.get("description")?,
            icon_key: row.get("icon_key")?,
            icon_hash: row.get("icon_hash")?,
            categories,
            transport: row.get("transport")?,
            command: row.get("command")?,
            args,
            url: row.get("url")?,
            connectivity: row.get("connectivity")?,
            official: row.get::<i32>("official")? == 1,
            tool_count: row.get("tool_count")?,
            security_score: row.get("security_score")?,
            added_at: row.get("added_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

pub async fn list_servers(
    db: &D1Database,
    query: Option<&str>,
    category: Option<&str>,
    limit: u32,
    cursor: u32,
) -> Result<(Vec<CatalogServer>, u32)> {
    let mut sql = String::from("SELECT * FROM catalog_servers WHERE 1=1");
    let mut count_sql = String::from("SELECT COUNT(*) as cnt FROM catalog_servers WHERE 1=1");

    if let Some(q) = query {
        let filter = format!(
            " AND (name LIKE '%{}%' OR description LIKE '%{}%')",
            q.replace('\'', "''"),
            q.replace('\'', "''")
        );
        sql.push_str(&filter);
        count_sql.push_str(&filter);
    }

    if let Some(cat) = category {
        let filter = format!(
            " AND categories LIKE '%\"{}\"'%",
            cat.replace('\'', "''")
        );
        sql.push_str(&filter);
        count_sql.push_str(&filter);
    }

    sql.push_str(&format!(" ORDER BY official DESC, name ASC LIMIT {} OFFSET {}", limit, cursor));

    let stmt = db.prepare(&sql);
    let result = stmt.all().await?;
    let rows = result.results::<serde_json::Value>()?;

    let mut servers = Vec::new();
    for row_val in &rows {
        // Parse from the raw D1 result
        let server: CatalogServer = serde_json::from_value(row_val.clone())
            .map_err(|e| worker::Error::RustError(e.to_string()))?;
        servers.push(server);
    }

    let count_stmt = db.prepare(&count_sql);
    let count_result = count_stmt.first::<serde_json::Value>(None).await?;
    let total = count_result
        .and_then(|v| v.get("cnt").and_then(|c| c.as_u64()))
        .unwrap_or(0) as u32;

    Ok((servers, total))
}

pub async fn get_server(db: &D1Database, id: &str) -> Result<Option<CatalogServer>> {
    let stmt = db.prepare("SELECT * FROM catalog_servers WHERE id = ?1");
    let result = stmt.bind(&[id.into()])?.first::<serde_json::Value>(None).await?;

    match result {
        Some(val) => {
            let server: CatalogServer = serde_json::from_value(val)
                .map_err(|e| worker::Error::RustError(e.to_string()))?;
            Ok(Some(server))
        }
        None => Ok(None),
    }
}
```

- [ ] **Step 3: Create routes/mod.rs**

```rust
pub mod catalog;
```

- [ ] **Step 4: Create routes/catalog.rs**

```rust
use crate::models::catalog;
use serde::Serialize;
use worker::*;

#[derive(Serialize)]
struct ListResponse {
    servers: Vec<catalog::CatalogServer>,
    total: u32,
    limit: u32,
    cursor: u32,
}

pub async fn list(req: &Request, env: &Env) -> Result<Response> {
    let db = env.d1("DB")?;

    let url = req.url()?;
    let params: Vec<(String, String)> = url.query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let query = params.iter().find(|(k, _)| k == "q").map(|(_, v)| v.as_str());
    let category = params.iter().find(|(k, _)| k == "category").map(|(_, v)| v.as_str());
    let limit: u32 = params.iter()
        .find(|(k, _)| k == "limit")
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(50)
        .min(100);
    let cursor: u32 = params.iter()
        .find(|(k, _)| k == "cursor")
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(0);

    let (servers, total) = catalog::list_servers(&db, query, category, limit, cursor).await?;

    let body = ListResponse { servers, total, limit, cursor };
    Response::from_json(&body)
}

pub async fn get(id: &str, env: &Env) -> Result<Response> {
    let db = env.d1("DB")?;

    match catalog::get_server(&db, id).await? {
        Some(server) => Response::from_json(&server),
        None => Response::error("Server not found", 404),
    }
}
```

- [ ] **Step 5: Update router.rs to add catalog routes**

Add to the match in `router.rs`:

```rust
use crate::routes;

// In the match block:
(Method::Get, "/v1/catalog/servers") => routes::catalog::list(&req, &env).await,

// For /v1/catalog/servers/:id, extract the ID:
_ if method == Method::Get && path.starts_with("/v1/catalog/servers/") => {
    let id = path.strip_prefix("/v1/catalog/servers/").unwrap_or("");
    if id.is_empty() {
        Response::error("Missing server ID", 400)
    } else {
        routes::catalog::get(id, &env).await
    }
}
```

- [ ] **Step 6: Update lib.rs to include modules**

```rust
mod error;
mod models;
mod router;
mod routes;
```

- [ ] **Step 7: Test locally**

```bash
npx wrangler dev
curl http://localhost:8787/v1/catalog/servers
curl http://localhost:8787/v1/catalog/servers/figma
```

Expected: Empty list (no data yet), and 404 for figma (not seeded yet).

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat: add catalog models and public list/get endpoints"
```

---

### Task 5: R2 icon serving

**Files:**
- Create: `plugmux-api/src/routes/icons.rs`
- Modify: `plugmux-api/src/routes/mod.rs`
- Modify: `plugmux-api/src/router.rs`

- [ ] **Step 1: Create R2 bucket**

```bash
npx wrangler r2 bucket create plugmux-icons
```

- [ ] **Step 2: Create routes/icons.rs**

```rust
use worker::*;

pub async fn get(id: &str, env: &Env) -> Result<Response> {
    let bucket = env.r2("ICONS")?;
    let key = format!("{}.svg", id);

    match bucket.get(&key).execute().await? {
        Some(object) => {
            let body = object.body()
                .ok_or_else(|| Error::RustError("Empty object body".into()))?;
            let bytes = body.bytes().await?;
            let mut resp = Response::from_bytes(bytes)?;
            resp.headers_mut().set("Content-Type", "image/svg+xml")?;
            resp.headers_mut().set("Cache-Control", "public, max-age=86400")?;
            Ok(resp)
        }
        None => Response::error("Icon not found", 404),
    }
}
```

- [ ] **Step 3: Update routes/mod.rs**

```rust
pub mod catalog;
pub mod icons;
```

- [ ] **Step 4: Add route to router.rs**

```rust
_ if method == Method::Get && path.starts_with("/v1/icons/") && path.ends_with(".svg") => {
    let id = path
        .strip_prefix("/v1/icons/")
        .and_then(|s| s.strip_suffix(".svg"))
        .unwrap_or("");
    if id.is_empty() {
        Response::error("Missing icon ID", 400)
    } else {
        routes::icons::get(id, &env).await
    }
}
```

- [ ] **Step 5: Upload test icon**

```bash
npx wrangler r2 object put plugmux-icons/figma.svg --file ../plugmux/catalog/icons/figma.svg --local
```

- [ ] **Step 6: Test locally**

```bash
curl http://localhost:8787/v1/icons/figma.svg
```

Expected: SVG content with `Content-Type: image/svg+xml`.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: add R2 icon serving endpoint"
```

---

### Task 6: JWT auth module

**Files:**
- Create: `plugmux-api/src/auth/mod.rs`
- Create: `plugmux-api/src/auth/jwt.rs`
- Create: `plugmux-api/src/auth/middleware.rs`
- Modify: `plugmux-api/src/lib.rs`

- [ ] **Step 1: Create auth/mod.rs**

```rust
pub mod jwt;
pub mod middleware;
pub mod github;
```

- [ ] **Step 2: Create auth/jwt.rs**

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // user_id
    pub username: String,   // github_username
    pub exp: usize,         // expiration timestamp
    pub iat: usize,         // issued at
}

pub fn sign(user_id: &str, username: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: now + (30 * 24 * 60 * 60), // 30 days
        iat: now,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub fn verify(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}
```

- [ ] **Step 3: Create auth/middleware.rs**

```rust
use crate::auth::jwt;
use crate::error::ApiError;
use worker::*;

pub struct AuthUser {
    pub user_id: String,
    pub username: String,
}

pub fn extract_user(req: &Request, env: &Env) -> Result<AuthUser> {
    let secret = env.secret("JWT_SECRET")?.to_string();

    let header = req.headers().get("Authorization")?
        .ok_or_else(|| Error::RustError("Missing Authorization header".into()))?;

    let token = header.strip_prefix("Bearer ")
        .ok_or_else(|| Error::RustError("Invalid Authorization format".into()))?;

    let claims = jwt::verify(token, &secret)
        .map_err(|e| Error::RustError(format!("Invalid token: {}", e)))?;

    Ok(AuthUser {
        user_id: claims.sub,
        username: claims.username,
    })
}

pub fn extract_admin(req: &Request, env: &Env) -> Result<()> {
    let secret = env.secret("ADMIN_SECRET")?.to_string();

    let header = req.headers().get("X-Admin-Secret")?
        .ok_or_else(|| Error::RustError("Missing admin secret".into()))?;

    if header != secret {
        return Err(Error::RustError("Invalid admin secret".into()));
    }

    Ok(())
}
```

- [ ] **Step 4: Update lib.rs**

```rust
mod auth;
mod error;
mod models;
mod router;
mod routes;
```

- [ ] **Step 5: Set secrets for local dev**

Create `.dev.vars` file (gitignored):

```
JWT_SECRET=dev-jwt-secret-change-in-prod
ADMIN_SECRET=dev-admin-secret
GITHUB_CLIENT_ID=your-github-oauth-app-id
GITHUB_CLIENT_SECRET=your-github-oauth-app-secret
```

Add `.dev.vars` to `.gitignore`.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add JWT signing, verification, and auth middleware"
```

---

### Task 7: GitHub OAuth flow

**Files:**
- Create: `plugmux-api/src/auth/github.rs`
- Create: `plugmux-api/src/models/user.rs`
- Modify: `plugmux-api/src/models/mod.rs`
- Modify: `plugmux-api/src/router.rs`

- [ ] **Step 1: Create models/user.rs**

```rust
use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub github_id: String,
    pub github_username: String,
    pub email: Option<String>,
    pub created_at: String,
    pub last_sync_at: Option<String>,
}

pub async fn find_by_github_id(db: &D1Database, github_id: &str) -> Result<Option<User>> {
    let stmt = db.prepare("SELECT * FROM users WHERE github_id = ?1");
    let result = stmt.bind(&[github_id.into()])?.first::<serde_json::Value>(None).await?;

    match result {
        Some(val) => {
            let user: User = serde_json::from_value(val)
                .map_err(|e| Error::RustError(e.to_string()))?;
            Ok(Some(user))
        }
        None => Ok(None),
    }
}

pub async fn create(db: &D1Database, id: &str, github_id: &str, username: &str, email: Option<&str>) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let stmt = db.prepare(
        "INSERT INTO users (id, github_id, github_username, email, created_at) VALUES (?1, ?2, ?3, ?4, ?5)"
    );
    stmt.bind(&[
        id.into(),
        github_id.into(),
        username.into(),
        email.map(|e| e.into()).unwrap_or(serde_json::Value::Null.into()),
        now.as_str().into(),
    ])?.run().await?;
    Ok(())
}

pub async fn find_by_id(db: &D1Database, user_id: &str) -> Result<Option<User>> {
    let stmt = db.prepare("SELECT * FROM users WHERE id = ?1");
    let result = stmt.bind(&[user_id.into()])?.first::<serde_json::Value>(None).await?;

    match result {
        Some(val) => {
            let user: User = serde_json::from_value(val)
                .map_err(|e| Error::RustError(e.to_string()))?;
            Ok(Some(user))
        }
        None => Ok(None),
    }
}

pub async fn delete(db: &D1Database, user_id: &str) -> Result<()> {
    // CASCADE deletes devices, favorites, changes
    let stmt = db.prepare("DELETE FROM users WHERE id = ?1");
    stmt.bind(&[user_id.into()])?.run().await?;
    Ok(())
}
```

- [ ] **Step 2: Update models/mod.rs**

```rust
pub mod catalog;
pub mod user;
```

- [ ] **Step 3: Create auth/github.rs**

```rust
use crate::auth::jwt;
use crate::models::user;
use serde::Deserialize;
use worker::*;

#[derive(Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GitHubUser {
    id: u64,
    login: String,
    email: Option<String>,
}

pub async fn redirect(req: &Request, env: &Env) -> Result<Response> {
    let client_id = env.secret("GITHUB_CLIENT_ID")?.to_string();

    // Generate state for CSRF protection
    let state = uuid::Uuid::new_v4().to_string();

    // Store state in a short-lived query param (stateless approach)
    // In production, use KV with TTL for proper state validation
    let url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&scope=read:user%20user:email&state={}",
        client_id, state
    );

    Response::redirect_with_status(Url::parse(&url)?, 302)
}

pub async fn callback(req: &Request, env: &Env) -> Result<Response> {
    let url = req.url()?;
    let params: Vec<(String, String)> = url.query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let code = params.iter()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.as_str())
        .ok_or_else(|| Error::RustError("Missing code parameter".into()))?;

    // Exchange code for access token
    let client_id = env.secret("GITHUB_CLIENT_ID")?.to_string();
    let client_secret = env.secret("GITHUB_CLIENT_SECRET")?.to_string();

    let token_body = serde_json::json!({
        "client_id": client_id,
        "client_secret": client_secret,
        "code": code,
    });

    let mut headers = Headers::new();
    headers.set("Accept", "application/json")?;
    headers.set("Content-Type", "application/json")?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(headers)
        .with_body(Some(serde_json::to_string(&token_body)?.into()));

    let token_req = Request::new_with_init(
        "https://github.com/login/oauth/access_token",
        &init,
    )?;

    let mut token_resp = Fetch::Request(token_req).send().await?;
    let token_data: GitHubTokenResponse = token_resp.json().await?;

    // Fetch GitHub user info
    let mut user_headers = Headers::new();
    user_headers.set("Authorization", &format!("Bearer {}", token_data.access_token))?;
    user_headers.set("User-Agent", "plugmux-api")?;

    let mut user_init = RequestInit::new();
    user_init.with_method(Method::Get).with_headers(user_headers);

    let user_req = Request::new_with_init("https://api.github.com/user", &user_init)?;
    let mut user_resp = Fetch::Request(user_req).send().await?;
    let github_user: GitHubUser = user_resp.json().await?;

    // Find or create user in D1
    let db = env.d1("DB")?;
    let github_id_str = github_user.id.to_string();

    let db_user = match user::find_by_github_id(&db, &github_id_str).await? {
        Some(existing) => existing,
        None => {
            let new_id = uuid::Uuid::new_v4().to_string();
            user::create(
                &db,
                &new_id,
                &github_id_str,
                &github_user.login,
                github_user.email.as_deref(),
            ).await?;
            user::find_by_github_id(&db, &github_id_str).await?
                .ok_or_else(|| Error::RustError("Failed to create user".into()))?
        }
    };

    // Sign JWT
    let jwt_secret = env.secret("JWT_SECRET")?.to_string();
    let token = jwt::sign(&db_user.id, &db_user.github_username, &jwt_secret)
        .map_err(|e| Error::RustError(format!("JWT error: {}", e)))?;

    // Return token as JSON
    let body = serde_json::json!({
        "token": token,
        "user": {
            "id": db_user.id,
            "username": db_user.github_username,
            "email": db_user.email,
        }
    });

    Response::from_json(&body)
}
```

- [ ] **Step 4: Add auth routes to router.rs**

```rust
(Method::Get, "/v1/auth/github") => auth::github::redirect(&req, &env).await,
(Method::Get, "/v1/auth/github/callback") => auth::github::callback(&req, &env).await,
```

Add `use crate::auth;` at top of router.rs.

- [ ] **Step 5: Test OAuth redirect locally**

```bash
curl -v http://localhost:8787/v1/auth/github
```

Expected: 302 redirect to `github.com/login/oauth/authorize?...`

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add GitHub OAuth flow with user creation"
```

---

### Task 8: Device registration and user endpoints

**Files:**
- Create: `plugmux-api/src/models/device.rs`
- Create: `plugmux-api/src/routes/devices.rs`
- Create: `plugmux-api/src/routes/user.rs`
- Modify: `plugmux-api/src/models/mod.rs`
- Modify: `plugmux-api/src/routes/mod.rs`
- Modify: `plugmux-api/src/router.rs`

- [ ] **Step 1: Create models/device.rs**

```rust
use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub last_seen_at: String,
}

pub async fn register(db: &D1Database, id: &str, user_id: &str, name: &str) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let stmt = db.prepare(
        "INSERT INTO devices (id, user_id, name, last_seen_at) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET last_seen_at = ?4, name = ?3"
    );
    stmt.bind(&[id.into(), user_id.into(), name.into(), now.as_str().into()])?.run().await?;
    Ok(())
}

pub async fn touch(db: &D1Database, device_id: &str) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let stmt = db.prepare("UPDATE devices SET last_seen_at = ?1 WHERE id = ?2");
    stmt.bind(&[now.as_str().into(), device_id.into()])?.run().await?;
    Ok(())
}
```

- [ ] **Step 2: Create routes/devices.rs**

```rust
use crate::auth::middleware::extract_user;
use crate::models::device;
use serde::Deserialize;
use worker::*;

#[derive(Deserialize)]
struct RegisterBody {
    device_id: String,
    name: String,
}

pub async fn register(mut req: Request, env: &Env) -> Result<Response> {
    let user = extract_user(&req, env)?;
    let db = env.d1("DB")?;

    let body: RegisterBody = req.json().await?;
    device::register(&db, &body.device_id, &user.user_id, &body.name).await?;

    Response::from_json(&serde_json::json!({"ok": true}))
}
```

- [ ] **Step 3: Create routes/user.rs**

```rust
use crate::auth::middleware::extract_user;
use crate::models::user;
use worker::*;

pub async fn profile(req: &Request, env: &Env) -> Result<Response> {
    let auth = extract_user(req, env)?;
    let db = env.d1("DB")?;

    match user::find_by_id(&db, &auth.user_id).await? {
        Some(u) => Response::from_json(&u),
        None => Response::error("User not found", 404),
    }
}

pub async fn delete(req: &Request, env: &Env) -> Result<Response> {
    let auth = extract_user(req, env)?;
    let db = env.d1("DB")?;
    user::delete(&db, &auth.user_id).await?;
    Response::from_json(&serde_json::json!({"ok": true, "message": "Account deleted"}))
}
```

- [ ] **Step 4: Update models/mod.rs and routes/mod.rs**

```rust
// models/mod.rs
pub mod catalog;
pub mod device;
pub mod user;

// routes/mod.rs
pub mod catalog;
pub mod devices;
pub mod icons;
pub mod user;
```

- [ ] **Step 5: Add routes to router.rs**

```rust
(Method::Post, "/v1/devices/register") => routes::devices::register(req, &env).await,
(Method::Get, "/v1/user/profile") => routes::user::profile(&req, &env).await,
(Method::Delete, "/v1/user/account") => routes::user::delete(&req, &env).await,
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add device registration and user profile/delete endpoints"
```

---

### Task 9: Sync endpoints (push, pull, snapshot)

**Files:**
- Create: `plugmux-api/src/models/change.rs`
- Create: `plugmux-api/src/models/favorite.rs`
- Create: `plugmux-api/src/routes/sync.rs`
- Modify: `plugmux-api/src/models/mod.rs`
- Modify: `plugmux-api/src/routes/mod.rs`
- Modify: `plugmux-api/src/router.rs`

- [ ] **Step 1: Create models/change.rs**

```rust
use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Change {
    pub id: i64,
    pub user_id: String,
    pub device_id: String,
    pub timestamp: String,
    pub action: String,
    pub payload: serde_json::Value,
}

pub async fn push_changes(db: &D1Database, user_id: &str, changes: &[NewChange]) -> Result<()> {
    for change in changes {
        let payload_str = serde_json::to_string(&change.payload)
            .map_err(|e| Error::RustError(e.to_string()))?;
        let stmt = db.prepare(
            "INSERT INTO changes (user_id, device_id, timestamp, action, payload) VALUES (?1, ?2, ?3, ?4, ?5)"
        );
        stmt.bind(&[
            user_id.into(),
            change.device_id.as_str().into(),
            change.timestamp.as_str().into(),
            change.action.as_str().into(),
            payload_str.as_str().into(),
        ])?.run().await?;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct NewChange {
    pub device_id: String,
    pub timestamp: String,
    pub action: String,
    pub payload: serde_json::Value,
}

pub async fn pull_changes(db: &D1Database, user_id: &str, since_id: i64, device_id: &str, limit: u32) -> Result<Vec<Change>> {
    let stmt = db.prepare(
        "SELECT * FROM changes WHERE user_id = ?1 AND id > ?2 AND device_id != ?3 ORDER BY id ASC LIMIT ?4"
    );
    let result = stmt.bind(&[
        user_id.into(),
        since_id.into(),
        device_id.into(),
        limit.into(),
    ])?.all().await?;

    let rows = result.results::<serde_json::Value>()?;
    let mut changes = Vec::new();
    for val in rows {
        let mut change: Change = serde_json::from_value(val)
            .map_err(|e| Error::RustError(e.to_string()))?;
        // Parse payload from string to JSON if stored as string
        if let serde_json::Value::String(ref s) = change.payload {
            if let Ok(parsed) = serde_json::from_str(s) {
                change.payload = parsed;
            }
        }
        changes.push(change);
    }
    Ok(changes)
}
```

- [ ] **Step 2: Create models/favorite.rs**

```rust
use worker::*;

pub async fn list(db: &D1Database, user_id: &str) -> Result<Vec<String>> {
    let stmt = db.prepare("SELECT server_id FROM favorites WHERE user_id = ?1");
    let result = stmt.bind(&[user_id.into()])?.all().await?;
    let rows = result.results::<serde_json::Value>()?;

    let ids: Vec<String> = rows.iter()
        .filter_map(|v| v.get("server_id").and_then(|s| s.as_str()).map(|s| s.to_string()))
        .collect();
    Ok(ids)
}
```

- [ ] **Step 3: Create routes/sync.rs**

```rust
use crate::auth::middleware::extract_user;
use crate::models::{change, favorite};
use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Deserialize)]
struct PushBody {
    changes: Vec<change::NewChange>,
}

#[derive(Serialize)]
struct PullResponse {
    changes: Vec<change::Change>,
    has_more: bool,
}

pub async fn push(mut req: Request, env: &Env) -> Result<Response> {
    let user = extract_user(&req, env)?;
    let db = env.d1("DB")?;

    let body: PushBody = req.json().await?;

    if body.changes.len() > 100 {
        return Response::error("Too many changes in single push (max 100)", 400);
    }

    // Apply side effects: update favorites table for favorite actions
    for c in &body.changes {
        match c.action.as_str() {
            "add_favorite" => {
                if let Some(server_id) = c.payload.get("server_id").and_then(|v| v.as_str()) {
                    let now = chrono::Utc::now().to_rfc3339();
                    let stmt = db.prepare(
                        "INSERT OR IGNORE INTO favorites (user_id, server_id, added_at) VALUES (?1, ?2, ?3)"
                    );
                    stmt.bind(&[user.user_id.as_str().into(), server_id.into(), now.as_str().into()])?.run().await?;
                }
            }
            "remove_favorite" => {
                if let Some(server_id) = c.payload.get("server_id").and_then(|v| v.as_str()) {
                    let stmt = db.prepare("DELETE FROM favorites WHERE user_id = ?1 AND server_id = ?2");
                    stmt.bind(&[user.user_id.as_str().into(), server_id.into()])?.run().await?;
                }
            }
            _ => {}
        }
    }

    change::push_changes(&db, &user.user_id, &body.changes).await?;

    Response::from_json(&serde_json::json!({"ok": true, "count": body.changes.len()}))
}

pub async fn pull(req: &Request, env: &Env) -> Result<Response> {
    let user = extract_user(req, env)?;
    let db = env.d1("DB")?;

    let url = req.url()?;
    let params: Vec<(String, String)> = url.query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let since_id: i64 = params.iter()
        .find(|(k, _)| k == "since_id")
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(0);

    let device_id = params.iter()
        .find(|(k, _)| k == "device_id")
        .map(|(_, v)| v.as_str())
        .unwrap_or("");

    let limit: u32 = 1000;
    let changes = change::pull_changes(&db, &user.user_id, since_id, device_id, limit + 1).await?;

    let has_more = changes.len() > limit as usize;
    let changes: Vec<change::Change> = changes.into_iter().take(limit as usize).collect();

    Response::from_json(&PullResponse { changes, has_more })
}

pub async fn snapshot(req: &Request, env: &Env) -> Result<Response> {
    let user = extract_user(req, env)?;
    let db = env.d1("DB")?;

    // Materialize current state by replaying all changes
    let all_changes = change::pull_changes(&db, &user.user_id, 0, "", 10000).await?;
    let favorites = favorite::list(&db, &user.user_id).await?;

    // Build environments from change log
    let mut envs: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut env_names: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut custom_servers: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();

    for c in &all_changes {
        match c.action.as_str() {
            "create_env" => {
                if let Some(name) = c.payload.get("name").and_then(|v| v.as_str()) {
                    let id = slug::slugify(name);
                    env_names.insert(id.clone(), name.to_string());
                    let servers: Vec<String> = c.payload.get("servers")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    envs.insert(id, servers);
                }
            }
            "delete_env" => {
                if let Some(env_id) = c.payload.get("env").and_then(|v| v.as_str()) {
                    envs.remove(env_id);
                    env_names.remove(env_id);
                }
            }
            "add_server_to_env" => {
                if let (Some(env_id), Some(server_id)) = (
                    c.payload.get("env").and_then(|v| v.as_str()),
                    c.payload.get("server_id").and_then(|v| v.as_str()),
                ) {
                    envs.entry(env_id.to_string())
                        .or_default()
                        .push(server_id.to_string());
                }
            }
            "remove_server_from_env" => {
                if let (Some(env_id), Some(server_id)) = (
                    c.payload.get("env").and_then(|v| v.as_str()),
                    c.payload.get("server_id").and_then(|v| v.as_str()),
                ) {
                    if let Some(servers) = envs.get_mut(env_id) {
                        servers.retain(|s| s != server_id);
                    }
                }
            }
            "update_env" => {
                if let (Some(env_id), Some(field), Some(value)) = (
                    c.payload.get("env").and_then(|v| v.as_str()),
                    c.payload.get("field").and_then(|v| v.as_str()),
                    c.payload.get("value").and_then(|v| v.as_str()),
                ) {
                    if field == "name" {
                        env_names.insert(env_id.to_string(), value.to_string());
                    }
                }
            }
            "add_custom_server" => {
                if let Some(id) = c.payload.get("id").and_then(|v| v.as_str()) {
                    custom_servers.insert(id.to_string(), c.payload.clone());
                }
            }
            "remove_custom_server" => {
                if let Some(id) = c.payload.get("id").and_then(|v| v.as_str()) {
                    custom_servers.remove(id);
                }
            }
            _ => {}
        }
    }

    // Get last change ID
    let last_id = all_changes.last().map(|c| c.id).unwrap_or(0);

    // Build response
    let environments: Vec<serde_json::Value> = envs.iter().map(|(id, servers)| {
        serde_json::json!({
            "id": id,
            "name": env_names.get(id).unwrap_or(id),
            "servers": servers,
        })
    }).collect();

    let custom: Vec<serde_json::Value> = custom_servers.values().cloned().collect();

    Response::from_json(&serde_json::json!({
        "environments": environments,
        "favorites": favorites,
        "custom_servers": custom,
        "last_change_id": last_id,
    }))
}
```

- [ ] **Step 4: Update models/mod.rs and routes/mod.rs**

```rust
// models/mod.rs
pub mod catalog;
pub mod change;
pub mod device;
pub mod favorite;
pub mod user;

// routes/mod.rs
pub mod catalog;
pub mod devices;
pub mod icons;
pub mod sync;
pub mod user;
```

- [ ] **Step 5: Add sync routes to router.rs**

```rust
(Method::Post, "/v1/sync/push") => routes::sync::push(req, &env).await,
(Method::Get, "/v1/sync/pull") => routes::sync::pull(&req, &env).await,
(Method::Get, "/v1/sync/snapshot") => routes::sync::snapshot(&req, &env).await,
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add sync push, pull, and snapshot endpoints"
```

---

### Task 10: Minimal admin endpoint

**Files:**
- Create: `plugmux-api/src/routes/admin.rs`
- Modify: `plugmux-api/src/routes/mod.rs`
- Modify: `plugmux-api/src/router.rs`

- [ ] **Step 1: Create routes/admin.rs**

```rust
use crate::auth::middleware::extract_admin;
use worker::*;

pub async fn upsert_server(mut req: Request, env: &Env, id: &str) -> Result<Response> {
    extract_admin(&req, env)?;
    let db = env.d1("DB")?;

    let body: serde_json::Value = req.json().await?;
    let now = chrono::Utc::now().to_rfc3339();

    let stmt = db.prepare(
        "INSERT INTO catalog_servers (id, name, description, icon_key, icon_hash, categories, transport, command, args, url, connectivity, official, tool_count, security_score, added_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
         ON CONFLICT(id) DO UPDATE SET
           name = ?2, description = ?3, icon_key = ?4, icon_hash = ?5, categories = ?6,
           transport = ?7, command = ?8, args = ?9, url = ?10, connectivity = ?11,
           official = ?12, tool_count = ?13, security_score = ?14, updated_at = ?16"
    );

    let categories = body.get("categories")
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "[]".into()))
        .unwrap_or_else(|| "[]".into());

    let args = body.get("args")
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "null".into()));

    stmt.bind(&[
        id.into(),
        body["name"].as_str().unwrap_or("").into(),
        body["description"].as_str().unwrap_or("").into(),
        body.get("icon_key").and_then(|v| v.as_str()).map(|s| s.into()).unwrap_or(serde_json::Value::Null.into()),
        body.get("icon_hash").and_then(|v| v.as_str()).map(|s| s.into()).unwrap_or(serde_json::Value::Null.into()),
        categories.as_str().into(),
        body["transport"].as_str().unwrap_or("stdio").into(),
        body.get("command").and_then(|v| v.as_str()).map(|s| s.into()).unwrap_or(serde_json::Value::Null.into()),
        args.as_deref().map(|s| s.into()).unwrap_or(serde_json::Value::Null.into()),
        body.get("url").and_then(|v| v.as_str()).map(|s| s.into()).unwrap_or(serde_json::Value::Null.into()),
        body["connectivity"].as_str().unwrap_or("online").into(),
        (if body.get("official").and_then(|v| v.as_bool()).unwrap_or(false) { 1i32 } else { 0i32 }).into(),
        body.get("tool_count").and_then(|v| v.as_i64()).map(|n| (n as i32).into()).unwrap_or(serde_json::Value::Null.into()),
        body.get("security_score").and_then(|v| v.as_str()).map(|s| s.into()).unwrap_or(serde_json::Value::Null.into()),
        now.as_str().into(),
        now.as_str().into(),
    ])?.run().await?;

    Response::from_json(&serde_json::json!({"ok": true, "id": id}))
}
```

- [ ] **Step 2: Update routes/mod.rs**

```rust
pub mod admin;
pub mod catalog;
pub mod devices;
pub mod icons;
pub mod sync;
pub mod user;
```

- [ ] **Step 3: Add admin route to router.rs**

```rust
_ if method == Method::Put && path.starts_with("/v1/admin/catalog/servers/") => {
    let id = path.strip_prefix("/v1/admin/catalog/servers/").unwrap_or("");
    if id.is_empty() {
        Response::error("Missing server ID", 400)
    } else {
        routes::admin::upsert_server(req, &env, id).await
    }
}
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add minimal admin endpoint for catalog upsert"
```

---

### Task 11: Seed catalog data and upload icons

**Files:**
- Create: `plugmux-api/seed/seed.sh`
- Copy: icons from `plugmux/catalog/icons/` to `plugmux-api/icons/`

- [ ] **Step 1: Copy icons**

```bash
cp -r ../plugmux/catalog/icons/* icons/
```

- [ ] **Step 2: Create seed script**

The seed script reads `plugmux/catalog/servers.json` and pushes each entry to the admin API.

```bash
#!/bin/bash
# seed/seed.sh — Seeds catalog data via admin API
# Usage: ./seed/seed.sh <api_url> <admin_secret>

API_URL="${1:-http://localhost:8787}"
ADMIN_SECRET="${2:-dev-admin-secret}"
SERVERS_JSON="../plugmux/catalog/servers.json"

echo "Seeding catalog from $SERVERS_JSON to $API_URL"

# Upload icons to R2 (local dev)
for svg in icons/*.svg; do
    name=$(basename "$svg" .svg)
    echo "Uploading icon: $name"
    npx wrangler r2 object put "plugmux-icons/${name}.svg" --file "$svg" --local 2>/dev/null
done

# Seed servers via admin API
python3 -c "
import json, subprocess, sys

with open('$SERVERS_JSON') as f:
    data = json.load(f)

for server in data['servers']:
    server_id = server['id']
    body = {
        'name': server['name'],
        'description': server['description'],
        'icon_key': server.get('icon', '').replace('.svg', '') + '.svg' if server.get('icon') else None,
        'categories': [server.get('category', 'dev-tools')],
        'transport': server['transport'],
        'command': server.get('command'),
        'args': server.get('args'),
        'url': server.get('url'),
        'connectivity': server.get('connectivity', 'online'),
        'official': True,
    }
    result = subprocess.run([
        'curl', '-s', '-X', 'PUT',
        f'$API_URL/v1/admin/catalog/servers/{server_id}',
        '-H', 'Content-Type: application/json',
        '-H', f'X-Admin-Secret: $ADMIN_SECRET',
        '-d', json.dumps(body)
    ], capture_output=True, text=True)
    print(f'  {server_id}: {result.stdout}')
"

echo "Done!"
```

- [ ] **Step 3: Run seed**

```bash
chmod +x seed/seed.sh
./seed/seed.sh
```

- [ ] **Step 4: Verify**

```bash
curl http://localhost:8787/v1/catalog/servers | python3 -m json.tool
curl http://localhost:8787/v1/icons/figma.svg | head -5
```

Expected: 12 catalog entries, SVG content for figma.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add seed script and icons"
```

---

### Task 12: Final router assembly and deploy

**Files:**
- Modify: `plugmux-api/src/router.rs` — final complete version
- Modify: `plugmux-api/wrangler.toml` — production database ID

- [ ] **Step 1: Write final router.rs**

Assemble the complete router with all routes:

```rust
use worker::*;
use crate::auth;
use crate::routes;

pub async fn handle(req: Request, env: Env) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();
    let method = req.method();

    let cors = |mut resp: Response| -> Result<Response> {
        let headers = resp.headers_mut();
        headers.set("Access-Control-Allow-Origin", "*")?;
        headers.set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")?;
        headers.set("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Admin-Secret")?;
        Ok(resp)
    };

    if method == Method::Options {
        return cors(Response::empty()?);
    }

    let resp = match (method.clone(), path.as_str()) {
        // Public
        (Method::Get, "/v1/health") => Response::ok("ok"),
        (Method::Get, "/v1/catalog/servers") => routes::catalog::list(&req, &env).await,

        // Auth
        (Method::Get, "/v1/auth/github") => auth::github::redirect(&req, &env).await,
        (Method::Get, "/v1/auth/github/callback") => auth::github::callback(&req, &env).await,

        // Protected
        (Method::Post, "/v1/devices/register") => routes::devices::register(req, &env).await,
        (Method::Get, "/v1/sync/pull") => routes::sync::pull(&req, &env).await,
        (Method::Post, "/v1/sync/push") => routes::sync::push(req, &env).await,
        (Method::Get, "/v1/sync/snapshot") => routes::sync::snapshot(&req, &env).await,
        (Method::Get, "/v1/user/profile") => routes::user::profile(&req, &env).await,
        (Method::Delete, "/v1/user/account") => routes::user::delete(&req, &env).await,

        // Dynamic path routes
        _ => {
            if method == Method::Get && path.starts_with("/v1/catalog/servers/") {
                let id = &path["/v1/catalog/servers/".len()..];
                routes::catalog::get(id, &env).await
            } else if method == Method::Get && path.starts_with("/v1/icons/") && path.ends_with(".svg") {
                let id = &path["/v1/icons/".len()..path.len() - 4];
                routes::icons::get(id, &env).await
            } else if method == Method::Put && path.starts_with("/v1/admin/catalog/servers/") {
                let id = &path["/v1/admin/catalog/servers/".len()..];
                routes::admin::upsert_server(req, &env, id).await
            } else {
                Response::error("Not Found", 404)
            }
        }
    };

    cors(resp?)
}
```

- [ ] **Step 2: Set production secrets**

```bash
npx wrangler secret put JWT_SECRET
npx wrangler secret put ADMIN_SECRET
npx wrangler secret put GITHUB_CLIENT_ID
npx wrangler secret put GITHUB_CLIENT_SECRET
```

- [ ] **Step 3: Apply D1 migration to production**

```bash
npx wrangler d1 migrations apply plugmux-db --remote
```

- [ ] **Step 4: Deploy**

```bash
npx wrangler deploy
```

- [ ] **Step 5: Verify production**

```bash
curl https://api.plugmux.com/v1/health
curl https://api.plugmux.com/v1/catalog/servers
```

- [ ] **Step 6: Seed production data**

```bash
./seed/seed.sh https://api.plugmux.com <real-admin-secret>
```

- [ ] **Step 7: Upload icons to production R2**

```bash
for svg in icons/*.svg; do
    name=$(basename "$svg")
    npx wrangler r2 object put "plugmux-icons/$name" --file "$svg"
done
```

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat: final router assembly, ready for deploy"
```

---

### Task 13: Reconciliation script

**Files:**
- Create: `plugmux-api/reconciler/reconcile.py`
- Create: `plugmux-api/reconciler/requirements.txt`

- [ ] **Step 1: Create requirements.txt**

```
requests
```

- [ ] **Step 2: Create reconcile.py**

```python
#!/usr/bin/env python3
"""
Manual reconciliation script for plugmux catalog.
Fetches tool counts from MCP servers and updates the API.

Usage:
    python reconcile.py --api-url https://api.plugmux.com --admin-secret <secret>
    python reconcile.py --api-url http://localhost:8787 --admin-secret dev-admin-secret --dry-run
"""

import argparse
import json
import subprocess
import requests
import sys

def fetch_catalog(api_url, admin_secret):
    """Fetch current catalog from API."""
    resp = requests.get(f"{api_url}/v1/catalog/servers?limit=200")
    resp.raise_for_status()
    return resp.json()["servers"]

def count_tools_stdio(command, args):
    """Connect to stdio MCP server and count tools."""
    try:
        full_cmd = [command] + (args or [])
        # Send initialize + tools/list via stdin
        init_msg = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": {"name": "plugmux-reconciler", "version": "0.1.0"}}})
        tools_msg = json.dumps({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}})
        stdin_data = init_msg + "\n" + tools_msg + "\n"

        result = subprocess.run(full_cmd, input=stdin_data, capture_output=True, text=True, timeout=30)

        # Parse responses
        for line in result.stdout.strip().split("\n"):
            try:
                resp = json.loads(line)
                if resp.get("id") == 2 and "result" in resp:
                    tools = resp["result"].get("tools", [])
                    return len(tools)
            except json.JSONDecodeError:
                continue
        return None
    except Exception as e:
        print(f"    Error: {e}")
        return None

def update_server(api_url, admin_secret, server_id, updates):
    """Update a catalog server via admin API."""
    resp = requests.put(
        f"{api_url}/v1/admin/catalog/servers/{server_id}",
        json=updates,
        headers={"X-Admin-Secret": admin_secret, "Content-Type": "application/json"}
    )
    resp.raise_for_status()
    return resp.json()

def main():
    parser = argparse.ArgumentParser(description="Reconcile plugmux catalog metadata")
    parser.add_argument("--api-url", required=True)
    parser.add_argument("--admin-secret", required=True)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    servers = fetch_catalog(args.api_url, args.admin_secret)
    print(f"Fetched {len(servers)} servers from catalog\n")

    for server in servers:
        sid = server["id"]
        print(f"[{sid}] {server['name']}")

        if server["transport"] == "stdio" and server.get("command"):
            tool_count = count_tools_stdio(server["command"], server.get("args"))
            if tool_count is not None:
                print(f"    Tools: {tool_count} (was: {server.get('tool_count', 'unknown')})")
                if not args.dry_run and tool_count != server.get("tool_count"):
                    update_server(args.api_url, args.admin_secret, sid, {**server, "tool_count": tool_count})
                    print(f"    Updated!")
            else:
                print(f"    Failed to connect")
        elif server["transport"] == "http" and server.get("url"):
            print(f"    HTTP server — manual check needed: {server['url']}")
        else:
            print(f"    Skipped (no command/url)")
        print()

    print("Done!" + (" (dry run)" if args.dry_run else ""))

if __name__ == "__main__":
    main()
```

- [ ] **Step 3: Test dry run**

```bash
cd reconciler
pip install -r requirements.txt
python reconcile.py --api-url http://localhost:8787 --admin-secret dev-admin-secret --dry-run
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add manual reconciliation script"
```

---

*End of Plan 1 — plugmux-api*
