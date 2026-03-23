//! Relay module — handles roots forwarding and future sampling/elicitation relay.
//!
//! Currently implements:
//! - Roots forwarding (agent → all backend servers) via ProxyLayer::broadcast_roots()
//!
//! Future (requires SSE transport upgrade):
//! - Sampling relay (backend → agent)
//! - Elicitation relay (backend → agent → user)
