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
//! - `shapes` → `crate::shapes::class_shape(...)` (the data-grounded predicate/datatype/
//!   cardinality constraints for one class IRI; structured grounding for a client LLM —
//!   no server-side model). [OPUS-4.8] sq-zak4f
//! - `stats` → graph triple count + introspection totals.
//! - `ask` (feature `nlq`, OFF by default) → `crate::nlq` (server-side NL→SPARQL→execute
//!   via `sparq-nlq`; embeds a configurable LLM call, degrades cleanly when none is
//!   configured). [OPUS-4.8] sq-jxjgr
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

/// The `shapes` tool: the data-grounded predicate/datatype/cardinality constraints for
/// one class IRI, as structured JSON for a CLIENT LLM to ground NL→SPARQL on. No
/// server-side model — this is the lean, structured grounding aid (`sq-zak4f`).
/// [OPUS-4.8]
pub const SHAPES: ToolSpec = ToolSpec {
    name: "shapes",
    description: "Given a class/type IRI, return the data-grounded SHACL-style shape of \
                  that class: which predicates instances actually use, each with its \
                  coverage, observed datatypes, object-kind split (IRI vs literal), \
                  observed range, and the cardinalities the data supports (min_count=1 \
                  for a predicate present on every instance; max_count=1 for a \
                  single-valued one — emitted only when the data proves the bound, never \
                  fabricated). Use this to ground a SPARQL query for instances of the \
                  class: it tells you the valid predicates and whether to bind a literal \
                  or an IRI. Constraints describe the EFFECTIVE schema (what the graph \
                  asserts), not an aspirational contract. No LLM is involved.",
    input_schema: || {
        json!({
            "type": "object",
            "properties": {
                "class": {
                    "type": "string",
                    "description": "The full class IRI (e.g. \
                                    \"http://xmlns.com/foaf/0.1/Person\"). Call \
                                    `introspect` first to discover the classes the \
                                    dataset uses."
                }
            },
            "required": ["class"],
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

/// The `ask` tool: server-side natural-language → SPARQL → execute, behind the opt-in
/// `nlq` feature (`sq-jxjgr`). HONEST framing in the description: it embeds a configurable
/// LLM call, so cost/quality depend on the user's model, and it returns the executed
/// SPARQL + an honest rendering of the result rows (and, with `nlq/citations`, in-graph
/// citations) — NOT a free-form prose paragraph the model could fabricate. The structured
/// `shapes` / `introspect` tools are the no-LLM default; this trades a model call for the
/// convenience of not writing SPARQL yourself. [OPUS-4.8]
#[cfg(feature = "nlq")]
pub const ASK: ToolSpec = ToolSpec {
    name: "ask",
    description: "Answer a natural-language question by generating, validating, and \
                  running a SPARQL query SERVER-SIDE, then returning the executed SPARQL \
                  plus the result rows it produced (and in-graph citations when available). \
                  HONEST FRAMING: this embeds a configurable LLM call — cost, latency, and \
                  answer quality depend entirely on the model/endpoint YOU configure (via \
                  ANTHROPIC_API_KEY, or an OpenAI-compatible SPARQ_NLQ_ENDPOINT_URL + \
                  _MODEL); the server ships no default model and never phones home. If no \
                  model is configured the tool returns a clear 'not configured' error — \
                  it NEVER fabricates an answer. The returned answer is grounded in the \
                  executed query's real result rows, not a free-form paragraph. For a \
                  no-LLM path, use `shapes` + `introspect` and write the query yourself.",
    input_schema: || {
        json!({
            "type": "object",
            "properties": {
                "question": {
                    "type": "string",
                    "description": "A natural-language question about the loaded dataset."
                }
            },
            "required": ["question"],
            "additionalProperties": false
        })
    },
};

/// The read-only tool set, always advertised in the default build. `ask` (feature `nlq`)
/// is appended by [`advertised`] only when a model backend is configured.
pub const READ_ONLY: &[&ToolSpec] = &[&QUERY, &CONSTRUCT, &INTROSPECT, &SHAPES, &STATS];

/// The full list of tools this server advertises, given its config — `UPDATE` is
/// appended only when [`McpServer`] was built with update enabled, and `ask` (feature
/// `nlq`) only when an LLM backend is configured (so a feature-on server with no key
/// configured does not advertise an unusable tool — it degrades cleanly).
pub fn advertised(server: &McpServer) -> Vec<&'static ToolSpec> {
    let mut tools: Vec<&'static ToolSpec> = READ_ONLY.to_vec();
    if server.allow_update() {
        tools.push(&UPDATE);
    }
    #[cfg(feature = "nlq")]
    if crate::nlq::backend_configured() {
        tools.push(&ASK);
    }
    tools
}
