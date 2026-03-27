//! End-to-end integration test for the plugmux gateway.
//!
//! Validates the full flow: start mock MCP server -> build gateway -> send
//! JSON-RPC requests -> verify responses -> cleanup.
//!
//! Architecture under test:
//! - `/env/global`    → PlugmuxLayer: exposes plugmux__ management tools
//! - `/env/test-env`  → ProxyLayer:   directly exposes upstream server tools

use std::sync::Arc;

use serde_json::{Value, json};
use tokio::sync::RwLock;

use plugmux_core::config::{Config, PermissionLevel, Permissions};
use plugmux_core::db::Db;
use plugmux_core::db::environments as db_env;
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
    resp.json::<Value>()
        .await
        .expect("failed to parse JSON response")
}

#[tokio::test]
async fn test_full_gateway_flow() {
    // -----------------------------------------------------------------------
    // 1. Build config and populate SQLite database with environments
    // -----------------------------------------------------------------------
    let config = Config {
        port: 4242,
        permissions: Permissions {
            enable_server: PermissionLevel::Allow,
            disable_server: PermissionLevel::Allow,
        },
        device_id: String::new(),
        onboarding_shown: false,
    };

    let db = Db::open_in_memory().expect("failed to open in-memory database");
    db_env::add_environment(&db, "test-env", "Test Environment").expect("failed to add test-env");
    db_env::add_server(&db, "test-env", "mock-echo").expect("failed to add mock-echo to test-env");

    // -----------------------------------------------------------------------
    // 2. Create ServerManager and start the mock server manually
    //    (mock-echo is not in the catalog, so we start it via ServerManager)
    // -----------------------------------------------------------------------
    let mock_server_config = ServerConfig {
        id: "mock-echo".to_string(),
        name: "Mock Echo".to_string(),
        transport: Transport::Stdio,
        command: Some(MOCK_SERVER_BIN.to_string()),
        args: Some(vec![]),
        url: None,
        connectivity: Connectivity::Local,
        description: Some("Mock echo server for testing".to_string()),
    };

    let manager = Arc::new(ServerManager::new());
    manager
        .start_server(mock_server_config)
        .await
        .expect("failed to start mock server");

    // Verify it's healthy
    assert!(
        manager.is_healthy("mock-echo").await,
        "mock server should be healthy"
    );

    // -----------------------------------------------------------------------
    // 3. Build the axum router and spawn the HTTP server on a random port
    // -----------------------------------------------------------------------
    let config = Arc::new(RwLock::new(config));
    let router = build_router(config.clone(), manager.clone(), Some(db), None);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind to random port");
    let addr = listener.local_addr().expect("failed to get local address");
    let base_url = format!("http://127.0.0.1:{}", addr.port());

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.ok();
    });

    let client = reqwest::Client::new();
    let env_url = format!("{base_url}/env/test-env");
    let global_url = format!("{base_url}/env/global");

    // -----------------------------------------------------------------------
    // 4. Test: initialize (works on any env)
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
    assert_eq!(server_info["version"], "0.2.0");

    // -----------------------------------------------------------------------
    // 5. Test: tools/list on /env/global — should return plugmux__ management tools
    // -----------------------------------------------------------------------
    let resp = jsonrpc_request(
        &client,
        &global_url,
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
    assert_eq!(
        tools.len(),
        4,
        "global env should expose 4 plugmux management tools"
    );

    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(tool_names.contains(&"plugmux__enable_server"));
    assert!(tool_names.contains(&"plugmux__disable_server"));
    assert!(tool_names.contains(&"plugmux__add_environment"));
    assert!(tool_names.contains(&"plugmux__confirm_action"));

    // -----------------------------------------------------------------------
    // 6. Test: tools/list on /env/test-env — should return upstream echo tool namespaced
    //    Proxy layer namespaces tools as server_id__tool_name (e.g. mock-echo__echo)
    // -----------------------------------------------------------------------
    let resp = jsonrpc_request(
        &client,
        &env_url,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/list",
            "params": {}
        }),
    )
    .await;

    let tools = resp["result"]["tools"]
        .as_array()
        .expect("tools should be an array");
    assert!(
        !tools.is_empty(),
        "test-env should expose at least one upstream tool"
    );
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(
        tool_names.contains(&"mock-echo__echo"),
        "test-env should expose 'mock-echo__echo' tool from mock-echo server; got: {tool_names:?}"
    );

    // -----------------------------------------------------------------------
    // 7. Test: tools/call on /env/test-env — call echo via proxy layer (namespaced)
    // -----------------------------------------------------------------------
    let resp = jsonrpc_request(
        &client,
        &env_url,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "mock-echo__echo",
                "arguments": {
                    "message": "hello from plugmux"
                }
            }
        }),
    )
    .await;

    assert!(
        resp["error"].is_null(),
        "echo tool call should not error: {resp}"
    );
    // Proxy layer returns the raw rmcp content array as the result
    let result = &resp["result"];
    let content = result
        .as_array()
        .unwrap_or_else(|| panic!("echo result should be a content array: {resp}"));
    assert!(
        !content.is_empty(),
        "echo result content should not be empty: {resp}"
    );
    let echo_text = content[0]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("unexpected echo result format: {resp}"));
    assert_eq!(echo_text, "hello from plugmux");

    // -----------------------------------------------------------------------
    // 8. Test: health endpoint
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
    // 9. Cleanup
    // -----------------------------------------------------------------------
    manager.shutdown_all().await;
    server_handle.abort();
}
