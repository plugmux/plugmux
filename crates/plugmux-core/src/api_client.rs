//! HTTP client for the plugmux cloud API.
//!
//! Fetches remote catalog, collections, and handles sync.

use serde::{Deserialize, Serialize};

/// API client for communicating with the plugmux backend.
#[derive(Debug, Clone)]
pub struct ApiClient {
    base_url: String,
    http: reqwest::Client,
    token: Option<String>,
}

/// A catalog server as returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCatalogServer {
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
    pub tool_count: Option<i64>,
    pub security_score: Option<String>,
    pub smithery_url: Option<String>,
    pub added_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogResponse {
    pub servers: Vec<RemoteCatalogServer>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCollection {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub sort_order: i64,
    #[serde(default)]
    pub server_ids: Vec<String>,
    #[serde(default)]
    pub servers: Vec<RemoteCatalogServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionsResponse {
    pub collections: Vec<RemoteCollection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: AuthUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub github_username: String,
    pub email: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("plugmux-app")
            .build()
            .expect("failed to create HTTP client");
        Self {
            base_url,
            http,
            token: None,
        }
    }

    /// Set the auth token (JWT) for protected requests.
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn clear_token(&mut self) {
        self.token = None;
    }

    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    // ── Public endpoints ──

    pub async fn health(&self) -> Result<HealthResponse, String> {
        self.get("/v1/health").await
    }

    pub async fn list_servers(
        &self,
        limit: Option<u32>,
        cursor: Option<&str>,
        search: Option<&str>,
        category: Option<&str>,
    ) -> Result<CatalogResponse, String> {
        let mut params = Vec::new();
        if let Some(l) = limit {
            params.push(format!("limit={l}"));
        }
        if let Some(c) = cursor {
            params.push(format!("cursor={c}"));
        }
        if let Some(q) = search {
            params.push(format!("q={}", urlencoding::encode(q)));
        }
        if let Some(cat) = category {
            params.push(format!("category={}", urlencoding::encode(cat)));
        }
        let qs = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };
        self.get(&format!("/v1/catalog/servers{qs}")).await
    }

    pub async fn get_server(&self, id: &str) -> Result<RemoteCatalogServer, String> {
        self.get(&format!("/v1/catalog/servers/{id}")).await
    }

    pub async fn list_collections(&self) -> Result<CollectionsResponse, String> {
        self.get("/v1/catalog/collections").await
    }

    pub async fn get_collection(&self, id: &str) -> Result<RemoteCollection, String> {
        self.get(&format!("/v1/catalog/collections/{id}")).await
    }

    /// Get the GitHub OAuth URL to redirect the user to.
    pub fn github_auth_url(&self) -> String {
        format!("{}/v1/auth/github", self.base_url)
    }

    // ── Protected endpoints ──

    pub async fn register_device(
        &self,
        device_id: &str,
        name: &str,
    ) -> Result<serde_json::Value, String> {
        self.post_json(
            "/v1/devices/register",
            &serde_json::json!({ "device_id": device_id, "name": name }),
        )
        .await
    }

    pub async fn get_profile(&self) -> Result<AuthUser, String> {
        self.get_authed("/v1/user/profile").await
    }

    // ── Internal helpers ──

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()));
        }

        resp.json::<T>()
            .await
            .map_err(|e| format!("parse error: {e}"))
    }

    async fn get_authed<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| "not authenticated".to_string())?;
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()));
        }

        resp.json::<T>()
            .await
            .map_err(|e| format!("parse error: {e}"))
    }

    async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<T, String> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| "not authenticated".to_string())?;
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .json(body)
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()));
        }

        resp.json::<T>()
            .await
            .map_err(|e| format!("parse error: {e}"))
    }
}
