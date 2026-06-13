//! SPARQL 1.1 Federated Query — `SERVICE` evaluation. [OPUS-4.8]
//!
//! `SERVICE <endpoint> { pattern }` evaluates `pattern` against a *remote* SPARQL
//! endpoint and joins the returned solutions back into the surrounding query, per
//! <https://www.w3.org/TR/sparql11-federated-query/>.
//!
//! ## How it works
//!
//! 1. The inner [`GraphPattern`] is wrapped as `SELECT * WHERE { <inner> }` using
//!    spargebra's `Display` impl (which round-trips algebra → SPARQL syntax), so the
//!    full pattern (BGPs, OPTIONAL, FILTER, sub-SELECT, …) is forwarded verbatim. We
//!    do NOT push surrounding bindings down (no "BindingsRestricted" / VALUES
//!    injection) — that is a correct, if not maximally-selective, evaluation: the
//!    remote relation is materialised and joined locally by the caller.
//! 2. The query is sent over HTTP (form-encoded POST, `Accept:
//!    application/sparql-results+json`).
//! 3. The SPARQL-Results-JSON response is parsed into a [`ServiceRelation`]
//!    (variable list + rows of optional [`Term`]s).
//! 4. The caller (`exec::eval_service`) interns those terms into the local/graph
//!    dictionaries — exactly like `VALUES` — and joins them with the rest of the query.
//!
//! ## `SERVICE SILENT`
//!
//! Any error (DNS, connection, non-2xx status, malformed body) is swallowed when the
//! pattern is `SILENT`: evaluation yields the join identity (a single empty solution),
//! so the surrounding query keeps its existing bindings rather than failing. Without
//! `SILENT`, the error propagates and fails the query.
//!
//! ## Transport seam (testability)
//!
//! The HTTP call sits behind the [`Transport`] trait. Production uses [`HttpTransport`]
//! (ureq, a tiny blocking client — gated off wasm). Tests inject a canned-response or
//! local-loopback transport, so the SRJ parser and the algebra integration are
//! exercised without a public network dependency.
//!
//! ## Out of scope
//!
//! * `SERVICE ?var` (a *variable* endpoint): the endpoint IRI is only known once the
//!   surrounding bindings are produced, which requires a per-solution remote call. We
//!   reject it with a clear error (or, under `SILENT`, the empty result) rather than
//!   silently mis-evaluating — see [`eval_service`] in `exec.rs`.

use oxrdf::{BlankNode, Literal, NamedNode, NamedOrBlankNode, Term, Triple, Variable};

/// A materialised remote SELECT result: the projected variables and one row per
/// solution (`None` = the variable is unbound in that solution).
pub(crate) struct ServiceRelation {
    pub vars: Vec<Variable>,
    pub rows: Vec<Vec<Option<Term>>>,
}

/// Abstracts the HTTP round-trip so tests can inject a fake endpoint. `query` is the
/// SPARQL query string; the return is the raw response body (expected to be
/// SPARQL-Results-JSON) or a transport error string.
pub(crate) trait Transport {
    fn fetch(&self, endpoint: &str, query: &str) -> Result<String, String>;
}

/// Evaluate one SERVICE call end-to-end: send `query` to `endpoint` via `transport`
/// and parse the response into a [`ServiceRelation`]. SILENT handling is the caller's
/// responsibility (it owns the join-identity fallback).
pub(crate) fn eval_remote(
    transport: &dyn Transport,
    endpoint: &str,
    query: &str,
) -> Result<ServiceRelation, String> {
    let body = transport.fetch(endpoint, query)?;
    parse_srj(&body)
}

// ---------------------------------------------------------------------------
// SPARQL Results JSON parser
// (https://www.w3.org/TR/sparql11-results-json/)
// ---------------------------------------------------------------------------

/// Parse a SELECT result document. ASK results (`{"boolean": …}`) are reported as an
/// error here — `SERVICE { … }` always wraps a SELECT in our forwarding, so a boolean
/// body indicates a misbehaving endpoint.
#[cfg(feature = "service")]
pub(crate) fn parse_srj(text: &str) -> Result<ServiceRelation, String> {
    let v: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("SERVICE: invalid results JSON: {e}"))?;
    if v.get("boolean").is_some() {
        return Err("SERVICE: endpoint returned an ASK boolean, expected SELECT bindings".into());
    }
    let vars: Vec<Variable> = v
        .pointer("/head/vars")
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str())
                .map(|s| Variable::new(s).map_err(|e| format!("SERVICE: bad result variable {s:?}: {e}")))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .ok_or_else(|| "SERVICE: results JSON missing head.vars".to_string())?;

    let mut rows: Vec<Vec<Option<Term>>> = Vec::new();
    for sol in v
        .pointer("/results/bindings")
        .and_then(|a| a.as_array())
        .ok_or_else(|| "SERVICE: results JSON missing results.bindings".to_string())?
    {
        let obj = sol
            .as_object()
            .ok_or_else(|| "SERVICE: a solution binding is not a JSON object".to_string())?;
        // Build a row positionally over `vars`; a variable absent from the solution
        // object is UNBOUND (`None`) — identical to the VALUES UNDEF semantics.
        let mut row: Vec<Option<Term>> = Vec::with_capacity(vars.len());
        for var in &vars {
            match obj.get(var.as_str()) {
                Some(cell) => row.push(Some(srj_term(cell)?)),
                None => row.push(None),
            }
        }
        rows.push(row);
    }
    Ok(ServiceRelation { vars, rows })
}

/// Reconstruct one term from an SRJ binding value object. Mirrors the conformance
/// suite's `srj_term` (uri / bnode / literal / SPARQL-1.2 triple terms).
#[cfg(feature = "service")]
fn srj_term(val: &serde_json::Value) -> Result<Term, String> {
    let get = |k: &str| val.get(k).and_then(|s| s.as_str());
    match get("type") {
        Some("uri") => {
            let iri = get("value").unwrap_or_default();
            Ok(Term::NamedNode(
                NamedNode::new(iri).map_err(|e| format!("SERVICE: bad IRI {iri:?}: {e}"))?,
            ))
        }
        Some("bnode") => {
            let id = get("value").unwrap_or_default();
            Ok(Term::BlankNode(
                BlankNode::new(id).map_err(|e| format!("SERVICE: bad bnode {id:?}: {e}"))?,
            ))
        }
        // Both "literal" and the legacy "typed-literal" (pre-2013 endpoints) map here.
        Some("literal") | Some("typed-literal") | None => {
            let value = get("value")
                .ok_or_else(|| "SERVICE: literal binding without value".to_string())?
                .to_string();
            if let Some(lang) = get("xml:lang") {
                Ok(Term::Literal(
                    Literal::new_language_tagged_literal(value, lang)
                        .map_err(|e| format!("SERVICE: bad language tag {lang:?}: {e}"))?,
                ))
            } else if let Some(dt) = get("datatype") {
                let dt = NamedNode::new(dt).map_err(|e| format!("SERVICE: bad datatype {dt:?}: {e}"))?;
                Ok(Term::Literal(Literal::new_typed_literal(value, dt)))
            } else {
                Ok(Term::Literal(Literal::new_simple_literal(value)))
            }
        }
        Some("triple") => {
            let v = val
                .get("value")
                .ok_or_else(|| "SERVICE: triple term without value".to_string())?;
            let part = |k: &str| -> Result<Term, String> {
                srj_term(v.get(k).ok_or_else(|| format!("SERVICE: triple term without {k}"))?)
            };
            let subject = match part("subject")? {
                Term::NamedNode(n) => NamedOrBlankNode::NamedNode(n),
                Term::BlankNode(b) => NamedOrBlankNode::BlankNode(b),
                other => return Err(format!("SERVICE: invalid triple-term subject: {other}")),
            };
            let predicate = match part("predicate")? {
                Term::NamedNode(n) => n,
                other => return Err(format!("SERVICE: invalid triple-term predicate: {other}")),
            };
            Ok(Term::Triple(Box::new(Triple {
                subject,
                predicate,
                object: part("object")?,
            })))
        }
        Some(other) => Err(format!("SERVICE: unknown binding type {other:?}")),
    }
}

// ---------------------------------------------------------------------------
// Production HTTP transport (ureq, blocking, native-only)
// ---------------------------------------------------------------------------

/// The real network transport: a blocking ureq POST with the SPARQL query
/// form-encoded in the body and `Accept: application/sparql-results+json`.
///
/// Gated to `cfg(not(wasm32))` AND the `service` feature so neither ureq nor any of
/// its TLS stack ever enters the wasm bundle.
#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
pub(crate) struct HttpTransport {
    timeout: std::time::Duration,
}

#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
impl HttpTransport {
    pub(crate) fn new() -> Self {
        // A finite default so an unreachable/slow endpoint cannot hang the engine
        // indefinitely; SILENT then turns this into an empty result.
        HttpTransport { timeout: std::time::Duration::from_secs(30) }
    }
}

#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
impl Transport for HttpTransport {
    fn fetch(&self, endpoint: &str, query: &str) -> Result<String, String> {
        let agent = ureq::AgentBuilder::new()
            .timeout(self.timeout)
            .user_agent(concat!("sparq-engine/", env!("CARGO_PKG_VERSION")))
            .build();
        // POST with the query in an `application/x-www-form-urlencoded` `query=` field
        // (SPARQL Protocol §2.1.2 "query via POST with URL-encoded parameters") — the
        // most broadly supported method and not subject to URL-length limits.
        let resp = agent
            .post(endpoint)
            .set("Accept", "application/sparql-results+json")
            .send_form(&[("query", query)]);
        match resp {
            Ok(r) => r
                .into_string()
                .map_err(|e| format!("SERVICE: reading response from {endpoint}: {e}")),
            // ureq surfaces non-2xx as `Error::Status`; treat both transport and HTTP
            // errors uniformly (the caller decides SILENT vs propagate).
            Err(ureq::Error::Status(code, _)) => {
                Err(format!("SERVICE: endpoint {endpoint} returned HTTP {code}"))
            }
            Err(e) => Err(format!("SERVICE: request to {endpoint} failed: {e}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests (parser + transport seam; no public network)
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "service"))]
mod tests {
    use super::*;

    #[test]
    fn parses_uri_and_literal_bindings() {
        let body = r#"{
            "head": { "vars": ["s", "name"] },
            "results": { "bindings": [
                { "s": {"type":"uri","value":"http://ex/a"},
                  "name": {"type":"literal","value":"Alice"} },
                { "s": {"type":"uri","value":"http://ex/b"},
                  "name": {"type":"literal","value":"Bob","xml:lang":"en"} }
            ] }
        }"#;
        let rel = parse_srj(body).unwrap();
        assert_eq!(rel.vars.len(), 2);
        assert_eq!(rel.rows.len(), 2);
        assert_eq!(
            rel.rows[0][0],
            Some(Term::NamedNode(NamedNode::new("http://ex/a").unwrap()))
        );
        assert_eq!(
            rel.rows[1][1],
            Some(Term::Literal(
                Literal::new_language_tagged_literal("Bob", "en").unwrap()
            ))
        );
    }

    #[test]
    fn unbound_variable_becomes_none() {
        let body = r#"{
            "head": { "vars": ["a", "b"] },
            "results": { "bindings": [ { "a": {"type":"uri","value":"http://ex/x"} } ] }
        }"#;
        let rel = parse_srj(body).unwrap();
        assert_eq!(rel.rows[0][0], Some(Term::NamedNode(NamedNode::new("http://ex/x").unwrap())));
        assert_eq!(rel.rows[0][1], None);
    }

    #[test]
    fn typed_literal_roundtrips() {
        let body = r#"{
            "head": { "vars": ["n"] },
            "results": { "bindings": [
                { "n": {"type":"literal","value":"42",
                        "datatype":"http://www.w3.org/2001/XMLSchema#integer"} }
            ] }
        }"#;
        let rel = parse_srj(body).unwrap();
        let want = Literal::new_typed_literal(
            "42",
            NamedNode::new("http://www.w3.org/2001/XMLSchema#integer").unwrap(),
        );
        assert_eq!(rel.rows[0][0], Some(Term::Literal(want)));
    }

    #[test]
    fn empty_results_is_ok() {
        let body = r#"{"head":{"vars":["x"]},"results":{"bindings":[]}}"#;
        let rel = parse_srj(body).unwrap();
        assert!(rel.rows.is_empty());
        assert_eq!(rel.vars.len(), 1);
    }

    #[test]
    fn malformed_json_errors() {
        assert!(parse_srj("not json at all").is_err());
        assert!(parse_srj(r#"{"head":{}}"#).is_err()); // no vars
        assert!(parse_srj(r#"{"boolean":true}"#).is_err()); // ASK, not SELECT
    }

    /// Canned-response transport: proves `eval_remote` wires the transport into the
    /// parser without touching the network.
    struct Canned(&'static str);
    impl Transport for Canned {
        fn fetch(&self, _endpoint: &str, _query: &str) -> Result<String, String> {
            Ok(self.0.to_string())
        }
    }

    #[test]
    fn eval_remote_uses_injected_transport() {
        let body = r#"{"head":{"vars":["x"]},
            "results":{"bindings":[{"x":{"type":"uri","value":"http://ex/1"}}]}}"#;
        let rel = eval_remote(&Canned(body), "http://unused/", "SELECT * WHERE {}").unwrap();
        assert_eq!(rel.rows.len(), 1);
    }

    struct Failing;
    impl Transport for Failing {
        fn fetch(&self, _e: &str, _q: &str) -> Result<String, String> {
            Err("connection refused".into())
        }
    }

    #[test]
    fn eval_remote_propagates_transport_error() {
        let err = eval_remote(&Failing, "http://unused/", "SELECT * WHERE {}").unwrap_err();
        assert!(err.contains("connection refused"));
    }
}
