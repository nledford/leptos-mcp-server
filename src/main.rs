//! Leptos MCP Server
//!
//! A Model Context Protocol server that provides Leptos documentation
//! and code assistance tools for AI agents.
//!
//! Implements MCP protocol via JSON-RPC over stdio.

use anyhow::Result;
use leptos_mcp_server::protocol::McpServer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive("leptos_mcp_server=info".parse()?))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting Leptos MCP Server...");

    let server = McpServer::new();
    server.run().await?;

    Ok(())
}
