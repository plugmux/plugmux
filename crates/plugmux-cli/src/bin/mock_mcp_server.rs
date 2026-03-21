//! Mock MCP server for integration testing.
//!
//! Speaks the MCP JSON-RPC 2.0 protocol over stdio using rmcp's server-side
//! handler infrastructure.  Exposes one tool: "echo" that takes a `message`
//! parameter and returns it verbatim.

use rmcp::{ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for the echo tool.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EchoRequest {
    /// The message to echo back.
    pub message: String,
}

/// A minimal MCP server that exposes a single "echo" tool.
#[derive(Debug, Clone)]
struct EchoServer {
    tool_router: ToolRouter<Self>,
}

impl EchoServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl EchoServer {
    /// Echo a message back to the caller.
    #[tool(name = "echo", description = "Echo a message back")]
    async fn echo(&self, params: Parameters<EchoRequest>) -> String {
        params.0.message
    }
}

#[tool_handler]
impl ServerHandler for EchoServer {}

#[tokio::main]
async fn main() {
    let server = EchoServer::new();
    let transport = rmcp::transport::io::stdio();
    let service = server.serve(transport).await.expect("failed to start MCP server");
    service.waiting().await.expect("server error");
}
