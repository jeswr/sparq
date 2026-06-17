//! Capability discovery (design §4.1) — **Phase 1**.
//!
//! Given a remote SPARQL endpoint this module fetches the two discovery documents a
//! `sparq-server` (and any standards-compliant endpoint) publishes, and distils them into
//! the planner's two inputs:
//!
//!   * a [`Capability`](crate::discovery::Capability) — *what the source can do* (full SPARQL
//!     1.1? which result formats? bind-join via VALUES? aggregates / property-paths /
//!     ORDER+LIMIT pushable?), parsed from the **SPARQL Service Description** (`GET <endpoint>`
//!     with no `query` parameter, per
//!     [SD §2](https://www.w3.org/TR/sparql11-service-description/)); and
//!   * an optional [`SourceDescriptor`](sparq_fedplan::SourceDescriptor) — the source's
//!     *statistics* (VoID counts +
//!     per-predicate / per-class partitions + the `scs:` characteristic sets), parsed from
//!     `GET <base>/.well-known/void`.
//!
//! ## What is net-new here, and what is reused
//!
//! The **VoID + `scs:` consumer already exists** end-to-end: the server writes it with
//! `sparq_introspect::Introspection::to_void_with_cs` and the planner reads it with
//! [`SourceDescriptor::from_void_nt`](sparq_fedplan::SourceDescriptor::from_void_nt). This
//! module **REUSES** that seam verbatim for the statistics half — it does NOT re-parse VoID.
//!
//! The **client-side Service-Description parser is the one genuinely new parser this layer
//! needs** (the server has only the *writer*, `sparq-server/src/descriptors.rs::sd_ntriples`).
//! [`parse_service_description`](crate::discovery::parse_service_description) is the exact
//! consumer of that writer's output, and the `sd_round_trip_against_served_fixture` test
//! round-trips a served-shape fixture through it.
//!
//! ## ASK-probe fallback
//!
//! When an endpoint publishes **no** SD and **no** VoID (a bare SPARQL endpoint), discovery
//! falls back to a FedX-style **`ASK { ... }` probe**: a source that answers an `ASK`
//! affirmatively is a full SPARQL endpoint, so a conservative "full SPARQL, VALUES bind-join,
//! no advertised stats" capability is assumed. This is the recall-safe default — it never
//! *claims* a feature the SD would have to confirm beyond core SELECT/ASK evaluation, and it
//! never invents statistics (the returned [`SourceDescriptor`](sparq_fedplan::SourceDescriptor)
//! is `None`, so the planner uses
//! its uniform fallback). Pattern-level ASK probes for cardinality live in a later phase
//! (§4.1 "to per-fragment count metadata (TPF)"); this slice ships the SD/VoID parse + the
//! capability-level ASK reachability probe.
//!
//! ## SSRF caution on every fetch
//!
//! Discovery makes outbound HTTP GETs to a *caller-supplied* endpoint URL — a textbook SSRF
//! sink (threat-model B4). The production [`HttpFetcher`](crate::discovery::HttpFetcher)
//! therefore installs the **same default-deny egress filter the engine's SERVICE transport
//! uses**: it resolves the host itself and refuses to dial loopback / RFC1918 / link-local
//! (incl. the `169.254.169.254` cloud-metadata IP) / unique-local / CGNAT addresses, vetting
//! the *resolved* IP just before connect so a DNS-rebind cannot smuggle an internal address
//! through. The IP classifier (`is_forbidden_ip`) mirrors
//! `sparq_engine::service::is_forbidden_ip` (which is `pub(crate)` there, so it is
//! re-derived here rather than imported); an embedder may relax the policy per-host with
//! `HttpFetcher::allow_private_host`.
//!
//! The network is behind the [`Fetcher`](crate::discovery::Fetcher) trait seam, so the parser
//! + orchestration are tested with an in-memory [`MapFetcher`](crate::discovery::MapFetcher)
//! and pay **zero** network in CI.
//
// [OPUS-4.8] sq-nfxl (epic sq-dnko): Phase-1 discovery client + Service-Description parser.
// Net-new: the SD->Capability parser + the SSRF-guarded fetch seam + the ASK fallback.
// Reused: SourceDescriptor::from_void_nt for the VoID+scs statistics half. Flagged for
// Fable re-review when available.

use sparq_fedplan::{SourceDescriptor, SourceId};

// ─── Vocabulary IRIs (must match sparq-server/src/descriptors.rs exactly) ────────────

/// `sd:` — the SPARQL 1.1 Service Description vocabulary namespace.
const SD_NS: &str = "http://www.w3.org/ns/sparql-service-description#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// `sd:SPARQL10Query` / `sd:SPARQL11Query` — the query-language feature IRIs.
const SD_SPARQL10_QUERY: &str = "http://www.w3.org/ns/sparql-service-description#SPARQL10Query";
const SD_SPARQL11_QUERY: &str = "http://www.w3.org/ns/sparql-service-description#SPARQL11Query";
/// `sd:BasicFederatedQuery` — the SERVICE-clause feature (the server advertises it only
/// when built with the `service` feature). We surface it as [`Capability::federated_query`].
const SD_BASIC_FEDERATED_QUERY: &str =
    "http://www.w3.org/ns/sparql-service-description#BasicFederatedQuery";

// ─── The capability model (§4.1) ─────────────────────────────────────────────────────
//
// NOTE on module ownership: the design names `Capability` / `Interface` / `BindJoin` etc. as
// living in `source.rs`. A SIBLING agent owns that module in this same crate, so to keep our
// edits non-overlapping (trivial merge) the discovery-specific capability model is defined
// HERE and re-exported from `discovery`. When the source-abstraction phase lands it can
// re-export or absorb these — they are deliberately self-contained.

/// The wire interface a source speaks. This Phase-1 slice discovers SPARQL `Endpoint`s; the
/// brTPF / TPF / local variants are named so the planner bridge (later phases) can match on a
/// stable enum, but discovery only ever *produces* `Endpoint` here (an ASK-reachable SPARQL
/// endpoint) — brTPF/TPF detection (Hydra control vocabulary) is a later phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interface {
    /// A full SPARQL 1.1 protocol endpoint.
    Endpoint,
    /// A bindings-restricted Triple Pattern Fragments server (brTPF).
    BrTpf,
    /// A plain Triple Pattern Fragments server (TPF).
    Tpf,
    /// The in-process `sparq-engine` over a local `Graph`.
    LocalEngine,
}

/// The highest SPARQL query language the source advertises (`sd:supportedLanguage`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SparqlVersion {
    /// `sd:SPARQL10Query`.
    Sparql10,
    /// `sd:SPARQL11Query`.
    Sparql11,
}

/// How much of a `FILTER` a source can evaluate remotely. Phase 1 reads only the *language*
/// (SD has no per-filter vocabulary), so it is set coarsely from the SPARQL version: a full
/// SPARQL 1.1 endpoint gets [`FilterClass::Full`]. Per-operator narrowing (e.g. a brTPF
/// server that evaluates no FILTER) lands with the pushdown phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterClass {
    /// No FILTER can be pushed (e.g. plain TPF) — everything is filtered locally.
    None,
    /// The full SPARQL 1.1 FILTER expression grammar evaluates remotely.
    Full,
}

/// The bind-join mode the source supports — the block-at-a-time pushdown lever (§4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindJoin {
    /// A SPARQL endpoint: send a `VALUES` block of bindings in one sub-query.
    Values,
    /// A brTPF server: at most `n` mappings per request (`hydra:maxMpR`-style).
    MaxMpR(u32),
    /// No bind pushdown (plain TPF / unknown): the planner keeps the join local.
    None,
}

/// A result/RDF media type the source can RETURN (`sd:resultFormat`), as the W3C
/// `<https://www.w3.org/ns/formats/...>` format IRI. Kept as the raw IRI string so the set is
/// loss-lessly round-trippable; convenience predicates ([`MediaType::is_sparql_results_json`])
/// cover the formats the client actually negotiates.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MediaType(pub String);

impl MediaType {
    /// The W3C format IRI.
    pub fn iri(&self) -> &str {
        &self.0
    }
    /// Whether this is `SPARQL_Results_JSON` — the format the client's SRJ transport reads.
    pub fn is_sparql_results_json(&self) -> bool {
        self.0 == "http://www.w3.org/ns/formats/SPARQL_Results_JSON"
    }
}

/// What a source can do — the planner's capability input (design §4.1). Far richer than a
/// coarse "service-description? / totalItems?" runtime check: each field is a statically
/// resolved fact about the source, so the pushdown phase can decide *per operator* what to
/// send remotely without another round-trip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    /// The wire interface (always [`Interface::Endpoint`] from Phase-1 discovery).
    pub interface: Interface,
    /// The SPARQL query language advertised (`sd:supportedLanguage`), if any.
    pub sparql_version: Option<SparqlVersion>,
    /// Which FILTER expressions evaluate remotely.
    pub pushable_filters: FilterClass,
    /// The bind-join mode (VALUES / maxMpR(n) / none).
    pub bind_join: BindJoin,
    /// Whether SPARQL aggregates (`GROUP BY` / `COUNT` …) can be pushed.
    pub aggregates: bool,
    /// Whether property paths can be pushed.
    pub property_paths: bool,
    /// Whether `ORDER BY` / `LIMIT` can be pushed.
    pub order_limit: bool,
    /// Whether the source advertises `sd:SPARQL11Update`.
    pub update: bool,
    /// Whether the source advertises `sd:BasicFederatedQuery` (a SERVICE-capable endpoint).
    pub federated_query: bool,
    /// The result formats the source can return (`sd:resultFormat`), sorted + de-duplicated.
    pub result_formats: Vec<MediaType>,
}

impl Capability {
    /// The conservative capability assumed for an endpoint that **answers an `ASK` but
    /// publishes no Service Description** — the FedX-style fallback (§4.1). It claims full
    /// SPARQL 1.1 with VALUES bind-join (every SPARQL endpoint supports `VALUES`) and the
    /// SPARQL-Results-JSON result format (the protocol's mandatory format), but advertises
    /// **no** aggregates / property-paths / ORDER+LIMIT *push* claims beyond core evaluation —
    /// recall-safe: if a push turns out unsupported the planner keeps that operator local, it
    /// never produces a wrong answer. No statistics are invented (the caller pairs this with
    /// `None` for the [`SourceDescriptor`]).
    pub fn ask_fallback() -> Capability {
        Capability {
            interface: Interface::Endpoint,
            sparql_version: Some(SparqlVersion::Sparql11),
            pushable_filters: FilterClass::Full,
            bind_join: BindJoin::Values,
            aggregates: false,
            property_paths: false,
            order_limit: false,
            update: false,
            federated_query: false,
            result_formats: vec![MediaType(
                "http://www.w3.org/ns/formats/SPARQL_Results_JSON".to_string(),
            )],
        }
    }

    /// Whether the source can return SPARQL-Results-JSON (the client's SRJ transport format).
    pub fn returns_sparql_results_json(&self) -> bool {
        self.result_formats
            .iter()
            .any(MediaType::is_sparql_results_json)
    }
}

// ─── The Service-Description parser (THE net-new parser) ──────────────────────────────

/// Parses a SPARQL **Service Description** document (the N-Triples
/// `sparq-server/src/descriptors.rs::sd_ntriples` writes, or any standards-compliant SD) into
/// a [`Capability`].
///
/// Recognised triples (subject = the `sd:Service`):
///   * `sd:supportedLanguage` → [`Capability::sparql_version`] (SPARQL11 wins over 10) and
///     the [`Capability::update`] flag (`sd:SPARQL11Update`);
///   * `sd:resultFormat` → [`Capability::result_formats`] (sorted + de-duplicated);
///   * `sd:feature sd:BasicFederatedQuery` → [`Capability::federated_query`].
///
/// `pushable_filters` is set [`FilterClass::Full`] for a SPARQL 1.1 endpoint (SD has no
/// per-filter vocabulary; the pushdown phase narrows it). `bind_join` is [`BindJoin::Values`]
/// for a SPARQL endpoint (every endpoint evaluates a `VALUES` block).
///
/// Returns `Err` only on malformed N-Triples; an SD that simply omits a feature yields the
/// recall-safe "not advertised ⇒ not pushed" default for that field. Returns `Ok(None)` when
/// the document contains **no `sd:Service`** (so the caller falls back to an ASK probe).
pub fn parse_service_description(nt: &str) -> Result<Option<Capability>, String> {
    use oxrdf::{NamedOrBlankNode, Term as OxTerm};
    let sd = |local: &str| format!("{SD_NS}{local}");

    // Per-subject predicate→objects, so we can find the sd:Service subject and read its props.
    // The SD is small (a fixed shape), so a flat scan into a per-subject map is cheap.
    use rustc_hash::FxHashMap;
    let mut by_subject: FxHashMap<String, FxHashMap<String, Vec<OxTerm>>> = FxHashMap::default();
    // Subjects explicitly typed sd:Service.
    let mut service_subjects: Vec<String> = Vec::new();

    for r in oxttl::NTriplesParser::new().for_slice(nt.as_bytes()) {
        let t = r.map_err(|e| format!("malformed service-description N-Triples: {e}"))?;
        let subj = match &t.subject {
            NamedOrBlankNode::NamedNode(n) => n.as_str().to_string(),
            NamedOrBlankNode::BlankNode(b) => format!("_:{}", b.as_str()),
        };
        let pred = t.predicate.as_str().to_string();
        if pred == RDF_TYPE && iri_eq(&t.object, &sd("Service")) {
            service_subjects.push(subj.clone());
        }
        by_subject
            .entry(subj)
            .or_default()
            .entry(pred)
            .or_default()
            .push(t.object.clone());
    }

    // No sd:Service ⇒ this is not a Service Description (caller falls back to ASK).
    let Some(service) = service_subjects.first() else {
        return Ok(None);
    };
    let props = by_subject.get(service).expect("service subject collected");

    // --- supportedLanguage → version (+ update flag).
    let mut version: Option<SparqlVersion> = None;
    let mut update = false;
    if let Some(langs) = props.get(&sd("supportedLanguage")) {
        for l in langs {
            match l {
                OxTerm::NamedNode(n) if n.as_str() == SD_SPARQL11_QUERY => {
                    version = Some(SparqlVersion::Sparql11);
                }
                OxTerm::NamedNode(n)
                    if n.as_str() == SD_SPARQL10_QUERY
                        && version != Some(SparqlVersion::Sparql11) =>
                {
                    version = Some(SparqlVersion::Sparql10);
                }
                OxTerm::NamedNode(n)
                    if n.as_str()
                        == "http://www.w3.org/ns/sparql-service-description#SPARQL11Update" =>
                {
                    update = true;
                }
                _ => {}
            }
        }
    }

    // --- resultFormat → media types (sorted + de-duplicated for a deterministic set).
    let mut result_formats: Vec<MediaType> = props
        .get(&sd("resultFormat"))
        .into_iter()
        .flatten()
        .filter_map(|t| match t {
            OxTerm::NamedNode(n) => Some(MediaType(n.as_str().to_string())),
            _ => None,
        })
        .collect();
    result_formats.sort();
    result_formats.dedup();

    // --- feature sd:BasicFederatedQuery.
    let federated_query = props
        .get(&sd("feature"))
        .map(|fs| fs.iter().any(|t| iri_eq(t, SD_BASIC_FEDERATED_QUERY)))
        .unwrap_or(false);

    // A SPARQL endpoint evaluates a full FILTER grammar and a VALUES bind-join block; a
    // document with no supportedLanguage at all is treated as an endpoint with unknown
    // version (recall-safe: still full FILTER + VALUES, since it answered as a SPARQL service).
    let pushable_filters = match version {
        Some(SparqlVersion::Sparql11) | None => FilterClass::Full,
        // SPARQL 1.0 has FILTER too, so still Full; kept explicit for clarity.
        Some(SparqlVersion::Sparql10) => FilterClass::Full,
    };

    Ok(Some(Capability {
        interface: Interface::Endpoint,
        sparql_version: version,
        pushable_filters,
        bind_join: BindJoin::Values,
        // SD does not enumerate per-feature push-ability beyond the feature IRIs above; the
        // recall-safe default is "do not claim a push the SD did not promise". Aggregates /
        // property-paths / ORDER+LIMIT are core SPARQL 1.1 a full endpoint evaluates, so a
        // SPARQL11 service is allowed to take them; anything weaker keeps them local.
        aggregates: version == Some(SparqlVersion::Sparql11),
        property_paths: version == Some(SparqlVersion::Sparql11),
        order_limit: version == Some(SparqlVersion::Sparql11),
        update,
        federated_query,
        result_formats,
    }))
}

fn iri_eq(t: &oxrdf::Term, iri: &str) -> bool {
    matches!(t, oxrdf::Term::NamedNode(n) if n.as_str() == iri)
}

// ─── The network seam (SSRF-guarded fetch) ───────────────────────────────────────────

/// One HTTP GET, behind a trait so discovery is testable without a network. Implemented by
/// the production [`HttpFetcher`] (ureq + SSRF egress filter) and the in-memory
/// [`MapFetcher`] (tests / a pre-fetched document cache).
///
/// `accept` is the desired `Accept` header (discovery asks for `application/n-triples` so the
/// served descriptor comes back in the exact syntax the parsers consume). The implementation
/// returns the body on a 2xx, or an `Err` describing the failure (4xx/5xx/transport/SSRF).
pub trait Fetcher {
    /// GET `url` with the given `Accept`, returning the response body or an error string.
    fn get(&self, url: &str, accept: &str) -> Result<String, String>;
}

/// An in-memory fetcher mapping exact URLs → response bodies. Any URL not in the map yields a
/// `404`-style error (so a missing SD/VoID exercises the ASK-probe fallback path in tests).
#[derive(Debug, Default, Clone)]
pub struct MapFetcher {
    responses: rustc_hash::FxHashMap<String, String>,
}

impl MapFetcher {
    /// An empty map (every GET 404s) — models an endpoint that publishes no descriptors.
    pub fn new() -> MapFetcher {
        MapFetcher::default()
    }
    /// Registers `body` as the response for an exact `url`.
    pub fn with(mut self, url: impl Into<String>, body: impl Into<String>) -> MapFetcher {
        self.responses.insert(url.into(), body.into());
        self
    }
}

impl Fetcher for MapFetcher {
    fn get(&self, url: &str, _accept: &str) -> Result<String, String> {
        self.responses
            .get(url)
            .cloned()
            .ok_or_else(|| format!("MapFetcher: no response registered for {url} (simulated 404)"))
    }
}

/// Classifies a resolved [`IpAddr`](std::net::IpAddr) as a forbidden (private / internal /
/// non-global) destination for a discovery fetch — the SSRF default-deny set.
///
/// This **mirrors `sparq_engine::service::is_forbidden_ip`** (which is `pub(crate)` in the
/// engine, hence re-derived here rather than imported). Forbidden ranges: loopback
/// (`127/8`, `::1`); RFC1918 private (`10/8`, `172.16/12`, `192.168/16`); link-local
/// (`169.254/16`, incl. the `169.254.169.254` cloud-metadata IP, and IPv6 `fe80::/10`);
/// unique-local IPv6 (`fc00::/7`); unspecified (`0.0.0.0`, `::`); broadcast; CGNAT
/// (`100.64/10`). IPv4-mapped IPv6 (`::ffff:a.b.c.d`) is unwrapped and re-classified as the
/// embedded IPv4, so a private v4 cannot ride in through a v6 host literal.
#[cfg(not(target_arch = "wasm32"))]
pub fn is_forbidden_ip(ip: std::net::IpAddr) -> bool {
    use std::net::IpAddr;
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || matches!(v4.octets(), [100, b, ..] if (64..=127).contains(&b))
            // 100.64/10 CGNAT
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_forbidden_ip(IpAddr::V4(v4));
            }
            v6.is_loopback()
                || v6.is_unspecified()
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // fe80::/10 link-local
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7 unique-local
        }
    }
}

/// The production network fetcher: a blocking ureq GET behind the **same default-deny SSRF
/// egress filter the engine's SERVICE transport uses**. Native-only — neither ureq nor its
/// TLS stack ever enters the wasm bundle.
#[cfg(not(target_arch = "wasm32"))]
pub struct HttpFetcher {
    timeout: std::time::Duration,
    /// Hosts (bare authority, lower-cased) exempt from the SSRF filter — for a trusted
    /// internal federation endpoint that resolves privately.
    allow_private: std::sync::Arc<std::collections::HashSet<String>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for HttpFetcher {
    fn default() -> Self {
        HttpFetcher::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl HttpFetcher {
    /// A fetcher with a finite default timeout (so an unreachable endpoint cannot hang
    /// discovery) and the strict default-deny SSRF policy (no private host allowed).
    pub fn new() -> HttpFetcher {
        HttpFetcher {
            timeout: std::time::Duration::from_secs(15),
            allow_private: std::sync::Arc::new(std::collections::HashSet::new()),
        }
    }

    /// Sets the per-request timeout.
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> HttpFetcher {
        self.timeout = timeout;
        self
    }

    /// Exempts a trusted `host` (bare authority, e.g. `sparql.internal`) from the SSRF
    /// filter — its resolved addresses are dialled even when private. Use ONLY for a
    /// deliberately-trusted internal endpoint.
    pub fn allow_private_host(mut self, host: impl Into<String>) -> HttpFetcher {
        std::sync::Arc::make_mut(&mut self.allow_private).insert(host.into().to_ascii_lowercase());
        self
    }
}

/// ureq resolver enforcing the SSRF egress policy on the *resolved* addresses
/// (DNS-rebinding-safe): it resolves the host, drops every [`is_forbidden_ip`] address
/// (unless the bare host is on the allowlist), and returns only survivors — so ureq dials
/// only vetted IPs. Mirrors the engine's `EgressFilterResolver`.
#[cfg(not(target_arch = "wasm32"))]
struct EgressFilterResolver {
    allow_private: std::sync::Arc<std::collections::HashSet<String>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl ureq::Resolver for EgressFilterResolver {
    fn resolve(&self, netloc: &str) -> std::io::Result<Vec<std::net::SocketAddr>> {
        use std::net::ToSocketAddrs;
        // `netloc` is `host:port`; the allowlist is keyed by the bare host. rsplit on ':' to
        // keep IPv6 literals, which arrive bracketed (`[::1]:80`) — strip the brackets.
        let host = match netloc.rsplit_once(':') {
            Some((h, _)) => h,
            None => netloc,
        };
        let host = host.trim_start_matches('[').trim_end_matches(']');
        let allowed = self.allow_private.contains(&host.to_ascii_lowercase());
        let permitted: Vec<std::net::SocketAddr> = netloc
            .to_socket_addrs()?
            .filter(|sa| allowed || !is_forbidden_ip(sa.ip()))
            .collect();
        if permitted.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!(
                    "discovery egress refused: {netloc} resolves only to private/internal \
                     addresses (default-deny SSRF policy; allow it with \
                     HttpFetcher::allow_private_host)"
                ),
            ));
        }
        Ok(permitted)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Fetcher for HttpFetcher {
    fn get(&self, url: &str, accept: &str) -> Result<String, String> {
        let agent = ureq::AgentBuilder::new()
            .timeout(self.timeout)
            .user_agent(concat!("sparq-fedclient/", env!("CARGO_PKG_VERSION")))
            .resolver(EgressFilterResolver {
                allow_private: std::sync::Arc::clone(&self.allow_private),
            })
            .build();
        match agent.get(url).set("Accept", accept).call() {
            Ok(r) => r
                .into_string()
                .map_err(|e| format!("discovery: reading response from {url}: {e}")),
            Err(ureq::Error::Status(code, _)) => {
                Err(format!("discovery: {url} returned HTTP {code}"))
            }
            Err(e) => Err(format!("discovery: request to {url} failed: {e}")),
        }
    }
}

// ─── Orchestration: discover() ────────────────────────────────────────────────────────

/// The N-Triples `Accept` discovery asks for, so the served descriptor comes back in the
/// exact syntax the [`parse_service_description`] / [`SourceDescriptor::from_void_nt`]
/// parsers consume (rather than negotiating Turtle/RDF-XML and re-parsing).
const NT_ACCEPT: &str = "application/n-triples";

/// `ASK { ?s ?p ?o }` — the FedX-style reachability probe. A source that answers it (with the
/// SPARQL-Results-JSON `boolean`) is a live SPARQL endpoint, so the ASK-fallback capability
/// applies.
const ASK_PROBE: &str = "ASK { ?s ?p ?o }";

/// The outcome of discovering one source: its capability and (when published) its statistics.
#[derive(Debug, Clone)]
pub struct Discovery {
    /// What the source can do.
    pub capability: Capability,
    /// The source's statistics (VoID + `scs:`), `None` when it publishes no VoID document.
    pub descriptor: Option<SourceDescriptor>,
    /// How the capability was determined — useful for diagnostics / logging.
    pub provenance: Provenance,
}

/// How a [`Discovery`]'s capability was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// Parsed from a served SPARQL Service Description.
    ServiceDescription,
    /// No SD published; an `ASK` probe confirmed a live SPARQL endpoint (conservative caps).
    AskProbe,
}

/// Derives the `/.well-known/void` URL for a SPARQL endpoint URL. The well-known URI is
/// resolved against the endpoint's **authority** (scheme + `host[:port]`), per RFC 8615 — e.g.
/// `https://host/sparql` and `https://host/db/sparql` both map to
/// `https://host/.well-known/void`. Returns `None` if `endpoint` has no parseable authority.
pub fn well_known_void_url(endpoint: &str) -> Option<String> {
    let scheme_end = endpoint.find("://")?;
    let after = scheme_end + 3;
    let rest = &endpoint[after..];
    let host_len = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    if host_len == 0 {
        return None;
    }
    Some(format!(
        "{}/.well-known/void",
        &endpoint[..after + host_len]
    ))
}

/// Discovers a source's [`Capability`] + statistics from a SPARQL `endpoint` URL, using
/// `fetcher` for every network access (so the caller controls the SSRF policy and tests
/// inject canned documents).
///
/// Steps (design §4.1):
///   1. GET the **Service Description** (`endpoint` with no `query`) → [`Capability`] via
///      [`parse_service_description`]. If the SD is absent or empty, fall through to (3).
///   2. GET `<authority>/.well-known/void` → [`SourceDescriptor`] via the REUSED
///      [`SourceDescriptor::from_void_nt`] seam. A missing VoID leaves the descriptor `None`.
///   3. If no SD was parsed, **ASK-probe** the endpoint: a live SPARQL endpoint gets the
///      conservative [`Capability::ask_fallback`]; an endpoint that answers nothing is an
///      error.
///
/// The `SourceId` for the descriptor is the `endpoint` URL (the planner's stable key). This
/// function is the one-shot uncached primitive; a caller that discovers repeatedly should
/// cache the [`Discovery`] (design §4.1 — "`discover()` is cached so the hot path pays
/// nothing"), which the source-adapter phase wires.
pub fn discover(endpoint: &str, fetcher: &dyn Fetcher) -> Result<Discovery, String> {
    // --- (1) Service Description (GET the endpoint with no query).
    let capability = match fetcher.get(endpoint, NT_ACCEPT) {
        Ok(body) => parse_service_description(&body)?,
        // A transport/HTTP failure on the SD GET is non-fatal: many bare endpoints 400 a
        // no-query GET. Fall through to the ASK probe.
        Err(_) => None,
    };

    // --- (2) VoID statistics (REUSE from_void_nt). Best-effort: a missing/!parseable VoID
    // leaves the descriptor None (the planner uses its uniform fallback).
    let descriptor = well_known_void_url(endpoint).and_then(|void_url| {
        fetcher
            .get(&void_url, NT_ACCEPT)
            .ok()
            .and_then(|nt| SourceDescriptor::from_void_nt(SourceId::new(endpoint), &nt).ok())
    });

    // --- (3) Resolve the capability: SD if present, else an ASK-probe fallback.
    match capability {
        Some(cap) => Ok(Discovery {
            capability: cap,
            descriptor,
            provenance: Provenance::ServiceDescription,
        }),
        None => {
            // The ASK probe proves *reachability*, not contents: ANY well-formed boolean
            // answer (true OR false) means the endpoint evaluated the query and is a live
            // SPARQL service. Only a transport/HTTP/parse failure (the `?`) means unreachable.
            let _answered: bool = ask_probe(endpoint, fetcher).map_err(|e| {
                format!(
                    "discovery: {endpoint} published no Service Description and did not answer an \
                     ASK probe — not a reachable SPARQL endpoint ({e})"
                )
            })?;
            Ok(Discovery {
                capability: Capability::ask_fallback(),
                descriptor,
                provenance: Provenance::AskProbe,
            })
        }
    }
}

/// Runs the FedX-style `ASK { ?s ?p ?o }` reachability probe by POSTing the query — here
/// modelled as a GET with the query in the URL (`?query=...`), the simplest protocol form a
/// [`Fetcher`] can express. Returns the boolean answer, or an error if the endpoint is
/// unreachable or returns a non-ASK body.
fn ask_probe(endpoint: &str, fetcher: &dyn Fetcher) -> Result<bool, String> {
    let url = format!("{endpoint}?query={}", urlencode(ASK_PROBE));
    let body = fetcher
        .get(&url, "application/sparql-results+json")
        .map_err(|e| format!("discovery: ASK probe to {endpoint} failed: {e}"))?;
    parse_ask_boolean(&body)
}

/// Extracts the `boolean` field from a SPARQL-Results-JSON ASK response. Deliberately a tiny
/// tolerant scan rather than a JSON dependency — the only shape we need is `"boolean": true`.
fn parse_ask_boolean(body: &str) -> Result<bool, String> {
    // Find the "boolean" key and read the following true/false token.
    let key = body
        .find("\"boolean\"")
        .ok_or_else(|| "discovery: ASK response had no \"boolean\" field".to_string())?;
    let after = &body[key + "\"boolean\"".len()..];
    let after = after.trim_start();
    let after = after
        .strip_prefix(':')
        .ok_or_else(|| "discovery: malformed ASK response (no ':' after \"boolean\")".to_string())?
        .trim_start();
    if after.starts_with("true") {
        Ok(true)
    } else if after.starts_with("false") {
        Ok(false)
    } else {
        Err("discovery: ASK response \"boolean\" was neither true nor false".to_string())
    }
}

/// Minimal percent-encoding for a SPARQL query in a URL query string — encodes everything
/// outside the unreserved set so spaces / braces / `?` in the query are safe. Kept tiny
/// (no url crate) because the only caller is the fixed ASK probe.
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ─── Tests (parser + orchestration; NO public network) ────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// A served-shape Service Description (the exact N-Triples
    /// `sparq-server/src/descriptors.rs::sd_ntriples` writes for a read-only, federation-
    /// capable endpoint). The round-trip test parses THIS and asserts the capability matches.
    const SERVED_SD_NT: &str = r#"<http://host/sparql> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sparql-service-description#Service> .
<http://host/sparql> <http://www.w3.org/ns/sparql-service-description#endpoint> <http://host/sparql> .
<http://host/sparql> <http://www.w3.org/ns/sparql-service-description#supportedLanguage> <http://www.w3.org/ns/sparql-service-description#SPARQL11Query> .
<http://host/sparql> <http://www.w3.org/ns/sparql-service-description#resultFormat> <http://www.w3.org/ns/formats/SPARQL_Results_JSON> .
<http://host/sparql> <http://www.w3.org/ns/sparql-service-description#resultFormat> <http://www.w3.org/ns/formats/SPARQL_Results_XML> .
<http://host/sparql> <http://www.w3.org/ns/sparql-service-description#resultFormat> <http://www.w3.org/ns/formats/Turtle> .
<http://host/sparql> <http://www.w3.org/ns/sparql-service-description#inputFormat> <http://www.w3.org/ns/formats/Turtle> .
<http://host/sparql> <http://www.w3.org/ns/sparql-service-description#feature> <http://www.w3.org/ns/sparql-service-description#BasicFederatedQuery> .
<http://host/sparql> <http://www.w3.org/ns/sparql-service-description#defaultDataset> _:dataset .
_:dataset <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sparql-service-description#Dataset> .
_:dataset <http://www.w3.org/ns/sparql-service-description#defaultGraph> _:defaultGraph .
_:defaultGraph <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sparql-service-description#Graph> .
_:defaultGraph <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://rdfs.org/ns/void#Dataset> .
<http://host/sparql> <http://purl.org/dc/terms/source> <http://host/.well-known/void#dataset> .
"#;

    /// A served-shape VoID document (the `from_void_nt` consumer's input — one property + one
    /// class partition + a top-level triple count).
    const SERVED_VOID_NT: &str = r#"<http://host/.well-known/void#dataset> <http://rdfs.org/ns/void#triples> "300"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://host/.well-known/void#dataset> <http://rdfs.org/ns/void#propertyPartition> _:p1 .
_:p1 <http://rdfs.org/ns/void#property> <http://xmlns.com/foaf/0.1/knows> .
_:p1 <http://rdfs.org/ns/void#triples> "200"^^<http://www.w3.org/2001/XMLSchema#integer> .
_:p1 <http://rdfs.org/ns/void#distinctSubjects> "100"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://host/.well-known/void#dataset> <http://rdfs.org/ns/void#classPartition> _:c1 .
_:c1 <http://rdfs.org/ns/void#class> <http://xmlns.com/foaf/0.1/Person> .
_:c1 <http://rdfs.org/ns/void#entities> "100"^^<http://www.w3.org/2001/XMLSchema#integer> .
"#;

    // ---- The net-new Service-Description parser ------------------------------------

    #[test]
    fn parse_sd_extracts_full_capability() {
        let cap = parse_service_description(SERVED_SD_NT)
            .expect("well-formed SD must parse")
            .expect("document has an sd:Service");
        assert_eq!(cap.interface, Interface::Endpoint);
        assert_eq!(cap.sparql_version, Some(SparqlVersion::Sparql11));
        assert_eq!(cap.pushable_filters, FilterClass::Full);
        assert_eq!(cap.bind_join, BindJoin::Values);
        assert!(
            cap.federated_query,
            "feature sd:BasicFederatedQuery present"
        );
        assert!(!cap.update, "no SPARQL11Update advertised ⇒ read-only");
        // result formats are sorted + de-duplicated.
        assert!(cap.returns_sparql_results_json());
        assert_eq!(cap.result_formats.len(), 3);
        assert!(cap.result_formats.windows(2).all(|w| w[0] < w[1]), "sorted");
        // SPARQL 1.1 ⇒ aggregates / paths / order-limit pushable.
        assert!(cap.aggregates && cap.property_paths && cap.order_limit);
    }

    #[test]
    fn parse_sd_with_update_sets_flag() {
        let nt = format!(
            "{SERVED_SD_NT}<http://host/sparql> \
             <http://www.w3.org/ns/sparql-service-description#supportedLanguage> \
             <http://www.w3.org/ns/sparql-service-description#SPARQL11Update> .\n"
        );
        let cap = parse_service_description(&nt).unwrap().unwrap();
        assert!(cap.update, "SPARQL11Update advertised ⇒ update flag set");
        // Adding the Update language must not downgrade the query version.
        assert_eq!(cap.sparql_version, Some(SparqlVersion::Sparql11));
    }

    #[test]
    fn parse_sd_without_service_returns_none() {
        // A VoID document (no sd:Service) must NOT parse as a Service Description.
        let cap = parse_service_description(SERVED_VOID_NT).expect("well-formed N-Triples");
        assert!(cap.is_none(), "no sd:Service ⇒ not an SD ⇒ None");
    }

    #[test]
    fn parse_sd_malformed_is_error() {
        let err = parse_service_description("this is not <ntriples at all");
        assert!(err.is_err(), "malformed N-Triples must surface an error");
    }

    #[test]
    fn parse_sd_sparql10_only() {
        let nt = "<http://h/s> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sparql-service-description#Service> .\n<http://h/s> <http://www.w3.org/ns/sparql-service-description#supportedLanguage> <http://www.w3.org/ns/sparql-service-description#SPARQL10Query> .\n";
        let cap = parse_service_description(nt).unwrap().unwrap();
        assert_eq!(cap.sparql_version, Some(SparqlVersion::Sparql10));
        // SPARQL 1.0 ⇒ no aggregate/path/order-limit push (those are 1.1).
        assert!(!cap.aggregates && !cap.property_paths && !cap.order_limit);
        // FILTER still evaluates (1.0 has FILTER), and VALUES bind-join still applies.
        assert_eq!(cap.pushable_filters, FilterClass::Full);
    }

    // ---- The round-trip test (THE deliverable): served SD/VoID → Discovery -----------

    #[test]
    fn sd_round_trip_against_served_fixture() {
        // The endpoint serves the EXACT documents sparq-server writes, at the conventional
        // URLs: the SD at the endpoint URL (no query) and the VoID at /.well-known/void.
        let endpoint = "http://host/sparql";
        let fetcher = MapFetcher::new()
            .with(endpoint, SERVED_SD_NT)
            .with("http://host/.well-known/void", SERVED_VOID_NT);

        let d = discover(endpoint, &fetcher).expect("discovery against a served fixture");

        // Capability came from the SD.
        assert_eq!(d.provenance, Provenance::ServiceDescription);
        assert_eq!(d.capability.sparql_version, Some(SparqlVersion::Sparql11));
        assert!(d.capability.federated_query);
        assert!(d.capability.returns_sparql_results_json());

        // Statistics came from the REUSED from_void_nt seam — and match the server's writer.
        let desc = d.descriptor.expect("VoID was served ⇒ descriptor present");
        assert_eq!(desc.id(), &SourceId::new(endpoint));
        let p = desc
            .predicate("http://xmlns.com/foaf/0.1/knows")
            .expect("served property partition");
        assert_eq!(p.triples, 200);
        assert_eq!(p.distinct_subjects, 100);
        assert!(desc.has_class("http://xmlns.com/foaf/0.1/Person"));
    }

    // ---- ASK-probe fallback when nothing is published --------------------------------

    #[test]
    fn ask_fallback_when_no_descriptor_published() {
        // The endpoint serves no SD (no-query GET 404s in the map) and no VoID, but answers
        // the ASK probe affirmatively.
        let endpoint = "http://bare/sparql";
        let ask_url = format!("{endpoint}?query={}", urlencode(ASK_PROBE));
        let fetcher = MapFetcher::new().with(ask_url, r#"{ "head": {}, "boolean": true }"#);

        let d = discover(endpoint, &fetcher).expect("ASK-reachable endpoint discovers");
        assert_eq!(d.provenance, Provenance::AskProbe);
        // The conservative fallback capability.
        assert_eq!(d.capability.interface, Interface::Endpoint);
        assert_eq!(d.capability.bind_join, BindJoin::Values);
        assert_eq!(d.capability.pushable_filters, FilterClass::Full);
        assert!(d.capability.returns_sparql_results_json());
        // No descriptor invented.
        assert!(d.descriptor.is_none());
    }

    #[test]
    fn unreachable_endpoint_is_an_error() {
        // No SD, no VoID, and the ASK probe is not registered either (so it errors) ⇒ the
        // endpoint is not a reachable SPARQL service.
        let endpoint = "http://dead/sparql";
        let fetcher = MapFetcher::new();
        let err = discover(endpoint, &fetcher);
        assert!(err.is_err(), "an endpoint that answers nothing must error");
    }

    #[test]
    fn ask_probe_negative_boolean_still_reachable() {
        // An ASK that returns false STILL proves the endpoint is a live SPARQL service (it
        // evaluated the query) — so discovery succeeds with the fallback capability.
        let endpoint = "http://empty/sparql";
        let ask_url = format!("{endpoint}?query={}", urlencode(ASK_PROBE));
        let fetcher = MapFetcher::new().with(ask_url, r#"{ "boolean": false }"#);
        let d = discover(endpoint, &fetcher).expect("a false ASK still proves reachability");
        assert_eq!(d.provenance, Provenance::AskProbe);
    }

    // ---- ASK boolean parsing -------------------------------------------------------

    #[test]
    fn ask_boolean_parsing() {
        assert!(parse_ask_boolean(r#"{"boolean": true}"#).unwrap());
        assert!(!parse_ask_boolean(r#"{"boolean":false}"#).unwrap());
        assert!(parse_ask_boolean(r#"{ "head": {}, "boolean" : true }"#).unwrap());
        assert!(parse_ask_boolean(r#"{"results":{}}"#).is_err());
    }

    // ---- well-known VoID URL derivation --------------------------------------------

    #[test]
    fn well_known_void_url_derivation() {
        assert_eq!(
            well_known_void_url("http://host/sparql").as_deref(),
            Some("http://host/.well-known/void")
        );
        assert_eq!(
            well_known_void_url("https://host:8080/db/sparql").as_deref(),
            Some("https://host:8080/.well-known/void")
        );
        // No authority ⇒ None.
        assert_eq!(well_known_void_url("not-a-url"), None);
    }

    // ---- SSRF classifier (mirrors the engine's is_forbidden_ip) ---------------------

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn ssrf_forbids_private_and_metadata_ranges() {
        use std::net::IpAddr;
        let forbid = |s: &str| is_forbidden_ip(s.parse::<IpAddr>().unwrap());
        // Forbidden: loopback, RFC1918, link-local incl. cloud-metadata, CGNAT, ::1, ULA.
        assert!(forbid("127.0.0.1"));
        assert!(forbid("10.1.2.3"));
        assert!(forbid("172.16.5.5"));
        assert!(forbid("192.168.1.1"));
        assert!(forbid("169.254.169.254")); // cloud metadata
        assert!(forbid("100.64.0.1")); // CGNAT
        assert!(forbid("::1"));
        assert!(forbid("fc00::1")); // unique-local
        assert!(forbid("fe80::1")); // link-local v6
        assert!(forbid("::ffff:127.0.0.1")); // v4-mapped loopback can't smuggle through
                                             // Permitted: public addresses.
        assert!(!forbid("8.8.8.8"));
        assert!(!forbid("1.1.1.1"));
        assert!(!forbid("2606:4700:4700::1111"));
    }

    // ---- EgressFilterResolver::resolve (the ureq resolver that fronts every fetch) ----
    //
    // The classifier above proves WHICH addresses are forbidden; these prove the *resolver*
    // that ureq actually calls applies it correctly — including the host-authority parse
    // (`rsplit(':')`), the IPv6 bracket strip, and the `allow_private_host` allowlist bypass.
    // All cases use IP-LITERAL netlocs so `to_socket_addrs` is hermetic (no DNS lookup, no
    // network), so the tests are deterministic. [OPUS-4.8] sq-73om (follow-up to sq-nfxl).

    #[cfg(not(target_arch = "wasm32"))]
    fn resolver_denying() -> EgressFilterResolver {
        EgressFilterResolver {
            allow_private: std::sync::Arc::new(std::collections::HashSet::new()),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn resolver_allowing(host: &str) -> EgressFilterResolver {
        let mut set = std::collections::HashSet::new();
        set.insert(host.to_ascii_lowercase());
        EgressFilterResolver {
            allow_private: std::sync::Arc::new(set),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn resolve_refuses_private_v4_literal() {
        use ureq::Resolver;
        // A bare host:port whose ONLY resolved address is private must be refused outright —
        // ureq never sees a dialable address.
        let err = resolver_denying()
            .resolve("10.0.0.5:443")
            .expect_err("a private-only host must be refused by the egress resolver");
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn resolve_refuses_cloud_metadata_literal() {
        use ureq::Resolver;
        // The 169.254.169.254 cloud-metadata IP is the canonical SSRF target — refused.
        let err = resolver_denying()
            .resolve("169.254.169.254:80")
            .expect_err("cloud-metadata IP must be refused");
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn resolve_strips_ipv6_brackets_and_refuses_loopback() {
        use ureq::Resolver;
        // The IPv6 loopback arrives bracketed (`[::1]:80`); the resolver must strip the
        // brackets to recover the host, classify ::1 as forbidden, and refuse.
        let err = resolver_denying()
            .resolve("[::1]:80")
            .expect_err("bracketed IPv6 loopback must be refused");
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn resolve_permits_public_v4_literal() {
        use ureq::Resolver;
        // A public address survives the filter and is handed back to ureq to dial.
        let addrs = resolver_denying()
            .resolve("8.8.8.8:80")
            .expect("a public address must survive the egress filter");
        assert_eq!(addrs, vec!["8.8.8.8:80".parse().unwrap()]);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn resolve_permits_public_v6_literal() {
        use ureq::Resolver;
        // A public IPv6 literal (bracketed) is bracket-stripped, classified public, kept.
        let addrs = resolver_denying()
            .resolve("[2606:4700:4700::1111]:443")
            .expect("a public IPv6 address must survive");
        assert_eq!(addrs, vec!["[2606:4700:4700::1111]:443".parse().unwrap()]);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn resolve_allowlist_reopens_private_v4_literal() {
        use ureq::Resolver;
        // The allowlist bypass (`HttpFetcher::allow_private_host`): the SAME private literal
        // refused by the default-deny resolver is now dialled because the bare host is on the
        // allowlist. This is the only path that lets a private federation endpoint through.
        let addrs = resolver_allowing("10.0.0.5")
            .resolve("10.0.0.5:443")
            .expect("an allowlisted private host must be permitted");
        assert_eq!(addrs, vec!["10.0.0.5:443".parse().unwrap()]);
        // And it is exactly that host that opens it: a DIFFERENT allowlist entry does not.
        let err = resolver_allowing("192.168.0.1")
            .resolve("10.0.0.5:443")
            .expect_err("allowlisting a different host must not re-open 10.0.0.5");
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn resolve_allowlist_match_is_case_insensitive() {
        use ureq::Resolver;
        // `allow_private_host` lower-cases its key; the resolver lower-cases the parsed host,
        // so an IPv6 literal with upper-case hex digits still matches its allowlist entry.
        // (`fc00::AB` is unique-local ⇒ forbidden unless allowlisted.)
        let addrs = resolver_allowing("fc00::ab")
            .resolve("[fc00::AB]:80")
            .expect("case-insensitive allowlist match must permit the private host");
        assert_eq!(addrs, vec!["[fc00::AB]:80".parse().unwrap()]);
    }
}
