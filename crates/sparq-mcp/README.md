<!-- [OPUS-4.8] sq-0z43i (gh #909): new opt-in crate. -->
# sparq-mcp

<p>
  <a href="https://crates.io/crates/sparq-mcp"><img src="https://img.shields.io/crates/v/sparq-mcp.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-mcp"><img src="https://docs.rs/sparq-mcp/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

A **Model Context Protocol (MCP)** server exposing a sparq RDF graph as **agent tools** —
an **opt-in** crate. An LLM/agent can use `query`, `introspect`, `stats`, `prefixes`, and `void` over a
dataset as first-class MCP tools; SPARQL **`update` is OFF by default**. It is a thin
wrapper over the existing engine read API (`sparq-engine` query path + `sparq-introspect`
schema mining): nothing in the workspace depends on it, the default engine build does not
compile it, and it adds **zero engine capability** and **no heavy dependency** (JSON-RPC
2.0 framing is hand-rolled over `serde_json`).

## 🚀 Quickstart

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use sparq_core::Graph;
use sparq_mcp::{McpServer, ServerConfig};

let graph = Graph::load_str("<a> <b> <c> .", "ntriples")?;

// Read-only by default (update tool not advertised or callable).
let mut server = McpServer::new(graph);

// Drive it with your own transport: feed one JSON-RPC line, get one line back.
let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
let reply = server.handle_message(init).unwrap();
assert!(reply.contains("serverInfo"));

// To allow writes, opt in EXPLICITLY:
let _writable = ServerConfig { allow_update: true, ..ServerConfig::default() };
# Ok(()) }
```

With the `stdio` feature, `serve_stdio(&mut server)` runs the standard MCP stdio
transport (line-delimited JSON-RPC 2.0 over this process's stdin/stdout).

## ✨ Tools

- **`query`** — run a read-only SELECT/ASK; returns SPARQL 1.1 Query Results JSON. Bounded by a configurable `QueryBudget` (default 30 s / 1M rows).
- **`construct`** — run a read-only CONSTRUCT/DESCRIBE; returns N-Triples text.
- **`introspect`** — the effective schema the graph actually uses (classes, predicates,
  prefixes, characteristic sets) as full JSON or a token-budgeted text summary for LLM
  grounding — exact counts from the store indexes, no sampling.
- **`shapes`** — given a class IRI, the data-grounded SHACL-style shape of that class:
  the valid predicates, their datatypes/object-kinds, observed range, and the
  cardinalities the data supports (`min_count`/`max_count` only when proven, never
  fabricated). Structured grounding so a **client** LLM can write NL→SPARQL — **no
  server-side model**. Ships in the default build.
- **`stats`** — dataset totals (triples, distinct subjects, typed entities, class / predicate / namespace counts).
- **`prefixes`** — list detected namespace declarations and distinct IRI term counts, largest namespace first. <!-- [GPT-5.6] sq-kx5b0 -->
- **`void`** — emit a W3C VoID dataset descriptor as N-Triples, optionally including characteristic-set statistics; `dataset` defaults to `urn:sparq:dataset`.
- **`ask`** *(feature `nlq`, OFF by default)* — answer a natural-language question
  **server-side**: NL→SPARQL→validate→execute via `sparq-nlq`, returning the executed
  SPARQL + the real result rows (+ in-graph citations). It embeds a **configurable** LLM
  call — cost/quality depend on the model **you** configure (`ANTHROPIC_API_KEY`, or an
  OpenAI-compatible `SPARQ_NLQ_ENDPOINT_URL`+`_MODEL`); no model is bundled. With no
  backend configured it is unadvertised and a call returns a clear "not configured"
  error — never a fabricated answer. This is an ergonomics/grounding aid (no
  token-saving claim); the structured tools are the no-LLM default.
- **`update`** *(gated, OFF by default)* — apply an atomic SPARQL 1.1 Update. Neither
  advertised in `tools/list` nor callable unless `ServerConfig::allow_update` is set.
- **`template_list` / `template_invoke`** *(feature `templates`, OFF by default)* — named
  parameterized templates (registered on `ServerConfig::templates`) invoked with **typed,
  fail-closed** JSON arguments through the #901 injection-safe algebra binding; an UPDATE
  template stays behind the same `allow_update` gate (sq-lsp7k.10).
- **`text_search`** *(feature `text`, OFF by default)* — BM25 full-text search over the
  graph's string literals (`sparq-text`; lazily built, incrementally reconciled).
- **`validate`** *(feature `shacl`, OFF by default)* — read-only validation against caller-supplied shapes; returns `{conforms, results}`, with parse failures as tool errors. [GPT-5.6] sq-lsp7k.22

**Pod mode** *(feature `solid`, OFF by default)*: `SolidMcpServer` serves a
`sparq-solid` `PodStore` (named graph per document, WAC/ACP-authorized, bound to one
session) with LDP tools per the MCP-Solid proposal draft — session-scoped `query`,
`resource_get`, `container_list` (containment from stored `ldp:contains` data, never
IRI-path guessing), and `update` / `resource_put` / `resource_delete` /
`container_create` behind the same off-by-default write gate. A resource the session
cannot read errors **identically to one that does not exist** (existence
non-disclosure), and `.acl`/`.acr` writes route through the pod store's atomic
fail-closed ACL write-through. RDF sources only (Turtle / N-Triples) in v1.

Each tool ships a proper MCP `inputSchema` (JSON-Schema); `tools/list` returns
`name` / `description` / `inputSchema` per tool. `tools/call` wraps output in the MCP
`CallToolResult` shape; a bad query is an MCP tool error (`isError: true`), not a
protocol error, so the agent can read it and retry.

## 🔒 Trust model — read this

This is a **local agent-tool server, not a hardened multi-tenant endpoint**. It has **no
built-in authentication or authorization**: the MCP transport (stdio) is a trust boundary
you, the operator, establish — whoever can speak to the server has exactly the access the
server was configured with. Run it only against a client you trust.

- **Read-only by default.** Default tools cannot mutate; feature-gated `validate` is read-only.
- **`update` is a mutation surface** and is exposed **only** when you set
  `ServerConfig::allow_update = true` (or a binary's `--allow-update` flag). Turn it on
  only when the client is trusted to issue writes. There is no per-tool ACL beyond this
  one switch.
- **Queries are bounded** by a `QueryBudget` (deadline + row cap) so one tool call cannot
  run the server unbounded — a blunt anti-DoS ceiling, not a fairness quota.

No overclaim: this does not add isolation, sandboxing, or auth that the host process does
not already provide.

## 📚 Learn more

- **How-to** — [`skills/agent-tools/SKILL.md`](../../skills/agent-tools/SKILL.md).
- **API reference** — [docs.rs/sparq-mcp](https://docs.rs/sparq-mcp).
- **MCP spec** — <https://modelcontextprotocol.io>.
- **Underlying engine** — [`sparq-engine`](../sparq-engine) (query path) and
  [`sparq-introspect`](../sparq-introspect) (schema mining).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
