//! End-to-end integration test for the plugmux gateway.
//!
//! Validates the full flow: start mock MCP server -> build gateway -> send
//! JSON-RPC requests -> verify responses -> cleanup.

use std::sync::Arc;

use serde_json::{Value, json};
use tokio::sync::RwLock;

use plugmux_core::config::{
    EnvironmentConfig, MainConfig, PlugmuxConfig,
};
use plugmux_core::gateway::router::build_router;
use plugmux_core::manager::ServerManager;
use plugmux_core::server::{Connectivity, ServerConfig, Transport};

/// Path to the mock-mcp-server binary, resolved at compile time by Cargo.
const MOCK_SERVER_BIN: &str = env!("CARGO_BIN_EXE_mock-mcp-server");

/// Helper: send a JSON-RPC request to the gateway and return the parsed response.
async fn jsonrpc_request(client: &reqwest::Client, url: &str, body: Value) -> Value {
    let resp = client
        .post(url)
        .json(&body)
        .send()
        .await
        .expect("HTTP request failed");
    assert_eq!(resp.status(), 200);
    resp.json::<Value>().await.expect("failed to parse JSON response")
}

#[tokio::test]
async fn test_full_gateway_flow() {
    // -----------------------------------------------------------------------
    // 1. Build config with a mock server in Main and one environment
    // -----------------------------------------------------------------------
    let config = PlugmuxConfig {
        main: MainConfig {
            servers: vec![ServerConfig {
                id: "mock-echo".to_string(),
                name: "Mock Echo".to_string(),
                transport: Transport::Stdio,
                command: Some(MOCK_SERVER_BIN.to_string()),
                args: Some(vec![]),
                url: None,
                connectivity: Connectivity::Local,
                enabled: true,
                description: Some("Mock echo server for testing".to_string()),
            }],
        },
        environments: vec![EnvironmentConfig {
            id: "main".to_string(),
            name: "Main".to_string(),
            endpoint: "http://localhost:0/env/main".to_string(),
            servers: vec![],
            overrides: vec![],
        }],
    };

    // -----------------------------------------------------------------------
    // 2. Create ServerManager and start the mock server
    // -----------------------------------------------------------------------
    let manager = Arc::new(ServerManager::new());
    manager
        .start_server(config.main.servers[0].clone())
        .await
        .expect("failed to start mock server");

    // Verify it's healthy
    assert!(manager.is_healthy("mock-echo").await, "mock server should be healthy");

    // -----------------------------------------------------------------------
    // 3. Build the axum router and spawn the HTTP server on a random port
    // -----------------------------------------------------------------------
    let config = Arc::new(RwLock::new(config));
    let router = build_router(config.clone(), manager.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind to random port");
    let addr = listener.local_addr().expect("failed to get local address");
    let base_url = format!("http://127.0.0.1:{}", addr.port());

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.ok();
    });

    let client = reqwest::Client::new();
    let env_url = format!("{base_url}/env/main");

    // -----------------------------------------------------------------------
    // 4. Test: initialize
    // -----------------------------------------------------------------------
    let resp = jsonrpc_request(
        &client,
        &env_url,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }),
    )
    .await;

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    let server_info = &resp["result"]["serverInfo"];
    assert_eq!(server_info["name"], "plugmux");
    assert_eq!(server_info["version"], "0.1.0");

    // -----------------------------------------------------------------------
    // 5. Test: tools/list — should return 5 gateway tools
    // -----------------------------------------------------------------------
    let resp = jsonrpc_request(
        &client,
        &env_url,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }),
    )
    .await;

    let tools = resp["result"]["tools"]
        .as_array()
        .expect("tools should be an array");
    assert_eq!(tools.len(), 5, "gateway should expose 5 tools");

    let tool_names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(tool_names.contains(&"list_servers"));
    assert!(tool_names.contains(&"get_tools"));
    assert!(tool_names.contains(&"execute"));
    assert!(tool_names.contains(&"enable_server"));
    assert!(tool_names.contains(&"disable_server"));

    // -----------------------------------------------------------------------
    // 6. Test: tools/call list_servers — mock server should be listed
    // -----------------------------------------------------------------------
    let resp = jsonrpc_request(
        &client,
        &env_url,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "list_servers",
                "arguments": {}
            }
        }),
    )
    .await;

    assert!(resp["error"].is_null(), "list_servers should not error: {resp}");
    let content_text = resp["result"]["content"][0]["text"]
        .as_str()
        .expect("should have text content");
    let servers: Vec<Value> =
        serde_json::from_str(content_text).expect("should be valid JSON array");
    assert_eq!(servers.len(), 1, "should have exactly one server");
    assert_eq!(servers[0]["id"], "mock-echo");
    assert_eq!(servers[0]["name"], "Mock Echo");
    assert_eq!(servers[0]["healthy"], true);
    assert!(
        servers[0]["tool_count"].as_u64().unwrap() >= 1,
        "mock server should expose at least 1 tool"
    );

    // -----------------------------------------------------------------------
    // 7. Test: tools/call get_tools — echo tool should be listed
    // -----------------------------------------------------------------------
    let resp = jsonrpc_request(
        &client,
        &env_url,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "get_tools",
                "arguments": {
                    "server_id": "mock-echo"
                }
            }
        }),
    )
    .await;

    assert!(resp["error"].is_null(), "get_tools should not error: {resp}");
    let content_text = resp["result"]["content"][0]["text"]
        .as_str()
        .expect("should have text content");
    let tools: Vec<Value> =
        serde_json::from_str(content_text).expect("should be valid JSON array");
    let echo_tool = tools
        .iter()
        .find(|t| t["name"] == "echo")
        .expect("echo tool should be listed");
    assert_eq!(echo_tool["description"], "Echo a message back");

    // -----------------------------------------------------------------------
    // 8. Test: tools/call execute — call echo tool, verify response
    // -----------------------------------------------------------------------
    let resp = jsonrpc_request(
        &client,
        &env_url,
        json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "execute",
                "arguments": {
                    "server_id": "mock-echo",
                    "tool_name": "echo",
                    "args": {
                        "message": "hello from plugmux"
                    }
                }
            }
        }),
    )
    .await;

    assert!(resp["error"].is_null(), "execute should not error: {resp}");
    // The execute response is the raw upstream result — an array of content items.
    let result = &resp["result"];
    // rmcp returns content as an array of {type, text} objects
    let echo_text = result
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text").or_else(|| item.get("raw").and_then(|r| r.get("text"))))
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("unexpected execute result format: {result}"));
    assert_eq!(echo_text, "hello from plugmux");

    // -----------------------------------------------------------------------
    // 9. Test: health endpoint
    // -----------------------------------------------------------------------
    let health_resp = client
        .get(format!("{base_url}/health"))
        .send()
        .await
        .expect("health request failed");
    assert_eq!(health_resp.status(), 200);
    let health_body: Value = health_resp.json().await.unwrap();
    assert_eq!(health_body["status"], "ok");

    // -----------------------------------------------------------------------
    // 10. Cleanup
    // -----------------------------------------------------------------------
    manager.shutdown_all().await;
    server_handle.abort();
}
