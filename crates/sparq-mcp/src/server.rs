//! The MCP server: configuration, the in-memory graph it serves, and the
//! transport-agnostic JSON-RPC dispatch core.
//!
//! [OPUS-4.8] (sq-0z43i, gh #909) [`McpServer::handle_message`] is the single
//! entry point: it takes one raw JSON-RPC message string and returns the raw
//! response string (or `None` for a notification). The stdio transport in the
//! `transport` module (feature `stdio`) is a thin loop around it; the round-trip
//! test calls it directly, so the test exercises the real dispatch path with no
//! process spawn.

use std::time::Duration;

use sparq_core::Graph;
use sparq_engine::QueryBudget;
use sparq_introspect::Introspection;
use serde_json::{json, Value};

use crate::jsonrpc::{
    Request, Response, RpcError, INTERNAL_ERROR, INVALID_PARAMS, INVALID_REQUEST,
    METHOD_NOT_FOUND,
};
use crate::tools;

/// The MCP protocol version this server implements (the value echoed in the
/// `initialize` result). Pinned to a published MCP revision; a client that speaks a
/// different revision still interoperates for the basic tools flow.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// Server configuration. The security-relevant field is [`Self::allow_update`]:
/// it is **`false` by default**, so a freshly-built server is strictly read-only
/// and never advertises or accepts the mutating `update` tool.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Whether the mutating `update` tool is exposed. **OFF by default.** When
    /// `false` the tool is neither advertised in `tools/list` nor callable via
    /// `tools/call` (a call returns a `METHOD_NOT_FOUND`-class tool error), so a
    /// default server cannot mutate the dataset at all. Turn it on only when the
    /// MCP client is trusted to issue writes — see the crate-level trust model.
    pub allow_update: bool,
    /// Wall-clock deadline applied to every tool-issued query/update, in seconds.
    /// Bounds an expensive or adversarial query so one `tools/call` cannot run the
    /// server unbounded. `None` disables the deadline (unbounded — not recommended
    /// for an agent-facing server).
    pub query_timeout_secs: Option<u64>,
    /// Upper bound on the rows of any materialised result, applied to every
    /// tool-issued query. `None` disables the row cap.
    pub max_rows: Option<usize>,
    /// The server name reported in the `initialize` handshake (`serverInfo.name`).
    pub server_name: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            // Security default: read-only.
            allow_update: false,
            // A generous-but-bounded default deadline so an agent's runaway query
            // cannot hang the server; opt out explicitly with `None`.
            query_timeout_secs: Some(30),
            max_rows: Some(1_000_000),
            server_name: "sparq-mcp".to_string(),
        }
    }
}

/// An MCP server over an in-memory sparq [`Graph`].
///
/// Construct it with a loaded graph and a [`ServerConfig`], then drive it either via
/// [`McpServer::handle_message`] (embed your own transport) or, with the `stdio`
/// feature, the `serve_stdio` loop (the standard MCP stdio transport).
pub struct McpServer {
    graph: Graph,
    config: ServerConfig,
}

impl McpServer {
    /// Build a read-only server over `graph` with the default [`ServerConfig`]
    /// (update disabled).
    pub fn new(graph: Graph) -> Self {
        McpServer { graph, config: ServerConfig::default() }
    }

    /// Build a server over `graph` with an explicit [`ServerConfig`].
    pub fn with_config(graph: Graph, config: ServerConfig) -> Self {
        McpServer { graph, config }
    }

    /// Whether the mutating `update` tool is exposed (mirrors
    /// [`ServerConfig::allow_update`]).
    pub fn allow_update(&self) -> bool {
        self.config.allow_update
    }

    /// The configured query budget for one tool call.
    fn budget(&self) -> QueryBudget {
        let mut b = QueryBudget::unlimited();
        if let Some(secs) = self.config.query_timeout_secs {
            b.deadline = Some(std::time::Instant::now() + Duration::from_secs(secs));
        }
        b.max_rows = self.config.max_rows;
        b
    }

    /// Read-only borrow of the served graph (for embedders / tests).
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Handle one raw JSON-RPC message. Returns `Ok(Some(response_json))` for a
    /// request, `Ok(None)` for a notification (no response is emitted), and a
    /// serialized error response for a malformed / failed request.
    ///
    /// The string is one complete JSON object; the transport is responsible for
    /// delimiting (the stdio transport uses one object per line).
    pub fn handle_message(&mut self, raw: &str) -> Option<String> {
        let req: Request = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => {
                // A parse error cannot be correlated to an id ⇒ id is JSON null.
                let resp = Response::err(
                    Value::Null,
                    RpcError::new(INVALID_REQUEST, format!("invalid JSON-RPC request: {}", e)),
                );
                return Some(serialize(&resp));
            }
        };
        let is_notification = req.is_notification();
        let id = req.id.clone().unwrap_or(Value::Null);
        let outcome = self.dispatch(&req);
        if is_notification {
            // Notifications never get a response, even on error.
            return None;
        }
        let resp = match outcome {
            Ok(result) => Response::ok(id, result),
            Err(e) => Response::err(id, e),
        };
        Some(serialize(&resp))
    }

    /// Route a request to its method handler. Returns the JSON `result` or a JSON-RPC
    /// error object.
    fn dispatch(&mut self, req: &Request) -> Result<Value, RpcError> {
        match req.method.as_str() {
            "initialize" => Ok(self.initialize_result()),
            // Lifecycle notification from the client; nothing to do.
            "notifications/initialized" | "initialized" => Ok(Value::Null),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(self.tools_list_result()),
            "tools/call" => self.tools_call(&req.params),
            other => Err(RpcError::new(
                METHOD_NOT_FOUND,
                format!("method not found: {}", other),
            )),
        }
    }

    /// The `initialize` handshake result: protocol version, server info, and the
    /// declared capabilities (just `tools`).
    fn initialize_result(&self) -> Value {
        json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": self.config.server_name,
                "version": env!("CARGO_PKG_VERSION"),
            }
        })
    }

    /// The `tools/list` result: every advertised tool's `name`/`description`/
    /// `inputSchema`. `update` appears here only when update is enabled.
    fn tools_list_result(&self) -> Value {
        let tools: Vec<Value> = tools::advertised(self).iter().map(|t| t.to_json()).collect();
        json!({ "tools": tools })
    }

    /// Handle a `tools/call`: validate the requested tool name + arguments, run it,
    /// and wrap the output in the MCP `CallToolResult` shape (a `content` array with a
    /// single text item, plus `isError` on a tool-level failure).
    fn tools_call(&mut self, params: &Value) -> Result<Value, RpcError> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| RpcError::new(INVALID_PARAMS, "tools/call requires a string `name`"))?;
        let args = params.get("arguments").cloned().unwrap_or(json!({}));

        // Reject the mutating tool up front when it is not enabled, *before* parsing
        // args — so a disabled `update` is never even attempted.
        if name == tools::UPDATE.name && !self.allow_update() {
            return Err(RpcError::new(
                METHOD_NOT_FOUND,
                "tool `update` is not available: this server is read-only \
                 (start it with update enabled to allow writes)",
            ));
        }

        let result = match name {
            "query" => self.tool_query(&args),
            "construct" => self.tool_construct(&args),
            "introspect" => self.tool_introspect(&args),
            "shapes" => self.tool_shapes(&args),
            "stats" => self.tool_stats(),
            #[cfg(feature = "nlq")]
            "ask" => self.tool_ask(&args),
            "update" => self.tool_update(&args),
            other => {
                return Err(RpcError::new(
                    METHOD_NOT_FOUND,
                    format!("unknown tool: {}", other),
                ))
            }
        };

        // A tool-execution failure (a bad SPARQL string, a budget trip) is reported
        // as an MCP tool error (`isError: true` + a text message), NOT a JSON-RPC
        // protocol error — the client/agent reads it and can retry. A protocol error
        // (bad method, bad params shape) is the JSON-RPC `Err` path above.
        match result {
            Ok(text) => Ok(tool_text_result(text, false)),
            Err(msg) => Ok(tool_text_result(msg, true)),
        }
    }

    /// `query`: SELECT/ASK → SPARQL 1.1 JSON results.
    fn tool_query(&self, args: &Value) -> Result<String, String> {
        let sparql = arg_str(args, "sparql")?;
        sparq_engine::query_json_with_budget(&self.graph, sparql, &self.budget())
    }

    /// `construct`: CONSTRUCT/DESCRIBE → N-Triples text.
    fn tool_construct(&self, args: &Value) -> Result<String, String> {
        let sparql = arg_str(args, "sparql")?;
        sparq_engine::construct_ntriples_with_budget(&self.graph, sparql, &self.budget())
    }

    /// `introspect`: the effective schema as JSON (default) or a text summary.
    fn tool_introspect(&self, args: &Value) -> Result<String, String> {
        let format = args.get("format").and_then(Value::as_str).unwrap_or("json");
        let ix = Introspection::build(&self.graph);
        match format {
            "json" => Ok(ix.to_json()),
            "text" => {
                let budget = args
                    .get("budget_chars")
                    .and_then(Value::as_u64)
                    .map(|n| n as usize)
                    .unwrap_or(4000);
                Ok(ix.to_text_summary(budget))
            }
            other => Err(format!("unknown format `{}` (expected \"json\" or \"text\")", other)),
        }
    }

    /// `shapes`: the data-grounded shape (valid predicates + datatypes + cardinalities)
    /// of one class IRI, as structured JSON for a client LLM to ground NL→SPARQL on. No
    /// server-side model — reuses the introspection schema miner. [OPUS-4.8] sq-zak4f
    fn tool_shapes(&self, args: &Value) -> Result<String, String> {
        let class_iri = arg_str(args, "class")?;
        let ix = Introspection::build(&self.graph);
        let shape = crate::shapes::class_shape(&ix, class_iri)?;
        Ok(serde_json::to_string_pretty(&shape).unwrap_or_else(|_| shape.to_string()))
    }

    /// `ask` (feature `nlq`): server-side NL→SPARQL→execute via sparq-nlq. Degrades
    /// cleanly — returns a clear "not configured" tool error when no LLM backend is set,
    /// never a fabricated answer or a panic. [OPUS-4.8] sq-jxjgr
    #[cfg(feature = "nlq")]
    fn tool_ask(&self, args: &Value) -> Result<String, String> {
        let question = arg_str(args, "question")?;
        crate::nlq::ask(&self.graph, question, &self.budget())
    }

    /// `stats`: dataset totals as a small JSON object.
    fn tool_stats(&self) -> Result<String, String> {
        let ix = Introspection::build(&self.graph);
        let stats = json!({
            "triples": self.graph.len(),
            "distinct_subjects": ix.subjects,
            "typed_entities": ix.entities,
            "classes": ix.classes.len(),
            "predicates": ix.predicates.len(),
            "namespaces": ix.vocabularies.distinct,
        });
        Ok(serde_json::to_string_pretty(&stats).unwrap_or_else(|_| stats.to_string()))
    }

    /// `update`: apply a SPARQL 1.1 Update atomically (only reachable when enabled).
    fn tool_update(&mut self, args: &Value) -> Result<String, String> {
        let sparql = arg_str(args, "sparql")?;
        let budget = self.budget();
        sparq_engine::update_in_place_atomic_with_budget(&mut self.graph, sparql, &budget)?;
        Ok(format!("ok; graph now has {} triples", self.graph.len()))
    }
}

/// Extract a required string argument or produce a tool-level error message.
pub(crate) fn arg_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing required string argument `{}`", key))
}

/// Wrap a tool's textual output in the MCP `CallToolResult` shape.
pub(crate) fn tool_text_result(text: String, is_error: bool) -> Value {
    json!({
        "content": [ { "type": "text", "text": text } ],
        "isError": is_error,
    })
}

/// Serialize a response, falling back to a hand-built internal-error object if
/// serialization itself somehow fails (it cannot for our concrete types, but we never
/// panic on the I/O path).
pub(crate) fn serialize(resp: &Response) -> String {
    serde_json::to_string(resp).unwrap_or_else(|_| {
        format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{{\"code\":{},\"message\":\"response serialization failed\"}}}}",
            INTERNAL_ERROR
        )
    })
}
