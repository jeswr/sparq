//! [FABLE-5] sq-u16eq — the pod-backed MCP server (feature `solid`, OFF by default):
//! LDP container/resource CRUD tools over a `sparq_solid::PodStore`, per the MCP-Solid
//! proposal draft (`site/specs/mcp-solid.typ` §6.4 / §7.3 / §9.3, sq-tag1q.1).
//!
//! # What this adds
//!
//! Where [`crate::McpServer`] serves ONE anonymous in-memory [`Graph`] with a
//! SPARQL-only surface, [`SolidMcpServer`] serves a **pod**: a named-graph-per-document
//! dataset with a materialized WAC/ACP authorization view, bound to ONE authenticated
//! session (the Solid-OIDC-derived agent/client/issuer triple fixed at construction —
//! the "local trusted agent" deployment mode of the draft §4.1). Tools:
//!
//! - `query` (Class R) — session-scoped SPARQL over the authorized graph set via the
//!   engine's zero-copy dataset view; the standing default graph is EMPTY and a query
//!   opts into the authorized union with the reserved `FROM` IRI (the
//!   `solid-sparql-query` semantics `sparq-solid` implements).
//! - `resource_get` (Class R) — one pod document, served from the SAME dataset the
//!   `query` tool evaluates over (draft §6.4: the two read surfaces cannot disagree).
//! - `container_list` (Class R) — the direct `ldp:contains` members of one container,
//!   derived ONLY from stored containment triples in the container's own graph — never
//!   from IRI-path guessing (draft §6.4).
//! - `update`, `resource_put`, `resource_delete`, `container_create` (Class U) — all
//!   gated behind the SAME off-by-default write enablement as the base server's
//!   `update` tool ([`SolidServerConfig::allow_update`], draft §7.1).
//!
//! # Existence non-disclosure (draft §9.3)
//!
//! For every resource-addressed tool, a session **lacking read access to the target
//! receives a tool error byte-identical to the resource-does-not-exist error**
//! ([`not_found_error`]), so resource existence is never disclosed to a session that
//! cannot read the resource. A session *with* read access but without the required
//! write mode gets a distinguishable denied error (permitted by the draft). Writes to
//! an access-control document are gated on `acl:Control` of the governed resource and
//! non-disclose the same way.
//!
//! # ACL write-through (draft §7.3)
//!
//! `resource_put` / `resource_delete` targeting an access-control document (`.acl` /
//! `.acr`) route through `sparq-solid`'s authoritative write-through
//! (`PodStore::put_acl` / `delete_acl`, or the `_acp` twins): the control graph swap
//! and the authorization-view re-materialization succeed or roll back as ONE
//! fail-closed atomic unit — a pod is never left with a policy the enforcement layer
//! could not evaluate. Content-document writes are parse-first + rollback-on-error the
//! same way, and re-materialize the view so a created document becomes visible (and a
//! deleted one invisible) to the very next tool call.
//!
//! # Honest v1 limits
//!
//! - RDF sources only: `content_type` must be Turtle or N-Triples, and `resource_get`
//!   serves `application/n-triples` (an `accept` for anything else is a tool error,
//!   never silent coercion). Non-RDF (binary) resources are out of scope.
//! - The SPARQL `update` tool delegates to the session-checked
//!   `PodStore::update_as` / `update_as_acp`, which applies the per-graph write
//!   permission check but not the wall-clock/row budget the read tools enforce.
//! - Time-windowed conditional grants fail closed unless [`SolidServerConfig::now`]
//!   supplies a request clock.

use std::time::Duration;

use oxrdf::{NamedNode, Term};
use serde_json::{json, Value};
use sparq_core::Graph;
use sparq_engine::QueryBudget;
use sparq_solid::{Mode, PodStore, Session};

use crate::jsonrpc::{
    Request, Response, RpcError, INVALID_PARAMS, INVALID_REQUEST, METHOD_NOT_FOUND,
};
use crate::server::{arg_str, serialize, tool_text_result};
use crate::tools::ToolSpec;

/// `ldp:contains` — the containment predicate a container's own graph stores.
const LDP_CONTAINS: &str = "http://www.w3.org/ns/ldp#contains";
/// `ldp:BasicContainer` — the type asserted on a container `container_create` makes.
const LDP_BASIC_CONTAINER: &str = "http://www.w3.org/ns/ldp#BasicContainer";
/// `ldp:Container` — asserted alongside [`LDP_BASIC_CONTAINER`].
const LDP_CONTAINER: &str = "http://www.w3.org/ns/ldp#Container";
/// `rdf:type`.
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// The reserved graph-name space a pod write may never target (the rewrite sentinel
/// and the materialized auth view live here; `PodStore::new` strips inbound graphs
/// under it, and this server refuses to re-introduce one).
const RESERVED_PREFIX: &str = "urn:sparq:";

/// Configuration for a [`SolidMcpServer`]: the ONE authenticated session the server is
/// bound to, the pod's access-control model, and the same security-default gates as
/// [`crate::ServerConfig`] (writes OFF, bounded query budget).
#[derive(Debug, Clone)]
pub struct SolidServerConfig {
    /// The authenticated agent's WebID (`None` = anonymous — a fresh pod with no
    /// public grant then fails closed to zero visibility).
    pub agent: Option<String>,
    /// The client identifier (WAC `acl:origin` / ACP `acp:client`); `None` = any.
    pub client: Option<String>,
    /// The OIDC issuer that vouched for the WebID (ACP only); `None` = any.
    pub issuer: Option<String>,
    /// The request clock as an `xsd:dateTime` lexical string, consulted only by
    /// time-windowed conditional grants. `None` ⇒ such grants fail closed.
    pub now: Option<String>,
    /// `false` = WAC pod (`.acl`, [`PodStore::materialize_wac`]); `true` = ACP pod
    /// (`.acr`, [`PodStore::materialize_acp`]). Selects which model every
    /// (re-)materialization this server performs runs — including the ACL
    /// write-through twins.
    pub acp: bool,
    /// Whether the mutating tools (`update`, `resource_put`, `resource_delete`,
    /// `container_create`) are exposed. **OFF by default** — a freshly-built pod
    /// server is strictly read-only, exactly like [`crate::ServerConfig`].
    pub allow_update: bool,
    /// Wall-clock deadline per tool-issued query, in seconds (`None` = unbounded).
    pub query_timeout_secs: Option<u64>,
    /// Row cap on any materialised query result (`None` = uncapped).
    pub max_rows: Option<usize>,
    /// The server name reported in the `initialize` handshake.
    pub server_name: String,
}

impl Default for SolidServerConfig {
    fn default() -> Self {
        SolidServerConfig {
            agent: None,
            client: None,
            issuer: None,
            now: None,
            acp: false,
            // Security default: read-only, exactly like the base server.
            allow_update: false,
            query_timeout_secs: Some(30),
            max_rows: Some(1_000_000),
            server_name: "sparq-mcp-solid".to_string(),
        }
    }
}

/// An MCP server over a `sparq-solid` [`PodStore`], bound to one authenticated
/// session. See the [module docs](self) for the tool surface and its draft-conformance
/// properties (existence non-disclosure, ACL write-through, shared dataset).
///
/// Construction (re-)materializes the authorization view under the configured model
/// ([`SolidServerConfig::acp`]), so the server never starts on a stale or
/// differently-modelled view.
pub struct SolidMcpServer {
    store: PodStore,
    config: SolidServerConfig,
}

/// The pod `query` tool (session-scoped — distinct semantics from the base server's).
pub const POD_QUERY: ToolSpec = ToolSpec {
    name: "query",
    description: "Run a read-only SPARQL 1.1 SELECT or ASK query against the pod, \
                  restricted to the named graphs (documents) this session may read, and \
                  return SPARQL 1.1 Query Results JSON. One named graph per pod \
                  document; the standing DEFAULT graph is EMPTY — use GRAPH patterns, \
                  or opt into the authorized union for one query with FROM \
                  <http://www.w3.org/ns/solid/sparql#union-default-graph>. An \
                  unauthorized document contributes nothing and raises no error.",
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

/// The `resource_get` tool: one pod document from the same dataset `query` evaluates.
pub const RESOURCE_GET: ToolSpec = ToolSpec {
    name: "resource_get",
    description: "Return the RDF representation of ONE pod resource (document), served \
                  from the same named-graph-per-document dataset the `query` tool \
                  evaluates over. Result: the resource content as N-Triples plus its \
                  media type. A resource this session cannot read is reported with the \
                  SAME error as a resource that does not exist.",
    input_schema: || {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The resource IRI (the pod document's named-graph name)."
                },
                "accept": {
                    "type": "string",
                    "description": "Optional preferred media type. This server serves \
                                    \"application/n-triples\"; any other value is a tool \
                                    error (no silent coercion)."
                }
            },
            "required": ["url"],
            "additionalProperties": false
        })
    },
};

/// The `container_list` tool: `ldp:contains` members from stored containment data.
pub const CONTAINER_LIST: ToolSpec = ToolSpec {
    name: "container_list",
    description: "List the DIRECT members of one pod container (the semantics of \
                  ldp:contains), derived ONLY from the containment triples stored in the \
                  container's own document — never guessed from IRI paths. Each member \
                  carries a container flag. A container this session cannot read is \
                  reported with the SAME error as one that does not exist.",
    input_schema: || {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The container IRI (conventionally slash-terminated)."
                }
            },
            "required": ["url"],
            "additionalProperties": false
        })
    },
};

/// The pod `update` tool (session-checked writes; gated like every mutating tool).
pub const POD_UPDATE: ToolSpec = ToolSpec {
    name: "update",
    description: "Apply a SPARQL 1.1 Update to the pod, enforcing this session's \
                  per-document WRITE authorization on every graph the update could \
                  touch (fail-closed: one unwritable target rejects the whole request \
                  before anything is applied; the default graph is never writable). \
                  Only available when the server was started with updates enabled.",
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

/// The `resource_put` tool: create or replace one document (LDP PUT).
pub const RESOURCE_PUT: ToolSpec = ToolSpec {
    name: "resource_put",
    description: "Create or replace ONE pod resource's RDF representation (LDP PUT \
                  semantics): replacement atomically swaps the document's named graph; \
                  creation also links the document into its parent containers' \
                  ldp:contains listings. Writing an access-control document (.acl/.acr) \
                  routes through the authoritative write-through: the policy swap and \
                  the authorization-view rebuild succeed or roll back as one unit. \
                  Requires write access; only available when updates are enabled.",
    input_schema: || {
        json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "The resource IRI to write." },
                "content": { "type": "string", "description": "The full new RDF body." },
                "content_type": {
                    "type": "string",
                    "description": "\"text/turtle\" or \"application/n-triples\"."
                }
            },
            "required": ["url", "content", "content_type"],
            "additionalProperties": false
        })
    },
};

/// The `resource_delete` tool: delete one document (LDP DELETE).
pub const RESOURCE_DELETE: ToolSpec = ToolSpec {
    name: "resource_delete",
    description: "Delete ONE pod resource (LDP DELETE semantics): removes the \
                  document's named graph and its parent's ldp:contains link. Deleting a \
                  non-empty container is rejected. Deleting an access-control document \
                  (.acl/.acr) routes through the authoritative write-through, so the \
                  removed rules stop granting immediately. Requires write access; only \
                  available when updates are enabled.",
    input_schema: || {
        json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "The resource IRI to delete." }
            },
            "required": ["url"],
            "additionalProperties": false
        })
    },
};

/// The `container_create` tool: create one empty container.
pub const CONTAINER_CREATE: ToolSpec = ToolSpec {
    name: "container_create",
    description: "Create ONE empty pod container (typed ldp:BasicContainer) and link it \
                  into its parent containers' ldp:contains listings. The IRI must be \
                  slash-terminated. Creating an existing container is an error. Requires \
                  write access; only available when updates are enabled.",
    input_schema: || {
        json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "The container IRI (must end in \"/\")." }
            },
            "required": ["url"],
            "additionalProperties": false
        })
    },
};

/// The tool names only reachable when [`SolidServerConfig::allow_update`] is on.
const MUTATING: [&str; 4] = [
    "update",
    "resource_put",
    "resource_delete",
    "container_create",
];

/// The existence-non-disclosure error (draft §9.3): used **byte-identically** for a
/// resource that does not exist and for one this session may not read, so no
/// resource-addressed tool discloses existence to a session without read access.
pub fn not_found_error(url: &str) -> String {
    format!("resource not found: <{url}>")
}

/// The distinguishable write-denied error for a session that CAN read the target but
/// lacks the required write mode (the draft permits distinguishing this case).
fn write_denied_error(url: &str) -> String {
    format!("write access denied: <{url}>")
}

/// Map a `content_type` / `accept` value to the engine's RDF format name. Media-type
/// parameters (`; charset=…`) are ignored; only the RDF text formats a pod document
/// can round-trip are accepted.
fn rdf_format(content_type: &str) -> Option<&'static str> {
    let base = content_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    match base.as_str() {
        "text/turtle" | "turtle" | "ttl" => Some("turtle"),
        "application/n-triples" | "ntriples" | "n-triples" | "nt" => Some("ntriples"),
        _ => None,
    }
}

/// Whether `iri` names an access-control document by the `.acl`/`.acr` convention
/// (mirrors `sparq-solid`'s write-through targeting rule).
fn is_control_doc(iri: &str) -> bool {
    iri.ends_with(".acl") || iri.ends_with(".acr")
}

/// The parent container of `iri` under LDP slash semantics, or `None` at (or above)
/// the origin root: `https://h/a/b` → `https://h/a/`, `https://h/a/` → `https://h/`,
/// `https://h/` → `None`.
fn parent_iri(iri: &str) -> Option<String> {
    let scheme_end = iri.find("://")? + 3;
    let root_slash = iri[scheme_end..].find('/')? + scheme_end;
    let cut = iri.strip_suffix('/').unwrap_or(iri);
    let pos = cut.rfind('/')?;
    if pos < root_slash {
        return None; // already the origin root (or a scheme slash)
    }
    Some(iri[..=pos].to_string())
}

/// Parse + validate a write-target IRI: syntactically an IRI and outside the reserved
/// `urn:sparq:` space.
fn valid_target(url: &str) -> Result<NamedNode, String> {
    if url.starts_with(RESERVED_PREFIX) {
        return Err(format!(
            "invalid target <{url}>: the `{RESERVED_PREFIX}` graph space is reserved"
        ));
    }
    NamedNode::new(url).map_err(|e| format!("invalid resource IRI <{url}>: {e}"))
}

/// Pretty-print a JSON tool result.
fn pretty(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
}

/// One recorded containment-link addition (for rollback): the parent graph's name and
/// the `(parent, ldp:contains, child)` triple inserted into it.
type AddedLink = (Term, [Term; 3]);

impl SolidMcpServer {
    /// Build a read-only pod server over `store` with the default
    /// [`SolidServerConfig`] (anonymous session, WAC, writes disabled), materializing
    /// the WAC authorization view.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the initial materialization fails (e.g. an ACL names a
    /// reserved-encoding principal); no server is built on an unevaluable policy.
    pub fn new(store: PodStore) -> Result<Self, String> {
        Self::with_config(store, SolidServerConfig::default())
    }

    /// Build a pod server over `store` with an explicit [`SolidServerConfig`],
    /// (re-)materializing the authorization view under the configured model so the
    /// served view is never stale and always matches [`SolidServerConfig::acp`].
    ///
    /// # Errors
    ///
    /// Returns `Err` if that materialization fails.
    pub fn with_config(store: PodStore, config: SolidServerConfig) -> Result<Self, String> {
        let mut server = SolidMcpServer { store, config };
        server.rematerialize()?;
        Ok(server)
    }

    /// Whether the mutating tools are exposed (mirrors
    /// [`SolidServerConfig::allow_update`]).
    pub fn allow_update(&self) -> bool {
        self.config.allow_update
    }

    /// Read-only borrow of the served pod store (for embedders / tests).
    pub fn store(&self) -> &PodStore {
        &self.store
    }

    /// Handle one raw JSON-RPC message — the pod twin of
    /// [`crate::McpServer::handle_message`], with the identical framing contract
    /// (`Some(response)` for a request, `None` for a notification).
    pub fn handle_message(&mut self, raw: &str) -> Option<String> {
        let req: Request = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response::err(
                    Value::Null,
                    RpcError::new(INVALID_REQUEST, format!("invalid JSON-RPC request: {e}")),
                );
                return Some(serialize(&resp));
            }
        };
        let is_notification = req.is_notification();
        let id = req.id.clone().unwrap_or(Value::Null);
        let outcome = self.dispatch(&req);
        if is_notification {
            return None;
        }
        let resp = match outcome {
            Ok(result) => Response::ok(id, result),
            Err(e) => Response::err(id, e),
        };
        Some(serialize(&resp))
    }

    /// The session this server is bound to (borrowing the config's owned strings).
    fn session(&self) -> Session<'_> {
        Session {
            agent: self.config.agent.as_deref(),
            client: self.config.client.as_deref(),
            issuer: self.config.issuer.as_deref(),
            now: self.config.now.as_deref(),
        }
    }

    /// The per-call query budget (same defaults as the base server).
    fn budget(&self) -> QueryBudget {
        let mut b = QueryBudget::unlimited();
        if let Some(secs) = self.config.query_timeout_secs {
            b.deadline = Some(std::time::Instant::now() + Duration::from_secs(secs));
        }
        b.max_rows = self.config.max_rows;
        b
    }

    /// Fail-closed point decision: may this session do `mode` on `url`?
    fn allowed(&self, url: &str, mode: Mode) -> bool {
        self.store.decide(&self.session(), url, mode).allow
    }

    /// (Re-)materialize the authorization view under the configured model.
    fn rematerialize(&mut self) -> Result<(), String> {
        if self.config.acp {
            self.store.materialize_acp().map(|_| ())
        } else {
            self.store.materialize_wac().map(|_| ())
        }
    }

    /// The advertised tool list: read tools always; mutating tools only when enabled.
    pub fn advertised(&self) -> Vec<&'static ToolSpec> {
        let mut tools: Vec<&'static ToolSpec> = vec![&POD_QUERY, &RESOURCE_GET, &CONTAINER_LIST];
        if self.allow_update() {
            tools.extend([
                &POD_UPDATE,
                &RESOURCE_PUT,
                &RESOURCE_DELETE,
                &CONTAINER_CREATE,
            ]);
        }
        tools
    }

    /// Route a request to its method handler (same protocol surface as the base
    /// server: `initialize`, `ping`, `tools/list`, `tools/call`).
    fn dispatch(&mut self, req: &Request) -> Result<Value, RpcError> {
        match req.method.as_str() {
            "initialize" => Ok(json!({
                "protocolVersion": crate::server::PROTOCOL_VERSION,
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": self.config.server_name,
                    "version": env!("CARGO_PKG_VERSION"),
                }
            })),
            "notifications/initialized" | "initialized" => Ok(Value::Null),
            "ping" => Ok(json!({})),
            "tools/list" => {
                let tools: Vec<Value> = self.advertised().iter().map(|t| t.to_json()).collect();
                Ok(json!({ "tools": tools }))
            }
            "tools/call" => self.tools_call(&req.params),
            other => Err(RpcError::new(
                METHOD_NOT_FOUND,
                format!("method not found: {other}"),
            )),
        }
    }

    /// Handle a `tools/call`: gate the mutating tools, run, wrap as `CallToolResult`.
    fn tools_call(&mut self, params: &Value) -> Result<Value, RpcError> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| RpcError::new(INVALID_PARAMS, "tools/call requires a string `name`"))?;
        let args = params.get("arguments").cloned().unwrap_or(json!({}));

        // Reject every mutating tool up front when writes are disabled — before any
        // argument is even parsed (the same fail-closed shape as the base server).
        if MUTATING.contains(&name) && !self.allow_update() {
            return Err(RpcError::new(
                METHOD_NOT_FOUND,
                format!(
                    "tool `{name}` is not available: this server is read-only \
                     (start it with updates enabled to allow writes)"
                ),
            ));
        }

        let result = match name {
            "query" => self.tool_query(&args),
            "resource_get" => self.tool_resource_get(&args),
            "container_list" => self.tool_container_list(&args),
            "update" => self.tool_update(&args),
            "resource_put" => self.tool_resource_put(&args),
            "resource_delete" => self.tool_resource_delete(&args),
            "container_create" => self.tool_container_create(&args),
            other => {
                return Err(RpcError::new(
                    METHOD_NOT_FOUND,
                    format!("unknown tool: {other}"),
                ))
            }
        };

        match result {
            Ok(text) => Ok(tool_text_result(text, false)),
            Err(msg) => Ok(tool_text_result(msg, true)),
        }
    }

    /// `query`: session-scoped SELECT/ASK through the zero-copy dataset view, with the
    /// spec-conformant empty-default-graph / union-opt-in rewrite and the configured
    /// budget. Authorization never errors: unauthorized graphs are invisible.
    fn tool_query(&self, args: &Value) -> Result<String, String> {
        let sparql = arg_str(args, "sparql")?;
        let wrapped = sparq_solid::wrap_for_view_opt_in(sparql)?;
        let view = self.store.view_for(&self.session(), Mode::Read);
        sparq_engine::query_json_view_with_budget(&view, &wrapped, &self.budget())
    }

    /// Borrow the named graph stored under `url`, if any.
    fn find_doc(&self, url: &str) -> Option<&Graph> {
        let name = NamedNode::new(url).ok()?;
        let term = Term::NamedNode(name);
        self.store
            .graph
            .named
            .iter()
            .find(|(n, _)| *n == term)
            .map(|(_, g)| g)
    }

    /// The index of the named-graph slot for `term`, if present.
    fn slot_index(&self, term: &Term) -> Option<usize> {
        self.store.graph.named.iter().position(|(n, _)| n == term)
    }

    /// `resource_get`: serve one document from the shared dataset (draft §6.4), with
    /// existence non-disclosure (draft §9.3).
    fn tool_resource_get(&self, args: &Value) -> Result<String, String> {
        let url = arg_str(args, "url")?;
        NamedNode::new(url).map_err(|e| format!("invalid resource IRI <{url}>: {e}"))?;
        if let Some(accept) = args.get("accept").and_then(Value::as_str) {
            if rdf_format(accept) != Some("ntriples") {
                return Err(format!(
                    "unsupported accept `{accept}`: this server serves application/n-triples"
                ));
            }
        }
        // NON-DISCLOSURE: unauthorized-read and nonexistent produce the SAME error.
        if !self.allowed(url, Mode::Read) {
            return Err(not_found_error(url));
        }
        let Some(doc) = self.find_doc(url) else {
            return Err(not_found_error(url));
        };
        let nt = sparq_engine::construct_ntriples_with_budget(
            doc,
            "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }",
            &self.budget(),
        )?;
        Ok(pretty(&json!({
            "url": url,
            "content_type": "application/n-triples",
            "content": nt,
        })))
    }

    /// `container_list`: direct `ldp:contains` members from the container's OWN stored
    /// graph — data-derived, never IRI-path guessing (draft §6.4).
    fn tool_container_list(&self, args: &Value) -> Result<String, String> {
        let url = arg_str(args, "url")?;
        NamedNode::new(url).map_err(|e| format!("invalid container IRI <{url}>: {e}"))?;
        if !self.allowed(url, Mode::Read) {
            return Err(not_found_error(url));
        }
        let Some(doc) = self.find_doc(url) else {
            return Err(not_found_error(url));
        };
        let q = format!("SELECT ?m WHERE {{ <{url}> <{LDP_CONTAINS}> ?m }}");
        let res = sparq_engine::query_with_budget(doc, &q, &self.budget())?;
        let mut members: Vec<Value> = Vec::with_capacity(res.rows.len());
        for row in &res.rows {
            if let Some(Some(Term::NamedNode(m))) = row.first() {
                members.push(json!({
                    "url": m.as_str(),
                    "container": m.as_str().ends_with('/'),
                }));
            }
        }
        members.sort_by(|a, b| a["url"].as_str().cmp(&b["url"].as_str()));
        Ok(pretty(&json!({ "url": url, "members": members })))
    }

    /// `update`: session-checked SPARQL Update (`PodStore::update_as` — every touched
    /// graph needs this session's write permission, fail-closed, atomic).
    fn tool_update(&mut self, args: &Value) -> Result<String, String> {
        let sparql = arg_str(args, "sparql")?.to_string();
        let config = self.config.clone();
        let session = Session {
            agent: config.agent.as_deref(),
            client: config.client.as_deref(),
            issuer: config.issuer.as_deref(),
            now: config.now.as_deref(),
        };
        if self.config.acp {
            self.store.update_as_acp(&session, &sparql)?;
        } else {
            self.store.update_as(&session, &sparql)?;
        }
        Ok("ok".to_string())
    }

    /// The authorization anchor for `url`: `url` itself when the store knows it (a
    /// stored graph, or a structural container prefix of one — those carry
    /// materialized grants), else the NEAREST ancestor container the store knows —
    /// the Solid creation rule: a `PUT` to a nonexistent resource is authorized by
    /// the closest existing parent container's policy. When no ancestor is known the
    /// url itself is returned and the decision fails closed (`NoAcl`).
    fn auth_anchor(&self, url: &str) -> String {
        let known = |iri: &str| {
            self.store.graph.named.iter().any(|(n, _)| match n {
                Term::NamedNode(nn) => {
                    nn.as_str() == iri || (iri.ends_with('/') && nn.as_str().starts_with(iri))
                }
                _ => false,
            })
        };
        if known(url) {
            return url.to_string();
        }
        let mut cur = url.to_string();
        while let Some(p) = parent_iri(&cur) {
            if known(&p) {
                return p;
            }
            cur = p;
        }
        url.to_string()
    }

    /// The uniform read/write gate for a mutating resource tool, evaluated at the
    /// target's [authorization anchor](Self::auth_anchor): no read ⇒ the
    /// non-disclosure not-found error (named for the TARGET, never the anchor); read
    /// but not `write_mode` ⇒ the distinguishable denied error.
    fn gate_write(&self, url: &str, write_mode: Mode) -> Result<(), String> {
        let anchor = self.auth_anchor(url);
        if !self.allowed(&anchor, Mode::Read) {
            return Err(not_found_error(url));
        }
        if !self.allowed(&anchor, write_mode) {
            return Err(write_denied_error(url));
        }
        Ok(())
    }

    /// The gate for writing an access-control document: `acl:Control` of the GOVERNED
    /// resource (the WAC/ACP rule — Control gates the control document), anchored the
    /// same way for a not-yet-existing subtree, with the same non-disclosure shape
    /// (no Control ⇒ not-found, never "denied").
    fn gate_acl_write(&self, acl_url: &str) -> Result<(), String> {
        let governed = &acl_url[..acl_url.len() - 4]; // strip ".acl"/".acr" (both len 4)
        let anchor = self.auth_anchor(governed);
        if !self.allowed(&anchor, Mode::Control) {
            return Err(not_found_error(acl_url));
        }
        Ok(())
    }

    /// `resource_put`: create/replace one document (LDP PUT). ACL documents route
    /// through the authoritative write-through; content documents are parse-first,
    /// containment-linked on create, and re-materialized with rollback-on-error.
    fn tool_resource_put(&mut self, args: &Value) -> Result<String, String> {
        let url = arg_str(args, "url")?.to_string();
        let content = arg_str(args, "content")?.to_string();
        let content_type = arg_str(args, "content_type")?.to_string();
        let name = valid_target(&url)?;
        let fmt = rdf_format(&content_type).ok_or_else(|| {
            format!("unsupported content_type `{content_type}`: use text/turtle or application/n-triples")
        })?;

        if is_control_doc(&url) {
            // ACL write-through (draft §7.3): authorize on Control of the governed
            // resource, then let sparq-solid swap + re-materialize atomically
            // (fail-closed rollback on an unevaluable policy).
            self.gate_acl_write(&url)?;
            let out = if self.config.acp {
                self.store.put_acl_acp(&url, &content, fmt)?
            } else {
                self.store.put_acl(&url, &content, fmt)?
            };
            return Ok(pretty(&json!({
                "url": url,
                "created": !out.existed,
                "triples": out.triples,
                "access_control": true,
            })));
        }

        if url.ends_with('/') {
            return Err(format!(
                "<{url}> is a container IRI — use `container_create` to create containers"
            ));
        }
        self.gate_write(&url, Mode::Write)?;

        // PARSE FIRST — a malformed body is rejected before anything mutates.
        let new_graph = Graph::load_str(&content, fmt)
            .map_err(|e| format!("content for <{url}> did not parse as {fmt}: {e}"))?;
        let triples = new_graph.len();

        // Swap the document slot (capturing the prior content for rollback).
        let term = Term::NamedNode(name);
        let prior = self.take_slot(&term);
        let existed = prior.is_some();
        self.store.graph.named.push((term.clone(), new_graph));

        // A CREATED document becomes a member of its parent containers (draft §7.3).
        let links = if existed {
            Ok((Vec::new(), Vec::new()))
        } else {
            self.link_containment(&url)
        };
        let (created_graphs, added_links) = match links {
            Ok(x) => x,
            Err(e) => {
                self.restore_slot(&term, prior);
                return Err(e);
            }
        };

        // Re-materialize so the write is visible (a created doc enters the authorized
        // sets; a replaced group document takes effect) — rollback on error.
        if let Err(e) = self.rematerialize() {
            self.unlink_containment(&created_graphs, &added_links);
            self.restore_slot(&term, prior);
            let _ = self.rematerialize(); // restore the prior (already-valid) view
            return Err(format!("resource_put rolled back for <{url}>: {e}"));
        }
        Ok(pretty(
            &json!({ "url": url, "created": !existed, "triples": triples }),
        ))
    }

    /// `resource_delete`: delete one document (LDP DELETE). Rejects non-empty
    /// containers; ACL documents route through the authoritative write-through.
    fn tool_resource_delete(&mut self, args: &Value) -> Result<String, String> {
        let url = arg_str(args, "url")?.to_string();
        let name = valid_target(&url)?;

        if is_control_doc(&url) {
            self.gate_acl_write(&url)?;
            let out = if self.config.acp {
                self.store.delete_acl_acp(&url)?
            } else {
                self.store.delete_acl(&url)?
            };
            return Ok(pretty(&json!({
                "url": url,
                "deleted": out.existed,
                "access_control": true,
            })));
        }

        if !self.allowed(&url, Mode::Read) {
            return Err(not_found_error(&url));
        }
        let term = Term::NamedNode(name);
        if self.slot_index(&term).is_none() {
            return Err(not_found_error(&url));
        }
        if !self.allowed(&url, Mode::Write) {
            return Err(write_denied_error(&url));
        }
        if url.ends_with('/') {
            // A container: reject when its STORED containment listing is non-empty.
            let doc = self.find_doc(&url).expect("slot checked above");
            let q = format!("ASK {{ <{url}> <{LDP_CONTAINS}> ?m }}");
            if sparq_engine::ask(doc, &q)? {
                return Err(format!(
                    "container <{url}> is not empty — delete its members first"
                ));
            }
        }

        // Remove the document and its parent's containment link (rollback-capable).
        let prior = self.take_slot(&term).expect("slot checked above");
        let removed_link = self.unlink_parent(&url);
        if let Err(e) = self.rematerialize() {
            if let Some((pterm, t)) = &removed_link {
                if let Some(i) = self.slot_index(pterm) {
                    let _ = self.store.graph.named[i].1.insert_triple(
                        t[0].clone(),
                        t[1].clone(),
                        t[2].clone(),
                    );
                }
            }
            self.restore_slot(&term, Some(prior));
            let _ = self.rematerialize();
            return Err(format!("resource_delete rolled back for <{url}>: {e}"));
        }
        Ok(pretty(&json!({ "url": url, "deleted": true })))
    }

    /// `container_create`: create one empty, typed container and link it into its
    /// parent containers.
    fn tool_container_create(&mut self, args: &Value) -> Result<String, String> {
        let url = arg_str(args, "url")?.to_string();
        let name = valid_target(&url)?;
        if !url.ends_with('/') {
            return Err(format!("container IRI must be slash-terminated: <{url}>"));
        }
        self.gate_write(&url, Mode::Write)?;
        let term = Term::NamedNode(name);
        if self.slot_index(&term).is_some() {
            return Err(format!("container <{url}> already exists"));
        }

        let container = container_graph(&url)?;
        self.store.graph.named.push((term.clone(), container));
        let (created_graphs, added_links) = match self.link_containment(&url) {
            Ok(x) => x,
            Err(e) => {
                self.restore_slot(&term, None);
                return Err(e);
            }
        };
        if let Err(e) = self.rematerialize() {
            self.unlink_containment(&created_graphs, &added_links);
            self.restore_slot(&term, None);
            let _ = self.rematerialize();
            return Err(format!("container_create rolled back for <{url}>: {e}"));
        }
        Ok(pretty(&json!({ "url": url, "created": true })))
    }

    /// Remove and return the named-graph slot for `term`, if present.
    fn take_slot(&mut self, term: &Term) -> Option<Graph> {
        let pos = self.slot_index(term)?;
        Some(self.store.graph.named.swap_remove(pos).1)
    }

    /// Put `prior` back into the slot for `term` (dropping whatever is there now).
    fn restore_slot(&mut self, term: &Term, prior: Option<Graph>) {
        let _ = self.take_slot(term);
        if let Some(g) = prior {
            self.store.graph.named.push((term.clone(), g));
        }
    }

    /// Walk the parent chain of a CREATED resource up to the origin root, inserting
    /// each missing `(parent, ldp:contains, child)` triple into the parent container's
    /// stored graph (creating a typed container graph for a missing parent). Returns
    /// what was added — `(created container-graph names, inserted links)` — so a
    /// failed re-materialization can roll every addition back precisely.
    fn link_containment(&mut self, url: &str) -> Result<(Vec<Term>, Vec<AddedLink>), String> {
        let mut created: Vec<Term> = Vec::new();
        let mut added: Vec<AddedLink> = Vec::new();
        let contains = NamedNode::new(LDP_CONTAINS).expect("valid constant IRI");
        let mut child = url.to_string();
        while let Some(parent) = parent_iri(&child) {
            let pnode = NamedNode::new(parent.as_str())
                .map_err(|e| format!("invalid parent container IRI <{parent}>: {e}"))?;
            let pterm = Term::NamedNode(pnode.clone());
            let pos = match self.slot_index(&pterm) {
                Some(i) => i,
                None => {
                    self.store
                        .graph
                        .named
                        .push((pterm.clone(), container_graph(&parent)?));
                    created.push(pterm.clone());
                    self.store.graph.named.len() - 1
                }
            };
            let q = format!("ASK {{ <{parent}> <{LDP_CONTAINS}> <{child}> }}");
            let present = sparq_engine::ask(&self.store.graph.named[pos].1, &q)?;
            if !present {
                let cnode = NamedNode::new(child.as_str())
                    .map_err(|e| format!("invalid member IRI <{child}>: {e}"))?;
                let triple = [
                    Term::NamedNode(pnode.clone()),
                    Term::NamedNode(contains.clone()),
                    Term::NamedNode(cnode),
                ];
                self.store.graph.named[pos].1.insert_triple(
                    triple[0].clone(),
                    triple[1].clone(),
                    triple[2].clone(),
                )?;
                added.push((pterm, triple));
            }
            child = parent;
        }
        Ok((created, added))
    }

    /// Roll back exactly what [`SolidMcpServer::link_containment`] added.
    fn unlink_containment(&mut self, created: &[Term], added: &[AddedLink]) {
        for (pterm, t) in added.iter().rev() {
            if let Some(i) = self.slot_index(pterm) {
                let _ = self.store.graph.named[i].1.remove_triple(
                    t[0].clone(),
                    t[1].clone(),
                    t[2].clone(),
                );
            }
        }
        for term in created.iter().rev() {
            let _ = self.take_slot(term);
        }
    }

    /// Remove the `(parent, ldp:contains, url)` link from the parent container's
    /// stored graph, returning it for rollback (`None` when absent).
    fn unlink_parent(&mut self, url: &str) -> Option<AddedLink> {
        let parent = parent_iri(url)?;
        let pterm = Term::NamedNode(NamedNode::new(parent.as_str()).ok()?);
        let pos = self.slot_index(&pterm)?;
        let q = format!("ASK {{ <{parent}> <{LDP_CONTAINS}> <{url}> }}");
        if !sparq_engine::ask(&self.store.graph.named[pos].1, &q).unwrap_or(false) {
            return None;
        }
        let triple = [
            Term::NamedNode(NamedNode::new(parent.as_str()).ok()?),
            Term::NamedNode(NamedNode::new(LDP_CONTAINS).expect("valid constant IRI")),
            Term::NamedNode(NamedNode::new(url).ok()?),
        ];
        let _ = self.store.graph.named[pos].1.remove_triple(
            triple[0].clone(),
            triple[1].clone(),
            triple[2].clone(),
        );
        Some((pterm, triple))
    }
}

/// A fresh, typed container document: `<url> a ldp:BasicContainer, ldp:Container .`
fn container_graph(url: &str) -> Result<Graph, String> {
    let nt = format!(
        "<{url}> <{RDF_TYPE}> <{LDP_BASIC_CONTAINER}> .\n<{url}> <{RDF_TYPE}> <{LDP_CONTAINER}> .\n"
    );
    Graph::load_str(&nt, "ntriples")
}

#[cfg(test)]
mod unit {
    use super::*;

    // Direct unit tests for the new public functions/consts (coverage-floor rule:
    // one direct test per new public fn), plus the private path helpers.

    #[test]
    fn parent_iri_walks_slash_semantics_to_the_origin_root() {
        assert_eq!(
            parent_iri("https://h.ex/a/b/doc.ttl").as_deref(),
            Some("https://h.ex/a/b/")
        );
        assert_eq!(
            parent_iri("https://h.ex/a/b/").as_deref(),
            Some("https://h.ex/a/")
        );
        assert_eq!(
            parent_iri("https://h.ex/a/").as_deref(),
            Some("https://h.ex/")
        );
        assert_eq!(
            parent_iri("https://h.ex/doc"),
            Some("https://h.ex/".to_string())
        );
        assert_eq!(parent_iri("https://h.ex/"), None);
        assert_eq!(parent_iri("urn:sparq:auth"), None); // no slash hierarchy
    }

    #[test]
    fn rdf_format_maps_media_types_and_rejects_non_rdf() {
        assert_eq!(rdf_format("text/turtle"), Some("turtle"));
        assert_eq!(rdf_format("text/turtle; charset=utf-8"), Some("turtle"));
        assert_eq!(rdf_format("application/n-triples"), Some("ntriples"));
        assert_eq!(rdf_format("application/octet-stream"), None);
        assert_eq!(rdf_format("application/ld+json"), None); // honest v1 limit
    }

    #[test]
    fn not_found_error_is_a_pure_function_of_the_url() {
        // Load-bearing for §9.3: the SAME template must serve both the nonexistent
        // and the unauthorized case, so it may depend on nothing but the URL.
        assert_eq!(
            not_found_error("https://p.ex/x"),
            "resource not found: <https://p.ex/x>"
        );
    }

    #[test]
    fn valid_target_rejects_the_reserved_graph_space() {
        assert!(valid_target("urn:sparq:auth").is_err());
        assert!(valid_target("not an iri").is_err());
        assert!(valid_target("https://p.ex/ok").is_ok());
    }

    #[test]
    fn default_config_is_read_only_wac_and_bounded() {
        let c = SolidServerConfig::default();
        assert!(!c.allow_update);
        assert!(!c.acp);
        assert_eq!(c.query_timeout_secs, Some(30));
        assert_eq!(c.max_rows, Some(1_000_000));
        assert_eq!(c.server_name, "sparq-mcp-solid");
        assert!(c.agent.is_none() && c.client.is_none() && c.issuer.is_none() && c.now.is_none());
    }

    #[test]
    fn tool_spec_consts_serialize_with_schema() {
        for spec in [
            &POD_QUERY,
            &RESOURCE_GET,
            &CONTAINER_LIST,
            &POD_UPDATE,
            &RESOURCE_PUT,
            &RESOURCE_DELETE,
            &CONTAINER_CREATE,
        ] {
            let j = spec.to_json();
            assert_eq!(j["name"].as_str(), Some(spec.name));
            assert_eq!(j["inputSchema"]["type"], "object");
        }
    }
}
