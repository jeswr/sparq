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
//!
//! # The ODRL lane (`odrl-authz` feature, sq-lrtc3.1 + sq-3mu76) [FABLE-5]
//!
//! Behind the OPT-IN `odrl-authz` cargo feature, ALL THREE endpoints (`/authz/query`,
//! `/authz/decide`, `/authz/wac-allow` — the latter two are sq-3mu76) grow an ODRL lane: when the
//! request dataset carries ODRL policy rules (`odrl:permission` / `odrl:prohibition` predicates),
//! the handler parses them from the UNION of the dataset's graphs (`sparq_policy::parse_policy`)
//! and runs the sparq-solid ODRL bridge (`PodStore::materialize_odrl_policy` — the Permit allow
//! side AND the matched-Prohibition deny side, deny-overrides) for the request's
//! (party = session agent, action = `odrl:read`, target = each rule's target graph) BEFORE
//! `query_json_as` / `decide` / `wac_allow` runs. The bridged grants/denies compose with the
//! WAC/ACP view through the UNCHANGED `∪ allow ∖ ∪ deny` enforcement — an ODRL prohibition beats
//! a static WAC grant, on the enforcement surface AND the advisory ones (pre-sq-3mu76, a dataset
//! carrying a prohibition could get a `/authz/decide` allow that `/authz/query` would refuse to
//! honour). Fail-closed refusals (never a silent allow): a malformed policy, an unimplementable
//! `odrl:conflict` strategy, a rule without a concrete target graph IRI (pattern targets are
//! sq-lrtc3.3), a non-read mode (the query-action contract is sq-lrtc3.2), or an anonymous
//! session each yield a 4xx. Stateless-lane scope only (sq-snopa.8 owns the stateful variant).
//!
//! ## Read-scoped advertisement (sq-3mu76) [FABLE-5]
//!
//! The lane evidences ONLY the `odrl:read` action until the sq-lrtc3.2 action contract lands, so
//! when it fires the ADVISORY mode advertisements are scoped down to what it evidenced:
//! `/authz/decide` masks `grantedModes` to the requested (read) mode, and `/authz/wac-allow`
//! reports at most `user="read"` with `public=""` (an anonymous session is REFUSED wherever the
//! lane fires, so public access is never advertised under an ODRL policy). Under-advertising is
//! the fail-closed direction — the `allow` bit / the advertised `read` are exactly what
//! `/authz/query` enforces; a WAC write grant is merely not repeated where the ODRL lane cannot
//! yet evaluate a write-action rule against it.

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
    let obj = v.as_object().ok_or_else(|| {
        json_error(
            StatusCode::BAD_REQUEST,
            "request body must be a JSON object",
        )
    })?;
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
        json_error(StatusCode::BAD_REQUEST, "'dataset' is not valid N-Quads")
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
///
/// [FABLE-5] sq-3mu76: behind the `odrl-authz` feature, a dataset carrying ODRL policy rules
/// runs the [`apply_odrl_lane`] materialisation first, so a prohibition denies HERE exactly as
/// `/authz/query` refuses to honour it — with the query lane's refusal matrix (non-read mode =>
/// 400 until sq-lrtc3.2, anonymous session => 403) and the read-scoped `grantedModes`
/// advertisement (module doc). NOTE the decision stays governing-ACL-scoped: with NO
/// discoverable `.acl`/`.acr` anywhere, `decide` still returns its fail-closed `noAcl` deny
/// even where a bridged ODRL grant lets `/authz/query` see rows — a deny-side-only divergence.
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
            return json_error(
                StatusCode::BAD_REQUEST,
                "'resource' (IRI string) is required",
            );
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
    // [FABLE-5] sq-3mu76 — the trust dispatch below decides over its OWN store and would
    // BYPASS the ODRL lane, silently dropping a prohibition (fail-OPEN through a side door).
    // Composing the two lanes is future work; until then the combination is refused up front.
    #[cfg(all(feature = "solid-authz-trust", feature = "odrl-authz"))]
    if state.config().solid_authz_trust
        && v.get("trust").is_some()
        && dataset_carries_odrl(&req.dataset)
    {
        return json_error(
            StatusCode::BAD_REQUEST,
            "the dataset carries ODRL policies; combining the trust extension with the \
             ODRL lane is not supported (fail-closed)",
        );
    }
    // [SONNET-4.6] sq-pfae.17 — DOUBLE-OPT-IN stateless trust-graph extension.
    // Activates ONLY when BOTH (a) the `solid-authz-trust` feature is compiled AND (b) the config
    // flag is set AND (c) the request carries a "trust" JSON block. Any failure => 403 DENY
    // (fail-closed: the trust path can only narrow, never broaden, the admission gate).
    #[cfg(feature = "solid-authz-trust")]
    if state.config().solid_authz_trust {
        if let Some(trust_val) = v.get("trust") {
            return trust_authz_decide(trust_val, &req, &resource, mode, &v);
        }
    }
    let store = match build_store(&req) {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    // [FABLE-5] sq-3mu76 — the OPT-IN ODRL lane, on the ADVISORY decision endpoint too: a
    // dataset-carried prohibition must deny here exactly as `/authz/query` refuses to honour
    // it. Same refusal matrix as the query lane (non-read mode => 400 until sq-lrtc3.2,
    // anonymous => 403). Feature OFF => this block does not exist, byte-identical behaviour.
    #[cfg(feature = "odrl-authz")]
    let (store, odrl_lane_fired) = {
        let mut store = store;
        let fired = dataset_carries_odrl(&req.dataset);
        if fired {
            if let Err(resp) = apply_odrl_lane(&mut store, &req, mode) {
                return resp;
            }
        }
        (store, fired)
    };
    // Deciding is CPU-light here (the materialise already ran), but keep the store owned in the
    // closure so the borrowed session must be rebuilt inside — do it inline (no spawn needed).
    let decision = store.decide(&req.session(), &resource, mode);
    // [FABLE-5] sq-3mu76 — read-scoped advertisement (module doc): when the lane fired, mask
    // the advisory `grantedModes` to the requested (read) mode — the lane evidenced no other.
    #[cfg(feature = "odrl-authz")]
    let decision = {
        let mut decision = decision;
        if odrl_lane_fired {
            mask_decision_to_mode(&mut decision, mode);
        }
        decision
    };
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

// ── [SONNET-4.6] sq-pfae.17 — stateless trust-graph extension ──────────────
//
// The `trust_authz_decide` function and all its helpers are compiled ONLY behind
// the `solid-authz-trust` feature (which implies `solid-authz` + `sparq-trust`
// with the `cert-graph` feature). Feature OFF => zero compiled code, byte-identical
// to `solid-authz`.
//
// FAIL-CLOSED INVARIANT: every parse/verify/dispatch failure returns a 403 DENY.
// The trust path can only NARROW (never broaden) the WAC/ACP admission gate: the
// cert-graph closure produces `direct_rules ++ derived_rules` (design §3.4), and
// the unchanged `sparq_trust::admit` gate vetoes each rule independently.
//
// POSITIONAL format args used throughout (CodeQL rust/unused-variable guard).

/// [SONNET-4.6] (sq-pfae.17) A minimal admission justification returned alongside the trust-path
/// decision: which rule sourced the deciding admit (its IRI), how many facts were admitted,
/// and whether the decision went through the cert-graph closure.
#[cfg(feature = "solid-authz-trust")]
#[cfg_attr(docsrs, doc(cfg(feature = "solid-authz-trust")))]
#[derive(Debug)]
struct TrustJustification {
    /// IRI of the `trust:source` of the first admitted fact (the deciding rule).
    admitted_source: String,
    /// Count of distinct admitted facts.
    admitted_count: usize,
    /// Whether the cert-graph closure produced at least one derived rule (beyond the direct rules).
    cert_graph_derived: bool,
}

/// [SONNET-4.6] (sq-pfae.17) Serialise a `TrustJustification` into the JSON fragment added to
/// the decide response body under the `"trust"` key. POSITIONAL format args (CodeQL guard).
#[cfg(feature = "solid-authz-trust")]
fn justification_to_json(j: &TrustJustification) -> serde_json::Value {
    serde_json::json!({
        "admittedSource": j.admitted_source,
        "admittedCount": j.admitted_count,
        "certGraphDerived": j.cert_graph_derived,
    })
}

/// [SONNET-4.6] (sq-pfae.17) A parsed trust policy rule from the JSON wire format.
///
/// Wire schema (one element of `"trust"."rules"` array):
/// ```json
/// {
///   "source": "https://issuer.example/",
///   "issuerKeyHex": "<64-byte-hex compressed public key>",
///   "scopeIri": "https://pod.ex/resource",
///   "freshWithinSecs": 86400,
///   "shapePredicateIri": "https://schema.org/age"
/// }
/// ```
///
/// `shapePredicateIri` is the `trust:forPredicate` IRI — desugared into the `ShapeRef`
/// that `sparq_trust::policy::TrustRule` carries.
#[cfg(feature = "solid-authz-trust")]
struct WireTrustRule {
    source: String,
    issuer_key_hex: String,
    scope_iri: String,
    fresh_within_secs: i64,
    shape_predicate_iri: String,
}

/// [SONNET-4.6] (sq-pfae.17) A parsed certification edge from the JSON wire format.
///
/// Wire schema (one element of `"trust"."certifications"` array):
/// ```json
/// {
///   "certifierIri": "https://gov.example/framework",
///   "certifierKeyHex": "<hex>",
///   "certifiedIssuerIri": "https://issuer.example/dvs",
///   "certifiedKeyHex": "<hex>",
///   "validFromUnixSecs": 0,
///   "validUntilUnixSecs": 9999999999,
///   "signatureHex": "<hex>",
///   "scopeKind": "anyService"
/// }
/// ```
///
/// `scopeKind` is `"anyService"` (no statement-type narrowing) or absent/other (fail-closed
/// deny: unknown scope kinds are never silently widened).
#[cfg(feature = "solid-authz-trust")]
struct WireCertification {
    certifier_iri: String,
    certifier_key_hex: String,
    certified_issuer_iri: String,
    certified_key_hex: String,
    valid_from_unix_secs: i64,
    valid_until_unix_secs: i64,
    signature_hex: String,
    scope_kind_any_service: bool,
}

/// [SONNET-4.6] (sq-pfae.17) A parsed presented credential from the JSON wire format.
///
/// Wire schema (one element of `"trust"."credentials"` array):
/// ```json
/// {
///   "graphNquads": "<N-Quads string>",
///   "issuerSignatureHex": "<hex>",
///   "saltHex": "<64-hex-byte>",
///   "issuedAtUnixSecs": 1720000000,
///   "revoked": false
/// }
/// ```
#[cfg(feature = "solid-authz-trust")]
struct WireCredential {
    graph_nquads: String,
    issuer_signature_hex: String,
    salt_hex: String,
    issued_at_unix_secs: i64,
    revoked: bool,
}

/// [SONNET-4.6] (sq-pfae.17) Parse a `WireTrustRule` from a JSON object. Fail-closed: any
/// missing/mistyped field => `Err(reason)`.
#[cfg(feature = "solid-authz-trust")]
fn parse_wire_rule(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<WireTrustRule, &'static str> {
    let source = obj
        .get("source")
        .and_then(|v| v.as_str())
        .ok_or("trust rule missing 'source'")?
        .to_owned();
    let issuer_key_hex = obj
        .get("issuerKeyHex")
        .and_then(|v| v.as_str())
        .ok_or("trust rule missing 'issuerKeyHex'")?
        .to_owned();
    let scope_iri = obj
        .get("scopeIri")
        .and_then(|v| v.as_str())
        .ok_or("trust rule missing 'scopeIri'")?
        .to_owned();
    let fresh_within_secs = obj
        .get("freshWithinSecs")
        .and_then(|v| v.as_i64())
        .ok_or("trust rule missing/invalid 'freshWithinSecs'")?;
    let shape_predicate_iri = obj
        .get("shapePredicateIri")
        .and_then(|v| v.as_str())
        .ok_or("trust rule missing 'shapePredicateIri'")?
        .to_owned();
    Ok(WireTrustRule {
        source,
        issuer_key_hex,
        scope_iri,
        fresh_within_secs,
        shape_predicate_iri,
    })
}

/// [SONNET-4.6] (sq-pfae.17) Parse a `WireCertification` from a JSON object. Fail-closed.
#[cfg(feature = "solid-authz-trust")]
fn parse_wire_certification(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<WireCertification, &'static str> {
    let certifier_iri = obj
        .get("certifierIri")
        .and_then(|v| v.as_str())
        .ok_or("certification missing 'certifierIri'")?
        .to_owned();
    let certifier_key_hex = obj
        .get("certifierKeyHex")
        .and_then(|v| v.as_str())
        .ok_or("certification missing 'certifierKeyHex'")?
        .to_owned();
    let certified_issuer_iri = obj
        .get("certifiedIssuerIri")
        .and_then(|v| v.as_str())
        .ok_or("certification missing 'certifiedIssuerIri'")?
        .to_owned();
    let certified_key_hex = obj
        .get("certifiedKeyHex")
        .and_then(|v| v.as_str())
        .ok_or("certification missing 'certifiedKeyHex'")?
        .to_owned();
    let valid_from_unix_secs = obj
        .get("validFromUnixSecs")
        .and_then(|v| v.as_i64())
        .ok_or("certification missing/invalid 'validFromUnixSecs'")?;
    let valid_until_unix_secs = obj
        .get("validUntilUnixSecs")
        .and_then(|v| v.as_i64())
        .ok_or("certification missing/invalid 'validUntilUnixSecs'")?;
    let signature_hex = obj
        .get("signatureHex")
        .and_then(|v| v.as_str())
        .ok_or("certification missing 'signatureHex'")?
        .to_owned();
    // Fail-closed on scope kind: only "anyService" is accepted; an absent or unrecognised scope
    // kind denies (never silently widened to AnyService).
    let scope_kind = obj
        .get("scopeKind")
        .and_then(|v| v.as_str())
        .ok_or("certification missing 'scopeKind'")?;
    let scope_kind_any_service = match scope_kind {
        "anyService" => true,
        _ => return Err("certification 'scopeKind' must be 'anyService'"),
    };
    Ok(WireCertification {
        certifier_iri,
        certifier_key_hex,
        certified_issuer_iri,
        certified_key_hex,
        valid_from_unix_secs,
        valid_until_unix_secs,
        signature_hex,
        scope_kind_any_service,
    })
}

/// [SONNET-4.6] (sq-pfae.17) Parse a `WireCredential` from a JSON object. Fail-closed.
#[cfg(feature = "solid-authz-trust")]
fn parse_wire_credential(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<WireCredential, &'static str> {
    let graph_nquads = obj
        .get("graphNquads")
        .and_then(|v| v.as_str())
        .ok_or("credential missing 'graphNquads'")?
        .to_owned();
    let issuer_signature_hex = obj
        .get("issuerSignatureHex")
        .and_then(|v| v.as_str())
        .ok_or("credential missing 'issuerSignatureHex'")?
        .to_owned();
    let salt_hex = obj
        .get("saltHex")
        .and_then(|v| v.as_str())
        .ok_or("credential missing 'saltHex'")?
        .to_owned();
    let issued_at_unix_secs = obj
        .get("issuedAtUnixSecs")
        .and_then(|v| v.as_i64())
        .ok_or("credential missing/invalid 'issuedAtUnixSecs'")?;
    let revoked = obj
        .get("revoked")
        .and_then(|v| v.as_bool())
        .ok_or("credential missing/invalid 'revoked'")?;
    Ok(WireCredential {
        graph_nquads,
        issuer_signature_hex,
        salt_hex,
        issued_at_unix_secs,
        revoked,
    })
}

/// [SONNET-4.6] (sq-pfae.17) Decode a 64-hex-char (32-byte) salt from a hex string. Fail-closed.
#[cfg(feature = "solid-authz-trust")]
fn decode_salt_hex(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let hi = hex_nibble(chunk[0])?;
        let lo = hex_nibble(chunk[1])?;
        out[i] = (hi << 4) | lo;
    }
    Some(out)
}

/// [SONNET-4.6] (sq-pfae.17) Decode a single hex nibble. Fail-closed.
#[cfg(feature = "solid-authz-trust")]
fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// [SONNET-4.6] (sq-pfae.17) Build a `sparq_trust::policy::ShapeRef` for a `trust:forPredicate`
/// IRI. This is the same desugaring `sparq_trust::policy` does internally: a single-predicate
/// SHACL node-shape targeting subjects of `predicate_iri` and requiring it present with
/// `sh:minCount 1`. Fail-closed if the IRI is not valid.
#[cfg(feature = "solid-authz-trust")]
fn predicate_shape(predicate_iri: &str) -> Option<sparq_trust::policy::ShapeRef> {
    use oxrdf::{BlankNode, Literal, NamedNode, Term, Triple};
    let pred = NamedNode::new(predicate_iri).ok()?;
    let root = BlankNode::default();
    let prop = BlankNode::default();
    let sh_target_subjects_of =
        NamedNode::new("http://www.w3.org/ns/shacl#targetSubjectsOf").expect("static IRI is valid");
    let sh_property =
        NamedNode::new("http://www.w3.org/ns/shacl#property").expect("static IRI is valid");
    let sh_path = NamedNode::new("http://www.w3.org/ns/shacl#path").expect("static IRI is valid");
    let sh_mincount =
        NamedNode::new("http://www.w3.org/ns/shacl#minCount").expect("static IRI is valid");
    Some(sparq_trust::policy::ShapeRef {
        root: Term::BlankNode(root.clone()),
        triples: vec![
            Triple::new(root.clone(), sh_target_subjects_of, pred.clone()),
            Triple::new(root.clone(), sh_property, prop.clone()),
            Triple::new(prop.clone(), sh_path, pred),
            Triple::new(prop, sh_mincount, Literal::new_simple_literal("1")),
        ],
    })
}

/// [SONNET-4.6] (sq-pfae.17) The core trust-path decision handler. Called from `decide_endpoint`
/// ONLY when BOTH (a) `solid-authz-trust` is compiled AND (b) the config flag is set AND (c) the
/// request carries a `"trust"` block. Returns a `Response` — always a complete HTTP response
/// (either an enriched allow/deny carrying a `"trust"` admission justification, or a plain 403
/// deny for any parse/verify failure — FAIL-CLOSED).
///
/// # Wire format for the `"trust"` block
///
/// ```json
/// {
///   "trust": {
///     "agentIri": "https://alice.ex/card#me",
///     "nowUnixSecs": 1720000000,
///     "rules": [ { "source": ..., "issuerKeyHex": ..., "scopeIri": ...,
///                  "freshWithinSecs": ..., "shapePredicateIri": ... }, ... ],
///     "certifications": [ { ... }, ... ],
///     "credentials": [ { "graphNquads": ..., "issuerSignatureHex": ...,
///                        "saltHex": ..., "issuedAtUnixSecs": ..., "revoked": ... }, ... ]
///   }
/// }
/// ```
///
/// # Fail-closed invariant
///
/// ANY malformed / unverifiable / stale / revoked / over-depth / broadening trust input => 403 DENY.
/// The WAC/ACP path is the SOLE gate on the dataset; the trust block can inject additional
/// `trust:admitted` facts (via `sparq_trust::admit`) but it NEVER short-circuits or weakens the
/// WAC/ACP materialisation.
#[cfg(feature = "solid-authz-trust")]
#[allow(clippy::result_large_err)]
fn trust_authz_decide(
    trust_val: &serde_json::Value,
    req: &AuthzRequest,
    resource: &str,
    mode: sparq_solid::Mode,
    full_body: &serde_json::Value,
) -> Response {
    use oxrdf::NamedNode;
    use sparq_trust::graph::{CertScope, Certification};
    use sparq_trust::{
        admit, derive_effective_rules, policy::TrustRule, AdmittedFact, PresentedCredential,
        Session as TrustSession,
    };
    use sparq_zk::sig::public_key_from_hex;

    // ── 1. Parse the trust block object ──────────────────────────────────────
    let trust_obj = match trust_val.as_object() {
        Some(o) => o,
        None => {
            return json_response(
                StatusCode::FORBIDDEN,
                trust_deny_json("trust block must be a JSON object"),
            );
        }
    };

    // ── 2. Parse the trust session (agent IRI + now) ──────────────────────────
    let agent_iri_str = match trust_obj.get("agentIri").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            return json_response(
                StatusCode::FORBIDDEN,
                trust_deny_json("trust block missing 'agentIri'"),
            );
        }
    };
    let agent_iri = match NamedNode::new(agent_iri_str) {
        Ok(n) => n,
        Err(_) => {
            return json_response(
                StatusCode::FORBIDDEN,
                trust_deny_json("trust 'agentIri' is not a valid IRI"),
            );
        }
    };
    let now_unix_secs = match trust_obj.get("nowUnixSecs").and_then(|v| v.as_i64()) {
        Some(n) => n,
        None => {
            return json_response(
                StatusCode::FORBIDDEN,
                trust_deny_json("trust block missing/invalid 'nowUnixSecs'"),
            );
        }
    };
    let trust_session = TrustSession {
        agent: agent_iri.clone(),
        now_unix_secs,
    };

    // ── 3. Parse direct trust rules ─────────────────────────────────────────
    let rules_val = match trust_obj.get("rules").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => {
            return json_response(
                StatusCode::FORBIDDEN,
                trust_deny_json("trust block missing 'rules' array"),
            );
        }
    };
    let mut direct_rules: Vec<TrustRule> = Vec::new();
    for (idx, rule_v) in rules_val.iter().enumerate() {
        let obj = match rule_v.as_object() {
            Some(o) => o,
            None => {
                // [SONNET-4.6] POSITIONAL format args (CodeQL guard).
                return json_response(
                    StatusCode::FORBIDDEN,
                    trust_deny_json(&format!("trust rule {} is not an object", idx)),
                );
            }
        };
        let wire_rule = match parse_wire_rule(obj) {
            Ok(r) => r,
            Err(reason) => {
                return json_response(StatusCode::FORBIDDEN, trust_deny_json(reason));
            }
        };
        let source = match NamedNode::new(&wire_rule.source) {
            Ok(n) => n,
            Err(_) => {
                return json_response(
                    StatusCode::FORBIDDEN,
                    trust_deny_json("trust rule 'source' is not a valid IRI"),
                );
            }
        };
        let scope = match NamedNode::new(&wire_rule.scope_iri) {
            Ok(n) => n,
            Err(_) => {
                return json_response(
                    StatusCode::FORBIDDEN,
                    trust_deny_json("trust rule 'scopeIri' is not a valid IRI"),
                );
            }
        };
        let issuer_key = match public_key_from_hex(&wire_rule.issuer_key_hex) {
            Some(k) => k,
            None => {
                return json_response(
                    StatusCode::FORBIDDEN,
                    trust_deny_json("trust rule 'issuerKeyHex' is not a valid public key"),
                );
            }
        };
        let shape = match predicate_shape(&wire_rule.shape_predicate_iri) {
            Some(s) => s,
            None => {
                return json_response(
                    StatusCode::FORBIDDEN,
                    trust_deny_json("trust rule 'shapePredicateIri' is not a valid IRI"),
                );
            }
        };
        direct_rules.push(TrustRule {
            source,
            issuer_key,
            shape,
            scope,
            fresh_within_secs: wire_rule.fresh_within_secs,
        });
    }

    // ── 4. Parse certifications ──────────────────────────────────────────────
    let certs_val = trust_obj
        .get("certifications")
        .and_then(|v| v.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[]);
    let mut certifications: Vec<Certification> = Vec::new();
    for (idx, cert_v) in certs_val.iter().enumerate() {
        let obj = match cert_v.as_object() {
            Some(o) => o,
            None => {
                return json_response(
                    StatusCode::FORBIDDEN,
                    trust_deny_json(&format!("certification {} is not an object", idx)),
                );
            }
        };
        let wire = match parse_wire_certification(obj) {
            Ok(c) => c,
            Err(reason) => {
                return json_response(StatusCode::FORBIDDEN, trust_deny_json(reason));
            }
        };
        let certifier_iri = match NamedNode::new(&wire.certifier_iri) {
            Ok(n) => n,
            Err(_) => {
                return json_response(
                    StatusCode::FORBIDDEN,
                    trust_deny_json("certification 'certifierIri' is not a valid IRI"),
                );
            }
        };
        let certified_issuer_iri = match NamedNode::new(&wire.certified_issuer_iri) {
            Ok(n) => n,
            Err(_) => {
                return json_response(
                    StatusCode::FORBIDDEN,
                    trust_deny_json("certification 'certifiedIssuerIri' is not a valid IRI"),
                );
            }
        };
        let certifier_key = match public_key_from_hex(&wire.certifier_key_hex) {
            Some(k) => k,
            None => {
                return json_response(
                    StatusCode::FORBIDDEN,
                    trust_deny_json("certification 'certifierKeyHex' is not a valid public key"),
                );
            }
        };
        let certified_key = match public_key_from_hex(&wire.certified_key_hex) {
            Some(k) => k,
            None => {
                return json_response(
                    StatusCode::FORBIDDEN,
                    trust_deny_json("certification 'certifiedKeyHex' is not a valid public key"),
                );
            }
        };
        let scope = if wire.scope_kind_any_service {
            CertScope::AnyService
        } else {
            // Fail-closed: only AnyService is currently supported; non-AnyService scopes
            // deny until the shape-scope wire format is specified (sq-pfae.17 follow-up).
            return json_response(
                StatusCode::FORBIDDEN,
                trust_deny_json("certification scopeKind not supported"),
            );
        };
        certifications.push(Certification {
            certifier: certifier_iri,
            certifier_key,
            certified_issuer: certified_issuer_iri,
            certified_key,
            scope,
            valid_from_unix_secs: wire.valid_from_unix_secs,
            valid_until_unix_secs: wire.valid_until_unix_secs,
            signature_hex: wire.signature_hex,
        });
    }

    // ── 5. Run the cert-graph closure ────────────────────────────────────────
    // Depth-1 closure: derived rules are NOT re-used as anchors (sq-tu4e discipline).
    let pre_count = direct_rules.len();
    let effective_rules = derive_effective_rules(&direct_rules, &certifications, now_unix_secs, 1);
    let cert_graph_derived = effective_rules.len() > pre_count;

    // ── 6. Parse presented credentials ──────────────────────────────────────
    let creds_val = match trust_obj.get("credentials").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => {
            return json_response(
                StatusCode::FORBIDDEN,
                trust_deny_json("trust block missing 'credentials' array"),
            );
        }
    };
    let resource_node = match NamedNode::new(resource) {
        Ok(n) => n,
        Err(_) => {
            return json_response(
                StatusCode::FORBIDDEN,
                trust_deny_json("'resource' is not a valid IRI"),
            );
        }
    };

    let mut all_admitted: Vec<AdmittedFact> = Vec::new();
    for (idx, cred_v) in creds_val.iter().enumerate() {
        let obj = match cred_v.as_object() {
            Some(o) => o,
            None => {
                return json_response(
                    StatusCode::FORBIDDEN,
                    trust_deny_json(&format!("credential {} is not an object", idx)),
                );
            }
        };
        let wire = match parse_wire_credential(obj) {
            Ok(c) => c,
            Err(reason) => {
                return json_response(StatusCode::FORBIDDEN, trust_deny_json(reason));
            }
        };
        // Decode the 32-byte salt from hex.
        let salt = match decode_salt_hex(&wire.salt_hex) {
            Some(s) => s,
            None => {
                return json_response(
                    StatusCode::FORBIDDEN,
                    trust_deny_json("credential 'saltHex' must be 64 hex chars (32 bytes)"),
                );
            }
        };
        // Parse the credential graph from N-Quads. Each quad's triple is extracted regardless of
        // its named graph (the default graph is the canonical credential graph; named-graph quads
        // are treated as additional context — fail-closed means unknown predicates admit nothing).
        let mut triples: Vec<oxrdf::Triple> = Vec::new();
        for result in oxttl::NQuadsParser::new().for_slice(wire.graph_nquads.as_bytes()) {
            match result {
                Ok(quad) => {
                    // Use the subject/predicate/object from the quad, regardless of graph name.
                    let t = oxrdf::Triple::new(quad.subject, quad.predicate, quad.object);
                    triples.push(t);
                }
                Err(_) => {
                    return json_response(
                        StatusCode::FORBIDDEN,
                        trust_deny_json("credential 'graphNquads' is not valid N-Quads"),
                    );
                }
            }
        }
        if triples.is_empty() {
            // An empty credential graph admits nothing; skip it rather than sending to the gate.
            continue;
        }
        let cred = PresentedCredential {
            graph: triples,
            issuer_signature_hex: wire.issuer_signature_hex,
            salt,
            issued_at_unix_secs: wire.issued_at_unix_secs,
            revoked: wire.revoked,
        };
        // Run the admission gate (fail-closed: empty result is fine — that credential just admits
        // nothing, only a complete trust-block failure => 403).
        let admitted = admit(&cred, &effective_rules, &trust_session, &resource_node);
        all_admitted.extend(admitted);
    }

    // ── 7. Build the WAC/ACP store and decide ───────────────────────────────
    // The trust block extends, never replaces, the WAC/ACP admission stratum: admitted facts
    // are injected INTO the pod dataset for materialisation, exactly as the design specifies.
    // Build the store over the original dataset with admitted facts injected.
    let store = match build_store_with_admitted(req, &all_admitted, resource, full_body) {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let decision = store.decide(&req.session(), resource, mode);
    let status = if decision.allow {
        StatusCode::OK
    } else {
        deny_status_code(decision.status)
    };

    // ── 8. Emit the response with a minimal admission justification ──────────
    let justification = if !all_admitted.is_empty() {
        let first_source = all_admitted[0].issuer.as_str().to_owned();
        Some(TrustJustification {
            admitted_source: first_source,
            admitted_count: all_admitted.len(),
            cert_graph_derived,
        })
    } else {
        None
    };
    let mut body_obj = match serde_json::from_str::<serde_json::Value>(&decision_to_json(&decision))
    {
        Ok(v) => v,
        Err(_) => {
            // decision_to_json is infallible for valid WacDecision; this branch is unreachable.
            return json_response(
                StatusCode::FORBIDDEN,
                trust_deny_json("internal serialisation error"),
            );
        }
    };
    if let Some(j) = justification {
        if let serde_json::Value::Object(ref mut map) = body_obj {
            map.insert("trustJustification".to_owned(), justification_to_json(&j));
        }
    }
    let acl_link = decision.acl_link_header();
    json_response_with_link(status, body_obj.to_string(), acl_link.as_deref())
}

/// [SONNET-4.6] (sq-pfae.17) Build a `PodStore` with admitted trust facts injected into the
/// dataset. Admitted facts are emitted into TWO positions so they reach the WAC/ACP materialiser:
///
/// 1. **Named-graph position** (`resource + ".acl"` graph) — the WAC loader ONLY reads named
///    graphs ending in `.acl`/`.acr`; injecting into that graph makes the facts visible to the
///    N3 reasoner. The convention `resource + ".acl"` matches Solid's ACL-by-naming-convention:
///    every resource `R` is governed by `R.acl` (or its nearest ancestor), and the loader emits
///    `<R> solidx:ownAcl <R.acl>` from a present `<R.acl>` graph.
///
/// 2. **Default-graph position** (no graph IRI) — belt-and-braces; the engine SPARQL query path
///    (`query_as` / `view_for`) sees the default graph alongside named graphs so trust-injected
///    facts are also queryable as plain RDF terms. Default-graph quads do NOT reach the WAC N3
///    reasoner (the loader skips them), so position 1 is the operative path for WAC decisions.
///
/// [SONNET-4.6] FIX (Fix 2, sq-zza2h): oxrdf's Display for `NamedOrBlankNode` (subject) and
/// `NamedNode` (predicate) already emits the `<…>` / `_:…` wrapper, and `Term` (object) emits
/// the correct N-Quads term. Do NOT re-wrap in `<{}>` — that would double-wrap a `NamedNode`
/// subject as `<<…>>`, producing invalid N-Quads that the parser rejects, so admitted facts
/// silently never reached the store before this fix. Emit `{} {} {} .` / `{} {} {} <graph> .`
/// and let oxrdf Display produce the canonical N-Quads term syntax.
///
/// HONEST SCOPE: this is a PoC injection path (anchored-not-proven; sq-qhy4 pending external
/// audit). A production wiring would use `trust:admitted`-named-graph tagging and epoch-based
/// cache invalidation (design `research/solid-trust-graph-authz-design.md`).
#[cfg(feature = "solid-authz-trust")]
#[allow(clippy::result_large_err)]
fn build_store_with_admitted(
    req: &AuthzRequest,
    admitted: &[sparq_trust::AdmittedFact],
    resource: &str,
    _full_body: &serde_json::Value,
) -> Result<sparq_solid::PodStore, Response> {
    use std::fmt::Write as _;
    // The ACL graph IRI for this resource (Solid naming convention: resource + ".acl").
    // Admitted facts emitted into this named graph are seen by the WAC N3 loader
    // (assemble_input processes all graphs whose name ends in ".acl").
    // [SONNET-4.6] POSITIONAL format args throughout (CodeQL rust/unused-variable guard).
    let acl_graph = format!("{}.acl", resource);
    let mut augmented = req.dataset.clone();
    for fact in admitted {
        // Named-graph position: reaches the WAC N3 reasoner via the .acl graph path.
        let _ = writeln!(
            &mut augmented,
            "{} {} {} <{}> .",
            fact.triple.subject, fact.triple.predicate, fact.triple.object, acl_graph,
        );
        // Default-graph position: reachable via SPARQL query on the full dataset.
        let _ = writeln!(
            &mut augmented,
            "{} {} {} .",
            fact.triple.subject, fact.triple.predicate, fact.triple.object,
        );
    }
    // Delegate to the unchanged build_store path (WAC/ACP inference + materialisation).
    let augmented_req = AuthzRequest {
        dataset: augmented,
        agent: req.agent.clone(),
        client: req.client.clone(),
        issuer: req.issuer.clone(),
        now: req.now.clone(),
        view: req.view,
    };
    build_store(&augmented_req)
}

/// [SONNET-4.6] (sq-pfae.17) Serialise a 403 trust-deny JSON body. The reason string is NEVER
/// echoed to the client verbatim from adversarial input (the caller always passes a static string
/// literal or a format!() from controlled values). POSITIONAL format args (CodeQL guard).
#[cfg(feature = "solid-authz-trust")]
fn trust_deny_json(reason: &str) -> String {
    serde_json::json!({
        "allow": false,
        "grantedModes": [],
        "governingAcl": null,
        "scope": null,
        "status": "resolved",
        "aclLink": null,
        "trustDenied": true,
        "trustDenyReason": reason,
    })
    .to_string()
}

/// `POST /authz/wac-allow` — the `WAC-Allow` header VALUE for a `(session, resource)` (FR-2/FR-4).
/// Body: `{ "dataset": nquads, "session": {..}, "resource": iri, "view"?: "wac"|"acp" }`. Returns
/// `{ "wacAllow": "user=\"…\",public=\"…\"" }` — the RFC-style permission advertisement the server
/// puts in a `WAC-Allow` response header.
///
/// Fail-closed: an unparseable dataset / materialisation failure is an error (never a wider
/// advertisement); a grant-less session yields `user="",public=""`.
///
/// [FABLE-5] sq-3mu76: behind the `odrl-authz` feature, a dataset carrying ODRL policy rules
/// runs the [`apply_odrl_lane`] materialisation (as `Mode::Read` — the endpoint has no mode
/// field and the lane evidences only the read action) before advertising, and the value is
/// scoped by [`read_scoped_wac_allow`]: a read prohibition is never advertised past, `user`
/// carries at most `read`, `public` is always empty (module doc). Same refusal matrix
/// (anonymous session => 403 etc.).
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
            return json_error(
                StatusCode::BAD_REQUEST,
                "'resource' (IRI string) is required",
            );
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
    // [FABLE-5] sq-3mu76 — the OPT-IN ODRL lane on the advertisement endpoint: a
    // dataset-carried read prohibition must not be advertised past (`user="read"` from a static
    // WAC grant that `/authz/query` would refuse to honour). The endpoint has no mode field —
    // the lane runs for `Mode::Read`, the ONLY action it evidences (sq-lrtc3.2 owns the rest),
    // and the advertisement is scoped down to that evidence below. Same refusal matrix
    // (anonymous => 403 etc.). Feature OFF => this block does not exist.
    #[cfg(feature = "odrl-authz")]
    let (store, odrl_lane_fired) = {
        let mut store = store;
        let fired = dataset_carries_odrl(&req.dataset);
        if fired {
            if let Err(resp) = apply_odrl_lane(&mut store, &req, Mode::Read) {
                return resp;
            }
        }
        (store, fired)
    };
    // [SONNET-4.6] sq-snopa.7 FR-5: resolve the governing ACL for the Link header BEFORE
    // `wac_allow` consumes the store. `resolve_acl` is a pure index lookup (no mode needed) and
    // returns `None` when no ACL was discovered — fail-closed: no header emitted in that case.
    // [SONNET-4.6] POSITIONAL format args (CodeQL rust/unused-variable guard).
    let acl_link: Option<String> = store
        .resolve_acl(&resource_iri)
        .map(|eff| format!("<{}>; rel=\"acl\"", eff.acl.as_str()));
    let value = store.wac_allow(&req.session(), &resource);
    // [FABLE-5] sq-3mu76 — read-scoped advertisement (module doc): the lane materialised
    // read-action outcomes only, so advertise at most `user="read"` and never a public mode
    // (an anonymous session is refused wherever the lane fires). Fail-closed: this can only
    // NARROW the advertisement, never widen it.
    #[cfg(feature = "odrl-authz")]
    let value = if odrl_lane_fired {
        read_scoped_wac_allow(&value)
    } else {
        value
    };
    let body = serde_json::json!({ "wacAllow": value }).to_string();
    json_response_with_link(StatusCode::OK, body, acl_link.as_deref())
}

// ── [FABLE-5] sq-lrtc3.1 + sq-3mu76 — the OPT-IN ODRL lane on `/authz/*` ───
//
// Everything below is compiled ONLY behind the `odrl-authz` feature (which implies
// `solid-authz` + `sparq-solid/odrl-bridge` + `sparq-policy`). Feature OFF => zero
// compiled code, byte-identical to `solid-authz`.
//
// FAIL-CLOSED INVARIANT: every refusal path returns a 4xx and materialises nothing —
// NEVER a silent allow. In particular a prohibition must never be silently DROPPED
// (dropping a deny widens access), so any input the lane cannot faithfully enforce
// (a targetless rule, a non-read mode, an anonymous session, an unimplementable
// `odrl:conflict` strategy, an unparseable policy) refuses the whole request.
//
// The lane deliberately uses the ONE-SHOT bridge entry point
// (`PodStore::materialize_odrl_policy`) rather than the persisted-conditional
// variants: the stateless per-request store dies with the response, so
// frozen-at-materialization and re-checked-per-session semantics coincide here —
// and the one-shot path evaluates `odrl:assignee` against the request party
// (the conditional path drops a bare-assignee restriction to an `auth:Public`
// head — a widening hazard tracked as its own sparq-solid bug bead, unsuitable
// for an enforcement surface).

/// Does the request dataset carry ODRL policy rules? A cheap substring scan for the
/// `odrl:permission` / `odrl:prohibition` predicate IRIs in N-Quads term brackets —
/// the same suffix-scan convenience shape as [`infer_view_is_acp`]. A false positive
/// (the IRI inside a literal) parses to a rule-less policy, which materialises
/// nothing — the lane then behaves exactly as if it had not fired (fail-closed
/// direction). Pure + directly unit-tested. [FABLE-5] sq-lrtc3.1.
#[cfg(feature = "odrl-authz")]
pub(crate) fn dataset_carries_odrl(dataset: &str) -> bool {
    dataset.contains("http://www.w3.org/ns/odrl/2/permission>")
        || dataset.contains("http://www.w3.org/ns/odrl/2/prohibition>")
}

/// The ODRL action IRI the query lane exercises for a WAC/ACP [`Mode`]: a SPARQL
/// query is an `odrl:read` exercise, so ONLY [`Mode::Read`] maps; every other mode is
/// `None` and the lane refuses (fail-closed — the full query-action contract,
/// including the deliberate non-mapping of `odrl:use`, is sq-lrtc3.2). Pure +
/// directly unit-tested. [FABLE-5] sq-lrtc3.1.
#[cfg(feature = "odrl-authz")]
pub(crate) fn odrl_action_for_mode(mode: Mode) -> Option<&'static str> {
    match mode {
        Mode::Read => Some("http://www.w3.org/ns/odrl/2/read"),
        Mode::Write | Mode::Append | Mode::Control => None,
    }
}

/// Mask an advisory decision's `granted_modes` to the SINGLE mode the ODRL lane evidenced
/// (the requested read mode) — the read-scoped-advertisement rule (module doc, sq-3mu76).
/// The `allow` bit is untouched: it concerns exactly the requested mode, which is never
/// masked out. Fail-closed direction only (a mode is removed from the advertisement, never
/// added). Pure + directly unit-tested. [FABLE-5] sq-3mu76.
#[cfg(feature = "odrl-authz")]
pub(crate) fn mask_decision_to_mode(decision: &mut WacDecision, mode: Mode) {
    decision.granted_modes.retain(|m| *m == mode);
}

/// Scope a [`sparq_solid::PodStore::wac_allow`] advertisement value down to what the ODRL
/// lane evidenced (the read-scoped-advertisement rule, module doc, sq-3mu76): `user` keeps at
/// most `read`, `public` is always empty (an anonymous session is REFUSED wherever the lane
/// fires, so no public mode is ever evidenced under an ODRL policy). The input is the
/// crate-controlled fixed format `user="<modes>",public="<modes>"` where `<modes>` is a
/// space-separated subset of the four mode names — `read` can only occur as the read-mode
/// token, so the substring test is exact. Fail-closed: the output advertises a subset of the
/// input's `user` field and nothing public. Pure + directly unit-tested. [FABLE-5] sq-3mu76.
#[cfg(feature = "odrl-authz")]
pub(crate) fn read_scoped_wac_allow(value: &str) -> String {
    let user_holds_read = value
        .split_once(",public=")
        .map(|(user, _)| user.contains("read"))
        .unwrap_or(false);
    let user = if user_holds_read { "read" } else { "" };
    format!(r#"user="{}",public="""#, user)
}

/// Union-project the request dataset's quads to TRIPLES so ODRL policies are parsed
/// wherever they live (the default graph OR a named graph). `sparq_policy::parse_policy`
/// queries bare triple patterns — the DEFAULT graph only — so a policy expressed inside
/// a named graph would otherwise be INVISIBLE, silently dropping its prohibitions
/// (fail-OPEN); the union projection makes it visible. Blank-node labels are
/// document-scoped in N-Quads, so the projection preserves node identity across graph
/// boundaries. Directly unit-tested. [FABLE-5] sq-lrtc3.1.
#[cfg(feature = "odrl-authz")]
fn union_policy_graph(dataset: &str) -> Result<sparq_core::Graph, String> {
    use std::fmt::Write as _;
    let mut nt = String::new();
    for quad in oxttl::NQuadsParser::new().for_slice(dataset.as_bytes()) {
        let quad = quad.map_err(|e| e.to_string())?;
        // oxrdf Display emits canonical N-Triples term syntax (no re-wrapping — the
        // sq-zza2h double-wrap lesson); the graph name is deliberately dropped.
        let _ = writeln!(
            &mut nt,
            "{} {} {} .",
            quad.subject, quad.predicate, quad.object
        );
    }
    sparq_core::Graph::load_str(&nt, "ntriples")
}

/// Run the ODRL lane over an already WAC/ACP-materialised `store`: parse the dataset's
/// ODRL policy and materialise its bridged grants/denies (both sides, deny-overrides)
/// into the auth view for the request's `(party, action, target)` per rule target.
/// Returns the fail-closed 4xx [`Response`] on any input the lane cannot faithfully
/// enforce (see the section comment above); `Ok(())` means every materialisation
/// outcome is now in the auth view and `query_json_as` / `decide` / `wac_allow` (all
/// three consume the same `∪ allow ∖ ∪ deny` index) can honour it. [FABLE-5]
/// sq-lrtc3.1 + sq-3mu76.
///
/// What the stateless lane can and cannot evidence (honest scope): the request context
/// carries the party (session agent), the delivery recipient (the same agent — the
/// query results go to the requester), and `odrl:dateTime` (the session `now`, when
/// supplied). A constraint dimension with no evidence (`odrl:purpose`, `odrl:count` —
/// stateful budgets are sq-snopa.8 territory) or an undischarged duty never grants
/// (the evaluator is fail-closed on missing evidence); it is never silently widened.
// `Response` is large; same `result_large_err` allow as the crate's other handler helpers.
#[cfg(feature = "odrl-authz")]
#[allow(clippy::result_large_err)]
fn apply_odrl_lane(store: &mut PodStore, req: &AuthzRequest, mode: Mode) -> Result<(), Response> {
    // 1. The query-action contract: only `read` maps (sq-lrtc3.2). Refuse any other
    //    mode rather than guess an ODRL action for it (fail-closed).
    let Some(action) = odrl_action_for_mode(mode) else {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "the dataset carries ODRL policies; the ODRL lane covers the read/query \
             action only (fail-closed)",
        ));
    };
    // 2. A concrete party: bridged grants AND denies are principal-scoped triples, so
    //    an anonymous session would leave an applicable prohibition unenforceable
    //    against any static public grant (fail-OPEN). Refuse instead.
    let Some(agent) = req.agent.as_deref() else {
        return Err(json_error(
            StatusCode::FORBIDDEN,
            "the dataset carries ODRL policies; the ODRL lane requires an \
             authenticated agent (fail-closed)",
        ));
    };
    // 3. Parse the policy from the UNION of the dataset's graphs. The parse detail is
    //    withheld from the client (info-leak posture), as everywhere on this surface.
    let policy_graph = union_policy_graph(&req.dataset)
        .map_err(|_| json_error(StatusCode::BAD_REQUEST, "'dataset' is not valid N-Quads"))?;
    let policy = sparq_policy::parse_policy(&policy_graph).map_err(|_| {
        json_error(
            StatusCode::BAD_REQUEST,
            "the dataset's ODRL policy did not parse (fail-closed)",
        )
    })?;
    // 4. Refuse an unimplementable `odrl:conflict` strategy UP FRONT: under e.g.
    //    `odrl:perm` NONE of the policy's rules can be trusted into deny-overrides
    //    enforcement, and a rule-less short-circuit below must not skip the check.
    if sparq_policy::conflict_admissibility(&policy).is_err() {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "the ODRL policy declares a conflict strategy this engine cannot honour \
             (fail-closed)",
        ));
    }
    // 5. Graph-granularity targets only: a rule with NO concrete target graph IRI
    //    applies to *any* target, which this lane cannot faithfully enumerate — and
    //    under-materialising a targetless PROHIBITION would widen access. Refuse
    //    (pattern/sub-graph targets are the sq-lrtc3.3 design track).
    let mut targets = std::collections::BTreeSet::new();
    for rule in policy.permissions.iter().chain(policy.prohibitions.iter()) {
        match rule.target.as_deref() {
            Some(t) => {
                targets.insert(t.to_owned());
            }
            None => {
                return Err(json_error(
                    StatusCode::BAD_REQUEST,
                    "an ODRL rule without a concrete target graph IRI is not supported \
                     on this lane (fail-closed; pattern targets are a tracked design \
                     follow-up)",
                ));
            }
        }
    }
    // 6. Materialise BOTH sides of the policy per target. Each call is independently
    //    fail-closed (a non-matching / unevidenced rule materialises nothing) and
    //    idempotent; the deny side wins at enforcement (∪ allow ∖ ∪ deny).
    for target in &targets {
        let mut request = sparq_policy::Request::new(action)
            .on(target.clone())
            .by(agent)
            // The requesting agent IS the delivery recipient of the query results.
            .with(
                sparq_policy::ODRL_RECIPIENT,
                sparq_policy::Value::Iri(agent.to_owned()),
            );
        if let Some(now) = req.now.as_deref() {
            request = request.at(now);
        }
        let outcome = store.materialize_odrl_policy(&policy, &request);
        if outcome.refused {
            // Defence-in-depth: step 4 already refused inadmissible strategies.
            return Err(json_error(
                StatusCode::BAD_REQUEST,
                "the ODRL policy was refused by the bridge (fail-closed)",
            ));
        }
    }
    Ok(())
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
            return json_error(
                StatusCode::BAD_REQUEST,
                "'query' (SPARQL string) is required",
            );
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
    // [FABLE-5] sq-lrtc3.1 — the OPT-IN ODRL lane: when the dataset carries ODRL policy
    // rules, materialise their bridged grants/denies into the auth view BEFORE the
    // access-controlled query runs. Feature OFF => this block does not exist and the
    // endpoint is byte-identical to plain `solid-authz`.
    #[cfg(feature = "odrl-authz")]
    let store = {
        let mut store = store;
        if dataset_carries_odrl(&req.dataset) {
            if let Err(resp) = apply_odrl_lane(&mut store, &req, mode) {
                return resp;
            }
        }
        store
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

    /// [FABLE-5] sq-3mu76 — the read-scoped-advertisement mask keeps ONLY the requested mode
    /// and never touches the `allow` bit.
    #[cfg(feature = "odrl-authz")]
    #[test]
    fn mask_decision_to_mode_keeps_only_the_requested_mode() {
        let mut d = WacDecision {
            allow: true,
            granted_modes: vec![Mode::Read, Mode::Write, Mode::Control],
            governing_acl: None,
            scope: None,
            status: AclStatus::Resolved,
        };
        mask_decision_to_mode(&mut d, Mode::Read);
        assert_eq!(d.granted_modes, vec![Mode::Read]);
        assert!(
            d.allow,
            "the allow bit is untouched by the advertisement mask"
        );
        // A mode the session does not hold is not conjured up (fail-closed direction only).
        let mut none = fail_closed_decision();
        mask_decision_to_mode(&mut none, Mode::Read);
        assert!(none.granted_modes.is_empty());
    }

    /// [FABLE-5] sq-3mu76 — the wac-allow value is scoped to at most `user="read"`,
    /// `public` always empty, and a `read` in the PUBLIC field never leaks into `user`.
    #[cfg(feature = "odrl-authz")]
    #[test]
    fn read_scoped_wac_allow_scopes_user_and_blanks_public() {
        assert_eq!(
            read_scoped_wac_allow(r#"user="read write append control",public="read""#),
            r#"user="read",public="""#
        );
        // No read held by the user: nothing advertised — the public `read` must NOT leak.
        assert_eq!(
            read_scoped_wac_allow(r#"user="write",public="read""#),
            r#"user="",public="""#
        );
        assert_eq!(
            read_scoped_wac_allow(r#"user="",public="""#),
            r#"user="",public="""#
        );
        // A value not in the crate-controlled format advertises NOTHING (fail-closed).
        assert_eq!(read_scoped_wac_allow("garbage"), r#"user="",public="""#);
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

// ── [FABLE-5] sq-lrtc3.1 — unit tests for the ODRL lane helpers ─────────────
// One DIRECT test per new fn (coverage ratchet), plus the lane's fail-closed
// refusal matrix and a positive/negative enforcement round-trip through the REAL
// store. Gated on the `odrl-authz` feature (zero code otherwise).
#[cfg(all(test, feature = "odrl-authz"))]
mod odrl_tests {
    use super::*;

    const ALICE: &str = "https://alice.ex/card#me";
    const BOB: &str = "https://bob.ex/card#me";
    const N1: &str = "https://pod.ex/notes/n1";

    /// One content graph, no static ACL — the only possible grants are bridged ones.
    const CONTENT: &str = "<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> \"hello\" <https://pod.ex/notes/n1> .\n";

    /// An ODRL permit for `agent` to read n1, as default-graph N-Quads.
    fn permit_nq(agent: &str) -> String {
        format!(
            "<urn:pol/p> <http://www.w3.org/ns/odrl/2/permission> _:perm .\n\
             _:perm <http://www.w3.org/ns/odrl/2/action> <http://www.w3.org/ns/odrl/2/read> .\n\
             _:perm <http://www.w3.org/ns/odrl/2/target> <{N1}> .\n\
             _:perm <http://www.w3.org/ns/odrl/2/assignee> <{agent}> .\n"
        )
    }

    /// An ODRL prohibition carving `agent` out of reading n1, as default-graph N-Quads.
    fn prohibit_nq(agent: &str) -> String {
        format!(
            "<urn:pol/p> <http://www.w3.org/ns/odrl/2/prohibition> _:proh .\n\
             _:proh <http://www.w3.org/ns/odrl/2/action> <http://www.w3.org/ns/odrl/2/read> .\n\
             _:proh <http://www.w3.org/ns/odrl/2/target> <{N1}> .\n\
             _:proh <http://www.w3.org/ns/odrl/2/assignee> <{agent}> .\n"
        )
    }

    fn authz_req(dataset: String, agent: Option<&str>) -> AuthzRequest {
        AuthzRequest {
            dataset,
            agent: agent.map(str::to_owned),
            client: None,
            issuer: None,
            now: None,
            view: Some(AuthView::Wac),
        }
    }

    /// Build the store exactly as the endpoint does, then run the lane.
    // Same `result_large_err` allow as the handler helpers under test.
    #[allow(clippy::result_large_err)]
    fn lane(req: &AuthzRequest, mode: Mode) -> Result<PodStore, Response> {
        let mut store = build_store(req).expect("fixture materialises");
        apply_odrl_lane(&mut store, req, mode)?;
        Ok(store)
    }

    fn reads_n1(store: &PodStore, agent: &str) -> bool {
        let s = Session {
            agent: Some(agent),
            client: None,
            issuer: None,
            now: None,
        };
        store
            .accessible(&s, Mode::Read)
            .iter()
            .any(|g| g.as_str() == N1)
    }

    #[test]
    fn dataset_carries_odrl_detects_rule_predicates() {
        assert!(dataset_carries_odrl(&permit_nq(ALICE)));
        assert!(dataset_carries_odrl(&prohibit_nq(ALICE)));
        // A WAC-only dataset does not fire the lane.
        assert!(!dataset_carries_odrl(CONTENT));
        assert!(!dataset_carries_odrl(""));
    }

    #[test]
    fn odrl_action_for_mode_is_read_only_fail_closed() {
        assert_eq!(
            odrl_action_for_mode(Mode::Read),
            Some("http://www.w3.org/ns/odrl/2/read")
        );
        // Every non-read mode is unmapped => the lane refuses (fail-closed).
        assert_eq!(odrl_action_for_mode(Mode::Write), None);
        assert_eq!(odrl_action_for_mode(Mode::Append), None);
        assert_eq!(odrl_action_for_mode(Mode::Control), None);
    }

    #[test]
    fn union_policy_graph_sees_named_graph_policies() {
        // The SAME permit, but tucked inside a named graph — parse_policy over the raw
        // dataset would MISS it (bare patterns match the default graph only); the
        // union projection must surface it.
        let named = permit_nq(ALICE)
            .lines()
            .map(|l| format!("{} <https://pod.ex/policies>{}\n", &l[..l.len() - 2], " ."))
            .collect::<String>();
        let g = union_policy_graph(&named).expect("projection parses");
        let policy = sparq_policy::parse_policy(&g).expect("policy parses");
        assert_eq!(
            policy.permissions.len(),
            1,
            "named-graph rule must be visible"
        );
        assert_eq!(policy.permissions[0].target.as_deref(), Some(N1));
    }

    #[test]
    fn lane_permit_grants_assignee_only() {
        let req = authz_req(format!("{CONTENT}{}", permit_nq(ALICE)), Some(ALICE));
        let store = lane(&req, Mode::Read).expect("lane applies");
        // alice (the assignee) reads through the REAL enforcement path…
        assert!(reads_n1(&store, ALICE));
        // …and the grant is scoped: bob + anonymous see nothing (fail-closed).
        assert!(!reads_n1(&store, BOB));
        assert!(store.accessible(&Session::default(), Mode::Read).is_empty());

        // The SAME dataset requested BY bob materialises nothing for bob.
        let req_bob = authz_req(format!("{CONTENT}{}", permit_nq(ALICE)), Some(BOB));
        let store_bob = lane(&req_bob, Mode::Read).expect("lane applies");
        assert!(!reads_n1(&store_bob, BOB));
    }

    #[test]
    fn lane_prohibition_overrides_static_wac_grant() {
        // alice holds a STATIC WAC grant on n1; the ODRL prohibition must beat it
        // (deny-overrides through the unchanged ∪ allow ∖ ∪ deny enforcement).
        let wac = r#"<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
<https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
"#;
        // WITHOUT the ODRL prohibition the WAC grant admits alice (guards vacuity).
        let req_wac = authz_req(wac.to_owned(), Some(ALICE));
        let store_wac = build_store(&req_wac).expect("fixture materialises");
        assert!(
            reads_n1(&store_wac, ALICE),
            "static WAC grant must admit alice"
        );
        // WITH it, the bridged deny wins.
        let req = authz_req(format!("{wac}{}", prohibit_nq(ALICE)), Some(ALICE));
        let store = lane(&req, Mode::Read).expect("lane applies");
        assert!(
            !reads_n1(&store, ALICE),
            "ODRL prohibition must beat the WAC grant"
        );
    }

    #[test]
    fn lane_refuses_anonymous_fail_closed() {
        let req = authz_req(format!("{CONTENT}{}", prohibit_nq(ALICE)), None);
        let err = lane(&req, Mode::Read).err().expect("anonymous must refuse");
        assert_eq!(err.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn lane_refuses_non_read_mode_fail_closed() {
        let req = authz_req(format!("{CONTENT}{}", permit_nq(ALICE)), Some(ALICE));
        for mode in [Mode::Write, Mode::Append, Mode::Control] {
            let err = lane(&req, mode).err().expect("non-read mode must refuse");
            assert_eq!(err.status(), StatusCode::BAD_REQUEST);
        }
    }

    #[test]
    fn lane_refuses_targetless_rule_fail_closed() {
        // A prohibition with NO odrl:target applies to any target; silently
        // under-materialising it would WIDEN access → the lane must refuse.
        let targetless = "<urn:pol/p> <http://www.w3.org/ns/odrl/2/prohibition> _:proh .\n\
             _:proh <http://www.w3.org/ns/odrl/2/action> <http://www.w3.org/ns/odrl/2/read> .\n\
             _:proh <http://www.w3.org/ns/odrl/2/assignee> <https://alice.ex/card#me> .\n";
        let req = authz_req(format!("{CONTENT}{targetless}"), Some(ALICE));
        let err = lane(&req, Mode::Read)
            .err()
            .expect("targetless rule must refuse");
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn lane_refuses_unimplementable_conflict_strategy_fail_closed() {
        // odrl:perm (permissions override prohibitions) is unrepresentable under the
        // bridge's deny-overrides enforcement → the whole request refuses.
        let perm_strategy = format!(
            "{}\
             <urn:pol/p> <http://www.w3.org/ns/odrl/2/conflict> <http://www.w3.org/ns/odrl/2/perm> .\n",
            permit_nq(ALICE)
        );
        let req = authz_req(format!("{CONTENT}{perm_strategy}"), Some(ALICE));
        let err = lane(&req, Mode::Read).err().expect("odrl:perm must refuse");
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn lane_ignores_marker_inside_literal_and_stays_wac_only() {
        // The predicate IRI appearing INSIDE A LITERAL is a detection false positive:
        // the parsed policy has no rules, nothing materialises, and the WAC-only
        // behaviour is preserved (the fail-closed direction).
        let literal_marker =
            "<urn:x> <urn:note> \"see http://www.w3.org/ns/odrl/2/permission> docs\" .\n";
        let dataset = format!("{CONTENT}{literal_marker}");
        assert!(dataset_carries_odrl(&dataset), "scan fires on the literal");
        let req = authz_req(dataset, Some(ALICE));
        let store = lane(&req, Mode::Read).expect("rule-less policy is a no-op");
        assert!(
            !reads_n1(&store, ALICE),
            "no grant appears from a literal marker"
        );
    }
}
