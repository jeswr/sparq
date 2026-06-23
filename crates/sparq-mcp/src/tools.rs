//! MCP tool definitions: the `name` / `description` / `inputSchema` triples an MCP
//! client receives from `tools/list`, plus the dispatch from a `tools/call` to the
//! sparq engine.
//!
//! [OPUS-4.8] (sq-0z43i, gh #909) Every tool is a thin wrapper over an existing
//! sparq read API — there is no new query/engine logic here:
//! - `query` → `sparq_engine::query_json` (SELECT/ASK; SPARQL 1.1 JSON results).
//! - `construct` → `sparq_engine::construct_ntriples` (CONSTRUCT/DESCRIBE; N-Triples).
//! - `introspect` → `sparq_introspect::Introspection::build(...).to_json()` /
//!   `to_text_summary(...)` (classes, predicates, prefixes, characteristic sets).
//! - `stats` → graph triple count + introspection totals.
//! - `update` (gated, OFF by default) → `sparq_engine::update_in_place_atomic`.

use serde_json::{json, Value};

use crate::server::McpServer;

/// A single MCP tool advertised in `tools/list`.
///
/// The `input_schema` is a JSON-Schema object describing the tool's arguments, exactly
/// as the MCP `Tool.inputSchema` field requires; clients use it to validate / form the
/// `arguments` they pass to `tools/call`.
pub struct ToolSpec {
    /// The tool name (the `name` a `tools/call` references).
    pub name: &'static str,
    /// One-line human/agent-facing description of what the tool does.
    pub description: &'static str,
    /// JSON-Schema (`type: object`) for the tool's `arguments`.
    pub input_schema: fn() -> Value,
}

impl ToolSpec {
    /// Render this spec as the JSON object MCP's `tools/list` returns per tool.
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "inputSchema": (self.input_schema)(),
        })
    }
}

/// The `query` tool: run a read-only SELECT or ASK and return SPARQL 1.1 JSON results.
pub const QUERY: ToolSpec = ToolSpec {
    name: "query",
    description: "Run a read-only SPARQL 1.1 SELECT or ASK query against the loaded RDF \
                  graph and return SPARQL 1.1 Query Results JSON \
                  (head.vars + results.bindings, or a boolean for ASK). Use `construct` \
                  for CONSTRUCT/DESCRIBE. This tool never mutates the graph.",
    input_schema: || {
        json!({
            "type": "object",
            "properties": {
                "sparql": {
                    "type": "string",
                    "description": "A SPARQL 1.1 SELECT or ASK query string."
                }
            },
            "required": ["sparql"],
            "additionalProperties": false
        })
    },
};

/// The `construct` tool: run a CONSTRUCT/DESCRIBE and return N-Triples text.
pub const CONSTRUCT: ToolSpec = ToolSpec {
    name: "construct",
    description: "Run a read-only SPARQL CONSTRUCT or DESCRIBE query against the loaded \
                  RDF graph and return the resulting triples as N-Triples text. This \
                  tool never mutates the graph.",
    input_schema: || {
        json!({
            "type": "object",
            "properties": {
                "sparql": {
                    "type": "string",
                    "description": "A SPARQL 1.1 CONSTRUCT or DESCRIBE query string."
                }
            },
            "required": ["sparql"],
            "additionalProperties": false
        })
    },
};

/// The `introspect` tool: mine and return the dataset's effective schema.
pub const INTROSPECT: ToolSpec = ToolSpec {
    name: "introspect",
    description: "Return the effective schema the loaded graph actually uses — classes \
                  (by instance count), predicates (by usage), namespace prefixes, and \
                  characteristic sets — mined directly from the store indexes (exact \
                  counts, no sampling). Set `format` to \"text\" for a compact \
                  token-budgeted summary suitable for LLM grounding, or \"json\" (the \
                  default) for the full machine-readable structure.",
    input_schema: || {
        json!({
            "type": "object",
            "properties": {
                "format": {
                    "type": "string",
                    "enum": ["json", "text"],
                    "description": "Output shape: \"json\" (full structure, default) or \
                                    \"text\" (compact prompt-ready summary)."
                },
                "budget_chars": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "For format=\"text\": approximate character budget for \
                                    the summary (default 4000)."
                }
            },
            "additionalProperties": false
        })
    },
};

/// The `stats` tool: dataset totals (triples, distinct subjects, typed entities, …).
pub const STATS: ToolSpec = ToolSpec {
    name: "stats",
    description: "Return summary statistics for the loaded graph: total triples, \
                  distinct subjects, typed entities, and the number of distinct \
                  classes / predicates / namespaces. Cheap, structured counts for \
                  deciding how to query the dataset.",
    input_schema: || {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        })
    },
};

/// The `update` tool: apply a SPARQL 1.1 Update. ONLY advertised / callable when the
/// server is started with update enabled (see [`crate::ServerConfig::allow_update`]).
pub const UPDATE: ToolSpec = ToolSpec {
    name: "update",
    description: "Apply a SPARQL 1.1 Update (INSERT/DELETE/LOAD/CLEAR/…) to the loaded \
                  graph, atomically (all-or-nothing per request). This MUTATES the \
                  dataset and is only available when the server was started with \
                  update explicitly enabled. Returns the new triple count on success.",
    input_schema: || {
        json!({
            "type": "object",
            "properties": {
                "sparql": {
                    "type": "string",
                    "description": "A SPARQL 1.1 Update request string."
                }
            },
            "required": ["sparql"],
            "additionalProperties": false
        })
    },
};

/// The read-only tool set, always advertised.
pub const READ_ONLY: &[&ToolSpec] = &[&QUERY, &CONSTRUCT, &INTROSPECT, &STATS];

/// The full list of tools this server advertises, given its config — `UPDATE` is
/// appended only when [`McpServer`] was built with update enabled.
pub fn advertised(server: &McpServer) -> Vec<&'static ToolSpec> {
    let mut tools: Vec<&'static ToolSpec> = READ_ONLY.to_vec();
    if server.allow_update() {
        tools.push(&UPDATE);
    }
    tools
}
