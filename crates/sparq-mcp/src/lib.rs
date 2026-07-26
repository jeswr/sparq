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
//! - `solid` (feature `solid`) — the pod-backed server (`SolidMcpServer`): LDP
//!   container/resource CRUD tools (`resource_get`/`container_list`, plus the gated
//!   `resource_put`/`resource_delete`/`container_create`) with WAC/ACP-authorized,
//!   session-scoped semantics over a `sparq_solid::PodStore`. [FABLE-5] sq-u16eq
//!   It also serves the MCP **`resources`** surface (`resources/list`, `resources/read`,
//!   and `resources/subscribe` / `resources/unsubscribe` with `subscribe: true`), bound
//!   to Solid Notifications semantics: content-free `notifications/resources/updated`
//!   messages, authorized at subscribe time AND at every delivery. [SONNET-4.6] sq-cmjmr
//! - `templates` (feature `templates`) — the `template_list` / `template_invoke` tools:
//!   named parameterized SPARQL templates registered on `ServerConfig::templates` and
//!   invoked with typed JSON arguments through the injection-safe algebra binding
//!   (fail-closed on unknown/missing/mistyped parameters); an UPDATE template stays
//!   behind the same `allow_update` gate as the raw `update` tool. [FABLE-5] sq-lsp7k.10
//! - `text` (feature `text`) — the `text_search` tool: BM25 full-text search over the
//!   graph's string literals via a lazily-built, incrementally-reconciled `sparq-text`
//!   index. [FABLE-5] sq-lsp7k.10
//! - `shacl` (feature `shacl`) — the read-only `validate` tool: validate the served
//!   graph against caller-supplied shapes via `sparq-shacl`. [GPT-5.6] sq-lsp7k.22
//!   The same feature lights the read-only `describe_form` tool: derive a shape-aware
//!   form for one focus node against caller-supplied shapes via `sparq-forms`,
//!   returning the `FormDescription` JSON verbatim. [FABLE-5] sq-lsp7k.1.6

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

// [FABLE-5] sq-u16eq: the pod-backed server with LDP resource tools, behind the opt-in
// `solid` feature.
#[cfg(feature = "solid")]
#[cfg_attr(docsrs, doc(cfg(feature = "solid")))]
pub mod solid;

// [SONNET-4.6] sq-cmjmr: the subscription registry + notification machinery the pod
// server's `resources/subscribe` surface is built on. Crate-internal — the public seam
// is `SolidMcpServer::take_notifications`.
#[cfg(feature = "solid")]
pub(crate) mod notifications;

pub use server::{McpServer, ServerConfig, PROTOCOL_VERSION, SUPPORTED_PROTOCOL_VERSIONS};
pub use tools::ToolSpec;

/// The pod-backed MCP server + its configuration (feature `solid`). See `solid`.
#[cfg(feature = "solid")]
#[cfg_attr(docsrs, doc(cfg(feature = "solid")))]
pub use solid::{SolidMcpServer, SolidServerConfig};

/// Serve the MCP protocol over this process's stdin/stdout (the standard MCP stdio
/// transport). Available with the `stdio` feature. See `transport::serve_stdio`.
#[cfg(feature = "stdio")]
#[cfg_attr(docsrs, doc(cfg(feature = "stdio")))]
pub use transport::{serve, serve_stdio};
