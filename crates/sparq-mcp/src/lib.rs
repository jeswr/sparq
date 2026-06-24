// [OPUS-4.8] (sq-0z43i, gh #909) Render the crate README as the docs.rs / rustdoc
// front page so crates.io and docs.rs show the same content.
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! ## Crate layout
//!
//! - [`McpServer`] — the server: hold a loaded [`sparq_core::Graph`] plus a
//!   [`ServerConfig`], then drive it via [`McpServer::handle_message`] (embed your own
//!   transport) or, with the `stdio` feature, the `serve_stdio` loop.
//! - [`jsonrpc`] — the minimal JSON-RPC 2.0 framing the MCP wire format uses.
//! - [`tools`] — the advertised tool specs (`name` / `description` / `inputSchema`).
//! - [`shapes`] — the structured `shapes` grounding tool (the data-grounded
//!   predicate/datatype/cardinality constraints for one class IRI; no server-side model).
//! - `nlq` (feature `nlq`) — the server-side natural-language `ask` tool (NL→SPARQL→execute
//!   via `sparq-nlq`; embeds a configurable LLM call, degrades cleanly when none is set).
//! - `transport` (feature `stdio`) — the line-delimited stdio serve loop.

pub mod jsonrpc;
pub mod server;
pub mod shapes;
pub mod tools;

// [OPUS-4.8] sq-jxjgr: the server-side NL `ask` tool, behind the opt-in `nlq` feature.
#[cfg(feature = "nlq")]
#[cfg_attr(docsrs, doc(cfg(feature = "nlq")))]
pub mod nlq;

#[cfg(feature = "stdio")]
#[cfg_attr(docsrs, doc(cfg(feature = "stdio")))]
pub mod transport;

pub use server::{McpServer, ServerConfig, PROTOCOL_VERSION};
pub use tools::ToolSpec;

/// Serve the MCP protocol over this process's stdin/stdout (the standard MCP stdio
/// transport). Available with the `stdio` feature. See `transport::serve_stdio`.
#[cfg(feature = "stdio")]
#[cfg_attr(docsrs, doc(cfg(feature = "stdio")))]
pub use transport::{serve, serve_stdio};
