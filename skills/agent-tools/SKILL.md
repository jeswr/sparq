---
name: agent-tools
description: Use when an LLM/agent should access a sparq RDF dataset over the Model Context Protocol (MCP) as first-class tools — run SPARQL queries, mine the dataset schema, read stats, optionally SHACL-validate against caller-supplied shapes, and (gated, off by default) apply SPARQL updates. Covers the opt-in sparq-mcp crate — a JSON-RPC 2.0 MCP server (initialize, tools/list, tools/call) exposing query (SELECT/ASK to SPARQL-JSON), construct (CONSTRUCT/DESCRIBE to N-Triples), introspect (effective schema as JSON or token-budgeted text), stats, an opt-in read-only validate tool, and a gated update tool that is OFF by default; the transport-agnostic handle_message dispatch core plus the optional stdio feature; and the honest trust model (a local agent-tool server with no built-in auth, read-only by default, queries bounded by a QueryBudget). Also covers the opt-in solid feature — SolidMcpServer, a pod-backed server over sparq-solid's PodStore with WAC/ACP-authorized session-scoped query plus LDP resource tools (resource_get, container_list from stored ldp:contains data, and gated resource_put/resource_delete/container_create with existence non-disclosure and atomic ACL write-through). Complements genai-retrieval (sparq-nlq/sparq-introspect) — that is the NL-to-SPARQL loop; this is the MCP front door.
---

# sparq agent-tools (MCP)

The **MCP front door** of the sparq RDF + SPARQL engine: the opt-in **`sparq-mcp`**
crate turns a loaded `sparq_core::Graph` into a **Model Context Protocol** server so an
LLM/agent can use the dataset as first-class tools.

It is **opt-in** (a separate crate; nothing in the workspace depends on it, the default
engine build does not compile it) and a **thin wrapper** over existing surfaces — it adds
no engine capability. JSON-RPC 2.0 framing is hand-rolled over `serde_json`; there is no
heavy MCP-SDK dependency.

## Tools

| tool | wraps | returns |
| --- | --- | --- |
| `query` | `sparq_engine::query_json` (SELECT/ASK) | SPARQL 1.1 Query Results JSON |
| `construct` | `sparq_engine::construct_ntriples` (CONSTRUCT/DESCRIBE) | N-Triples text |
| `introspect` | `sparq_introspect::Introspection` | effective schema — JSON or token-budgeted text |
| `shapes` | `sparq_introspect::Introspection` (per-class) | data-grounded SHACL-style shape for one class IRI |
| `stats` | graph count + introspection totals | small JSON object |
| `ask` *(feature `nlq`, OFF by default)* | `sparq_nlq` NL→SPARQL loop | executed SPARQL + real result rows (+ citations) |
| `update` *(gated, OFF by default)* | `sparq_engine::update_in_place_atomic` | new triple count |
| `template_list` / `template_invoke` *(feature `templates`, OFF by default)* | `sparq_engine::templates` (#901 injection-safe binding) | definitions / typed fail-closed invocation |
| `text_search` *(feature `text`, OFF by default)* | `sparq_text::TextIndex` (BM25, lazily built + reconciled) | ranked literal hits JSON |
| `validate` *(feature `shacl`, OFF by default)* | `sparq_shacl::validate` | `{conforms, results}` JSON |

### SHACL validation (feature `shacl`, OFF by default) — [GPT-5.6] sq-lsp7k.22

Call `validate` with `{"shapes": "...Turtle..."}` and optionally `format` (defaults to
`turtle`). The tool parses the shapes as a separate graph, validates the server's data
graph, and returns `conforms` plus disallowed-severity results containing `focusNode`,
`path`, `severity`, and `message`. It never mutates the served graph. A malformed shapes
string returns an MCP tool result with `isError: true`, not a JSON-RPC protocol error.

### Named templates + full-text search (features `templates` / `text`, OFF by default) — [FABLE-5] sq-lsp7k.10

Register validated `sparq_engine::templates::Template`s on `ServerConfig::templates`; the
server then advertises `template_list` (definitions: name/kind/text/typed parameters) and
`template_invoke` (bind typed JSON arguments through the #901 **algebra rewrite** — never
string concatenation — and execute; **fail-closed** on an unknown template or an
unknown/missing/mistyped parameter). An **UPDATE template stays behind the same
`allow_update` gate** as the raw `update` tool: listed on a read-only server, refused at
invocation. `text_search` (feature `text`) BM25-ranks the graph's string literals via a
lazily-built `sparq-text` index, reconciled incrementally (`O(new dict terms)`) before every
search so it stays current across updates; modes `and`/`any`, `tok*` prefix matching for
autocomplete-style discovery. The HTTP server exposes the same template layer at
`/templates` (see the `http-server` skill §5g). Facet-count and richer IRI-autocomplete
tools are deliberately deferred to their own beads (the facet/autocomplete engine features).

### Pod mode (feature `solid`, OFF by default) — [FABLE-5] sq-u16eq

`SolidMcpServer` serves a **`sparq_solid::PodStore`** (one named graph per pod document
+ a materialized WAC/ACP authorization view), bound to ONE authenticated session fixed
at construction — the MCP-Solid proposal draft's local-trusted-agent deployment mode
(`site/specs/mcp-solid.typ` §6.4/§7.3/§9.3). Tools:

| tool | wraps | notes |
| --- | --- | --- |
| `query` | `wrap_for_view_opt_in` + `query_json_view_with_budget` | session-scoped; empty default graph, union opt-in via the reserved `FROM` IRI |
| `resource_get` | the document's named graph, serialized | same dataset `query` reads — the two surfaces cannot disagree |
| `container_list` | `ldp:contains` triples in the container's OWN graph | data-derived, never IRI-path guessing |
| `update` *(gated)* | `PodStore::update_as` / `update_as_acp` | per-graph session write check, fail-closed |
| `resource_put` *(gated)* | atomic named-graph swap (+ containment link on create) | `.acl`/`.acr` route through `put_acl`/`put_acl_acp` |
| `resource_delete` *(gated)* | slot removal + containment unlink | non-empty containers rejected; `.acl` via `delete_acl` |
| `container_create` *(gated)* | typed `ldp:BasicContainer` graph + containment | slash-terminated IRIs only |

Key contracts: a resource the session cannot read errors **byte-identically** to a
nonexistent one (existence non-disclosure, draft §9.3); ACL writes are gated on
`acl:Control` of the governed resource and re-derive authorization **atomically with
fail-closed rollback**; creation is authorized at the closest existing parent container
(the Solid creation rule); every content write re-materializes the view so the next
tool call sees it. RDF sources only (Turtle / N-Triples) in v1; non-RDF binaries and
notifications are out of scope (spec gaps stay beaded, not spun).

Every tool ships a proper MCP `inputSchema` (JSON-Schema). `tools/list` returns
`name` / `description` / `inputSchema` per tool. A `tools/call` result is the MCP
`CallToolResult` shape (`content` text item + `isError`); a bad SPARQL string is an MCP
**tool error** (`isError: true`), not a JSON-RPC protocol error, so the agent can read it
and retry.

### Two grounding tools (2026-06-23 design call — `shapes` + `ask`)

Two complementary ways to turn a natural-language question into a SPARQL answer, chosen
**both, opt-in** on the 2026-06-23 call:

- **`shapes`** (structured, **no LLM**, default build) — give it a **class IRI** and it
  returns the predicates instances of that class actually use, each with coverage,
  observed datatypes / object-kind (IRI vs literal), observed range, and the
  cardinalities the data proves (`min_count`/`max_count` emitted **only** when the data
  establishes the bound). The **client's own** LLM grounds NL→SPARQL on this. Reuses the
  introspection miner; describes the **effective** schema (what the graph asserts), not an
  aspirational contract.
- **`ask`** (NL, **opt-in `nlq` feature**) — runs the whole NL→SPARQL→validate→execute
  loop **server-side** via `sparq-nlq` and returns the **executed SPARQL** + the **real
  result rows** (+ in-graph citations). It embeds a **configurable** LLM call: cost and
  quality depend on the model **you** configure — `ANTHROPIC_API_KEY`, or an
  OpenAI-compatible `SPARQ_NLQ_ENDPOINT_URL` + `SPARQ_NLQ_ENDPOINT_MODEL` (+ optional
  `_KEY`). **No model is bundled**, nothing phones home. With the feature ON but **no**
  backend configured, `ask` is unadvertised and a direct call returns a clear *"not
  configured"* error — **never** a fabricated answer, never a panic. The answer is the
  query's real rows, not a free-form paragraph.

These are **ergonomics / grounding aids pending measurement** — *not* a token-saving
claim (the project measured representation/token tricks as duds). `shapes` is the lean
no-model default; `ask` trades a model call for not writing SPARQL yourself. The two
overlap deliberately (`shapes` puts the model on the client; `ask` on the server).

## Use it

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use sparq_core::Graph;
use sparq_mcp::{McpServer, ServerConfig};

let graph = Graph::load_str("<a> <b> <c> .", "ntriples")?;

// Embed: feed one JSON-RPC line, get one line back (no transport needed).
let mut server = McpServer::new(graph); // read-only by default
let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
let _reply = server.handle_message(init).unwrap();

// Allow writes EXPLICITLY:
let cfg = ServerConfig { allow_update: true, ..ServerConfig::default() };
# let _ = cfg;
# Ok(()) }
```

The standard MCP stdio transport is behind the **`stdio`** feature:
`sparq_mcp::serve_stdio(&mut server)` runs the line-delimited JSON-RPC loop over this
process's stdin/stdout. For an arbitrary reader/writer pair use `sparq_mcp::serve`.

## Trust model (read this — no overclaim)

This is a **local agent-tool server, not a hardened multi-tenant endpoint**. There is
**no built-in authentication or authorization**: the MCP transport is the trust boundary
you, the operator, establish, and whoever can speak to the server has exactly the access it
was configured with.

- **Read-only by default** — a default `McpServer` advertises and accepts only
  `query` / `construct` / `introspect` / `stats`; it cannot mutate the dataset.
- **`update` is a mutation surface**, exposed **only** when `ServerConfig::allow_update`
  is set (or a binary's `--allow-update` flag). It is the single write switch; there is no
  finer per-tool ACL.
- **Queries are bounded** by a `QueryBudget` (deadline + row cap; default 30 s / 1M rows)
  so one `tools/call` cannot run the server unbounded — a blunt anti-DoS ceiling, not a
  fairness quota.

## Status / scope

Opt-in crate at workspace v0.1.0; verified against branch `main` (2026-06-23 [OPUS-4.8];
pod mode 2026-07-11 [FABLE-5]).
Tested by a real in-memory MCP round-trip (default features) **and** a real stdio
serve-loop round-trip (feature `stdio`). Only the **stdio** transport plus the embeddable
`handle_message` ship today; SSE/HTTP transports are **not implemented** (follow-up beads).
The read-only default is proven fail-closed (a disabled `update` returns `-32601` and does
not mutate the graph).

## Learn more

- **Crate** — [`sparq-mcp`](../../crates/sparq-mcp) (README + rustdoc).
- **MCP spec** — <https://modelcontextprotocol.io>.
- **NL→SPARQL & schema grounding** — [`genai-retrieval`](../genai-retrieval/SKILL.md).
- **Underlying query API** — [`sparql-query`](../sparql-query/SKILL.md).
