//! The axum HTTP surface: routing, extraction, status/error semantics.
//!
//! Two route groups:
//!   * `/sparql` — the SPARQL 1.1 Protocol `query` operation (GET + the two POST forms).
//!   * Graph Store HTTP Protocol READ — `GET`/`HEAD` on `/graphs/*path` (direct) and on
//!     `/sparql/graph` via `?graph=<uri>` / `?default` (indirect). Write verbs → 501.
//!
//! Shared state is a [`sparq_core::Graph`] behind an `RwLock<Arc<…>>`: queries take a cheap
//! lock-free `Arc` snapshot; SPARQL Update (T11b) rebuilds the graph and swaps it in atomically.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::{
    body::Bytes,
    extract::{Query, RawQuery, State},
    http::{header, HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{any, get},
    Router,
};

use sparq_core::Graph;

use crate::exec::{prepare, PrepareError, QueryForm};
use crate::negotiate::{negotiate, Format};
use crate::results;

/// Shared server state: the dataset under query, behind an `RwLock` so SPARQL Update can swap in a
/// rebuilt graph atomically. Queries take a cheap `Arc` snapshot (lock-free for the duration of the
/// request); an update takes the write lock briefly to install the new graph.
#[derive(Clone)]
pub struct AppState {
    graph: Arc<RwLock<Arc<Graph>>>,
}

impl AppState {
    pub fn new(graph: Graph) -> Self {
        Self { graph: Arc::new(RwLock::new(Arc::new(graph))) }
    }

    /// A cheap, consistent snapshot of the current graph for the duration of a request.
    pub fn snapshot(&self) -> Arc<Graph> {
        self.graph.read().expect("graph lock poisoned").clone()
    }

    /// Apply a SPARQL Update, atomically swapping in the rebuilt graph (T11b).
    pub fn apply_update(&self, sparql: &str) -> Result<(), String> {
        let next = sparq_engine::update(&self.snapshot(), sparql)?;
        *self.graph.write().expect("graph lock poisoned") = Arc::new(next);
        Ok(())
    }
}

/// Builds the application router for the SPARQL Protocol + GSP-read endpoints.
pub fn router(state: AppState) -> Router {
    Router::new()
        // SPARQL 1.1 Protocol — query operation. `any` so we can return a proper 405 with
        // an `Allow` header for unsupported methods (update verbs are T11b).
        .route("/sparql", any(sparql_endpoint))
        // Graph Store HTTP Protocol (indirect graph identification): ?graph=<uri> / ?default
        .route("/sparql/graph", any(graph_store_indirect))
        // Graph Store HTTP Protocol (direct graph identification).
        .route("/graphs/*path", any(graph_store_direct))
        // Liveness.
        .route("/health", get(|| async { "ok" }))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// SPARQL 1.1 Protocol — query operation
// ---------------------------------------------------------------------------

const SPARQL_QUERY_CT: &str = "application/sparql-query";
const FORM_CT: &str = "application/x-www-form-urlencoded";

async fn sparql_endpoint(
    State(state): State<AppState>,
    method: Method,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    match method {
        Method::GET | Method::HEAD => {
            // Query string carries `query=` (+ optional dataset params). Per protocol, a
            // GET without a `query` parameter is a malformed request (400).
            let params = parse_form(raw_query.as_deref().unwrap_or(""));
            match params.get("query") {
                Some(q) => run_query(&state, q, &headers, method == Method::HEAD),
                None => bad_request("missing 'query' parameter"),
            }
        }
        Method::POST => handle_post(&state, &headers, &body),
        _ => method_not_allowed(&[Method::GET, Method::HEAD, Method::POST]),
    }
}

fn handle_post(state: &AppState, headers: &HeaderMap, body: &Bytes) -> Response {
    let ct = content_type(headers);
    if ct.starts_with(SPARQL_QUERY_CT) {
        // POST directly — body IS the SPARQL query.
        match std::str::from_utf8(body) {
            Ok(q) => run_query(state, q, headers, false),
            Err(_) => bad_request("request body is not valid UTF-8"),
        }
    } else if ct.starts_with(FORM_CT) {
        // POST url-encoded — `query=` in the body.
        let s = match std::str::from_utf8(body) {
            Ok(s) => s,
            Err(_) => return bad_request("request body is not valid UTF-8"),
        };
        let params = parse_form(s);
        match params.get("query") {
            Some(q) => run_query(state, q, headers, false),
            None => bad_request("missing 'query' parameter in url-encoded body"),
        }
    } else if ct.starts_with("application/sparql-update") {
        // SPARQL 1.1 Protocol — update operation (T11b). Body IS the update; success → 204.
        match std::str::from_utf8(body) {
            Ok(u) => match state.apply_update(u) {
                Ok(()) => StatusCode::NO_CONTENT.into_response(),
                Err(e) => bad_request(&format!("update failed: {e}")),
            },
            Err(_) => bad_request("request body is not valid UTF-8"),
        }
    } else {
        // Unsupported media type for the POST query operation.
        (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "POST requires Content-Type 'application/sparql-query' or 'application/x-www-form-urlencoded'",
        )
            .into_response()
    }
}

/// Executes a query string and renders the negotiated representation.
fn run_query(state: &AppState, sparql: &str, headers: &HeaderMap, head_only: bool) -> Response {
    let prepared = match prepare(sparql) {
        Ok(p) => p,
        Err(PrepareError::Malformed(msg)) => return bad_request(&format!("malformed query: {msg}")),
    };
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok());
    let fmt = negotiate(accept);

    match prepared.form {
        QueryForm::Select => {
            let select = prepared.runnable_select.expect("SELECT always has a runnable query");
            render_select(state, &select, fmt, head_only)
        }
        QueryForm::Ask => {
            let select = prepared.runnable_select.expect("ASK always rewrites to a SELECT");
            match sparq_engine::count(&state.snapshot(), &select) {
                Ok(n) => {
                    let value = n > 0;
                    let (body, ct) = match fmt {
                        Format::Xml => (results::ask_to_xml(value), fmt.ask_content_type()),
                        _ => (results::ask_to_json(value), fmt.ask_content_type()),
                    };
                    text_response(StatusCode::OK, ct, body, head_only)
                }
                Err(e) => execution_error(&e),
            }
        }
        QueryForm::Construct | QueryForm::Describe => not_implemented(
            "CONSTRUCT/DESCRIBE are not yet supported: the engine exposes no RDF-graph result API (documented follow-up)",
        ),
    }
}

/// Runs a SELECT on the engine and serialises it in the negotiated format. JSON uses the
/// engine's direct id→JSON fast path; the others go through the materialised `QueryResult`.
fn render_select(state: &AppState, select: &str, fmt: Format, head_only: bool) -> Response {
    let ct = fmt.select_content_type();
    let body = match fmt {
        Format::Json => match sparq_engine::query_json(&state.snapshot(), select) {
            Ok(json) => json,
            Err(e) => return execution_error(&e),
        },
        _ => {
            let result = match sparq_engine::query(&state.snapshot(), select) {
                Ok(r) => r,
                Err(e) => return execution_error(&e),
            };
            match fmt {
                Format::Xml => results::select_to_xml(&result),
                Format::Csv => results::select_to_csv(&result),
                Format::Tsv => results::select_to_tsv(&result),
                Format::Json => unreachable!(),
            }
        }
    };
    text_response(StatusCode::OK, ct, body, head_only)
}

// ---------------------------------------------------------------------------
// Graph Store HTTP Protocol — READ side
// ---------------------------------------------------------------------------

/// Indirect graph identification: `?default` or `?graph=<iri>`.
async fn graph_store_indirect(
    State(state): State<AppState>,
    method: Method,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let is_default = params.contains_key("default");
    let named = params.get("graph").cloned();
    if !is_default && named.is_none() {
        return bad_request("indirect graph identification requires '?default' or '?graph=<uri>'");
    }
    graph_store_read(&state, &method, named.as_deref(), &headers)
}

/// Direct graph identification: the path IS (a path under) the graph's resource.
async fn graph_store_direct(
    State(state): State<AppState>,
    method: Method,
    axum::extract::Path(path): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Response {
    // The named graph here is the request's effective URI path; with no named-graph store,
    // every direct graph maps onto the default graph (documented limitation).
    let _ = path;
    graph_store_read(&state, &method, None, &headers)
}

/// Shared GSP handler. READ (GET/HEAD) serialises the (default) graph; write verbs → 501.
fn graph_store_read(state: &AppState, method: &Method, named: Option<&str>, headers: &HeaderMap) -> Response {
    match *method {
        Method::GET | Method::HEAD => {
            // The engine has no named-graph store: only the default graph is materialised.
            // A request for a *named* graph other than default cannot be served as that
            // graph; we serve the default-graph contents and document the limitation.
            let _ = named;
            let head_only = *method == Method::HEAD;
            serialise_default_graph(state, headers, head_only)
        }
        Method::PUT | Method::POST | Method::DELETE | Method::PATCH => not_implemented(
            "Graph Store write operations (PUT/POST/DELETE) are not implemented (thread T11b)",
        ),
        _ => method_not_allowed(&[Method::GET, Method::HEAD]),
    }
}

/// Serialises the default graph as N-Triples (the one RDF serialisation we can emit from
/// the engine's term-level scan without a full Turtle writer). Negotiation is minimal:
/// `text/turtle` and `application/n-triples` both yield N-Triples (valid Turtle), default
/// `application/n-triples`.
fn serialise_default_graph(state: &AppState, headers: &HeaderMap, head_only: bool) -> Response {
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    // N-Triples is a syntactic subset of Turtle, so it satisfies a text/turtle Accept too.
    let ct = if accept.contains("text/turtle") {
        "text/turtle; charset=utf-8"
    } else {
        "application/n-triples; charset=utf-8"
    };
    let body = nt_dump(&state.snapshot());
    text_response(StatusCode::OK, ct, body, head_only)
}

/// Dumps the whole default graph as N-Triples by reusing the engine's SELECT path
/// (`?s ?p ?o`) and the materialised terms — no private store API needed.
fn nt_dump(graph: &Graph) -> String {
    let r = match sparq_engine::query(graph, "SELECT ?s ?p ?o WHERE { ?s ?p ?o }") {
        Ok(r) => r,
        Err(_) => return String::new(),
    };
    let mut out = String::with_capacity(r.rows.len() * 64);
    for row in &r.rows {
        let (Some(s), Some(p), Some(o)) = (&row[0], &row[1], &row[2]) else { continue };
        nt_term(&mut out, s);
        out.push(' ');
        nt_term(&mut out, p);
        out.push(' ');
        nt_term(&mut out, o);
        out.push_str(" .\n");
    }
    out
}

fn nt_term(out: &mut String, t: &oxrdf::Term) {
    // oxrdf's Display for Term already produces canonical N-Triples term syntax.
    use std::fmt::Write;
    let _ = write!(out, "{t}");
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parses an `application/x-www-form-urlencoded` string into a map (last value wins).
fn parse_form(s: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in s.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        map.insert(form_decode(k), form_decode(v));
    }
    map
}

/// Decodes a single `application/x-www-form-urlencoded` component (`+` → space, `%XX`).
fn form_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hi = hex_val(bytes[i + 1]);
                let lo = hex_val(bytes[i + 2]);
                match (hi, lo) {
                    (Some(h), Some(l)) => {
                        out.push((h << 4) | l);
                        i += 3;
                    }
                    _ => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn content_type(headers: &HeaderMap) -> String {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// Builds a `text`-ish response with the given content type; for HEAD, omits the body but
/// keeps the `Content-Type` (and an accurate `Content-Length` via the header) so HEAD
/// mirrors GET.
fn text_response(status: StatusCode, content_type: &str, body: String, head_only: bool) -> Response {
    let len = body.len();
    // For HEAD we advertise the same Content-Length the GET would have, with an empty body.
    let out_body = if head_only { String::new() } else { body };
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, len)
        .body(out_body.into())
        .unwrap()
}

fn bad_request(msg: &str) -> Response {
    (StatusCode::BAD_REQUEST, msg.to_string()).into_response()
}

fn execution_error(msg: &str) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("query execution error: {msg}")).into_response()
}

fn not_implemented(msg: &str) -> Response {
    (StatusCode::NOT_IMPLEMENTED, msg.to_string()).into_response()
}

fn method_not_allowed(allow: &[Method]) -> Response {
    let allow_value = allow.iter().map(|m| m.as_str()).collect::<Vec<_>>().join(", ");
    Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .header(header::ALLOW, allow_value)
        .body(axum::body::Body::from("method not allowed"))
        .unwrap()
}
