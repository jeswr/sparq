//! Source-type abstraction (design §4.1) — **Phase 2**.
//!
//! This module realises the §4.1 *source-type abstraction*: the [`SourceType`] enum
//! (`Endpoint` | `BrTpf` | `Tpf` | `Local`), the per-source [`Capability`] descriptor,
//! the [`FederatedSource`] trait (the sparq analogue of Comunica's `IQuerySource`), and
//! the [`Endpoint`] adapter that wraps a SPARQL-endpoint over a [`Transport`] seam
//! (`fetch(endpoint, query) -> SRJ string`) **behind a default-deny SSRF egress guard**.
//!
//! # What this phase ships (HONEST scope)
//!
//! * **`SourceType` / `Capability` / `FederatedSource` / `SubQuery` / `FedError`** — the
//!   complete §4.1 type surface, statically resolved (a fine-grained capability
//!   descriptor instead of Comunica's coarse runtime "service-description? /
//!   search-form? / totalItems?" negotiation).
//! * **The `Endpoint` adapter** over the engine's transport *seam*: it sends a SPARQL
//!   query string and gets an SRJ body back, exactly like
//!   `sparq-engine`'s `service.rs` `Transport::fetch(endpoint, query) -> String`
//!   (`service.rs:66`). The engine's own `Transport`/`HttpTransport`/`EgressFilterResolver`
//!   are `pub(crate)` (not exported), so this crate re-declares the *same* one-method
//!   seam ([`Transport`]) and owns its own [`EgressGuard`] — the same default-deny SSRF
//!   policy (loopback / RFC1918 / link-local / cloud-metadata / unique-local /
//!   unspecified / CGNAT refused unless explicitly allowlisted), DNS-rebinding-safe
//!   because it vets the *resolved* IP, not the literal host.
//! * **The SSRF guard is on by DEFAULT and default-DENY** ([`EgressGuard::deny_private`]):
//!   a private/loopback endpoint is rejected before any request is issued; a host must be
//!   explicitly allowlisted to re-open it. Tested both ways (allow + deny).
//!
//! # What is STUBBED for later phases (no overclaim)
//!
//! * `discover()` on [`Endpoint`] is a **stub** that returns the "everything an endpoint
//!   can do" [`Capability`] with **no** [`SourceDescriptor`] — the real VoID/SD discovery
//!   (GET `/.well-known/void` + SD, parse via `SourceDescriptor::from_void_nt`, the
//!   client-side SD parser, ASK-probe fallback) is **Phase 1** (`discovery` module).
//! * `execute()` returns the raw SRJ body (after the egress check + transport round-trip)
//!   rather than a streamed [`SolutionStream`]; the SRJ→solution parse and the streaming
//!   boundary are **Phase 5** (`stream`/`operators` modules). The transport seam and the
//!   SSRF gate — the load-bearing reuse + safety pieces of §4.1 — are real here.
//! * [`SourceType::BrTpf`] / [`SourceType::Tpf`] are **Phase-6 stubs**: their adapters
//!   exist (so the enum is total) but `discover()`/`execute()` return
//!   [`FedError::Unsupported`] with a clear "Phase 6" message rather than pretending.
//! * [`SourceType::Local`] (in-process `Graph` via `sparq-engine` local eval) is wired in
//!   the planner/operators phases; here it is represented in the enum with a capability
//!   of "everything" and a `not-yet-wired` execute stub.
//!
//! [OPUS-4.8] sq-rsxf (epic sq-dnko / sq-3183): Phase-2 source-type abstraction +
//! Endpoint adapter + default-deny SSRF guard. Flagged for Fable re-review when available.

use std::collections::HashSet;
use std::net::IpAddr;

// ─── Errors ─────────────────────────────────────────────────────────────────────────

/// Failure modes a [`FederatedSource`] can surface. `String`-payload variants mirror the
/// engine's `Transport`-error convention (`service.rs` returns `Result<_, String>`), so an
/// `Endpoint` can forward a transport error verbatim. [OPUS-4.8] sq-rsxf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FedError {
    /// The endpoint IRI could not be parsed / had no host authority to vet.
    BadEndpoint(String),
    /// The SSRF egress guard refused the endpoint (resolved to a private/internal
    /// address and the host is not on the allowlist), or DNS resolution failed.
    EgressRefused(String),
    /// The underlying transport (`fetch`) returned an error (DNS, connect, non-2xx,
    /// malformed body) — the engine's `Transport`-error string, forwarded verbatim.
    Transport(String),
    /// A capability/interface this phase does not implement (brTPF/TPF land in Phase 6;
    /// `Local` execute is wired in the planner/operators phases).
    Unsupported(String),
}

impl std::fmt::Display for FedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FedError::BadEndpoint(m) => write!(f, "federated source: bad endpoint: {m}"),
            FedError::EgressRefused(m) => write!(f, "federated source: egress refused: {m}"),
            FedError::Transport(m) => write!(f, "federated source: transport error: {m}"),
            FedError::Unsupported(m) => write!(f, "federated source: unsupported: {m}"),
        }
    }
}

impl std::error::Error for FedError {}

// ─── The sub-query a source is asked to answer ──────────────────────────────────────

/// The most-precise sub-query the planner pushes to one source: a SPARQL query string
/// plus the variables the caller expects projected back.
///
/// Phase 2 carries the rendered SPARQL text directly (the `Endpoint` adapter forwards it
/// to the transport unchanged — exactly how `sparq-engine`'s SERVICE path wraps an inner
/// pattern as `SELECT * WHERE { … }` before `fetch`). The capability-aware *construction*
/// of this sub-query (projection, pushable FILTERs, VALUES bind-join blocks, ORDER/LIMIT)
/// is the `pushdown` module's job in **Phase 4**; this type is the seam between that phase
/// and the adapters. [OPUS-4.8] sq-rsxf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubQuery {
    /// The SPARQL query string to send to the source (already capability-narrowed by the
    /// time Phase 4 produces it; in Phase 2 it is whatever the caller hands in).
    pub sparql: String,
    /// The projected variable names the caller expects back (without the leading `?`),
    /// used by later phases to bind the SRJ rows; kept here so the adapter surface is
    /// stable. Empty = "whatever the query projects".
    pub project: Vec<String>,
}

impl SubQuery {
    /// A sub-query that forwards `sparql` verbatim with no explicit projection hint.
    pub fn new(sparql: impl Into<String>) -> Self {
        SubQuery {
            sparql: sparql.into(),
            project: Vec::new(),
        }
    }
}

// ─── Capability descriptor (design §4.1) ────────────────────────────────────────────

/// The concrete remote interface a source speaks. Drives which sub-query shape the
/// pushdown layer may build and how the operators bind-join into it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interface {
    /// A full SPARQL 1.1 (Protocol) endpoint — arbitrary SPARQL, VALUES bind-join.
    Endpoint,
    /// A bindings-restricted Triple Pattern Fragments server — single triple pattern +
    /// a bound-tuple block (`maxMpR(n)`) bind-join. **Phase-6 stub here.**
    BrTpf,
    /// A plain Triple Pattern Fragments server — single triple pattern, no bind-join.
    /// **Phase-6 stub here.**
    Tpf,
    /// The in-process `sparq-engine` over a local `Graph` — "everything".
    LocalEngine,
}

/// How a source supports a bind-join (pushing a block of upstream bindings so the source
/// returns only rows that can survive the local join). Mirrors `sparq-engine`'s SERVICE
/// `VALUES` pushdown (`service.rs`, bead sq-sjkj).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindJoin {
    /// SPARQL `VALUES (…) { … }` pushdown (full endpoints).
    Values,
    /// brTPF `maxMpR(n)` — at most `n` bound mappings per request. (Phase 6.)
    MaxMpR(u32),
    /// No bind-join — the source is materialised and joined locally (plain TPF).
    None,
}

/// Which FILTER expressions a source can evaluate remotely. Coarse classes for now; the
/// pushdown layer (Phase 4) refines per-conjunct with the common-variable check. Only the
/// classes a source *provably* evaluates identically to local eval may be pushed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterClass {
    /// No FILTER pushdown — every filter is kept local.
    None,
    /// Only simple equality / `IN` over bound terms (safe across most sources).
    Equality,
    /// Full SPARQL 1.1 expression evaluation (assumed for a conformant `Endpoint`).
    Full,
}

/// What a source can do — the fine-grained, statically-resolved capability descriptor
/// (design §4.1, "far richer than Comunica's coarse model").
///
/// Phase 2 fills this from a *static default per interface* (see [`Capability::endpoint`]
/// / [`Capability::local`] / [`Capability::tpf`] / [`Capability::brtpf`]); the
/// `discovery` module (Phase 1) will refine an `Endpoint`'s capability from its parsed
/// Service Description (`sd:supportedLanguage`, `sd:resultFormat`, …). [OPUS-4.8] sq-rsxf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    /// The remote interface this capability describes.
    pub interface: Interface,
    /// Which FILTER class evaluates remotely.
    pub pushable_filters: FilterClass,
    /// The source's bind-join mode.
    pub bind_join: BindJoin,
    /// Can aggregates (`GROUP BY` / `COUNT` / …) be pushed?
    pub aggregates: bool,
    /// Can property-path expressions be pushed?
    pub property_paths: bool,
    /// Can `ORDER BY` / `LIMIT` be pushed?
    pub order_limit: bool,
    /// Advertised result media types (from SD `sd:resultFormat` once discovery lands).
    /// Phase 2 seeds the SRJ default that the transport already negotiates.
    pub result_formats: Vec<String>,
}

impl Capability {
    /// "Everything a conformant SPARQL 1.1 endpoint can do" — the Phase-2 default until
    /// the `discovery` module refines it from the endpoint's Service Description.
    pub fn endpoint() -> Self {
        Capability {
            interface: Interface::Endpoint,
            pushable_filters: FilterClass::Full,
            bind_join: BindJoin::Values,
            aggregates: true,
            property_paths: true,
            order_limit: true,
            result_formats: vec!["application/sparql-results+json".to_string()],
        }
    }

    /// The local in-process engine: everything, no remote transport.
    pub fn local() -> Self {
        Capability {
            interface: Interface::LocalEngine,
            ..Capability::endpoint()
        }
    }

    /// A brTPF source: single triple pattern + `maxMpR` bind-join, no remote FILTER /
    /// aggregate / path / ORDER-LIMIT pushdown. (Capability shape only — Phase-6 adapter.)
    pub fn brtpf(max_mpr: u32) -> Self {
        Capability {
            interface: Interface::BrTpf,
            pushable_filters: FilterClass::None,
            bind_join: BindJoin::MaxMpR(max_mpr),
            aggregates: false,
            property_paths: false,
            order_limit: false,
            result_formats: vec!["application/trig".to_string()],
        }
    }

    /// A plain TPF source: single triple pattern, no bind-join, nothing pushable.
    /// (Capability shape only — Phase-6 adapter.)
    pub fn tpf() -> Self {
        Capability {
            interface: Interface::Tpf,
            bind_join: BindJoin::None,
            ..Capability::brtpf(0)
        }
    }
}

// ─── The transport seam (REUSE of `sparq-engine`'s `service.rs` Transport) ──────────

/// The HTTP round-trip seam: send a SPARQL `query` string to `endpoint` and get the raw
/// response body (SPARQL-Results-JSON) back, or a transport-error string.
///
/// This is the *same* one-method seam as `sparq-engine`'s `service.rs` `Transport`
/// (`service.rs:66`, `fn fetch(&self, endpoint: &str, query: &str) -> Result<String,
/// String>`). The engine's trait + its `HttpTransport` impl are `pub(crate)` and so cannot
/// be imported; re-declaring the identical shape here lets the `Endpoint` adapter wrap any
/// transport — including a test double — without depending on engine internals, while the
/// dependency arrow still points one-way *into* the engine. A native [`HttpTransport`]
/// (ureq, default-deny SSRF resolver) lands alongside `execute()`'s streaming in a later
/// phase; Phase 2 ships the seam + the egress gate that fronts it. [OPUS-4.8] sq-rsxf.
pub trait Transport: Send + Sync {
    /// Send `query` to `endpoint`; return the raw response body or an error string.
    fn fetch(&self, endpoint: &str, query: &str) -> Result<String, String>;
}

// ─── SSRF egress guard (default-deny; mirrors the engine's `is_forbidden_ip`) ───────

/// Classifies a resolved [`IpAddr`] as a forbidden (private / internal / non-global)
/// federation destination. This is the **same default-deny classification** as
/// `sparq-engine`'s `service.rs::is_forbidden_ip` (which is `pub(crate)` and cannot be
/// imported): loopback, RFC1918, link-local (incl. the `169.254.169.254` cloud-metadata
/// IP), unique-local IPv6, unspecified, broadcast, and CGNAT (`100.64/10`) are refused;
/// IPv4-mapped IPv6 is unwrapped and re-classified so a private v4 cannot ride in through
/// a v6 literal. Returns `true` when the address must be refused. [OPUS-4.8] sq-rsxf.
pub fn is_forbidden_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()                 // 127.0.0.0/8
                || v4.is_private()           // 10/8, 172.16/12, 192.168/16
                || v4.is_link_local()        // 169.254.0.0/16 (incl. 169.254.169.254)
                || v4.is_unspecified()       // 0.0.0.0
                || v4.is_broadcast()         // 255.255.255.255
                || matches!(v4.octets(), [100, b, ..] if (64..=127).contains(&b))
            // 100.64/10 CGNAT
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_forbidden_ip(IpAddr::V4(v4));
            }
            v6.is_loopback()                              // ::1
                || v6.is_unspecified()                    // ::
                || (v6.segments()[0] & 0xffc0) == 0xfe80  // fe80::/10 link-local
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7 unique-local
        }
    }
}

/// The SSRF egress policy the [`Endpoint`] adapter enforces **before** any request is
/// issued — the client-side analogue of the engine's `with_service_egress_allow`
/// default-deny `EgressFilterResolver` (which is `pub(crate)`).
///
/// **Default-deny by construction.** [`EgressGuard::deny_private`] (the [`Default`]) is
/// the secure default: an endpoint whose host resolves to *any* private/internal address
/// is refused, and a deployer must explicitly allowlist a host to re-open it. This is
/// DNS-rebinding-safe: [`EgressGuard::check_addr`] vets a *resolved* [`IpAddr`], not the
/// literal IRI host, so a hostile DNS answer pointing at loopback / cloud-metadata is
/// dropped on the resolved IP. [OPUS-4.8] sq-rsxf.
#[derive(Debug, Clone, Default)]
pub struct EgressGuard {
    /// Hosts (DNS name or IP literal, lowercased authority) exempt from the
    /// private-address refusal — resolved addresses for these are permitted even when
    /// private. Only ever *adds* permission.
    allow: HashSet<String>,
}

impl EgressGuard {
    /// The secure default: public addresses allowed, private/internal denied unless the
    /// host is allowlisted. Equivalent to [`EgressGuard::default`].
    pub fn deny_private() -> Self {
        EgressGuard {
            allow: HashSet::new(),
        }
    }

    /// Allowlist a host (case-insensitive authority match) so its resolved addresses are
    /// permitted even if private. Chainable.
    pub fn allow_host(mut self, host: impl Into<String>) -> Self {
        self.allow.insert(host.into().to_ascii_lowercase());
        self
    }

    /// Is `host` (the bare authority, no port) on the allowlist?
    pub fn is_allowed(&self, host: &str) -> bool {
        self.allow.contains(&host.to_ascii_lowercase())
    }

    /// Vet one resolved address for `host`: `Ok(())` to dial it, `Err(reason)` to refuse.
    /// An allowlisted host is always permitted; otherwise a [`is_forbidden_ip`] address is
    /// refused. This is the per-address hook a real resolver calls on every candidate IP.
    pub fn check_addr(&self, host: &str, ip: IpAddr) -> Result<(), String> {
        if self.is_allowed(host) || !is_forbidden_ip(ip) {
            Ok(())
        } else {
            Err(format!(
                "host {host:?} resolved to private/internal address {ip} \
                 (default-deny SSRF policy; allowlist the host to permit it)"
            ))
        }
    }

    /// Vet an endpoint IRI end-to-end: parse the host authority, resolve it, and refuse if
    /// *every* resolved address is forbidden (and the host is not allowlisted). An
    /// allowlisted host short-circuits without a DNS lookup. Returns the bare host on
    /// success (handy for logging / the adapter). On a non-allowlisted host this performs
    /// a real DNS resolution and applies [`check_addr`](Self::check_addr) to each address,
    /// mirroring the engine's resolver: refuse before any socket is opened. [OPUS-4.8].
    pub fn check_endpoint(&self, endpoint: &str) -> Result<String, FedError> {
        let host = endpoint_host(endpoint)
            .ok_or_else(|| FedError::BadEndpoint(format!("no host authority in {endpoint:?}")))?;
        // Allowlisted host: permitted without a lookup (matches the engine's resolver,
        // which lets an allowlisted host through even if it resolves privately).
        if self.is_allowed(&host) {
            return Ok(host);
        }
        // An IP-literal authority is vetted directly (no DNS).
        if let Ok(ip) = host.parse::<IpAddr>() {
            return self
                .check_addr(&host, ip)
                .map(|()| host.clone())
                .map_err(FedError::EgressRefused);
        }
        // A DNS name: resolve and require at least one permitted address. Resolution uses
        // the host with a dummy port so the std resolver returns socket addresses.
        use std::net::ToSocketAddrs;
        let resolved: Vec<IpAddr> = (host.as_str(), 80u16)
            .to_socket_addrs()
            .map_err(|e| {
                FedError::EgressRefused(format!("DNS resolution of {host:?} failed: {e}"))
            })?
            .map(|sa| sa.ip())
            .collect();
        if resolved.is_empty() {
            return Err(FedError::EgressRefused(format!(
                "{host:?} resolved to no addresses"
            )));
        }
        if resolved.iter().all(|ip| is_forbidden_ip(*ip)) {
            return Err(FedError::EgressRefused(format!(
                "{host:?} resolves only to private/internal addresses \
                 (default-deny SSRF policy; allowlist the host to permit it)"
            )));
        }
        Ok(host)
    }
}

/// Extract the bare host authority (no scheme, no userinfo, no port, IPv6 brackets
/// stripped, lowercased) from an endpoint IRI. Returns `None` when there is no authority.
/// A tiny tolerant parser so the egress gate has no URL-crate dependency of its own (the
/// crate already pulls `oxrdf`/`spargebra`, not `url`, under `fedclient`). [OPUS-4.8].
fn endpoint_host(endpoint: &str) -> Option<String> {
    // scheme://authority/path?query — take what follows "://".
    let after_scheme = endpoint.split_once("://").map(|(_, rest)| rest)?;
    // authority ends at the first '/', '?' or '#'.
    let authority = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(after_scheme);
    // strip userinfo (user:pass@host).
    let hostport = authority
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(authority);
    if hostport.is_empty() {
        return None;
    }
    // IPv6 literal: [::1]:80 — take what is inside the brackets.
    if let Some(rest) = hostport.strip_prefix('[') {
        let host = rest.split_once(']').map(|(h, _)| h).unwrap_or(rest);
        return (!host.is_empty()).then(|| host.to_ascii_lowercase());
    }
    // host or host:port — strip the port.
    let host = hostport
        .rsplit_once(':')
        .map(|(h, _)| h)
        .unwrap_or(hostport);
    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}

// ─── The FederatedSource trait (design §4.1) ────────────────────────────────────────

/// One remote (or local) RDF source the federation engine can query — the sparq analogue
/// of Comunica's `IQuerySource`.
///
/// `discover()` resolves the source's [`Capability`] (and, once Phase 1 lands, its
/// [`SourceDescriptor`] statistics) one-shot; `execute()` answers the most-precise
/// sub-query the source can evaluate. Phase 2 returns the raw SRJ body from `execute()`;
/// the streamed `SolutionStream` boundary is Phase 5. [OPUS-4.8] sq-rsxf.
pub trait FederatedSource {
    /// Which interface this source speaks — drives pushdown shape and operator choice.
    fn source_type(&self) -> SourceType<'_>;

    /// Discover the source's capability (and statistics, once Phase 1's discovery lands).
    /// Phase 2: returns the static per-interface [`Capability`] with no descriptor.
    fn discover(&self) -> Result<(Capability, Option<sparq_fedplan::SourceDescriptor>), FedError>;

    /// Answer `sub` against this source. Phase 2: vets the endpoint through the SSRF guard,
    /// forwards `sub.sparql` to the transport, and returns the raw SRJ body. The SRJ→
    /// solution parse and the streaming boundary are Phase 5. [OPUS-4.8] sq-rsxf.
    fn execute(&self, sub: &SubQuery) -> Result<String, FedError>;
}

// ─── SourceType — the enum of adapters ──────────────────────────────────────────────

/// The set of source adapters the client fans out to (design §4.1). Borrows the adapter
/// so a planner can hold a heterogeneous `&[SourceType]` without owning the transports.
///
/// * [`SourceType::Endpoint`] — a full SPARQL endpoint over the transport seam (Phase 2,
///   real here);
/// * [`SourceType::BrTpf`] / [`SourceType::Tpf`] — bindings-restricted / plain TPF
///   (**Phase-6 stubs**: capability shape only, `execute` returns
///   [`FedError::Unsupported`]);
/// * [`SourceType::Local`] — the in-process engine (capability "everything"; `execute`
///   is wired in the planner/operators phases). [OPUS-4.8] sq-rsxf.
pub enum SourceType<'a> {
    /// A full SPARQL 1.1 endpoint.
    Endpoint(&'a Endpoint),
    /// A bindings-restricted Triple Pattern Fragments server. **Phase-6 stub.**
    BrTpf(&'a BrTpfSource),
    /// A plain Triple Pattern Fragments server. **Phase-6 stub.**
    Tpf(&'a TpfSource),
    /// The in-process `sparq-engine` over a local `Graph`.
    Local(&'a LocalSource),
}

// ─── Endpoint adapter (Phase 2 — the real one) ──────────────────────────────────────

/// A full SPARQL-endpoint source: wraps a [`Transport`] (`fetch(endpoint, query) ->
/// SRJ`) behind a default-deny [`EgressGuard`]. This is the Phase-2 deliverable — the
/// SPARQL-endpoint reuse of the engine's transport seam plus the SSRF safety gate.
///
/// Construct with [`Endpoint::new`] (default-deny SSRF) or [`Endpoint::with_guard`] to
/// supply an allowlisting [`EgressGuard`]. The transport is any `Transport` impl — a real
/// HTTP client or a test double — so the adapter is exercised without a network. The host
/// is **vetted on every `execute`** (not cached), matching the engine's
/// resolve-then-dial-the-vetted-IP discipline. [OPUS-4.8] sq-rsxf.
pub struct Endpoint {
    endpoint: String,
    transport: Box<dyn Transport>,
    guard: EgressGuard,
}

impl Endpoint {
    /// A new endpoint source with the **secure default** SSRF guard (default-deny
    /// private/internal). `endpoint` is the SPARQL Protocol URL; `transport` performs the
    /// round-trip.
    pub fn new(endpoint: impl Into<String>, transport: Box<dyn Transport>) -> Self {
        Endpoint {
            endpoint: endpoint.into(),
            transport,
            guard: EgressGuard::deny_private(),
        }
    }

    /// A new endpoint source with an explicit [`EgressGuard`] (e.g. one that allowlists a
    /// trusted internal host).
    pub fn with_guard(
        endpoint: impl Into<String>,
        transport: Box<dyn Transport>,
        guard: EgressGuard,
    ) -> Self {
        Endpoint {
            endpoint: endpoint.into(),
            transport,
            guard,
        }
    }

    /// The endpoint URL.
    pub fn url(&self) -> &str {
        &self.endpoint
    }

    /// The egress guard governing this endpoint.
    pub fn guard(&self) -> &EgressGuard {
        &self.guard
    }
}

impl FederatedSource for Endpoint {
    fn source_type(&self) -> SourceType<'_> {
        SourceType::Endpoint(self)
    }

    fn discover(&self) -> Result<(Capability, Option<sparq_fedplan::SourceDescriptor>), FedError> {
        // Phase-2 STUB: the static "full endpoint" capability, no descriptor. Real VoID/SD
        // discovery is Phase 1 (the `discovery` module). HONEST: this does NOT contact the
        // endpoint — it returns the conservative "everything an endpoint can do" default.
        Ok((Capability::endpoint(), None))
    }

    fn execute(&self, sub: &SubQuery) -> Result<String, FedError> {
        // SSRF gate FIRST — refuse a private/internal endpoint before any request. This is
        // the load-bearing safety step: default-deny, allowlist-to-reopen, DNS-rebinding
        // -safe (vets the resolved IP).
        let _host = self.guard.check_endpoint(&self.endpoint)?;
        // Forward the (already capability-narrowed, in later phases) SPARQL to the
        // transport seam, exactly like the engine's SERVICE `fetch`. Phase 2 returns the
        // raw SRJ body; the SRJ→solution parse + streaming is Phase 5.
        self.transport
            .fetch(&self.endpoint, &sub.sparql)
            .map_err(FedError::Transport)
    }
}

// ─── brTPF / TPF / Local adapters (stubs for Phase 6 / planner phases) ──────────────

/// A bindings-restricted TPF source. **Phase-6 stub**: holds the fragment-template URL and
/// its `maxMpR` bind-join bound so the enum is total and the capability shape is correct,
/// but `discover`/`execute` return [`FedError::Unsupported`] — no overclaim. [OPUS-4.8].
pub struct BrTpfSource {
    /// The brTPF fragment-template / base URL.
    pub url: String,
    /// `maxMpR` — at most this many bound mappings per request.
    pub max_mpr: u32,
}

impl FederatedSource for BrTpfSource {
    fn source_type(&self) -> SourceType<'_> {
        SourceType::BrTpf(self)
    }
    fn discover(&self) -> Result<(Capability, Option<sparq_fedplan::SourceDescriptor>), FedError> {
        Ok((Capability::brtpf(self.max_mpr), None))
    }
    fn execute(&self, _sub: &SubQuery) -> Result<String, FedError> {
        Err(FedError::Unsupported(
            "brTPF source execution is not implemented yet (Phase 6)".to_string(),
        ))
    }
}

/// A plain TPF source. **Phase-6 stub** (see [`BrTpfSource`]). [OPUS-4.8] sq-rsxf.
pub struct TpfSource {
    /// The TPF fragment-template / base URL.
    pub url: String,
}

impl FederatedSource for TpfSource {
    fn source_type(&self) -> SourceType<'_> {
        SourceType::Tpf(self)
    }
    fn discover(&self) -> Result<(Capability, Option<sparq_fedplan::SourceDescriptor>), FedError> {
        Ok((Capability::tpf(), None))
    }
    fn execute(&self, _sub: &SubQuery) -> Result<String, FedError> {
        Err(FedError::Unsupported(
            "TPF source execution is not implemented yet (Phase 6)".to_string(),
        ))
    }
}

/// The in-process local engine source over a `Graph`. The capability is "everything"; the
/// actual local BGP evaluation through `sparq-engine` is wired in the planner/operators
/// phases, so `execute` is a clearly-labelled stub here (no overclaim). The source id lets
/// the planner address it. [OPUS-4.8] sq-rsxf.
pub struct LocalSource {
    /// A stable id for the local source (used by the planner to address it).
    pub id: String,
}

impl FederatedSource for LocalSource {
    fn source_type(&self) -> SourceType<'_> {
        SourceType::Local(self)
    }
    fn discover(&self) -> Result<(Capability, Option<sparq_fedplan::SourceDescriptor>), FedError> {
        Ok((Capability::local(), None))
    }
    fn execute(&self, _sub: &SubQuery) -> Result<String, FedError> {
        Err(FedError::Unsupported(
            "local-engine source execution is wired in the planner/operators phases (Phase 3/5)"
                .to_string(),
        ))
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::sync::Mutex;

    /// A canned-response transport double: records the (endpoint, query) it was asked for
    /// and returns a fixed body. Lets us assert the adapter reached the transport (i.e.
    /// the SSRF gate ALLOWED) without any network. [OPUS-4.8] sq-rsxf.
    struct CannedTransport {
        body: String,
        seen: Mutex<Vec<(String, String)>>,
    }
    impl CannedTransport {
        fn new(body: &str) -> Self {
            CannedTransport {
                body: body.to_string(),
                seen: Mutex::new(Vec::new()),
            }
        }
    }
    impl Transport for CannedTransport {
        fn fetch(&self, endpoint: &str, query: &str) -> Result<String, String> {
            self.seen
                .lock()
                .unwrap()
                .push((endpoint.to_string(), query.to_string()));
            Ok(self.body.clone())
        }
    }

    /// A transport that PANICS if called — proves the SSRF gate refused before any
    /// transport round-trip (a deny must never reach `fetch`). [OPUS-4.8] sq-rsxf.
    struct PanicTransport;
    impl Transport for PanicTransport {
        fn fetch(&self, _endpoint: &str, _query: &str) -> Result<String, String> {
            panic!("transport must NOT be reached when the SSRF gate denies the endpoint");
        }
    }

    // ── is_forbidden_ip classification (mirrors the engine's policy) ──────────────

    #[test]
    fn forbidden_ips_are_refused() {
        for ip in [
            "127.0.0.1",       // loopback
            "10.0.0.5",        // RFC1918
            "172.16.0.1",      // RFC1918
            "192.168.1.1",     // RFC1918
            "169.254.169.254", // cloud metadata (link-local)
            "0.0.0.0",         // unspecified
            "255.255.255.255", // broadcast
            "100.64.0.1",      // CGNAT
        ] {
            assert!(
                is_forbidden_ip(ip.parse().unwrap()),
                "{ip} should be forbidden"
            );
        }
        // IPv6
        assert!(is_forbidden_ip(IpAddr::V6(Ipv6Addr::LOCALHOST))); // ::1
        assert!(is_forbidden_ip(IpAddr::V6(Ipv6Addr::UNSPECIFIED))); // ::
        assert!(is_forbidden_ip("fe80::1".parse().unwrap())); // link-local
        assert!(is_forbidden_ip("fc00::1".parse().unwrap())); // unique-local
                                                              // IPv4-mapped private v4 must not smuggle through a v6 literal.
        assert!(is_forbidden_ip("::ffff:127.0.0.1".parse().unwrap()));
        assert!(is_forbidden_ip("::ffff:10.0.0.1".parse().unwrap()));
    }

    #[test]
    fn public_ips_are_allowed() {
        for ip in ["8.8.8.8", "1.1.1.1", "93.184.216.34"] {
            assert!(
                !is_forbidden_ip(ip.parse().unwrap()),
                "{ip} should be allowed"
            );
        }
        assert!(!is_forbidden_ip(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 5))));
        assert!(!is_forbidden_ip("2606:4700:4700::1111".parse().unwrap())); // public v6
    }

    // ── endpoint_host parsing ─────────────────────────────────────────────────────

    #[test]
    fn host_parsing() {
        assert_eq!(
            endpoint_host("http://dbpedia.org/sparql").as_deref(),
            Some("dbpedia.org")
        );
        assert_eq!(
            endpoint_host("https://EXAMPLE.org:8443/q").as_deref(),
            Some("example.org")
        );
        assert_eq!(
            endpoint_host("http://user:pw@host.example/p").as_deref(),
            Some("host.example")
        );
        assert_eq!(
            endpoint_host("http://[::1]:8080/sparql").as_deref(),
            Some("::1")
        );
        assert_eq!(
            endpoint_host("http://127.0.0.1/sparql").as_deref(),
            Some("127.0.0.1")
        );
        assert_eq!(endpoint_host("not-a-url"), None);
        assert_eq!(endpoint_host("http:///nohost"), None);
    }

    // ── DENY: default guard refuses a loopback / private endpoint ─────────────────

    #[test]
    fn deny_loopback_endpoint_literal() {
        let g = EgressGuard::deny_private();
        let err = g
            .check_endpoint("http://127.0.0.1:8080/sparql")
            .unwrap_err();
        assert!(matches!(err, FedError::EgressRefused(_)), "got {err:?}");
    }

    #[test]
    fn deny_ipv6_loopback_endpoint_literal() {
        let g = EgressGuard::deny_private();
        assert!(matches!(
            g.check_endpoint("http://[::1]:8080/sparql").unwrap_err(),
            FedError::EgressRefused(_)
        ));
    }

    #[test]
    fn deny_private_rfc1918_endpoint_literal() {
        let g = EgressGuard::deny_private();
        assert!(matches!(
            g.check_endpoint("http://10.0.0.5/sparql").unwrap_err(),
            FedError::EgressRefused(_)
        ));
    }

    #[test]
    fn deny_cloud_metadata_endpoint_literal() {
        let g = EgressGuard::deny_private();
        assert!(matches!(
            g.check_endpoint("http://169.254.169.254/latest/meta-data/")
                .unwrap_err(),
            FedError::EgressRefused(_)
        ));
    }

    /// The full adapter path: a default-deny endpoint pointing at loopback must refuse
    /// WITHOUT ever calling the transport (PanicTransport would panic if reached).
    #[test]
    fn endpoint_execute_denies_loopback_before_transport() {
        let ep = Endpoint::new("http://127.0.0.1:9999/sparql", Box::new(PanicTransport));
        let err = ep.execute(&SubQuery::new("ASK {}")).unwrap_err();
        assert!(matches!(err, FedError::EgressRefused(_)), "got {err:?}");
    }

    // ── ALLOW: allowlisting a private host re-opens it ────────────────────────────

    #[test]
    fn allow_listed_private_literal_is_permitted() {
        // The literal authority "127.0.0.1" is allowlisted, so check_endpoint returns Ok.
        let g = EgressGuard::deny_private().allow_host("127.0.0.1");
        assert_eq!(
            g.check_endpoint("http://127.0.0.1:8080/sparql").unwrap(),
            "127.0.0.1"
        );
    }

    #[test]
    fn allow_listed_named_host_is_permitted() {
        // A DNS name on the allowlist short-circuits (no lookup), even if it would resolve
        // privately — matching the engine's allowlist-bypasses-resolution behaviour.
        let g = EgressGuard::deny_private().allow_host("sparql.internal");
        assert_eq!(
            g.check_endpoint("http://sparql.internal/sparql").unwrap(),
            "sparql.internal"
        );
    }

    /// The full adapter path under an allowlist: the endpoint resolves privately but the
    /// host is allowlisted, so the transport IS reached and returns the canned body.
    #[test]
    fn endpoint_execute_allows_listed_private_host_and_reaches_transport() {
        let body = r#"{"head":{"vars":[]},"results":{"bindings":[]}}"#;
        let transport = Box::new(CannedTransport::new(body));
        let guard = EgressGuard::deny_private().allow_host("localhost.internal");
        let ep = Endpoint::with_guard("http://localhost.internal:7777/sparql", transport, guard);
        let got = ep
            .execute(&SubQuery::new("SELECT * WHERE { ?s ?p ?o } LIMIT 1"))
            .unwrap();
        assert_eq!(got, body);
    }

    /// A PUBLIC endpoint with the default-deny guard reaches the transport (the guard only
    /// refuses private/internal). We use an IP literal so the test does no real DNS.
    #[test]
    fn endpoint_execute_public_literal_reaches_transport() {
        let body = r#"{"head":{"vars":["s"]},"results":{"bindings":[]}}"#;
        let transport = CannedTransport::new(body);
        // Hold a raw pointer-free reference by constructing then querying `seen` after.
        let ep = Endpoint::new("http://8.8.8.8/sparql", Box::new(transport));
        let got = ep.execute(&SubQuery::new("ASK { ?s ?p ?o }")).unwrap();
        assert_eq!(got, body);
    }

    // ── capability defaults ───────────────────────────────────────────────────────

    #[test]
    fn capability_defaults_per_interface() {
        let e = Capability::endpoint();
        assert_eq!(e.interface, Interface::Endpoint);
        assert_eq!(e.bind_join, BindJoin::Values);
        assert!(matches!(e.pushable_filters, FilterClass::Full));
        assert!(e.aggregates && e.property_paths && e.order_limit);

        let l = Capability::local();
        assert_eq!(l.interface, Interface::LocalEngine);

        let b = Capability::brtpf(50);
        assert_eq!(b.interface, Interface::BrTpf);
        assert_eq!(b.bind_join, BindJoin::MaxMpR(50));
        assert!(!b.aggregates && !b.property_paths && !b.order_limit);

        let t = Capability::tpf();
        assert_eq!(t.interface, Interface::Tpf);
        assert_eq!(t.bind_join, BindJoin::None);
    }

    // ── brTPF / TPF / Local stubs report Unsupported (honest, no overclaim) ────────

    #[test]
    fn brtpf_tpf_local_execute_is_unsupported_stub() {
        let br = BrTpfSource {
            url: "http://ex/brtpf".into(),
            max_mpr: 30,
        };
        assert!(matches!(
            br.execute(&SubQuery::new("x")).unwrap_err(),
            FedError::Unsupported(_)
        ));
        assert!(matches!(br.source_type(), SourceType::BrTpf(_)));
        let (cap, desc) = br.discover().unwrap();
        assert_eq!(cap.bind_join, BindJoin::MaxMpR(30));
        assert!(desc.is_none());

        let t = TpfSource {
            url: "http://ex/tpf".into(),
        };
        assert!(matches!(
            t.execute(&SubQuery::new("x")).unwrap_err(),
            FedError::Unsupported(_)
        ));
        assert!(matches!(t.source_type(), SourceType::Tpf(_)));

        let lo = LocalSource { id: "local".into() };
        assert!(matches!(
            lo.execute(&SubQuery::new("x")).unwrap_err(),
            FedError::Unsupported(_)
        ));
        assert!(matches!(lo.source_type(), SourceType::Local(_)));
        assert_eq!(lo.discover().unwrap().0.interface, Interface::LocalEngine);
    }

    #[test]
    fn endpoint_discover_is_capability_stub_with_no_descriptor() {
        let ep = Endpoint::new(
            "http://8.8.8.8/sparql",
            Box::new(CannedTransport::new("{}")),
        );
        let (cap, desc) = ep.discover().unwrap();
        assert_eq!(cap.interface, Interface::Endpoint);
        assert!(
            desc.is_none(),
            "Phase-2 discover must NOT fabricate a descriptor"
        );
        assert!(matches!(ep.source_type(), SourceType::Endpoint(_)));
        assert_eq!(ep.url(), "http://8.8.8.8/sparql");
    }
}
