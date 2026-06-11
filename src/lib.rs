//! Library surface for the Leptos MCP server.
//!
//! Protocol code (`protocol`) validates and routes JSON-RPC/MCP messages;
//! transport code (`transport`) handles stdio line framing; domain modules
//! (`docs`, `tools`, `prompts`, `recipes`, `diagnostics`) provide Leptos MCP
//! capabilities, with `catalog` assembling their public capability metadata.

pub mod api;
pub mod app;
pub mod catalog;
pub mod diagnostics;
pub mod docs;
pub mod prompts;
pub mod protocol;
pub mod recipes;
pub mod tools;
mod transport;
