//! Shared ureq-3 SSRF-resolver helpers. [OPUS-4.8] sq-g2xs.
//!
//! ureq 3 (the migration off the unmaintained `rustls-pemfile`, RUSTSEC-2025-0134) changed the
//! [`Resolver`](ureq::unversioned::resolver::Resolver) trait: it now takes a parsed
//! [`http::Uri`](ureq::http::Uri) (plus the agent [`Config`](ureq::config::Config) and a
//! timeout) and returns an `ArrayVec<SocketAddr, 16>` ([`ResolvedSocketAddrs`]) rather than
//! ureq 2's `&str` netloc → `Vec<SocketAddr>`.
//!
//! The three sparq-fedclient transports (`source::HttpTransport`,
//! `source::HttpFragmentTransport`, `discovery::HttpFetcher`) install a custom resolver that
//! enforces the **same** default-deny SSRF policy: resolve the host, drop every private/internal
//! address (unless the bare host is explicitly allowlisted), and return only the survivors — so
//! ureq dials only vetted IPs and there is no DNS-rebinding re-resolve window. This module factors
//! the ureq-3 boilerplate (URI→host:port parse, the capacity-bounded filtered resolve, the refusal
//! error) into one place so those three transports share one implementation. Native-only; only
//! built when the `fedclient` feature pulls ureq in.
//!
//! The engine SERVICE resolver (`sparq-engine`'s `service.rs`) applies the *same* egress/SSRF
//! logic, but keeps its own equivalent inline copy of these helpers because `sparq-engine` must
//! not depend on `sparq-fedclient`. The two implementations are equivalent and unit-tested (not
//! externally security-audited); they do not share this module.

use std::net::IpAddr;
use std::net::ToSocketAddrs;

/// Capacity of ureq-3's [`ResolvedSocketAddrs`] (`ArrayVec<SocketAddr, 16>`); matches ureq's own
/// `MAX_ADDRS`. We never push more than this, so `ArrayVec::push` cannot overrun its array.
const RESOLVED_ADDRS_CAP: usize = 16;

/// `host:port` (for resolution) + the bare host (the allowlist key — IPv6 brackets stripped,
/// lowercased) + the numeric `port` from a ureq-3 request [`Uri`](ureq::http::Uri). `port` falls
/// back to the scheme default (443 for https, 80 otherwise). `None` when the URI carries no host
/// authority. The numeric port lets the SSRF resolver apply a PORT-SCOPED allowlist entry (a
/// `host:port` allowlist entry permits only that exact dialled port), consistent with the engine's
/// SERVICE egress guard. [OPUS-4.8] sq-vbnyc.
pub(crate) fn uri_host_port(uri: &ureq::http::Uri) -> Option<(String, String, u16)> {
    let authority = uri.authority()?;
    let host = authority.host();
    if host.is_empty() {
        return None;
    }
    let port = authority.port_u16().unwrap_or(match uri.scheme_str() {
        Some("https") => 443,
        _ => 80,
    });
    // The authority host keeps IPv6 brackets (`[::1]`); strip them for the allowlist key and for
    // `to_socket_addrs` (which wants the bare host + a separate port).
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    Some((format!("{bare}:{port}"), bare.to_ascii_lowercase(), port))
}

/// True iff any allowlist `entry` permits dialling `host` on `port`, using the engine's shared
/// SERVICE-egress per-entry rule ([`sparq_engine::allowlist_entry_permits`]). The federation
/// SSRF resolvers call this instead of a bare `allow.contains(host)`, so a PORT-SCOPED allowlist
/// entry (`host:port`) re-opens a private host ONLY on its exact dialled port — byte-for-byte the
/// same host:port matching the engine guard applies (one source of truth, bead sq-vbnyc). A
/// host-level entry (no `:port`) still re-opens every port (backward compatible). [OPUS-4.8].
pub(crate) fn allowlist_permits(
    allow: &std::collections::HashSet<String>,
    host: &str,
    port: u16,
) -> bool {
    allow
        .iter()
        .any(|entry| sparq_engine::allowlist_entry_permits(entry, host, port))
}

/// Wrap a refusal `reason` as a `PermissionDenied` [`ureq::Error::Io`], preserving the kind (the
/// resolver tests assert on it) and the reason text.
pub(crate) fn egress_refused(reason: String) -> ureq::Error {
    ureq::Error::Io(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        reason,
    ))
}

/// An empty [`ResolvedSocketAddrs`] backing store (a fixed-capacity `ArrayVec`; logical length 0,
/// the same idiom ureq's own `DefaultResolver::empty` uses).
fn empty_resolved() -> ureq::unversioned::resolver::ResolvedSocketAddrs {
    use std::net::{Ipv4Addr, SocketAddr};
    ureq::unversioned::resolver::ArrayVec::from_fn(|_| {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)
    })
}

/// Resolve `host_port`, keep only addresses that pass the SSRF policy, and return them as a
/// ureq-3 [`ResolvedSocketAddrs`]. An `allowed` host (on the resolver's allowlist) bypasses the
/// `forbidden` classifier; otherwise every `forbidden(ip)` address is dropped. If nothing
/// survives, refuse with a `PermissionDenied` error carrying `refusal()` — a HARD error, never an
/// empty set, so ureq cannot fall through to an unguarded dial.
pub(crate) fn filter_resolved(
    host_port: &str,
    allowed: bool,
    forbidden: fn(IpAddr) -> bool,
    refusal: impl FnOnce() -> String,
) -> Result<ureq::unversioned::resolver::ResolvedSocketAddrs, ureq::Error> {
    let resolved = host_port.to_socket_addrs().map_err(ureq::Error::Io)?;
    let mut permitted = empty_resolved();
    for sa in resolved
        .filter(|sa| allowed || !forbidden(sa.ip()))
        .take(RESOLVED_ADDRS_CAP)
    {
        permitted.push(sa);
    }
    if permitted.is_empty() {
        return Err(egress_refused(refusal()));
    }
    Ok(permitted)
}
