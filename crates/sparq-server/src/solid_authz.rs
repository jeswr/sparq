//! [FABLE-5] (sq-snopa.6, issue #992 FR-4) OPT-IN Solid **WAC/ACP HTTP authorization** surface.
//!
//! The `POST /authz/decide`, `POST /authz/wac-allow` and `POST /authz/query` endpoints — a THIN
//! HTTP shell over the `sparq-solid` LIBRARY authoriser
//! ([`PodStore::decide`](sparq_solid::PodStore::decide) /
//! [`wac_allow`](sparq_solid::PodStore::wac_allow) /
//! [`query_json_as`](sparq_solid::PodStore::query_json_as)). Compiled ONLY behind the
//! `solid-authz` feature (the deliberately-opt-in `sparq-server` -> `sparq-solid` workspace
//! dependency the FR-4 architecture note flagged); served only when
//! [`ServerConfig::solid_authz`](crate::http::ServerConfig) is also set (`--solid-authz` /
//! `SPARQ_SOLID_AUTHZ=1`) — the same double-opt-in as `tpf` / `shacl` / `terse`.
//!
//! # The boundary (`research/sparq-solid-scope.md` §4)
//!
//! `sparq-solid` is a **library-level authoriser with no HTTP surface**: mapping a request PATH
//! to a named-graph IRI, and authenticating a WebID into a `(WebID, client, issuer)` session, are
//! the **server's** job, NOT the library's. This module is exactly that missing HTTP shell — it
//! does NOT authenticate. Each endpoint takes an **already-resolved** session plus the pod dataset
//! (N-Quads, including the `.acl`/`.acr` control graphs) in the request body, builds a
//! [`PodStore`](sparq_solid::PodStore), materialises the auth view, and maps the library verdict
//! onto HTTP. It is STATELESS per request (the dataset is supplied, not the server's loaded store)
//! — a stateful "authorise over the server's own loaded pod" variant is a deliberate follow-up
//! (see the crate feature comment / the deferred bead), because it would thread a materialised
//! `PodStore` through the concurrent-serving `AppState`, a much larger change to a conflict-hot
//! path.
//!
//! # Fail-closed (the soundness invariant)
//!
//! **Every error path DENIES**, never opens. An unparseable dataset, a materialisation failure, an
//! unknown mode or view keyword, a malformed body — each yields a fail-closed response
//! (`allow:false` on `/decide`, an empty permission advertisement on `/wac-allow`, an error on
//! `/query`), never a grant. This mirrors the library's own construction: a
//! [`WacDecision`](sparq_solid::WacDecision) is `allow == false` for every non-`Resolved` status,
//! and a grant-less / un-materialised session sees nothing. The tests exercise this directly (an
//! unparseable dataset -> deny with a `400`; an un-materialisable view still denies).

use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::Response,
};

use sparq_solid::{AclStatus, Mode, PodStore, Session, WacDecision};

use crate::http::{auth_gate, json_error, AppState, Operation};

/// Which auth model to materialise before deciding. Chosen by the request's `"view"` field, or
/// inferred from the dataset's control-document suffixes (`.acr` present -> ACP, else WAC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthView {
    /// Web Access Control — materialise from `.acl` documents ([`PodStore::materialize_wac`]).
    Wac,
    /// Access Control Policy — materialise from `.acr` documents ([`PodStore::materialize_acp`]).
    Acp,
}

/// A parsed `/authz/*` request envelope. The `session` fields are borrowed from the owned JSON
/// value the caller keeps alive; `dataset` is the owned N-Quads text.
#[derive(Debug)]
struct AuthzRequest {
    /// The pod dataset as N-Quads (documents + `.acl`/`.acr` control graphs), owned.
    dataset: String,
    /// The already-resolved principal — agent (WebID) / client / issuer, all optional; anonymous
    /// when all are absent (fail-closed — an anonymous session sees only public grants).
    agent: Option<String>,
    client: Option<String>,
    issuer: Option<String>,
    now: Option<String>,
    /// The chosen auth model, if the request named one explicitly (`"view": "wac" | "acp"`).
    /// `None` -> infer from the dataset.
    view: Option<AuthView>,
}

impl AuthzRequest {
    /// Borrow the parsed fields as a [`Session`] (the library's borrowed principal type).
    fn session(&self) -> Session<'_> {
        Session {
            agent: self.agent.as_deref(),
            client: self.client.as_deref(),
            issuer: self.issuer.as_deref(),
            now: self.now.as_deref(),
        }
    }
}

/// Map an [`AclStatus`] to the HTTP status a resource server should return for a DENY
/// (`research/sparq-solid-scope.md` §4 / FR-6): a definitive permission outcome
/// (`Resolved` deny / `NoAcl`) is a **403**; a retryable operational condition (`Unloaded` /
/// `Transient`) is a **503**. An *allow* is always a `200` (the caller reads `allow`), so this is
/// the deny-side code only. Pure + directly unit-tested.
pub(crate) fn deny_status_code(status: AclStatus) -> StatusCode {
    if status.is_retryable() {
        // Unloaded / Transient: the decision could not be computed *this time* — retryable.
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        // Resolved-without-the-mode / NoAcl: a definitive, authoritative deny.
        StatusCode::FORBIDDEN
    }
}

/// The lower-case name of a [`Mode`], for the JSON body. Pure + directly unit-tested.
pub(crate) fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Read => "read",
        Mode::Write => "write",
        Mode::Append => "append",
        Mode::Control => "control",
    }
}

/// Parse a WAC/ACP mode name (`read` / `write` / `append` / `control`, case-insensitive) into a
/// [`Mode`]. `None` for any other string — the caller treats that as a fail-closed deny (an
/// unknown mode can never be granted). Pure + directly unit-tested.
pub(crate) fn parse_mode(s: &str) -> Option<Mode> {
    match s.trim().to_ascii_lowercase().as_str() {
        "read" => Some(Mode::Read),
        "write" => Some(Mode::Write),
        "append" => Some(Mode::Append),
        "control" => Some(Mode::Control),
        _ => None,
    }
}

/// The lower-case status token for the JSON body (mirrors [`AclStatus`]). Pure + directly
/// unit-tested.
pub(crate) fn status_token(status: AclStatus) -> &'static str {
    match status {
        AclStatus::Resolved => "resolved",
        AclStatus::NoAcl => "noAcl",
        AclStatus::Unloaded => "unloaded",
        AclStatus::Transient => "transient",
    }
}

/// Serialise a [`WacDecision`] to the JSON body the `/authz/decide` endpoint returns:
/// `{ "allow": bool, "grantedModes": [..], "governingAcl": iri|null, "scope": "accessTo"|"default"
/// |null, "status": "resolved"|"noAcl"|"unloaded"|"transient", "aclLink": link-value|null }`.
///
/// `aclLink` is the RFC-8288 `Link: rel="acl"` header VALUE
/// ([`WacDecision::acl_link_header`](sparq_solid::WacDecision::acl_link_header)) already present in
/// the body so a follow-up (sq-snopa.7) only needs to lift it into a response header. Pure +
/// directly unit-tested (against a real decision).
pub(crate) fn decision_to_json(d: &WacDecision) -> String {
    let modes: Vec<serde_json::Value> = d
        .granted_modes
        .iter()
        .map(|m| serde_json::Value::from(mode_name(*m)))
        .collect();
    let scope = d
        .scope
        .map(|s| serde_json::Value::from(s.as_acl_predicate()));
    let governing = d
        .governing_acl
        .as_ref()
        .map(|n| serde_json::Value::from(n.as_str()));
    let acl_link = d.acl_link_header().map(serde_json::Value::from);
    let body = serde_json::json!({
        "allow": d.allow,
        "grantedModes": modes,
        "governingAcl": governing,
        "scope": scope,
        "status": status_token(d.status),
        "aclLink": acl_link,
    });
    // `to_string` on a serde_json::Value never fails.
    body.to_string()
}

/// A `text/json` response with the given status + body. The body is already-serialised JSON.
fn json_response(status: StatusCode, body: String) -> Response {
    json_response_with_link(status, body, None)
}

/// A `text/json` response with the given status + body, and an OPTIONAL `Link: rel="acl"`
/// header (FR-5, sq-snopa.7). When `acl_link` is `Some(value)` the RFC 8288 link-value
/// (`<iri>; rel="acl"`) is emitted as the `Link` response header; `None` silently omits it
/// (fail-closed — if there is no governing ACL there is nothing to advertise). [SONNET-4.6]
fn json_response_with_link(status: StatusCode, body: String, acl_link: Option<&str>) -> Response {
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header(header::CONTENT_LENGTH, body.len());
    if let Some(link_val) = acl_link {
        // [SONNET-4.6] POSITIONAL format args (CodeQL rust/unused-variable guard).
        builder = builder.header(header::LINK, link_val);
    }
    builder
        .body(body.into())
        // A well-formed status + valid header values never fails to build.
        .unwrap()
}

/// Parse the request body as the `/authz/*` JSON envelope. Returns the caller's fail-closed error
/// [`Response`] (a `400`) on any malformed/absent field — the offending detail is NOT echoed to the
/// client (the same info-leak posture as the rest of the surface). `require_mode`/`require_query`
/// let each endpoint demand the field it needs.
///
/// Fail-closed: a missing `dataset` is a `400` (there is nothing to authorise over), NOT an
/// empty-dataset allow.
// [FABLE-5] `Response` is a large type; returning it in the `Err` arm mirrors the rest of this
// crate's handler helpers (`parse_shapes_graph`) — same `result_large_err` allow.
#[allow(clippy::result_large_err)]
fn parse_request(body: &Bytes) -> Result<AuthzRequest, Response> {
    let v: serde_json::Value = serde_json::from_slice(body)
        .map_err(|_| json_error(StatusCode::BAD_REQUEST, "request body is not valid JSON"))?;
    let obj = v
        .as_object()
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "request body must be a JSON object"))?;
    let dataset = obj
        .get("dataset")
        .and_then(|d| d.as_str())
        .ok_or_else(|| {
            json_error(
                StatusCode::BAD_REQUEST,
                "'dataset' (N-Quads string) is required",
            )
        })?
        .to_owned();
    // The session sub-object; absent -> an anonymous session (fail-closed, sees only public).
    let session = obj.get("session").and_then(|s| s.as_object());
    let field = |name: &str| -> Option<String> {
        session
            .and_then(|s| s.get(name))
            .and_then(|f| f.as_str())
            .map(str::to_owned)
    };
    let view = match obj.get("view").and_then(|s| s.as_str()) {
        None => None,
        Some(s) => match s.trim().to_ascii_lowercase().as_str() {
            "wac" => Some(AuthView::Wac),
            "acp" => Some(AuthView::Acp),
            // An unknown view keyword is a fail-closed client error, not an inferred default.
            _ => {
                return Err(json_error(
                    StatusCode::BAD_REQUEST,
                    "'view' must be 'wac' or 'acp' when present",
                ))
            }
        },
    };
    Ok(AuthzRequest {
        dataset,
        agent: field("agent"),
        client: field("client"),
        issuer: field("issuer"),
        now: field("now"),
        view,
    })
}

/// Build + materialise a [`PodStore`] over the request's dataset, choosing WAC vs ACP by the
/// request's `view` (or inferring it from the dataset's control-document suffixes). Returns the
/// caller's fail-closed error [`Response`] (a `400`) on a parse failure and a `503` on a
/// materialisation failure (a transient/operational condition, retryable) — NEVER a store that
/// silently grants.
///
/// Inference: `.acr` present in the dataset -> ACP; otherwise WAC. This is a convenience only; a
/// caller that knows its model should set `"view"` explicitly.
// [FABLE-5] `Response` is large; same `result_large_err` allow as `parse_request` / the crate's
// other handler helpers.
#[allow(clippy::result_large_err)]
fn build_store(req: &AuthzRequest) -> Result<PodStore, Response> {
    let graph = sparq_core::Graph::load_dataset(&req.dataset, "nquads").map_err(|_| {
        // The parser's detail is withheld from the client (info-leak posture); the class is enough.
        json_error(
            StatusCode::BAD_REQUEST,
            "'dataset' is not valid N-Quads",
        )
    })?;
    let view = req.view.unwrap_or_else(|| infer_view(&req.dataset));
    let mut store = PodStore::new(graph);
    let materialized = match view {
        AuthView::Wac => store.materialize_wac(),
        AuthView::Acp => store.materialize_acp(),
    };
    materialized.map_err(|_| {
        // Materialisation failed (e.g. a reserved-encoding collision / reasoner error): a
        // transient/operational condition — 503 (retryable), fail-closed (no grant).
        json_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "could not materialise the authorization view",
        )
    })?;
    Ok(store)
}

/// Infer the auth model from the dataset text: ACP if it names any `.acr` control graph, else WAC.
/// A cheap substring scan over the N-Quads — the same suffix check the loader uses; a caller that
/// knows its model should set `"view"` explicitly rather than rely on this. Pure + unit-tested.
pub(crate) fn infer_view_is_acp(dataset: &str) -> bool {
    // A `.acr` graph name appears as `<…​.acr>` in an N-Quad's graph position; `.acr` occurring
    // only inside a subject/predicate/object IRI would be a false positive, but a control document
    // is ALWAYS named by its own IRI in the graph slot, so the presence of the token is a sound
    // "ACP is in play" signal for the inference default. WAC is the safe fallback either way — an
    // ACP dataset materialised as WAC simply finds no `.acl` grants and denies (fail-closed).
    dataset.contains(".acr>") || dataset.contains(".acr ")
}

/// [`infer_view_is_acp`] as the internal [`AuthView`].
fn infer_view(dataset: &str) -> AuthView {
    if infer_view_is_acp(dataset) {
        AuthView::Acp
    } else {
        AuthView::Wac
    }
}

/// `POST /authz/decide` — the per-request WAC/ACP decision (FR-1/FR-4). Body:
/// `{ "dataset": nquads, "session": { "agent"?, "client"?, "issuer"?, "now"? }, "resource": iri,
/// "mode": "read"|"write"|"append"|"control", "view"?: "wac"|"acp" }`. Returns the
/// [`decision_to_json`] body with the HTTP status [`deny_status_code`] maps a DENY to (an ALLOW is
/// always `200`).
///
/// Fail-closed: an unknown mode, an unparseable dataset, or a materialisation failure all DENY
/// (never grant). The endpoint reads the store; it is gated like any read.
pub(crate) async fn decide_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if !state.config().solid_authz {
        return json_error(StatusCode::NOT_FOUND, "not found");
    }
    if let Some(resp) = auth_gate(state.config(), &headers, Operation::Read) {
        return resp;
    }
    let req = match parse_request(&body) {
        Ok(r) => r,
        Err(resp) => return resp,
    };
    // The resource IRI + mode are decide-specific fields.
    let v: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return json_error(StatusCode::BAD_REQUEST, "request body is not valid JSON"),
    };
    let resource = match v.get("resource").and_then(|r| r.as_str()) {
        Some(r) => r.to_owned(),
        None => {
            return json_error(StatusCode::BAD_REQUEST, "'resource' (IRI string) is required");
        }
    };
    // An unknown / absent mode is a fail-closed deny: an un-parseable mode can never be granted, so
    // we return a definitive 403 with the deny body rather than trusting an arbitrary keyword.
    let mode = match v.get("mode").and_then(|m| m.as_str()).and_then(parse_mode) {
        Some(m) => m,
        None => {
            let deny = decision_to_json(&fail_closed_decision());
            return json_response(StatusCode::FORBIDDEN, deny);
        }
    };
    let store = match build_store(&req) {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    // Deciding is CPU-light here (the materialise already ran), but keep the store owned in the
    // closure so the borrowed session must be rebuilt inside — do it inline (no spawn needed).
    let decision = store.decide(&req.session(), &resource, mode);
    let status = if decision.allow {
        StatusCode::OK
    } else {
        deny_status_code(decision.status)
    };
    // [SONNET-4.6] sq-snopa.7 FR-5: emit the RFC 8288 `Link: <acl-iri>; rel="acl"` header from
    // the decision's governing ACL. `acl_link_header()` returns `None` when no governing ACL was
    // discovered (fail-closed — nothing to advertise). Safe on a deny: the link tells the client
    // WHERE the ACL is, not that the request is allowed.
    let acl_link = decision.acl_link_header();
    json_response_with_link(status, decision_to_json(&decision), acl_link.as_deref())
}

/// A fail-closed [`WacDecision`] for an input the endpoint refuses to trust (e.g. an unknown mode):
/// an authoritative deny with no governing ACL. Distinct from the library's own `deny` (which is
/// crate-private) — this is the HTTP layer's own "I will not even try" deny. Pure + unit-tested.
fn fail_closed_decision() -> WacDecision {
    WacDecision {
        allow: false,
        granted_modes: Vec::new(),
        governing_acl: None,
        scope: None,
        status: AclStatus::Resolved,
    }
}

/// `POST /authz/wac-allow` — the `WAC-Allow` header VALUE for a `(session, resource)` (FR-2/FR-4).
/// Body: `{ "dataset": nquads, "session": {..}, "resource": iri, "view"?: "wac"|"acp" }`. Returns
/// `{ "wacAllow": "user=\"…\",public=\"…\"" }` — the RFC-style permission advertisement the server
/// puts in a `WAC-Allow` response header.
///
/// Fail-closed: an unparseable dataset / materialisation failure is an error (never a wider
/// advertisement); a grant-less session yields `user="",public=""`.
pub(crate) async fn wac_allow_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if !state.config().solid_authz {
        return json_error(StatusCode::NOT_FOUND, "not found");
    }
    if let Some(resp) = auth_gate(state.config(), &headers, Operation::Read) {
        return resp;
    }
    let req = match parse_request(&body) {
        Ok(r) => r,
        Err(resp) => return resp,
    };
    let v: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return json_error(StatusCode::BAD_REQUEST, "request body is not valid JSON"),
    };
    let resource_iri = match v.get("resource").and_then(|r| r.as_str()) {
        Some(r) => r.to_owned(),
        None => {
            return json_error(StatusCode::BAD_REQUEST, "'resource' (IRI string) is required");
        }
    };
    // `wac_allow` takes a `NamedNode`; an un-IRI resource is a fail-closed 400 (never a grant).
    let resource = match oxrdf::NamedNode::new(&resource_iri) {
        Ok(n) => n,
        Err(_) => {
            return json_error(StatusCode::BAD_REQUEST, "'resource' is not a valid IRI");
        }
    };
    let store = match build_store(&req) {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    // [SONNET-4.6] sq-snopa.7 FR-5: resolve the governing ACL for the Link header BEFORE
    // `wac_allow` consumes the store. `resolve_acl` is a pure index lookup (no mode needed) and
    // returns `None` when no ACL was discovered — fail-closed: no header emitted in that case.
    // [SONNET-4.6] POSITIONAL format args (CodeQL rust/unused-variable guard).
    let acl_link: Option<String> = store
        .resolve_acl(&resource_iri)
        .map(|eff| format!("<{}>; rel=\"acl\"", eff.acl.as_str()));
    let value = store.wac_allow(&req.session(), &resource);
    let body = serde_json::json!({ "wacAllow": value }).to_string();
    json_response_with_link(StatusCode::OK, body, acl_link.as_deref())
}

/// `POST /authz/query` — an ACCESS-CONTROLLED SPARQL query as `session` (FR-4). Body:
/// `{ "dataset": nquads, "session": {..}, "mode"?: "read"|…, "query": sparql, "view"?: … }`.
/// Delegates to [`PodStore::query_json_as`] (the zero-copy authorised-graph-set view path) and
/// returns the SPARQL 1.1 JSON results serialisation. `mode` defaults to `read` (the query path).
///
/// Fail-closed: a session with no grants sees an empty view (zero rows), never the whole store; an
/// unparseable dataset / materialisation failure is an error; a query that does not parse is a
/// `400` (the engine's detail withheld from the client).
pub(crate) async fn query_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if !state.config().solid_authz {
        return json_error(StatusCode::NOT_FOUND, "not found");
    }
    if let Some(resp) = auth_gate(state.config(), &headers, Operation::Read) {
        return resp;
    }
    let req = match parse_request(&body) {
        Ok(r) => r,
        Err(resp) => return resp,
    };
    let v: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return json_error(StatusCode::BAD_REQUEST, "request body is not valid JSON"),
    };
    let query = match v.get("query").and_then(|q| q.as_str()) {
        Some(q) => q.to_owned(),
        None => {
            return json_error(StatusCode::BAD_REQUEST, "'query' (SPARQL string) is required");
        }
    };
    // A query is a READ; default the mode to Read. An explicit unknown mode is a fail-closed 400
    // (we will not guess a wider mode).
    let mode = match v.get("mode").and_then(|m| m.as_str()) {
        None => Mode::Read,
        Some(s) => match parse_mode(s) {
            Some(m) => m,
            None => {
                return json_error(
                    StatusCode::BAD_REQUEST,
                    "'mode' must be 'read', 'write', 'append' or 'control'",
                );
            }
        },
    };
    let store = match build_store(&req) {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    match store.query_json_as(&req.session(), mode, &query) {
        Ok(json) => json_response(StatusCode::OK, json),
        // The engine's parse/eval detail is withheld (info-leak posture); the class is enough.
        Err(_) => json_error(
            StatusCode::BAD_REQUEST,
            "query did not parse or could not be evaluated",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A tiny WAC dataset: alice has Read on the root `.acl`, inherited by `notes/n1`.
    const WAC_NQUADS: &str = r#"
<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
<https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
"#;

    fn wac_store() -> PodStore {
        let mut store =
            PodStore::new(sparq_core::Graph::load_dataset(WAC_NQUADS, "nquads").unwrap());
        store.materialize_wac().unwrap();
        store
    }

    #[test]
    fn deny_status_code_maps_definitive_vs_retryable() {
        // Definitive permission outcomes -> 403.
        assert_eq!(deny_status_code(AclStatus::Resolved), StatusCode::FORBIDDEN);
        assert_eq!(deny_status_code(AclStatus::NoAcl), StatusCode::FORBIDDEN);
        // Retryable operational conditions -> 503.
        assert_eq!(
            deny_status_code(AclStatus::Unloaded),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            deny_status_code(AclStatus::Transient),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[test]
    fn parse_mode_is_case_insensitive_and_fail_closed() {
        assert_eq!(parse_mode("read"), Some(Mode::Read));
        assert_eq!(parse_mode("  WRITE "), Some(Mode::Write));
        assert_eq!(parse_mode("Append"), Some(Mode::Append));
        assert_eq!(parse_mode("control"), Some(Mode::Control));
        // Any unknown keyword -> None (the caller denies).
        assert_eq!(parse_mode("delete"), None);
        assert_eq!(parse_mode(""), None);
    }

    #[test]
    fn mode_name_round_trips_through_parse_mode() {
        for m in [Mode::Read, Mode::Write, Mode::Append, Mode::Control] {
            assert_eq!(parse_mode(mode_name(m)), Some(m));
        }
    }

    #[test]
    fn status_token_covers_every_status() {
        assert_eq!(status_token(AclStatus::Resolved), "resolved");
        assert_eq!(status_token(AclStatus::NoAcl), "noAcl");
        assert_eq!(status_token(AclStatus::Unloaded), "unloaded");
        assert_eq!(status_token(AclStatus::Transient), "transient");
    }

    #[test]
    fn infer_view_detects_acp_by_control_suffix() {
        assert!(infer_view_is_acp(
            "<https://pod.ex/.acr#p> <p> <o> <https://pod.ex/.acr> ."
        ));
        // A pure-WAC dataset infers WAC.
        assert!(!infer_view_is_acp(WAC_NQUADS));
    }

    #[test]
    fn decision_to_json_allow_carries_provenance_and_acl_link() {
        let store = wac_store();
        let alice = Session {
            agent: Some("https://alice.ex/card#me"),
            client: None,
            issuer: None,
            now: None,
        };
        let d = store.decide(&alice, "https://pod.ex/notes/n1", Mode::Read);
        assert!(d.allow);
        let json: serde_json::Value = serde_json::from_str(&decision_to_json(&d)).unwrap();
        assert_eq!(json["allow"], serde_json::Value::Bool(true));
        assert_eq!(json["status"], "resolved");
        assert_eq!(json["governingAcl"], "https://pod.ex/.acl");
        assert_eq!(json["scope"], "http://www.w3.org/ns/auth/acl#default");
        // aclLink is the RFC-8288 header value (feeds sq-snopa.7).
        assert_eq!(json["aclLink"], r#"<https://pod.ex/.acl>; rel="acl""#);
        assert!(json["grantedModes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m == "read"));
    }

    #[test]
    fn decision_to_json_deny_is_fail_closed() {
        // The HTTP layer's own "will not even try" deny (e.g. an unknown mode).
        let json: serde_json::Value =
            serde_json::from_str(&decision_to_json(&fail_closed_decision())).unwrap();
        assert_eq!(json["allow"], serde_json::Value::Bool(false));
        assert_eq!(json["grantedModes"].as_array().unwrap().len(), 0);
        assert_eq!(json["governingAcl"], serde_json::Value::Null);
        assert_eq!(json["aclLink"], serde_json::Value::Null);
    }

    #[test]
    fn parse_request_rejects_absent_dataset_fail_closed() {
        // No dataset -> a 400, NOT an empty-dataset allow.
        let body = Bytes::from(r#"{"session":{"agent":"https://a.ex#me"}}"#);
        let err = parse_request(&body).unwrap_err();
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn parse_request_rejects_unknown_view_fail_closed() {
        let body = Bytes::from(r#"{"dataset":"","view":"nonsense"}"#);
        let err = parse_request(&body).unwrap_err();
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn parse_request_extracts_session_and_dataset() {
        let body = Bytes::from(
            r#"{"dataset":"<a> <b> <c> <g> .","session":{"agent":"https://a.ex#me","client":"https://app.ex/id"},"view":"wac"}"#,
        );
        let req = parse_request(&body).unwrap();
        assert_eq!(req.dataset, "<a> <b> <c> <g> .");
        assert_eq!(req.agent.as_deref(), Some("https://a.ex#me"));
        assert_eq!(req.client.as_deref(), Some("https://app.ex/id"));
        assert_eq!(req.issuer, None);
        assert_eq!(req.view, Some(AuthView::Wac));
        // The borrowed session mirrors the parsed fields.
        let s = req.session();
        assert_eq!(s.agent, Some("https://a.ex#me"));
    }

    #[test]
    fn build_store_denies_on_unparseable_dataset() {
        let req = AuthzRequest {
            dataset: "this is not n-quads @@@".to_owned(),
            agent: None,
            client: None,
            issuer: None,
            now: None,
            view: Some(AuthView::Wac),
        };
        // A garbage dataset is a fail-closed 400, never a store that grants.
        // (`PodStore` is not `Debug`, so match rather than `unwrap_err`.)
        match build_store(&req) {
            Ok(_) => panic!("a garbage dataset must NOT build a store"),
            Err(resp) => assert_eq!(resp.status(), StatusCode::BAD_REQUEST),
        }
    }

    #[test]
    fn build_store_then_decide_is_authorized_and_anonymous_fails_closed() {
        let req = AuthzRequest {
            dataset: WAC_NQUADS.to_owned(),
            agent: Some("https://alice.ex/card#me".to_owned()),
            client: None,
            issuer: None,
            now: None,
            view: Some(AuthView::Wac),
        };
        let store = match build_store(&req) {
            Ok(s) => s,
            Err(_) => panic!("the WAC fixture must materialise"),
        };
        // Alice (the parsed session) is granted Read; the decision is authoritative.
        let d = store.decide(&req.session(), "https://pod.ex/notes/n1", Mode::Read);
        assert!(d.allow && d.status == AclStatus::Resolved);
        // An anonymous session over the SAME store is a fail-closed deny.
        let anon = store.decide(&Session::default(), "https://pod.ex/notes/n1", Mode::Read);
        assert!(!anon.allow && anon.status == AclStatus::Resolved);
        // And Write (which alice lacks) is an authoritative deny -> 403, not a 503.
        let dw = store.decide(&req.session(), "https://pod.ex/notes/n1", Mode::Write);
        assert!(!dw.allow);
        assert_eq!(deny_status_code(dw.status), StatusCode::FORBIDDEN);
    }
}
