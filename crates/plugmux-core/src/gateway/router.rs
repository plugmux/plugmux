//! HTTP router for the plugmux gateway.
//!
//! Dispatches JSON-RPC requests to either the **plugmux layer** (for
//! `/env/global`) or the **proxy layer** (for project environments).

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};
use futures::stream::Stream;
use serde_json::{Value, json};
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::config::Config;
use crate::db::Db;
use crate::manager::ServerManager;
use crate::plugmux_layer::PlugmuxLayer;
use crate::proxy::{PromptInfo, ProxyError, ResourceInfo, ToolInfo};
use crate::proxy_layer::ProxyLayer;

use super::{agent_detect, logging};

use crate::config::GLOBAL_ENV;

// ---------------------------------------------------------------------------
// Gateway callbacks — called from the hot path on every request
// ---------------------------------------------------------------------------

/// Information passed to the gateway callback on each JSON-RPC request.
#[derive(Debug, Clone)]
pub struct RequestEvent {
    pub agent_id: Option<String>,
    pub method: String,
    pub env_id: String,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// Callback type for gateway events.
/// The Tauri layer provides an implementation that emits UI events.
pub type OnRequest = Arc<dyn Fn(RequestEvent) + Send + Sync>;

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    plugmux: Arc<PlugmuxLayer>,
    proxy: Arc<ProxyLayer>,
    db: Option<Arc<Db>>,
    on_request: Option<OnRequest>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build the axum [`Router`] with all gateway routes.
pub fn build_router(
    config: Arc<RwLock<Config>>,
    manager: Arc<ServerManager>,
    db: Option<Arc<Db>>,
    on_request: Option<OnRequest>,
) -> Router {
    let plugmux = Arc::new(PlugmuxLayer::new(config.clone(), manager.clone(), db.clone()));
    let proxy = Arc::new(ProxyLayer::new(config, manager, db.clone()));
    let state = AppState { plugmux, proxy, db, on_request };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers(Any);

    Router::new()
        .route("/env/{env_id}", post(handle_jsonrpc).get(handle_sse))
        .route("/health", get(handle_health))
        .layer(cors)
        .with_state(state)
}

/// Start the axum HTTP server on the given port.
pub async fn start_server(
    config: Arc<RwLock<Config>>,
    manager: Arc<ServerManager>,
    port: u16,
    db: Option<Arc<Db>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let router = build_router(config, manager, db, None);
    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("plugmux gateway listening on http://{addr}");
    axum::serve(listener, router).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Health endpoint
// ---------------------------------------------------------------------------

async fn handle_health() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

// ---------------------------------------------------------------------------
// SSE endpoint (GET) — Streamable HTTP transport
// ---------------------------------------------------------------------------

/// MCP Streamable HTTP requires a GET endpoint that returns an SSE stream.
/// This keeps the connection open for server-initiated messages (notifications,
/// sampling, elicitation). For now it sends a keep-alive ping.
async fn handle_sse(
    Path(_env_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = futures::stream::unfold((), |()| async {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        Some((Ok(Event::default().comment("keep-alive")), ()))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// ---------------------------------------------------------------------------
// JSON-RPC handler
// ---------------------------------------------------------------------------

async fn handle_jsonrpc(
    State(state): State<AppState>,
    Path(env_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let id = body.get("id").cloned().unwrap_or(Value::Null);
    let method = body.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = body.get("params").cloned().unwrap_or(Value::Null);

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let agent_id = user_agent.as_deref().and_then(agent_detect::detect_agent);

    let result = dispatch(&state, &env_id, method, &params).await;
    let duration = start.elapsed();

    // Log to DB
    if let Some(ref db) = state.db {
        let log_result = match &result {
            Ok(v) => Ok(v.clone()),
            Err(e) => Err(e.to_string()),
        };
        logging::log_request(&logging::LogRequestParams {
            db,
            env_id: &env_id,
            method,
            params: &params,
            result: &log_result,
            duration,
            user_agent: user_agent.as_deref(),
            agent_id: agent_id.as_deref(),
            session_id: "default-session",
        });
    }

    // Notify the Tauri layer
    if let Some(ref cb) = state.on_request {
        cb(RequestEvent {
            agent_id: agent_id.clone(),
            method: method.to_string(),
            env_id: env_id.clone(),
            duration_ms: duration.as_millis() as u64,
            error: match &result {
                Err(e) => Some(e.to_string()),
                Ok(_) => None,
            },
        });
    }

    // Streamable HTTP requires Mcp-Session-Id header
    let session_id = format!("plugmux-{env_id}");
    let mut resp_headers = HeaderMap::new();
    if let Ok(val) = session_id.parse() {
        resp_headers.insert("mcp-session-id", val);
    }

    let body = match result {
        Ok(value) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": value,
        }),
        Err(ProxyError::ApprovalRequired { action_id, message }) => json!({
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
            }
        }),
        Err(err) => {
            error!(method = %method, env = %env_id, error = %err, "JSON-RPC error");
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32603,
                    "message": err.to_string(),
                },
            })
        }
    };

    (StatusCode::OK, resp_headers, Json(body))
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

/// Dispatch a JSON-RPC method to the appropriate layer.
async fn dispatch(
    state: &AppState,
    env_id: &str,
    method: &str,
    params: &Value,
) -> Result<Value, ProxyError> {
    match method {
        "initialize" => Ok(handle_initialize()),
        "notifications/initialized" => Ok(Value::Null),
        "ping" => Ok(json!({})),
        "tools/list" => dispatch_tools_list(state, env_id).await,
        "tools/call" => dispatch_tools_call(state, env_id, params).await,
        "resources/list" => dispatch_resources_list(state, env_id).await,
        "resources/read" => dispatch_resources_read(state, env_id, params).await,
        "prompts/list" => dispatch_prompts_list(state, env_id).await,
        "prompts/get" => dispatch_prompts_get(state, env_id, params).await,
        "notifications/roots/updated" => {
            if env_id != GLOBAL_ENV {
                state.proxy.broadcast_roots(env_id, params.clone()).await?;
            }
            Ok(Value::Null)
        }
        _ => Err(ProxyError::Transport(format!("unknown method: {method}"))),
    }
}

// ---------------------------------------------------------------------------
// initialize
// ---------------------------------------------------------------------------

fn handle_initialize() -> Value {
    json!({
        "protocolVersion": "2025-03-26",
        "capabilities": {
            "tools": { "listChanged": true },
            "resources": { "subscribe": false, "listChanged": true },
            "prompts": { "listChanged": true }
        },
        "serverInfo": {
            "name": "plugmux",
            "version": "0.2.0"
        }
    })
}

// ---------------------------------------------------------------------------
// Dispatch helpers
// ---------------------------------------------------------------------------

async fn dispatch_tools_list(state: &AppState, env_id: &str) -> Result<Value, ProxyError> {
    let tools: Vec<ToolInfo> = if env_id == GLOBAL_ENV {
        state.plugmux.list_tools()
    } else {
        state.proxy.list_tools(env_id).await?
    };

    let tools_json: Vec<Value> = tools
        .into_iter()
        .map(|t| {
            let mut obj = json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema,
            });
            if let Some(output_schema) = t.output_schema {
                obj["outputSchema"] = output_schema;
            }
            if let Some(annotations) = t.annotations {
                obj["annotations"] = annotations;
            }
            obj
        })
        .collect();

    Ok(json!({ "tools": tools_json }))
}

async fn dispatch_tools_call(
    state: &AppState,
    env_id: &str,
    params: &Value,
) -> Result<Value, ProxyError> {
    let name = params
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| ProxyError::Transport("missing 'name' in tools/call params".to_string()))?;

    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));

    if env_id == GLOBAL_ENV {
        state.plugmux.call_tool(name, args).await
    } else {
        state.proxy.call_tool(name, args).await
    }
}

async fn dispatch_resources_list(state: &AppState, env_id: &str) -> Result<Value, ProxyError> {
    let resources: Vec<ResourceInfo> = if env_id == GLOBAL_ENV {
        state.plugmux.list_resources()
    } else {
        state.proxy.list_resources(env_id).await?
    };

    let resources_json: Vec<Value> = resources
        .into_iter()
        .map(|r| {
            let mut obj = json!({
                "uri": r.uri,
                "name": r.name,
            });
            if let Some(desc) = r.description {
                obj["description"] = json!(desc);
            }
            if let Some(mime) = r.mime_type {
                obj["mimeType"] = json!(mime);
            }
            obj
        })
        .collect();

    Ok(json!({ "resources": resources_json }))
}

async fn dispatch_resources_read(
    state: &AppState,
    env_id: &str,
    params: &Value,
) -> Result<Value, ProxyError> {
    let uri = params.get("uri").and_then(|u| u.as_str()).ok_or_else(|| {
        ProxyError::Transport("missing 'uri' in resources/read params".to_string())
    })?;

    if env_id == GLOBAL_ENV {
        state.plugmux.read_resource(uri).await
    } else {
        state.proxy.read_resource(uri).await
    }
}

async fn dispatch_prompts_list(state: &AppState, env_id: &str) -> Result<Value, ProxyError> {
    if env_id == GLOBAL_ENV {
        // Plugmux layer does not expose prompts.
        Ok(json!({ "prompts": [] }))
    } else {
        let prompts: Vec<PromptInfo> = state.proxy.list_prompts(env_id).await?;

        let prompts_json: Vec<Value> = prompts
            .into_iter()
            .map(|p| {
                let args_json: Vec<Value> = p
                    .arguments
                    .into_iter()
                    .map(|a| {
                        let mut obj = json!({ "name": a.name, "required": a.required });
                        if let Some(desc) = a.description {
                            obj["description"] = json!(desc);
                        }
                        obj
                    })
                    .collect();

                let mut obj = json!({
                    "name": p.name,
                    "arguments": args_json,
                });
                if let Some(desc) = p.description {
                    obj["description"] = json!(desc);
                }
                obj
            })
            .collect();

        Ok(json!({ "prompts": prompts_json }))
    }
}

async fn dispatch_prompts_get(
    state: &AppState,
    env_id: &str,
    params: &Value,
) -> Result<Value, ProxyError> {
    if env_id == GLOBAL_ENV {
        return Err(ProxyError::Transport(
            "prompts are not available on the global environment".to_string(),
        ));
    }

    let name = params
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| ProxyError::Transport("missing 'name' in prompts/get params".to_string()))?;

    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));

    state.proxy.get_prompt(name, args).await
}
