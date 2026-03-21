//! HTTP router for the plugmux gateway.
//!
//! Exposes a per-environment MCP JSON-RPC endpoint and a health check.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::{Value, json};
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::config::Config;
use crate::gateway::tools::GatewayTools;
use crate::manager::ServerManager;
use crate::proxy::ProxyError;

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    tools: Arc<GatewayTools>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build the axum [`Router`] with all gateway routes.
pub fn build_router(
    config: Arc<RwLock<Config>>,
    manager: Arc<ServerManager>,
) -> Router {
    let tools = Arc::new(GatewayTools::new(config, manager));
    let state = AppState { tools };

    Router::new()
        .route("/env/{env_id}", post(handle_jsonrpc))
        .route("/health", get(handle_health))
        .with_state(state)
}

/// Start the axum HTTP server on the given port.
pub async fn start_server(
    config: Arc<RwLock<Config>>,
    manager: Arc<ServerManager>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let router = build_router(config, manager);
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
// JSON-RPC handler
// ---------------------------------------------------------------------------

async fn handle_jsonrpc(
    State(state): State<AppState>,
    Path(env_id): Path<String>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let id = body.get("id").cloned().unwrap_or(Value::Null);
    let method = body
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("");
    let params = body.get("params").cloned().unwrap_or(Value::Null);

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
        Err(ProxyError::ApprovalRequired {
            action_id,
            message,
        }) => (
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
                }
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
}

/// Dispatch a JSON-RPC method to the appropriate handler.
async fn dispatch(
    tools: &GatewayTools,
    env_id: &str,
    method: &str,
    params: &Value,
) -> Result<Value, ProxyError> {
    match method {
        "initialize" => Ok(handle_initialize()),
        "tools/list" => Ok(handle_tools_list()),
        "tools/call" => handle_tools_call(tools, env_id, params).await,
        _ => Err(ProxyError::Transport(format!("unknown method: {method}"))),
    }
}

// ---------------------------------------------------------------------------
// initialize
// ---------------------------------------------------------------------------

fn handle_initialize() -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "plugmux",
            "version": "0.1.0"
        }
    })
}

// ---------------------------------------------------------------------------
// tools/list — return schemas for our 6 gateway tools
// ---------------------------------------------------------------------------

fn handle_tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "list_servers",
                "description": "List all MCP servers available in this environment",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            },
            {
                "name": "get_tools",
                "description": "Get the full tool list for a specific MCP server",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "server_id": {
                            "type": "string",
                            "description": "The server identifier"
                        }
                    },
                    "required": ["server_id"]
                }
            },
            {
                "name": "execute",
                "description": "Execute a tool on a specific MCP server",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "server_id": {
                            "type": "string",
                            "description": "The server identifier"
                        },
                        "tool_name": {
                            "type": "string",
                            "description": "The tool to execute"
                        },
                        "args": {
                            "type": "object",
                            "description": "Arguments to pass to the tool"
                        }
                    },
                    "required": ["server_id", "tool_name"]
                }
            },
            {
                "name": "enable_server",
                "description": "Add a server to this environment (server must exist in catalog or custom servers)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "server_id": {
                            "type": "string",
                            "description": "The server identifier"
                        }
                    },
                    "required": ["server_id"]
                }
            },
            {
                "name": "disable_server",
                "description": "Remove a server from this environment",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "server_id": {
                            "type": "string",
                            "description": "The server identifier"
                        }
                    },
                    "required": ["server_id"]
                }
            },
            {
                "name": "confirm_action",
                "description": "Confirm a pending action that requires user approval",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action_id": {
                            "type": "string",
                            "description": "The action ID"
                        }
                    },
                    "required": ["action_id"]
                }
            }
        ]
    })
}

// ---------------------------------------------------------------------------
// tools/call — route to the appropriate GatewayTools method
// ---------------------------------------------------------------------------

async fn handle_tools_call(
    tools: &GatewayTools,
    env_id: &str,
    params: &Value,
) -> Result<Value, ProxyError> {
    let tool_name = params
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| {
            ProxyError::Transport("missing 'name' in tools/call params".to_string())
        })?;

    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));

    match tool_name {
        "list_servers" => {
            let servers = tools.list_servers(env_id).await?;

            let result: Vec<Value> = servers
                .into_iter()
                .map(|s| {
                    json!({
                        "id": s.id,
                        "name": s.name,
                        "healthy": s.healthy,
                        "tool_count": s.tool_count,
                    })
                })
                .collect();

            Ok(wrap_content(&serde_json::to_string(&result).unwrap()))
        }

        "get_tools" => {
            let server_id = args
                .get("server_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ProxyError::Transport("missing 'server_id' argument".to_string())
                })?;

            let tool_list = tools.get_tools(server_id).await?;

            let result: Vec<Value> = tool_list
                .into_iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "inputSchema": t.input_schema,
                    })
                })
                .collect();

            Ok(wrap_content(&serde_json::to_string(&result).unwrap()))
        }

        "execute" => {
            let server_id = args
                .get("server_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ProxyError::Transport("missing 'server_id' argument".to_string())
                })?;

            let exec_tool_name = args
                .get("tool_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ProxyError::Transport("missing 'tool_name' argument".to_string())
                })?;

            let exec_args = args
                .get("args")
                .cloned()
                .unwrap_or(Value::Object(Default::default()));

            let result = tools
                .execute(server_id, exec_tool_name, exec_args)
                .await?;

            // Pass through the upstream result directly.
            Ok(result)
        }

        "enable_server" => {
            let server_id = args
                .get("server_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ProxyError::Transport("missing 'server_id' argument".to_string())
                })?;

            tools.enable_server(env_id, server_id).await?;

            Ok(wrap_content("server added to environment"))
        }

        "disable_server" => {
            let server_id = args
                .get("server_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ProxyError::Transport("missing 'server_id' argument".to_string())
                })?;

            tools.disable_server(env_id, server_id).await?;

            Ok(wrap_content("server removed from environment"))
        }

        "confirm_action" => {
            let action_id = args
                .get("action_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ProxyError::Transport("missing 'action_id' argument".to_string())
                })?;

            tools.confirm_action(action_id).await?;

            Ok(wrap_content("action confirmed and executed"))
        }

        _ => Err(ProxyError::Transport(format!(
            "unknown tool: {tool_name}"
        ))),
    }
}

/// Wrap a text string into the MCP content format.
fn wrap_content(text: &str) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": text,
            }
        ]
    })
}
