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
#[derive(Debug)]
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
// SSRF egress policy (default-deny private / internal ranges) [OPUS-4.8]
// ---------------------------------------------------------------------------
//
// The `SERVICE` clause turns an attacker-controlled SPARQL string into an
// outbound HTTP request from the engine host (threat-model B4 / T-SERVICE-SSRF,
// bead sq-2v6f). With no egress filter that is a textbook SSRF primitive into
// the internal network — the worst case being the cloud-metadata endpoint
// 169.254.169.254, which on most clouds hands out credentials. The DEFAULT here
// is therefore DENY: a SERVICE endpoint that resolves to any non-global address
// is refused, and a deployer must explicitly opt a host/range back in via the
// allowlist (mirroring how `update.rs` gates `LOAD file://` behind
// `with_load_base`).
//
// DNS-rebinding safety: the check runs on the *resolved* IP, not the literal IRI
// host, and the production transport installs the policy as ureq's `Resolver`.
// ureq then connects only to the addresses the resolver returns, so the IP that
// is vetted is exactly the IP that is dialled — a hostile DNS answer that points
// at 127.0.0.1 / 169.254.169.254 is dropped before any socket is opened, and
// there is no resolve-then-reresolve TOCTOU window.

/// Classifies a resolved [`IpAddr`](std::net::IpAddr) as a forbidden (private /
/// internal / non-global) destination for SERVICE federation. Returns `true`
/// when the address is in a range the default-deny policy refuses.
///
/// Forbidden ranges:
/// * loopback — `127.0.0.0/8`, `::1`
/// * RFC1918 private — `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`
/// * link-local — `169.254.0.0/16` (incl. the `169.254.169.254` cloud-metadata
///   IP) and IPv6 `fe80::/10`
/// * unique-local IPv6 — `fc00::/7`
/// * unspecified — `0.0.0.0`, `::`
/// * IPv4-mapped IPv6 (`::ffff:a.b.c.d`) is unwrapped and re-classified as the
///   embedded IPv4 address, so a private v4 cannot be smuggled through a v6 host.
#[cfg(feature = "service")]
pub(crate) fn is_forbidden_ip(ip: std::net::IpAddr) -> bool {
    use std::net::IpAddr;
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()            // 127.0.0.0/8
                || v4.is_private()      // 10/8, 172.16/12, 192.168/16
                || v4.is_link_local()   // 169.254.0.0/16 (incl. 169.254.169.254)
                || v4.is_unspecified()  // 0.0.0.0
                // Defence-in-depth on ranges that are also internal but not
                // covered above: broadcast, shared CGNAT (100.64/10), benchmarking.
                || v4.is_broadcast()    // 255.255.255.255
                || matches!(v4.octets(), [100, b, ..] if (64..=127).contains(&b)) // 100.64/10 CGNAT
        }
        IpAddr::V6(v6) => {
            // Unwrap IPv4-mapped (::ffff:a.b.c.d) and re-check as IPv4 so a
            // private v4 can't ride in through a v6 literal.
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_forbidden_ip(IpAddr::V4(v4));
            }
            v6.is_loopback()            // ::1
                || v6.is_unspecified()  // ::
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // fe80::/10 link-local
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7 unique-local
        }
    }
}

/// SERVICE egress allowlist + policy mode. A host (DNS name or IP literal, exactly
/// as written in the SERVICE IRI authority) on this list is exempt from
/// [`is_forbidden_ip`] — its resolved addresses are permitted even when private.
///
/// Two modes (the [`Mode`] flag), both default-deny but at different strictnesses:
///
/// * **`Mode::DenyPrivate`** (the engine's standalone default, installed by
///   [`with_service_egress_allow`]): public IPs are reachable, private/internal IPs
///   are refused unless the host is on the allowlist. Allowlist entries only *add*
///   permission (re-open a private host).
/// * **`Mode::AllowlistOnly`** (the strict mode the *server* uses, installed by
///   [`with_service_egress_policy`]): ONLY hosts on the allowlist may be reached at
///   all — every other host is refused even if it resolves to a public address. An
///   empty allowlist in this mode is therefore "deny ALL SERVICE", which is the
///   server's safe default for the network-exposed surface.
///
/// Empty + `DenyPrivate` (the thread-local default before any scope installs a
/// policy) preserves the original behaviour: public allowed, private denied.
/// Installed for a scope via [`with_service_egress_allow`] /
/// [`with_service_egress_policy`], mirroring `update.rs`'s `with_load_base`
/// thread-local allowlist pattern. [OPUS-4.8] (sq-4w18)
#[cfg(feature = "service")]
pub(crate) mod egress_policy {
    use std::cell::RefCell;
    use std::collections::HashSet;

    /// How the allowlist is interpreted for hosts NOT on it. [OPUS-4.8] (sq-4w18)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum Mode {
        /// Hosts off the allowlist are reachable iff they resolve to a *public*
        /// address (private/internal IPs are refused). The engine's standalone
        /// default and the semantics of [`super::with_service_egress_allow`].
        DenyPrivate,
        /// Hosts off the allowlist are refused unconditionally (even public IPs).
        /// The server installs this so federation is restricted to exactly the
        /// operator-listed hosts; an empty allowlist = deny ALL SERVICE.
        AllowlistOnly,
    }

    struct Policy {
        allow: HashSet<String>,
        mode: Mode,
    }

    thread_local! {
        static POLICY: RefCell<Policy> =
            RefCell::new(Policy { allow: HashSet::new(), mode: Mode::DenyPrivate });
    }

    /// Restores the previous policy when the installing scope returns (also on
    /// unwind, so a panicking SERVICE call never leaks a relaxed policy).
    pub(crate) struct Guard(Option<Policy>);
    impl Drop for Guard {
        fn drop(&mut self) {
            if let Some(prev) = self.0.take() {
                POLICY.with(|p| *p.borrow_mut() = prev);
            }
        }
    }

    /// Installs `hosts` (lower-cased) + `mode` as the active SERVICE egress policy
    /// for the duration of the returned guard; the previous policy is restored on
    /// drop.
    pub(crate) fn install(hosts: impl IntoIterator<Item = String>, mode: Mode) -> Guard {
        let allow: HashSet<String> = hosts.into_iter().map(|h| h.to_ascii_lowercase()).collect();
        let next = Policy { allow, mode };
        // Swap in the new policy and hand the previous one to the Guard for restore.
        POLICY.with(|p| Guard(Some(std::mem::replace(&mut *p.borrow_mut(), next))))
    }

    /// True if `host` (case-insensitive) is on the active allowlist. An entry is
    /// matched two ways: [OPUS-4.8] (sq-4w18)
    ///   * **exact** — the entry equals the host (`"sparql.example.org"`).
    ///   * **suffix wildcard** — an entry beginning with a dot (`".example.org"`)
    ///     matches any host ending in that suffix INCLUDING the bare apex
    ///     (`example.org`, `a.example.org`, `a.b.example.org`). This is the engine
    ///     representation of the server's `*.example.org` pattern. The leading-dot
    ///     boundary means `.example.org` does NOT match `notexample.org`.
    pub(crate) fn is_allowed(host: &str) -> bool {
        let h = host.to_ascii_lowercase();
        POLICY.with(|p| {
            let allow = &p.borrow().allow;
            if allow.contains(&h) {
                return true;
            }
            // Suffix-wildcard entries (".suffix"): match the apex and any subdomain.
            allow.iter().any(|e| {
                if let Some(suffix) = e.strip_prefix('.') {
                    h == suffix || h.ends_with(e.as_str())
                } else {
                    false
                }
            })
        })
    }

    /// The active policy mode.
    pub(crate) fn mode() -> Mode {
        POLICY.with(|p| p.borrow().mode)
    }
}

/// Runs `f` with `hosts` allowlisted for SERVICE federation: each host's resolved
/// addresses are permitted even if they fall in a private/internal range that the
/// default-deny SSRF policy would otherwise refuse. A host is matched
/// case-insensitively against the *authority* of the SERVICE IRI (DNS name or IP
/// literal, e.g. `"localhost"`, `"10.0.0.5"`, `"sparql.internal"`).
///
/// Without an installed allowlist, every SERVICE endpoint that resolves to a
/// loopback / RFC1918 / link-local / unique-local / unspecified address is
/// REJECTED — the secure default. This mirrors [`crate::with_load_base`], which
/// gates `LOAD file://` the same way. Only effective with the `service` feature.
///
/// ```no_run
/// # #[cfg(feature = "service")] {
/// // Permit federation to a trusted internal endpoint that resolves privately.
/// sparq_engine::with_service_egress_allow(["sparql.internal".to_string()], || {
///     // ... run a query containing `SERVICE <http://sparql.internal/> { ... }`
/// });
/// # }
/// ```
#[cfg(feature = "service")]
pub fn with_service_egress_allow<R>(
    hosts: impl IntoIterator<Item = String>,
    f: impl FnOnce() -> R,
) -> R {
    let _guard = egress_policy::install(hosts, egress_policy::Mode::DenyPrivate);
    f()
}

/// Runs `f` under a STRICT SERVICE egress policy: only the listed `hosts` may be
/// reached, and EVERY other host is refused — even one resolving to a public
/// address. This is the policy the network-exposed **server** installs (bead
/// sq-4w18): the SERVICE clause turns attacker-controlled SPARQL into outbound HTTP,
/// so federation is restricted to exactly the operator-configured endpoints.
///
/// `strict = false` is identical to [`with_service_egress_allow`] (default-deny
/// *private*: public hosts reachable, private/internal only if allowlisted). The
/// server wires this directly to its `--service-allow` config so the same call site
/// expresses both "no federation at all" (strict + empty list) and "an explicit
/// allowlist" (strict + hosts). Host matching is case-insensitive against the
/// SERVICE IRI *authority* (DNS name or IP literal), exactly like
/// [`with_service_egress_allow`].
///
/// ```no_run
/// # #[cfg(feature = "service")] {
/// // Restrict SERVICE to a single trusted endpoint; anything else is refused.
/// sparq_engine::with_service_egress_policy(true, ["sparql.example.org".to_string()], || {
///     // ... run a query that may contain `SERVICE <…> { ... }`
/// });
/// // Strict + empty list = federation fully disabled (deny ALL SERVICE).
/// sparq_engine::with_service_egress_policy(true, std::iter::empty(), || { /* ... */ });
/// # }
/// ```
#[cfg(feature = "service")]
pub fn with_service_egress_policy<R>(
    strict: bool,
    hosts: impl IntoIterator<Item = String>,
    f: impl FnOnce() -> R,
) -> R {
    let mode = if strict {
        egress_policy::Mode::AllowlistOnly
    } else {
        egress_policy::Mode::DenyPrivate
    };
    let _guard = egress_policy::install(hosts, mode);
    f()
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

/// ureq [`Resolver`](ureq::Resolver) wrapper that enforces the SSRF egress policy
/// on the *resolved* addresses (DNS-rebinding-safe). [OPUS-4.8]
///
/// It resolves `netloc` with the standard system resolver, drops every address
/// the [`is_forbidden_ip`] policy refuses (unless the host is on the active
/// allowlist), and returns only the survivors — so ureq dials only vetted IPs.
/// If resolution yields nothing but forbidden addresses, it returns a
/// `PermissionDenied` error rather than an empty set, which surfaces to the
/// caller as a SERVICE failure (and an empty result under SILENT).
#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
struct EgressFilterResolver;

#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
impl ureq::Resolver for EgressFilterResolver {
    fn resolve(&self, netloc: &str) -> std::io::Result<Vec<std::net::SocketAddr>> {
        use std::net::ToSocketAddrs;
        // `netloc` is `host:port`; the allowlist is keyed by the bare host (the
        // authority without the port). rsplit on ':' to keep IPv6 literals — those
        // come bracketed as `[::1]:80`, so strip the brackets too.
        let host = match netloc.rsplit_once(':') {
            Some((h, _)) => h,
            None => netloc,
        };
        let host = host.trim_start_matches('[').trim_end_matches(']');
        let allowed = egress_policy::is_allowed(host);
        // [OPUS-4.8] (sq-4w18) STRICT (AllowlistOnly) mode — the server's policy —
        // refuses any host not on the allowlist BEFORE resolving DNS, so a host that
        // is not explicitly permitted never triggers even a lookup (no network at all,
        // and no public-IP escape hatch). An empty allowlist here = deny ALL SERVICE.
        if !allowed && egress_policy::mode() == egress_policy::Mode::AllowlistOnly {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!(
                    "SERVICE egress refused: host {host:?} is not on the SERVICE allowlist \
                     (strict allowlist-only policy; add it via --service-allow / SPARQ_SERVICE_ALLOW \
                     on the server, or with_service_egress_policy in an embedder)"
                ),
            ));
        }
        let all: Vec<std::net::SocketAddr> = netloc.to_socket_addrs()?.collect();
        let permitted: Vec<std::net::SocketAddr> = all
            .into_iter()
            .filter(|sa| allowed || !is_forbidden_ip(sa.ip()))
            .collect();
        if permitted.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!(
                    "SERVICE egress refused: {netloc} resolves only to private/internal addresses \
                     (default-deny SSRF policy; allowlist the host via with_service_egress_allow)"
                ),
            ));
        }
        Ok(permitted)
    }
}

#[cfg(all(feature = "service", not(target_arch = "wasm32")))]
impl Transport for HttpTransport {
    fn fetch(&self, endpoint: &str, query: &str) -> Result<String, String> {
        let agent = ureq::AgentBuilder::new()
            .timeout(self.timeout)
            .user_agent(concat!("sparq-engine/", env!("CARGO_PKG_VERSION")))
            // Default-deny SSRF egress filter: vets the resolved IP before connect,
            // so a SERVICE endpoint pointing at loopback / RFC1918 / link-local /
            // cloud-metadata is refused (unless allowlisted). DNS-rebinding-safe:
            // ureq connects only to the addresses this resolver returns. [OPUS-4.8]
            .resolver(EgressFilterResolver)
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

    // ---------------------------------------------------------------------
    // SSRF egress policy [OPUS-4.8] (bead sq-2v6f)
    // ---------------------------------------------------------------------

    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    fn v4(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(a, b, c, d))
    }

    #[test]
    fn loopback_is_forbidden() {
        assert!(is_forbidden_ip(v4(127, 0, 0, 1)));
        assert!(is_forbidden_ip(v4(127, 255, 255, 254))); // anywhere in 127/8
        assert!(is_forbidden_ip(IpAddr::V6(Ipv6Addr::LOCALHOST))); // ::1
    }

    #[test]
    fn rfc1918_private_is_forbidden() {
        assert!(is_forbidden_ip(v4(10, 0, 0, 1))); // 10/8
        assert!(is_forbidden_ip(v4(10, 255, 255, 255)));
        assert!(is_forbidden_ip(v4(172, 16, 0, 1))); // 172.16/12 (low edge)
        assert!(is_forbidden_ip(v4(172, 31, 255, 255))); // 172.16/12 (high edge)
        assert!(is_forbidden_ip(v4(192, 168, 0, 1))); // 192.168/16
    }

    #[test]
    fn link_local_and_cloud_metadata_are_forbidden() {
        assert!(is_forbidden_ip(v4(169, 254, 0, 1))); // 169.254/16
        // The cloud-metadata endpoint — the highest-value SSRF target.
        assert!(is_forbidden_ip(v4(169, 254, 169, 254)));
        // IPv6 link-local fe80::/10.
        assert!(is_forbidden_ip(IpAddr::V6("fe80::1".parse().unwrap())));
        assert!(is_forbidden_ip(IpAddr::V6("febf::1".parse().unwrap()))); // top of /10
    }

    #[test]
    fn unique_local_v6_is_forbidden() {
        assert!(is_forbidden_ip(IpAddr::V6("fc00::1".parse().unwrap()))); // fc00::/7
        assert!(is_forbidden_ip(IpAddr::V6("fd12:3456::1".parse().unwrap())));
    }

    #[test]
    fn unspecified_is_forbidden() {
        assert!(is_forbidden_ip(v4(0, 0, 0, 0)));
        assert!(is_forbidden_ip(IpAddr::V6(Ipv6Addr::UNSPECIFIED))); // ::
    }

    #[test]
    fn cgnat_and_broadcast_are_forbidden() {
        assert!(is_forbidden_ip(v4(100, 64, 0, 1))); // 100.64/10 CGNAT (low)
        assert!(is_forbidden_ip(v4(100, 127, 255, 255))); // CGNAT (high)
        assert!(!is_forbidden_ip(v4(100, 63, 0, 1))); // just below CGNAT — public
        assert!(!is_forbidden_ip(v4(100, 128, 0, 1))); // just above CGNAT — public
        assert!(is_forbidden_ip(v4(255, 255, 255, 255))); // broadcast
    }

    #[test]
    fn ipv4_mapped_v6_is_unwrapped_and_classified() {
        // ::ffff:127.0.0.1 must be refused as the embedded private v4.
        assert!(is_forbidden_ip(IpAddr::V6("::ffff:127.0.0.1".parse().unwrap())));
        assert!(is_forbidden_ip(IpAddr::V6("::ffff:10.0.0.1".parse().unwrap())));
        assert!(is_forbidden_ip(IpAddr::V6("::ffff:169.254.169.254".parse().unwrap())));
        // A public v4 mapped into v6 is still allowed.
        assert!(!is_forbidden_ip(IpAddr::V6("::ffff:8.8.8.8".parse().unwrap())));
    }

    #[test]
    fn public_addresses_are_allowed() {
        assert!(!is_forbidden_ip(v4(8, 8, 8, 8))); // Google DNS
        assert!(!is_forbidden_ip(v4(1, 1, 1, 1))); // Cloudflare DNS
        assert!(!is_forbidden_ip(v4(93, 184, 216, 34))); // example.com (historical)
        assert!(!is_forbidden_ip(v4(172, 15, 0, 1))); // just below 172.16/12 — public
        assert!(!is_forbidden_ip(v4(172, 32, 0, 1))); // just above 172.16/12 — public
        assert!(!is_forbidden_ip(IpAddr::V6("2001:4860:4860::8888".parse().unwrap()))); // public v6
    }

    #[test]
    fn allowlist_plumbing_install_and_restore() {
        // Default: nothing is allowlisted.
        assert!(!egress_policy::is_allowed("localhost"));
        {
            let _g = egress_policy::install(
                ["localhost".to_string(), "10.0.0.5".to_string()],
                egress_policy::Mode::DenyPrivate,
            );
            assert!(egress_policy::is_allowed("localhost"));
            assert!(egress_policy::is_allowed("LOCALHOST")); // case-insensitive
            assert!(egress_policy::is_allowed("10.0.0.5"));
            assert!(!egress_policy::is_allowed("other.host"));
        }
        // Restored to empty on guard drop.
        assert!(!egress_policy::is_allowed("localhost"));
    }

    #[test]
    fn with_service_egress_allow_scopes_the_allowlist() {
        assert!(!egress_policy::is_allowed("sparql.internal"));
        let seen = with_service_egress_allow(["sparql.internal".to_string()], || {
            egress_policy::is_allowed("sparql.internal")
        });
        assert!(seen);
        // Allowlist is gone after the scope returns.
        assert!(!egress_policy::is_allowed("sparql.internal"));
    }

    #[test]
    fn strict_allowlist_only_mode_scopes_and_restores() {
        // [OPUS-4.8] (sq-4w18) Strict mode: only listed hosts are allowed; the mode
        // and allowlist both restore on scope exit.
        assert_eq!(egress_policy::mode(), egress_policy::Mode::DenyPrivate);
        assert!(!egress_policy::is_allowed("a.example"));
        with_service_egress_policy(true, ["a.example".to_string()], || {
            assert_eq!(egress_policy::mode(), egress_policy::Mode::AllowlistOnly);
            assert!(egress_policy::is_allowed("a.example"));
            assert!(egress_policy::is_allowed("A.EXAMPLE")); // case-insensitive
            assert!(!egress_policy::is_allowed("b.example"));
        });
        assert_eq!(egress_policy::mode(), egress_policy::Mode::DenyPrivate);
        assert!(!egress_policy::is_allowed("a.example"));
    }

    #[test]
    fn suffix_wildcard_allowlist_matches_apex_and_subdomains() {
        // [OPUS-4.8] (sq-4w18) A ".example.org" entry matches the apex and any
        // subdomain, but not a host that merely ends in the same letters.
        with_service_egress_policy(true, [".example.org".to_string()], || {
            assert!(egress_policy::is_allowed("example.org")); // apex
            assert!(egress_policy::is_allowed("sparql.example.org")); // subdomain
            assert!(egress_policy::is_allowed("a.b.example.org")); // deep subdomain
            assert!(egress_policy::is_allowed("SPARQL.EXAMPLE.ORG")); // case-insensitive
            assert!(!egress_policy::is_allowed("notexample.org")); // boundary respected
            assert!(!egress_policy::is_allowed("example.org.evil.com")); // suffix only
        });
    }

    #[test]
    fn non_strict_policy_matches_allow_helper() {
        // strict=false behaves exactly like with_service_egress_allow (DenyPrivate).
        with_service_egress_policy(false, ["c.example".to_string()], || {
            assert_eq!(egress_policy::mode(), egress_policy::Mode::DenyPrivate);
            assert!(egress_policy::is_allowed("c.example"));
        });
    }

    #[test]
    fn allowlist_restores_on_unwind() {
        // A panic inside the scope must still restore the previous (empty) policy —
        // a relaxed allowlist must never leak past the scope on unwind.
        let _ = std::panic::catch_unwind(|| {
            with_service_egress_allow(["leaky.host".to_string()], || {
                assert!(egress_policy::is_allowed("leaky.host"));
                panic!("boom");
            });
        });
        assert!(!egress_policy::is_allowed("leaky.host"));
    }

    // The resolver path is native-only (it wraps ureq's Resolver).
    #[cfg(not(target_arch = "wasm32"))]
    mod resolver {
        use super::*;
        use ureq::Resolver;

        #[test]
        fn resolver_refuses_loopback_endpoint() {
            // 127.0.0.1 resolves to itself; with no allowlist the policy must
            // refuse it with PermissionDenied — never returning a dial-able addr.
            let r = EgressFilterResolver;
            let err = r.resolve("127.0.0.1:8080").unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        }

        #[test]
        fn resolver_refuses_cloud_metadata_endpoint() {
            let r = EgressFilterResolver;
            let err = r.resolve("169.254.169.254:80").unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        }

        #[test]
        fn resolver_refuses_ipv6_loopback_endpoint() {
            let r = EgressFilterResolver;
            // ureq passes IPv6 netlocs bracketed.
            let err = r.resolve("[::1]:80").unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        }

        #[test]
        fn resolver_allows_public_endpoint() {
            let r = EgressFilterResolver;
            // 8.8.8.8 is a literal so no DNS lookup happens; it is global, so it
            // passes the filter and comes back as a dial-able address.
            let addrs = r.resolve("8.8.8.8:443").unwrap();
            assert_eq!(addrs.len(), 1);
            assert_eq!(addrs[0].ip(), v4(8, 8, 8, 8));
        }

        #[test]
        fn resolver_permits_allowlisted_private_endpoint() {
            // With 127.0.0.1 on the allowlist, the loopback endpoint is dial-able.
            let r = EgressFilterResolver;
            let addrs = with_service_egress_allow(["127.0.0.1".to_string()], || {
                r.resolve("127.0.0.1:8080")
            })
            .unwrap();
            assert_eq!(addrs.len(), 1);
            assert!(addrs[0].ip().is_loopback());
        }

        // [OPUS-4.8] (sq-4w18) Strict allowlist-only mode — the server's policy.

        #[test]
        fn strict_refuses_public_host_off_the_allowlist() {
            let r = EgressFilterResolver;
            let err = with_service_egress_policy(true, std::iter::empty(), || r.resolve("8.8.8.8:443"))
                .unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        }

        #[test]
        fn strict_empty_allowlist_denies_all() {
            let r = EgressFilterResolver;
            for netloc in ["8.8.8.8:443", "1.1.1.1:80", "127.0.0.1:8080"] {
                let err = with_service_egress_policy(true, std::iter::empty(), || r.resolve(netloc))
                    .unwrap_err();
                assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied, "{netloc} must be refused");
            }
        }

        #[test]
        fn strict_permits_allowlisted_host() {
            let r = EgressFilterResolver;
            let addrs = with_service_egress_policy(true, ["8.8.8.8".to_string()], || r.resolve("8.8.8.8:443"))
                .unwrap();
            assert_eq!(addrs.len(), 1);
            assert_eq!(addrs[0].ip(), v4(8, 8, 8, 8));

            let addrs = with_service_egress_policy(true, ["127.0.0.1".to_string()], || r.resolve("127.0.0.1:8080"))
                .unwrap();
            assert_eq!(addrs.len(), 1);
            assert!(addrs[0].ip().is_loopback());
        }

        #[test]
        fn non_strict_resolver_allows_public_off_list() {
            let r = EgressFilterResolver;
            let addrs = with_service_egress_policy(false, std::iter::empty(), || r.resolve("8.8.8.8:443"))
                .unwrap();
            assert_eq!(addrs.len(), 1);
        }
    }
}
