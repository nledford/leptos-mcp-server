//! Library surface for the Leptos MCP server.
//!
//! The SDK adapter owns MCP protocol, transport, and capability registration;
//! domain modules (`docs`, `tools`, `prompts`, `recipes`, `diagnostics`) provide
//! Leptos MCP capabilities.

pub mod api;
pub mod app;
pub mod diagnostics;
pub mod docs;
pub mod prompts;
pub mod recipes;
pub mod sdk;
#[cfg(feature = "stdio")]
mod stdio_transport;
pub mod tools;
