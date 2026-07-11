// AUTHORED-BY Claude Opus 4.8
//! Per-source rate limiting — a PRE-CRYPTO per-IP token bucket that rejects abusive traffic with a
//! **429** *before* the expensive DPoP signature verification ever runs.
//!
//! ## Why (the cost problem this solves)
//! The single most expensive thing this server does per request is the asymmetric DPoP/JWS signature
//! verification (an ES256 verify is ~hundreds of µs). [`crate::overload`] already caps GLOBAL
//! concurrency — but that is not enough: a single source can BOTH eat admission slots AND force a full
//! crypto verify for every bogus proof it sends, making attacker traffic cost the server its most
//! expensive operation per request. This layer adds a **per-IP token bucket OUTSIDE auth/WAC/crypto**:
//! a source that exceeds its rate gets a cheap 429 and its request NEVER reaches the verifier, so a
//! flood from one IP cannot make every bogus proof pay the crypto cost.
//!
//! ## Where it sits (the security-critical ordering)
//! Like admission control, this layer wraps the APP routes only (NOT the health routes) and runs
//! OUTSIDE auth — a rate-limited request is rejected (429) before authentication. This is correct AND
//! a security property, but the reasoning is the MIRROR of admission control's, so be explicit:
//!
//! - 🔒 **This layer only REJECTS EARLIER. It NEVER weakens auth/WAC.** A 429 grants strictly LESS
//!   access than the request would otherwise have gotten — it can never turn an unauthorized request
//!   into a success, never grant access, never let through a request the verifier would reject. A
//!   rate-limited request gets 429 and never reaches auth. (Compare [`crate::overload`]'s 503: same
//!   fail-safe-by-construction argument.)
//! - 🔒 **Fail-OPEN stance for the LIMITER ITSELF.** A limiter bug — or the (should-never-happen) case
//!   where [`ConnectInfo`] is absent from the request extensions — must NOT deny all traffic. So when
//!   the peer IP cannot be determined, the request is allowed to PROCEED to the normal auth stack,
//!   which still gates it. **Fail-open here means "proceed to auth, which still gates the request",
//!   NEVER "bypass auth".** The limiter is a cheap front door that can only ever turn a request away
//!   early; it has zero authority to admit one. So failing open costs only the rate-limit protection
//!   for that request, never any authorization guarantee.
//! - **Health/readiness endpoints are EXEMPT.** They are mounted OUTSIDE this layer (see
//!   [`crate::app::build_router_with_overload`]), so they are already exempt. As defence-in-depth, the
//!   middleware ALSO skips `/livez` + `/readyz` by path in case the layer is ever applied more broadly
//!   — a readiness probe must never be rate-limited (rate-limiting a healthy instance's probe would
//!   make a load balancer pull it).
//!
//! ## The bucket (hand-rolled, dependency-free — a deliberate choice)
//! A textbook per-IP **token bucket**: each source IP has a bucket holding up to `burst` tokens that
//! refill at `rate` tokens/second; a request costs one token; an empty bucket ⇒ 429. We hand-roll it
//! rather than pull in `governor` ON PURPOSE: `governor` is the reputable, widely-used standard Rust
//! rate-limiter, but it transitively adds ~15 crates (a second `getrandom 0.3`/`rand 0.9` line,
//! `quanta`, `futures-timer`, `spinning_top`, `web-time`, …) — a meaningful expansion of the
//! audit surface for a SECURITY-CRITICAL layer whose whole job is a ~50-line algorithm. The house rule
//! is a **minimal, audit-small surface** for security-critical paths; a hand-rolled bucket adds ZERO
//! new crates, reuses the SAME `getrandom 0.2` jitter source as [`crate::overload`], and matches its
//! house style (atomics, pure parse cores, exhaustive tests, fail-safe constants) exactly. The state
//! lives in a SHARDED set of `Mutex<HashMap<IpAddr, Bucket>>` so per-IP lock contention is bounded
//! without a concurrent-map dependency; each critical section is tiny (a map lookup + arithmetic, no
//! I/O, no lock held across an `.await`).
//!
//! ## Sizing + the internal-IP exemption — the default MUST NOT trip the conformance run or normal use
//! The default per-IP rate is DELIBERATELY GENEROUS (see [`DEFAULT_RATE_PER_IP`] /
//! [`DEFAULT_BURST`]) — high enough that it never sheds normal use. More importantly, **INTERNAL
//! source IPs are EXEMPT by default** ([`ENV_EXEMPT_INTERNAL`] / [`is_internal_ip`]: loopback + RFC
//! 1918 private + link-local + IPv6 ULA). This is BOTH a footgun guard and the conformance fix:
//! - **Footgun guard:** behind a reverse proxy / docker-bridge / k8s service without a configured
//!   [`ENV_TRUSTED_PROXY`], EVERY client shares ONE internal hop IP, so a per-IP bucket keyed on that
//!   hop is meaningless (it throttles all clients together). Exempting internal IPs means a
//!   misconfigured internal hop is simply not rate-limited (it proceeds to auth, which still gates it)
//!   rather than mis-throttling everyone onto one bucket.
//! - **Conformance:** the CTH reaches the server via a `--network host` socat sidecar forwarding from
//!   `host.docker.internal` — a NON-loopback PRIVATE Docker-VM gateway IP. The narrower loopback-only
//!   exemption did NOT cover it, so the WAC suite's parallel single-source bursts drained the bucket →
//!   429s → failed features. The internal-range exemption covers it; and the conformance script ALSO
//!   sets `SOLID_SERVER_RATE_LIMIT_PER_IP=off` as the explicit primary belt (the CTH is a trusted
//!   single-source load generator — the limiter's real protection is validated by the unit + HTTP
//!   tests, not the harness).
//!
//! 🔒 Security framing: with this default the limiter protects against **PUBLIC-internet per-source
//! floods**, NOT a flood arriving via a trusted internal proxy hop — for THAT you set
//! [`ENV_TRUSTED_PROXY`] so the real PUBLIC client IP is keyed (not the internal hop). Both the
//! internal exemption and the loopback exemption are env-toggleable; the `off` sentinel on
//! [`ENV_RATE_PER_IP`] disables the layer entirely. See the env constants.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Request, State};
use axum::http::{header, HeaderName, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::app::{LIVEZ_PATH, READYZ_PATH};

// --- Env knobs + their testable parse cores -------------------------------------------------------

/// Env var: the per-IP sustained request rate (requests/second/IP) the token bucket refills at.
/// Unset / empty / invalid / `0` ⇒ [`DEFAULT_RATE_PER_IP`]. The sentinel `off` (case-insensitive)
/// DISABLES the rate-limit layer entirely (see [`parse_rate_per_ip`]).
pub const ENV_RATE_PER_IP: &str = "SOLID_SERVER_RATE_LIMIT_PER_IP";

/// Env var: the per-IP burst capacity (the bucket's max token count — how many requests a source may
/// make back-to-back before being throttled to the sustained rate). Unset / empty / invalid / `0` ⇒
/// [`DEFAULT_BURST`].
pub const ENV_BURST: &str = "SOLID_SERVER_RATE_LIMIT_BURST";

/// Env var: the X-Forwarded-For trusted-proxy hop COUNT. Unset / empty / invalid / `0` ⇒ XFF is NOT
/// trusted (the direct peer IP is used — the safe default, since an untrusted XFF is spoofable). A
/// positive integer N ⇒ trust N reverse-proxy hops in front of us and take the **N-th XFF entry from
/// the RIGHT** (0-based index `N-1`) as the client IP (standard XFF semantics — see
/// [`client_ip_from_xff`] for the derivation). See [`parse_trusted_proxy_hops`] and
/// [`client_ip_from_xff`].
pub const ENV_TRUSTED_PROXY: &str = "SOLID_SERVER_TRUSTED_PROXY";

/// Env var: whether to EXEMPT loopback source IPs from the limit. `0`/`false`/`no`/`off`
/// (case-insensitive) disables the exemption; anything else / absent ⇒ exemption ON (the default).
/// Loopback is exempt by default because the conformance harness's socat sidecar forwards from
/// loopback, so a loopback exemption guarantees the CTH is never rate-limited regardless of the
/// configured rate. An operator behind a loopback-bound reverse proxy who relies on XFF should turn
/// this OFF (and configure [`ENV_TRUSTED_PROXY`]). NOTE: [`ENV_EXEMPT_INTERNAL`] is the broader knob
/// that ALSO covers loopback — this narrower one stays for back-compat + the loopback-only case.
pub const ENV_EXEMPT_LOOPBACK: &str = "SOLID_SERVER_RATE_LIMIT_EXEMPT_LOOPBACK";

/// Env var: whether to EXEMPT all INTERNAL source IPs from the per-IP limit. `0`/`false`/`no`/`off`
/// (case-insensitive) disables the exemption; anything else / absent ⇒ exemption ON (**the default**).
///
/// "Internal" = loopback (`127.0.0.0/8`, `::1`) + RFC 1918 private (`10/8`, `172.16/12`,
/// `192.168/16`) + link-local (`169.254/16`, `fe80::/10`) + IPv6 unique-local (`fc00::/7`) — see
/// [`is_internal_ip`]. This is ON by default for two load-bearing reasons:
///  1. **It removes a footgun the per-IP design otherwise has.** When the server sits behind a
///     reverse proxy / docker-bridge / k8s service WITHOUT a configured [`ENV_TRUSTED_PROXY`], EVERY
///     client shares ONE internal hop IP as their peer — so a per-IP bucket keyed on that hop is
///     meaningless (it throttles ALL clients together, or — worse — lets a flood from one real client
///     exhaust the shared bucket for everyone). Exempting internal source IPs means a misconfigured
///     internal hop is simply NOT rate-limited (it proceeds to auth, which still gates it) rather than
///     mis-throttling. To actually rate-limit clients behind a TRUSTED proxy, set [`ENV_TRUSTED_PROXY`]
///     so the real (public) client IP is keyed instead of the internal hop.
///  2. **It covers the conformance harness's `host.docker.internal` hop** — the CTH's `--network host`
///     socat sidecar forwards from the Docker-VM gateway, a NON-loopback PRIVATE IP, so the
///     loopback-only exemption did not cover it and the WAC suite's parallel setup bursts (all one
///     source IP) drained the bucket → 429s → failed features. Exempting the private range fixes it.
///
/// 🔒 Security framing (be explicit): with this default, the per-IP limiter protects against
/// **PUBLIC-internet per-source floods**, NOT against a flood arriving via a trusted internal proxy
/// hop (for that you MUST set [`ENV_TRUSTED_PROXY`] so the limiter keys the real public client IP).
/// Turning this OFF rate-limits internal source IPs too (e.g. a directly-exposed internal network you
/// genuinely want throttled per-hop).
pub const ENV_EXEMPT_INTERNAL: &str = "SOLID_SERVER_RATE_LIMIT_EXEMPT_INTERNAL";

/// The default sustained per-IP rate (requests/second). Chosen DELIBERATELY HIGH so it never trips
/// during normal use OR the conformance run (which, while it hammers from one IP, stays well under
/// this AND is loopback-exempt by default). This is a safety bound against per-source flooding, not a
/// throughput throttle; an operator tunes it down to defend a small box.
pub const DEFAULT_RATE_PER_IP: f64 = 500.0;

/// The default per-IP burst capacity (max tokens). Generous so a legitimate client's bursty traffic
/// (e.g. a page load firing many parallel sub-resource requests) is never throttled, while a sustained
/// flood is still bounded to [`DEFAULT_RATE_PER_IP`]/s after the burst drains.
pub const DEFAULT_BURST: f64 = 2000.0;

/// The base `Retry-After` (seconds) returned on a 429, before jitter — mirrors
/// [`crate::overload::RETRY_AFTER_BASE_SECS`]. Small because a per-IP throttle clears within a second
/// of refill for a well-behaved client.
pub const RETRY_AFTER_BASE_SECS: u64 = 1;
/// The maximum extra jitter (seconds) on top of [`RETRY_AFTER_BASE_SECS`]; the returned value is
/// `base + rand(0..=JITTER)` so throttled clients spread their retries (a thundering-herd guard),
/// mirroring [`crate::overload::RETRY_AFTER_JITTER_SECS`].
pub const RETRY_AFTER_JITTER_SECS: u64 = 4;

/// The number of shards in the per-IP bucket map. A power of two so the shard index is a cheap mask of
/// the IP hash. 64 keeps per-shard lock contention low under realistic peer-IP cardinality while
/// keeping memory trivial. Not security-relevant — purely a contention/throughput knob.
const NUM_SHARDS: usize = 64;

/// The idle TTL after which an untouched bucket is GC'd from its shard, so the map cannot grow without
/// bound under a churn of distinct source IPs (a memory-exhaustion guard). A bucket idle longer than
/// this would have fully refilled anyway, so dropping it loses no throttling state — a returning IP
/// just starts fresh at full burst, which is the correct (most-permissive-to-a-quiet-source) behaviour.
const IDLE_TTL: Duration = Duration::from_secs(600);

/// A HARD per-shard bucket cap — the RESIDENT-MEMORY CEILING (not just a GC trigger). The `IDLE_TTL`
/// retain above only sweeps buckets idle past the TTL; under a sustained churn of DISTINCT source IPs
/// faster than that, the resident map would still grow to ≈ `arrival_rate × IDLE_TTL` of buckets —
/// attacker-influenceable memory. This cap bounds each shard's map to a fixed maximum: at the cap, a
/// NEW IP first triggers the idle-GC, and if the shard is STILL full, the OLDEST-`last_seen` bucket
/// (the coldest source) is EVICTED before inserting the newcomer. So total resident buckets are
/// bounded by `NUM_SHARDS × MAX_BUCKETS_PER_SHARD` regardless of churn, and — load-bearing — a HOT IP
/// (recently seen, hence NOT the coldest) is NEVER evicted in favour of a cold sprayed IP, so an
/// attacker spraying distinct cold IPs cannot reset a throttled hot IP's bucket. `8192` per shard ×
/// `64` shards = ~512k buckets max (~tens of MB), well above realistic legitimate peer cardinality.
const MAX_BUCKETS_PER_SHARD: usize = 8192;

/// Resolve the per-IP rate config from the env value into a [`RateConfig`].
/// - the sentinel `off`/`OFF`/`Off`/`disabled` (case-insensitive) ⇒ [`RateConfig::Disabled`]
///   (the layer is not installed — see [`crate::app`]);
/// - absent / empty / non-numeric / `<= 0` ⇒ [`DEFAULT_RATE_PER_IP`] (never silently disable on a typo
///   — disabling requires the explicit `off` sentinel);
/// - a positive number ⇒ that rate.
pub fn rate_per_ip_from_env() -> RateConfig {
    parse_rate_per_ip(std::env::var(ENV_RATE_PER_IP).ok())
}

/// Testable core of [`rate_per_ip_from_env`]. See that fn for the rules.
pub fn parse_rate_per_ip(raw: Option<String>) -> RateConfig {
    match raw.as_deref().map(str::trim) {
        Some(s) if s.eq_ignore_ascii_case("off") || s.eq_ignore_ascii_case("disabled") => {
            RateConfig::Disabled
        }
        None | Some("") => RateConfig::Enabled(DEFAULT_RATE_PER_IP),
        Some(s) => match s.parse::<f64>() {
            Ok(n) if n.is_finite() && n > 0.0 => RateConfig::Enabled(n),
            // non-numeric, NaN/inf, or <= 0 ⇒ the safe default (never silently brick on a typo; a
            // `0` rate would refill nothing and throttle everything — disabling needs the `off` sentinel).
            _ => RateConfig::Enabled(DEFAULT_RATE_PER_IP),
        },
    }
}

/// Resolve the per-IP burst capacity from the env value.
/// - absent / empty / non-numeric / `<= 0` ⇒ [`DEFAULT_BURST`];
/// - a positive number ⇒ that capacity.
pub fn burst_from_env() -> f64 {
    parse_burst(std::env::var(ENV_BURST).ok())
}

/// Testable core of [`burst_from_env`]. See that fn for the rules.
pub fn parse_burst(raw: Option<String>) -> f64 {
    match raw.as_deref().map(str::trim) {
        None | Some("") => DEFAULT_BURST,
        Some(s) => match s.parse::<f64>() {
            Ok(n) if n.is_finite() && n > 0.0 => n,
            _ => DEFAULT_BURST,
        },
    }
}

/// Resolve the trusted-proxy hop count from the env value (see [`ENV_TRUSTED_PROXY`]).
/// - absent / empty / non-numeric / `0` ⇒ `0` (XFF NOT trusted — the safe default);
/// - a positive integer N ⇒ trust N proxy hops.
pub fn trusted_proxy_hops_from_env() -> usize {
    parse_trusted_proxy_hops(std::env::var(ENV_TRUSTED_PROXY).ok())
}

/// Testable core of [`trusted_proxy_hops_from_env`]. See that fn for the rules. A non-numeric or
/// negative value resolves to `0` (do NOT trust XFF) — fail-safe: an XFF misconfig must not silently
/// enable a spoofable bypass.
pub fn parse_trusted_proxy_hops(raw: Option<String>) -> usize {
    match raw.as_deref().map(str::trim) {
        None | Some("") => 0,
        Some(s) => s.parse::<usize>().unwrap_or(0),
    }
}

/// Resolve whether loopback source IPs are exempt (see [`ENV_EXEMPT_LOOPBACK`]).
/// `0`/`false` (case-insensitive) ⇒ NOT exempt; anything else / absent ⇒ exempt (the default).
pub fn exempt_loopback_from_env() -> bool {
    parse_exempt_loopback(std::env::var(ENV_EXEMPT_LOOPBACK).ok())
}

/// Testable core of [`exempt_loopback_from_env`]. See that fn for the rules. The falsey set is matched
/// **case-INSENSITIVELY** (`0`/`false`/`no`/`off` in ANY casing), so `OFF`/`No`/`FALSE` correctly
/// DISABLE the exemption instead of silently slipping through to the on-by-default branch.
pub fn parse_exempt_loopback(raw: Option<String>) -> bool {
    !is_falsey(raw.as_deref())
}

/// Shared case-insensitive falsey test for the boolean-ish exemption env knobs: `true` for `0` /
/// `false` / `no` / `off` in ANY casing (trimmed); `false` for absent / empty / anything else (so the
/// default for these knobs is ON). Single-sourced so [`parse_exempt_loopback`] and
/// [`parse_exempt_internal`] agree on the falsey grammar (and fixes the prior casing gap where only
/// specific casings were matched).
fn is_falsey(raw: Option<&str>) -> bool {
    match raw.map(str::trim) {
        Some(s) => {
            s.eq_ignore_ascii_case("0")
                || s.eq_ignore_ascii_case("false")
                || s.eq_ignore_ascii_case("no")
                || s.eq_ignore_ascii_case("off")
        }
        None => false,
    }
}

/// Resolve whether INTERNAL source IPs are exempt (see [`ENV_EXEMPT_INTERNAL`]).
/// `0`/`false`/`no`/`off` (case-insensitive) ⇒ NOT exempt; anything else / absent ⇒ exempt (the
/// default).
pub fn exempt_internal_from_env() -> bool {
    parse_exempt_internal(std::env::var(ENV_EXEMPT_INTERNAL).ok())
}

/// Testable core of [`exempt_internal_from_env`]. See that fn for the rules.
pub fn parse_exempt_internal(raw: Option<String>) -> bool {
    !is_falsey(raw.as_deref())
}

/// Classify a source IP as INTERNAL (a host on a private / link-local / loopback / unique-local
/// network) vs PUBLIC (a routable internet address). Used by the default exemption ([`ENV_EXEMPT_INTERNAL`]):
/// an internal source IP is almost always a reverse-proxy / docker-bridge / k8s-internal hop, for which
/// a per-IP bucket is meaningless (all clients share the one hop IP), so it is exempted by default and
/// the operator keys the real public client via [`ENV_TRUSTED_PROXY`] instead.
///
/// Internal ⇔ any of:
/// - **IPv4** loopback `127.0.0.0/8`, RFC 1918 private (`10/8`, `172.16/12`, `192.168/16`), or
///   link-local `169.254.0.0/16`;
/// - **IPv6** loopback `::1`, unique-local `fc00::/7`, link-local `fe80::/10`, OR an
///   IPv4-mapped/-compatible address whose embedded IPv4 is itself internal (so a `::ffff:10.0.0.1`
///   peer is still treated as internal — closing the trivial mapped-address bypass).
pub fn is_internal_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_internal_v4(v4),
        IpAddr::V6(v6) => {
            if v6.is_loopback() {
                return true;
            }
            // IPv4-mapped (`::ffff:a.b.c.d`) / IPv4-compatible (`::a.b.c.d`): classify by the embedded
            // v4 so a mapped internal address can't masquerade as a "public" v6.
            if let Some(v4) = v6.to_ipv4() {
                return is_internal_v4(&v4);
            }
            let seg0 = v6.segments()[0];
            // fc00::/7 (unique-local) ⇒ top 7 bits == 1111110. fe80::/10 (link-local) ⇒ top 10 bits
            // == 1111111010. (`Ipv6Addr::is_unique_local`/`is_unicast_link_local` are unstable, so
            // classify by the prefix directly — a stable, dependency-free check.)
            (seg0 & 0xfe00) == 0xfc00 || (seg0 & 0xffc0) == 0xfe80
        }
    }
}

/// IPv4 internal-range test: loopback `127/8`, RFC 1918 private (`10/8`, `172.16/12`, `192.168/16`),
/// or link-local `169.254/16`. (`Ipv4Addr::is_private`/`is_link_local`/`is_loopback` are stable.)
fn is_internal_v4(v4: &std::net::Ipv4Addr) -> bool {
    v4.is_loopback() || v4.is_private() || v4.is_link_local()
}

/// The resolved rate config: either ENABLED at a sustained rate or DISABLED entirely.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RateConfig {
    /// The rate-limit layer is installed with this sustained per-IP rate (requests/second).
    Enabled(f64),
    /// The rate-limit layer is NOT installed (the `off` sentinel) — every request proceeds to auth.
    Disabled,
}

// --- The token bucket -----------------------------------------------------------------------------

/// A single source's token bucket. `tokens` is a continuous (fractional) count that refills at
/// `rate`/s up to `capacity`; a request costs one token. `last_refill` timestamps the last update so
/// we can compute the elapsed refill lazily on each access (no background timer). `last_seen` drives
/// idle-bucket GC.
#[derive(Debug)]
struct Bucket {
    tokens: f64,
    last_refill: Instant,
    last_seen: Instant,
}

impl Bucket {
    fn new(capacity: f64, now: Instant) -> Self {
        Self {
            tokens: capacity,
            last_refill: now,
            last_seen: now,
        }
    }

    /// Refill lazily by the elapsed time, then try to spend one token. Returns `true` if a token was
    /// available (the request is ALLOWED) and `false` if the bucket was empty (the request is LIMITED).
    fn try_take(&mut self, rate: f64, capacity: f64, now: Instant) -> bool {
        let elapsed = now
            .saturating_duration_since(self.last_refill)
            .as_secs_f64();
        // Refill, clamped to capacity. `last_refill` advances to `now` regardless so refill is monotone.
        self.tokens = (self.tokens + elapsed * rate).min(capacity);
        self.last_refill = now;
        self.last_seen = now;
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Observability counter: a monotonic total of requests rate-limited (429'd) since boot. A plain
/// atomic so a future `/metrics` exporter can read it lock-free (mirrors [`crate::overload`]).
#[derive(Debug, Default)]
pub struct RateLimitMetrics {
    limited_total: AtomicU64,
}

impl RateLimitMetrics {
    /// Total requests rate-limited (429'd) since boot.
    pub fn limited_total(&self) -> u64 {
        self.limited_total.load(Ordering::Relaxed)
    }
}

/// The shared rate-limiter state: the per-IP token-bucket map (sharded), the rate/burst sizing, the
/// XFF trust + loopback-exemption policy, and the metrics. `Clone` is cheap (`Arc` bumps) so it can be
/// handed to `from_fn_with_state` like [`crate::overload::AdmissionControl`].
#[derive(Clone)]
pub struct RateLimiter {
    shards: Arc<Vec<Mutex<HashMap<IpAddr, Bucket>>>>,
    rate: f64,
    capacity: f64,
    /// Number of trusted reverse-proxy hops in front of us; `0` ⇒ XFF is not trusted (use the peer IP).
    trusted_proxy_hops: usize,
    /// Whether loopback source IPs are exempt from the limit.
    exempt_loopback: bool,
    /// Whether ALL internal source IPs (loopback + private + link-local + ULA) are exempt — the
    /// default. See [`ENV_EXEMPT_INTERNAL`] / [`is_internal_ip`].
    exempt_internal: bool,
    /// The HARD per-shard bucket cap (resident-memory ceiling). Defaults to [`MAX_BUCKETS_PER_SHARD`];
    /// the test-only [`Self::set_max_buckets_per_shard_for_test`] lowers it so the eviction path can be
    /// exercised without spraying hundreds of thousands of IPs.
    max_buckets_per_shard: usize,
    metrics: Arc<RateLimitMetrics>,
}

impl RateLimiter {
    /// Build a rate limiter. `rate` (tokens/s) and `capacity` (burst) are clamped to a small positive
    /// floor so the type is always safe to construct directly (e.g. in tests); the env parsers already
    /// reject non-positive values, this just makes the constructor total. `exempt_internal` exempts the
    /// broader internal-IP set (loopback + private + link-local + ULA — see [`is_internal_ip`]) and is
    /// the default-on footgun guard for a proxy/docker/k8s hop; `exempt_loopback` is the narrower
    /// loopback-only knob (kept for back-compat). When `exempt_internal` is on it already covers
    /// loopback, so the two compose (either exemption matching ⇒ exempt).
    pub fn new(
        rate: f64,
        capacity: f64,
        trusted_proxy_hops: usize,
        exempt_loopback: bool,
        exempt_internal: bool,
    ) -> Self {
        let rate = if rate.is_finite() && rate > 0.0 {
            rate
        } else {
            DEFAULT_RATE_PER_IP
        };
        let capacity = if capacity.is_finite() && capacity >= 1.0 {
            capacity
        } else {
            DEFAULT_BURST
        };
        let mut shards = Vec::with_capacity(NUM_SHARDS);
        for _ in 0..NUM_SHARDS {
            shards.push(Mutex::new(HashMap::new()));
        }
        Self {
            shards: Arc::new(shards),
            rate,
            capacity,
            trusted_proxy_hops,
            exempt_loopback,
            exempt_internal,
            max_buckets_per_shard: MAX_BUCKETS_PER_SHARD,
            metrics: Arc::new(RateLimitMetrics::default()),
        }
    }

    /// The metrics handle (limited counter), e.g. for a `/metrics` exporter.
    pub fn metrics(&self) -> Arc<RateLimitMetrics> {
        self.metrics.clone()
    }

    /// Pick the shard for an IP by a cheap hash (the IP's octet bytes), masked to the shard count.
    fn shard_for(&self, ip: &IpAddr) -> &Mutex<HashMap<IpAddr, Bucket>> {
        // A small FNV-1a over the address bytes — deterministic, fast, and good enough for shard
        // distribution (this is NOT a security boundary; it only spreads lock contention).
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let octets: &[u8] = match ip {
            IpAddr::V4(v4) => &v4.octets()[..],
            IpAddr::V6(v6) => &v6.octets()[..],
        };
        for &b in octets {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        let idx = (hash as usize) & (NUM_SHARDS - 1);
        &self.shards[idx]
    }

    /// Try to admit one request from `ip`. `true` ⇒ allowed (a token was available, or the IP is
    /// exempt); `false` ⇒ rate-limited (the bucket was empty). GCs idle buckets in the touched shard
    /// AND enforces a HARD per-shard cap (evicting the coldest bucket when full) so the map's resident
    /// memory is bounded regardless of distinct-IP churn.
    fn allow(&self, ip: IpAddr) -> bool {
        // Source-IP exemptions (checked here, not in the middleware, so they are part of the testable
        // core). `exempt_internal` (default-on) covers loopback + private + link-local + ULA — the
        // footgun guard for a proxy/docker/k8s hop AND the CTH `host.docker.internal` private-IP hop;
        // `exempt_loopback` is the narrower back-compat loopback-only knob. Either matching ⇒ exempt.
        if self.exempt_internal && is_internal_ip(&ip) {
            return true;
        }
        if self.exempt_loopback && ip.is_loopback() {
            return true;
        }
        let now = Instant::now();
        let shard = self.shard_for(&ip);
        // The critical section is tiny — a map lookup + arithmetic (+ at most one O(shard) GC/eviction
        // pass), NO I/O and NO `.await` held — so a plain Mutex per shard is fine. A poisoned lock (a
        // panic while held) is recovered via `into_inner` so a single panicking request can never wedge
        // the limiter for an IP shard.
        let mut map = match shard.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };

        // HARD per-shard memory cap. If this is a NEW IP and the shard is at the cap, make room before
        // inserting: first GC genuinely-idle buckets, and if STILL full, EVICT the coldest
        // (oldest-`last_seen`) bucket. A HOT IP is the most-recently-seen, hence never the coldest, so
        // it survives a spray of cold distinct IPs — its throttle is not reset by churn (the
        // memory-exhaustion + hot-IP-eviction guard). We only do this work for a NEW key at the cap, so
        // the steady-state hot path (an existing key) pays nothing.
        if map.len() >= self.max_buckets_per_shard && !map.contains_key(&ip) {
            map.retain(|_, b| now.saturating_duration_since(b.last_seen) < IDLE_TTL);
            if map.len() >= self.max_buckets_per_shard {
                if let Some(coldest) = map.iter().min_by_key(|(_, b)| b.last_seen).map(|(k, _)| *k)
                {
                    map.remove(&coldest);
                }
            }
        }

        let bucket = map
            .entry(ip)
            .or_insert_with(|| Bucket::new(self.capacity, now));
        let allowed = bucket.try_take(self.rate, self.capacity, now);
        // Opportunistic GC of idle buckets in this shard (amortised — only the touched shard, and only
        // when it has grown past a small threshold, so it is O(shard size) at most occasionally). This
        // keeps the resident set small in the common case; the hard cap above is the worst-case ceiling.
        if map.len() > 1024 {
            map.retain(|_, b| now.saturating_duration_since(b.last_seen) < IDLE_TTL);
        }
        allowed
    }

    /// Drive one request for `ip` through the limiter for tests (exposes the testable [`allow`] core).
    #[doc(hidden)]
    pub fn allow_for_test(&self, ip: IpAddr) -> bool {
        self.allow(ip)
    }

    /// The TOTAL number of resident buckets across all shards (for the memory-cap tests). Sums each
    /// shard's map length under its lock — a poisoned lock is recovered so the count is always readable.
    #[doc(hidden)]
    pub fn bucket_count_for_test(&self) -> usize {
        self.shards
            .iter()
            .map(|s| match s.lock() {
                Ok(g) => g.len(),
                Err(p) => p.into_inner().len(),
            })
            .sum()
    }

    /// Lower the per-shard hard cap for the memory-cap tests, so the eviction path can be exercised
    /// without spraying hundreds of thousands of distinct IPs. Clamped to >= 1. Test-only.
    #[doc(hidden)]
    pub fn set_max_buckets_per_shard_for_test(&mut self, cap: usize) {
        self.max_buckets_per_shard = cap.max(1);
    }
}

// --- Peer-IP extraction (direct peer + optional trusted XFF) --------------------------------------

/// The HTTP `X-Forwarded-For` header name (lowercase, as stored in the header map).
const XFF: HeaderName = HeaderName::from_static("x-forwarded-for");

/// Resolve the effective CLIENT IP for rate-limiting from the direct peer IP + (only when trusted) the
/// `X-Forwarded-For` header.
///
/// - `trusted_proxy_hops == 0` (the default) ⇒ ALWAYS use the direct `peer` IP and IGNORE any XFF. An
///   untrusted XFF is attacker-controlled and spoofable, so trusting it would let an attacker dodge the
///   per-IP limit by rotating a fake XFF on every request — a bypass. Fail-safe: ignore it by default.
/// - `trusted_proxy_hops == N > 0` ⇒ we sit behind N reverse proxies. **Standard XFF semantics:** a
///   proxy appends the IP it RECEIVED the connection FROM; the DIRECT proxy in front of us is the TCP
///   peer (`ConnectInfo`) and does NOT appear as an XFF entry it wrote about itself. So a request that
///   traversed N proxies carries exactly N XFF entries, the rightmost of which is the client as seen by
///   the OUTERMOST-from-us proxy — and the real client IP is the **N-th entry from the RIGHT** (0-based
///   index `N-1`). If XFF has FEWER than `N` valid entries (a malformed/short header, or a request that
///   did not actually traverse N proxies), fall back to the direct peer IP — never trust a too-short
///   XFF, which could otherwise be spoofed to inject an arbitrary "client" IP.
pub fn resolve_client_ip(peer: IpAddr, xff: Option<&str>, trusted_proxy_hops: usize) -> IpAddr {
    if trusted_proxy_hops == 0 {
        return peer; // XFF untrusted — direct peer only.
    }
    match xff.and_then(|h| client_ip_from_xff(h, trusted_proxy_hops)) {
        Some(ip) => ip,
        None => peer, // too-short/malformed XFF ⇒ fall back to the direct peer (fail-safe).
    }
}

/// Parse the client IP from an `X-Forwarded-For` value given `trusted_proxy_hops` (`N`) trusted hops.
/// Returns the **N-th entry from the RIGHT (0-based index `N-1`)**, or `None` if the header has fewer
/// than `N` valid entries (so the caller falls back to the direct peer IP). Entries that don't parse as
/// an IP are skipped from the right, so a proxy that wrote a junk token can't shift the index.
///
/// ## Why index `N-1` (the off-by-one this corrects)
/// Standard XFF: each proxy APPENDS the IP it received the connection from; the DIRECT proxy in front
/// of us is our TCP peer ([`ConnectInfo`]) and writes NOTHING about itself into the header. So for `N`
/// proxies the header has exactly `N` entries and the real client is the LEFTMOST (the first proxy's
/// view of the client), i.e. the N-th from the right = 0-based index `N-1`:
/// - `N = 1`, one trusted proxy receiving DIRECTLY from the client ⇒ header `XFF: <client>` ⇒ the
///   client is the single RIGHTMOST entry = index `0` (`N-1`). (The earlier code used index `N` = `1`
///   here, which is `None` ⇒ it wrongly fell back to the proxy peer, collapsing every client behind
///   the proxy onto ONE bucket.)
/// - `N = 2` ⇒ header `XFF: <client>, <proxy1>` ⇒ client = index `1` (`N-1`) from the right.
pub fn client_ip_from_xff(xff: &str, trusted_proxy_hops: usize) -> Option<IpAddr> {
    // A zero-hop call never trusts XFF (the caller short-circuits to the peer); guard here too so the
    // `N-1` index can't underflow.
    if trusted_proxy_hops == 0 {
        return None;
    }
    // Collect the right-to-left sequence of PARSEABLE IPs.
    let ips: Vec<IpAddr> = xff
        .split(',')
        .rev()
        .filter_map(|tok| tok.trim().parse::<IpAddr>().ok())
        .collect();
    // The client is the N-th entry from the right, 0-based index `N-1`: with `N` trusted proxies the
    // header carries `N` client-appended entries and the real client is the leftmost of them.
    ips.get(trusted_proxy_hops - 1).copied()
}

// --- The 429 response -----------------------------------------------------------------------------

/// Compute the jittered `Retry-After` (seconds) for a 429: `base + rand(0..=jitter)`. Reuses the same
/// OS-RNG (`getrandom 0.2`) jitter approach as [`crate::overload::jittered_retry_after_secs`] — a
/// jitter value needs no cryptographic strength, but reusing the one OS source keeps the surface small.
/// On the (vanishingly unlikely) `getrandom` failure, fall back to no jitter (just `base`) — the 429 is
/// still correct, only the retry spread is lost.
fn jittered_retry_after_secs() -> u64 {
    let mut buf = [0u8; 8];
    let jitter = if getrandom::getrandom(&mut buf).is_ok() {
        u64::from_le_bytes(buf) % (RETRY_AFTER_JITTER_SECS + 1)
    } else {
        0
    };
    RETRY_AFTER_BASE_SECS + jitter
}

/// Build the 429 response: `429 Too Many Requests` + `Retry-After: <jittered seconds>` +
/// `Cache-Control: no-store` + a short plain-text body. Mirrors [`crate::overload`]'s `shed_response`
/// but with 429 (a per-source rate limit) rather than 503 (a global overload). This is a FAIL-SAFE
/// response: it grants strictly less than the request would otherwise have gotten, so it can NEVER be
/// an authorization bypass.
fn rate_limited_response() -> Response {
    let retry_after = jittered_retry_after_secs();
    (
        StatusCode::TOO_MANY_REQUESTS,
        [
            (header::RETRY_AFTER, retry_after.to_string()),
            (header::CACHE_CONTROL, "no-store".to_string()),
        ],
        "429 Too Many Requests: per-source rate limit exceeded. Retry after the indicated delay.\n",
    )
        .into_response()
}

// --- The middleware -------------------------------------------------------------------------------

/// The pre-crypto per-IP rate-limit middleware. Sits OUTSIDE auth/WAC/crypto (alongside admission
/// control). For each request it resolves the source IP, charges the source's token bucket, and either
/// LIMITS it (429 + jittered `Retry-After` — the inner stack, incl. the verifier, is NOT run) or lets
/// it PROCEED to the normal auth stack.
///
/// 🔒 Two security invariants (see the module docs):
/// 1. This layer can only REJECT a request early; it has zero authority to admit one. A 429 is strictly
///    less access than auth would grant, so it can never be a bypass.
/// 2. FAIL-OPEN for the limiter: if `ConnectInfo` is absent (should not happen once wired — see
///    `main`), proceed to auth (which still gates the request) rather than denying. "Fail-open" means
///    "proceed to auth", NEVER "bypass auth".
pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    req: Request,
    next: Next,
) -> Response {
    // Defence-in-depth: health probes are EXEMPT. They are already mounted outside this layer (so this
    // path is normally unreachable for them), but skip them by path too in case the layer is ever
    // applied more broadly — a readiness probe must never be rate-limited.
    let path = req.uri().path();
    if path == LIVEZ_PATH || path == READYZ_PATH {
        return next.run(req).await;
    }

    // Resolve the direct peer IP from ConnectInfo. ABSENT ⇒ FAIL OPEN (proceed to auth — which still
    // gates the request — never deny-all on a limiter wiring gap).
    let peer_ip = match req.extensions().get::<ConnectInfo<SocketAddr>>() {
        Some(ConnectInfo(addr)) => addr.ip(),
        None => return next.run(req).await,
    };

    // Only consult XFF when a trusted-proxy count is configured; otherwise the direct peer IP is used
    // (an untrusted XFF is spoofable — see `resolve_client_ip`).
    let xff = req
        .headers()
        .get(&XFF)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let client_ip = resolve_client_ip(peer_ip, xff.as_deref(), limiter.trusted_proxy_hops);

    if limiter.allow(client_ip) {
        next.run(req).await
    } else {
        let limited = limiter
            .metrics
            .limited_total
            .fetch_add(1, Ordering::Relaxed)
            + 1;
        // Log the first 429 and then periodically, so a flood is visible without flooding the log.
        if limited == 1 || limited % 100 == 0 {
            eprintln!(
                "  RATE-LIMIT: per-source limit exceeded — returned 429 (limited_total={limited}, \
                 rate={}/s burst={}). The request never reached auth/crypto.",
                limiter.rate, limiter.capacity
            );
        }
        rate_limited_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn ipv4(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(a, b, c, d))
    }

    // --- env parser tests (mirror overload.rs's pure-core tests) ---

    #[test]
    fn rate_default_on_absent_empty_invalid_or_nonpositive() {
        // Never silently disable on a typo — disabling requires the `off` sentinel.
        assert_eq!(
            parse_rate_per_ip(None),
            RateConfig::Enabled(DEFAULT_RATE_PER_IP)
        );
        assert_eq!(
            parse_rate_per_ip(Some("".into())),
            RateConfig::Enabled(DEFAULT_RATE_PER_IP)
        );
        assert_eq!(
            parse_rate_per_ip(Some("  ".into())),
            RateConfig::Enabled(DEFAULT_RATE_PER_IP)
        );
        assert_eq!(
            parse_rate_per_ip(Some("abc".into())),
            RateConfig::Enabled(DEFAULT_RATE_PER_IP)
        );
        assert_eq!(
            parse_rate_per_ip(Some("0".into())),
            RateConfig::Enabled(DEFAULT_RATE_PER_IP)
        );
        assert_eq!(
            parse_rate_per_ip(Some("-5".into())),
            RateConfig::Enabled(DEFAULT_RATE_PER_IP)
        );
        assert_eq!(
            parse_rate_per_ip(Some("nan".into())),
            RateConfig::Enabled(DEFAULT_RATE_PER_IP)
        );
    }

    #[test]
    fn rate_explicit_positive_is_honoured() {
        assert_eq!(
            parse_rate_per_ip(Some("1".into())),
            RateConfig::Enabled(1.0)
        );
        assert_eq!(
            parse_rate_per_ip(Some("250.5".into())),
            RateConfig::Enabled(250.5)
        );
        assert_eq!(
            parse_rate_per_ip(Some("  4096  ".into())),
            RateConfig::Enabled(4096.0)
        );
    }

    #[test]
    fn rate_off_sentinel_disables() {
        // The ONLY way to disable — and case-insensitive, with a `disabled` alias.
        assert_eq!(parse_rate_per_ip(Some("off".into())), RateConfig::Disabled);
        assert_eq!(parse_rate_per_ip(Some("OFF".into())), RateConfig::Disabled);
        assert_eq!(parse_rate_per_ip(Some("Off".into())), RateConfig::Disabled);
        assert_eq!(
            parse_rate_per_ip(Some("disabled".into())),
            RateConfig::Disabled
        );
    }

    #[test]
    fn burst_rules() {
        assert_eq!(parse_burst(None), DEFAULT_BURST);
        assert_eq!(parse_burst(Some("".into())), DEFAULT_BURST);
        assert_eq!(parse_burst(Some("garbage".into())), DEFAULT_BURST);
        assert_eq!(parse_burst(Some("0".into())), DEFAULT_BURST);
        assert_eq!(parse_burst(Some("-1".into())), DEFAULT_BURST);
        assert_eq!(parse_burst(Some("16".into())), 16.0);
        assert_eq!(parse_burst(Some("  64  ".into())), 64.0);
    }

    #[test]
    fn trusted_proxy_hops_rules() {
        // Default 0 (do NOT trust XFF) on absent/empty/invalid/negative — fail-safe.
        assert_eq!(parse_trusted_proxy_hops(None), 0);
        assert_eq!(parse_trusted_proxy_hops(Some("".into())), 0);
        assert_eq!(parse_trusted_proxy_hops(Some("abc".into())), 0);
        assert_eq!(parse_trusted_proxy_hops(Some("-1".into())), 0);
        assert_eq!(parse_trusted_proxy_hops(Some("0".into())), 0);
        assert_eq!(parse_trusted_proxy_hops(Some("1".into())), 1);
        assert_eq!(parse_trusted_proxy_hops(Some("  2  ".into())), 2);
    }

    #[test]
    fn exempt_loopback_rules() {
        // Default ON; explicit falsey values (in ANY casing) turn it off.
        assert!(parse_exempt_loopback(None));
        assert!(parse_exempt_loopback(Some("1".into())));
        assert!(parse_exempt_loopback(Some("true".into())));
        assert!(
            parse_exempt_loopback(Some("  ".into())),
            "blank ⇒ default ON"
        );
        assert!(!parse_exempt_loopback(Some("0".into())));
        assert!(!parse_exempt_loopback(Some("false".into())));
        assert!(!parse_exempt_loopback(Some("no".into())));
        assert!(!parse_exempt_loopback(Some("off".into())));
        // CASING FIX (was the Low): these MUST disable the exemption — the old code only matched
        // specific casings, so `OFF`/`No`/`FALSE` silently slipped through to the on-by-default branch.
        assert!(
            !parse_exempt_loopback(Some("OFF".into())),
            "OFF must disable"
        );
        assert!(!parse_exempt_loopback(Some("No".into())), "No must disable");
        assert!(
            !parse_exempt_loopback(Some("FALSE".into())),
            "FALSE must disable"
        );
        assert!(!parse_exempt_loopback(Some("Off".into())));
        assert!(!parse_exempt_loopback(Some("NO".into())));
        assert!(!parse_exempt_loopback(Some(" off ".into())), "trimmed");
    }

    // --- token-bucket core tests ---

    #[test]
    fn burst_then_throttle_then_refill() {
        // capacity 3, rate "0" effectively (use a tiny rate so refill in the test window is negligible).
        // We exempt-loopback AND exempt-internal OFF and drive a PUBLIC (TEST-NET) IP.
        let rl = RateLimiter::new(0.0001, 3.0, 0, false, false);
        let ip = ipv4(203, 0, 113, 7);
        // The first `capacity` requests pass (the burst), the next is limited.
        assert!(rl.allow(ip), "burst 1");
        assert!(rl.allow(ip), "burst 2");
        assert!(rl.allow(ip), "burst 3");
        assert!(!rl.allow(ip), "4th exceeds the burst ⇒ limited");
    }

    #[test]
    fn refill_grants_more_after_wait() {
        // DETERMINISTIC de-flake (was flaky at rate=1000/s: >1ms could elapse between the two
        // back-to-back allow() calls on a loaded box and refill a full token before the
        // "immediately exhausted" assertion, ~1-in-6 flake). The fix keeps what it PROVES
        // (burst → exhaust → refill-grants-more) but removes the race:
        //   - rate = 5/s, capacity 1: a token takes 200ms to refill. The microseconds between the two
        //     consecutive allow() calls refill << 0.001 token — it CANNOT cross the 1.0 boundary even
        //     if the box stalls for tens of ms between them — so "immediately exhausted" can't race.
        //   - then sleep a GENEROUS 400ms (2× the 200ms refill period) so ≥1 token is unambiguously
        //     refilled regardless of scheduler jitter, and the follow-up request passes.
        let rl = RateLimiter::new(5.0, 1.0, 0, false, false);
        let ip = ipv4(203, 0, 113, 8);
        assert!(rl.allow(ip), "first token (the single-capacity burst)");
        assert!(
            !rl.allow(ip),
            "immediately exhausted (capacity 1; at 5/s the sub-ms inter-call refill is << 1 token, \
             so this cannot race even under load)"
        );
        std::thread::sleep(Duration::from_millis(400)); // 5/s ⇒ a token every 200ms; 400ms ⇒ ≥1 refilled
        assert!(
            rl.allow(ip),
            "a full token has refilled after the 400ms wait ⇒ allowed"
        );
    }

    #[test]
    fn per_ip_isolation_a_floods_b_unaffected() {
        // MUTATION KILL (per-IP isolation): a shared/global bucket would let A's flood throttle B.
        // capacity 2, negligible refill. A exhausts its bucket; B in the SAME window is unaffected.
        let rl = RateLimiter::new(0.0001, 2.0, 0, false, false);
        let a = ipv4(198, 51, 100, 1);
        let b = ipv4(198, 51, 100, 2);
        assert!(rl.allow(a), "A 1");
        assert!(rl.allow(a), "A 2");
        assert!(!rl.allow(a), "A is now throttled");
        // B must still have its FULL burst — a shared-bucket mutation would fail these.
        assert!(rl.allow(b), "B 1 unaffected by A's flood");
        assert!(rl.allow(b), "B 2 unaffected by A's flood");
        assert!(
            !rl.allow(b),
            "B then hits its OWN limit (proves B has an independent bucket)"
        );
    }

    #[test]
    fn does_not_trip_at_modest_sequential_rate_under_default() {
        // A modest sequential burst under the DEFAULT config must never be limited (the conformance/
        // normal-use guarantee). DEFAULT_BURST is generous; loopback-exempt is irrelevant here (use a
        // public IP), the burst alone covers it.
        let rl = RateLimiter::new(DEFAULT_RATE_PER_IP, DEFAULT_BURST, 0, false, false);
        let ip = ipv4(192, 0, 2, 50);
        for i in 0..(DEFAULT_BURST as usize) {
            assert!(
                rl.allow(ip),
                "request {i} under the default burst must pass"
            );
        }
    }

    #[test]
    fn loopback_is_exempt_when_enabled_and_not_when_disabled() {
        // Exempt-loopback ON (and exempt-internal OFF so the loopback-only knob is what's tested):
        // loopback never limited even past a tiny capacity.
        let rl_exempt = RateLimiter::new(0.0001, 1.0, 0, true, false);
        let lo = IpAddr::V4(Ipv4Addr::LOCALHOST);
        for _ in 0..10 {
            assert!(rl_exempt.allow(lo), "loopback exempt ⇒ always allowed");
        }
        // BOTH exemptions OFF: loopback is subject to the bucket like any IP.
        let rl_strict = RateLimiter::new(0.0001, 1.0, 0, false, false);
        assert!(rl_strict.allow(lo), "loopback 1 (capacity 1)");
        assert!(
            !rl_strict.allow(lo),
            "loopback IS limited when both exemptions are off"
        );
    }

    #[test]
    fn internal_exempt_covers_loopback_private_linklocal_ula() {
        // Exempt-internal ON (the DEFAULT): every internal source IP is exempt regardless of the bucket,
        // EVEN with the loopback-only knob OFF. This is the footgun guard + the CTH host.docker.internal
        // (private-IP) hop fix. Tiny capacity so a non-exempt IP would be limited after one request.
        let rl = RateLimiter::new(0.0001, 1.0, 0, false, true);
        let internal = [
            IpAddr::V4(Ipv4Addr::LOCALHOST),    // 127.0.0.1 loopback
            ipv4(10, 1, 2, 3),                  // RFC1918 10/8
            ipv4(172, 16, 5, 6),                // RFC1918 172.16/12
            ipv4(172, 31, 255, 254),            // RFC1918 172.16/12 (upper edge)
            ipv4(192, 168, 0, 1),               // RFC1918 192.168/16
            ipv4(169, 254, 10, 20),             // link-local 169.254/16
            "::1".parse().unwrap(),             // IPv6 loopback
            "fc00::1".parse().unwrap(),         // IPv6 unique-local fc00::/7
            "fd12:3456::1".parse().unwrap(),    // IPv6 unique-local (fd in fc00::/7)
            "fe80::1".parse().unwrap(),         // IPv6 link-local fe80::/10
            "::ffff:10.0.0.1".parse().unwrap(), // IPv4-mapped internal ⇒ still internal
        ];
        for ip in internal {
            for _ in 0..5 {
                assert!(
                    rl.allow(ip),
                    "internal IP {ip} must be exempt under exempt_internal (never limited)"
                );
            }
        }
        // A PUBLIC IP (172.32.x is OUTSIDE the 172.16/12 private block) is NOT internal ⇒ still limited.
        let public = ipv4(172, 32, 1, 1);
        assert!(rl.allow(public), "public 1 (capacity 1)");
        assert!(
            !rl.allow(public),
            "a PUBLIC IP is NOT exempt under exempt_internal ⇒ limited past its burst"
        );
    }

    #[test]
    fn is_internal_ip_classifies_ranges() {
        // Positive: the full internal set.
        for ip in [
            "127.0.0.1",
            "127.255.255.255",
            "10.0.0.0",
            "10.255.255.255",
            "172.16.0.0",
            "172.31.255.255",
            "192.168.0.0",
            "192.168.255.255",
            "169.254.0.1",
            "::1",
            "fc00::1",
            "fdff::1",
            "fe80::1",
            "febf::1",
            "::ffff:192.168.1.1",
        ] {
            assert!(
                is_internal_ip(&ip.parse().unwrap()),
                "{ip} must classify as INTERNAL"
            );
        }
        // Negative: public / boundary-just-outside addresses.
        for ip in [
            "8.8.8.8",
            "1.1.1.1",
            "172.15.255.255", // just below 172.16/12
            "172.32.0.0",     // just above 172.16/12
            "192.167.255.255",
            "192.169.0.0",
            "9.255.255.255",
            "11.0.0.0",
            "169.253.255.255",
            "169.255.0.0",
            "2606:4700::1111", // public v6 (Cloudflare)
            "fbff::1",         // just below fc00::/7
            "fec0::1",         // just above fe80::/10
            "::ffff:8.8.8.8",  // IPv4-mapped PUBLIC ⇒ public
        ] {
            assert!(
                !is_internal_ip(&ip.parse().unwrap()),
                "{ip} must classify as PUBLIC (not internal)"
            );
        }
    }

    #[test]
    fn exempt_internal_rules() {
        // Default ON; explicit falsey (any casing) turns it off.
        assert!(parse_exempt_internal(None));
        assert!(parse_exempt_internal(Some("1".into())));
        assert!(parse_exempt_internal(Some("true".into())));
        assert!(!parse_exempt_internal(Some("0".into())));
        assert!(!parse_exempt_internal(Some("false".into())));
        assert!(!parse_exempt_internal(Some("OFF".into())));
        assert!(!parse_exempt_internal(Some("No".into())));
    }

    // --- XFF trust tests ---

    #[test]
    fn xff_ignored_by_default_spoof_does_not_dodge_limit() {
        // MUTATION KILL (XFF-spoof bypass): with trusted_proxy_hops=0, the direct peer IP is used and a
        // ROTATING fake XFF must NOT let one peer dodge the per-IP limit. We simulate the middleware's
        // IP resolution: same peer, different spoofed XFF each call ⇒ all map to the SAME peer bucket.
        let peer = ipv4(203, 0, 113, 9);
        let r1 = resolve_client_ip(peer, Some("1.2.3.4"), 0);
        let r2 = resolve_client_ip(peer, Some("5.6.7.8"), 0);
        assert_eq!(r1, peer, "XFF ignored when untrusted ⇒ direct peer");
        assert_eq!(r2, peer, "a rotated XFF still resolves to the same peer");
        // Drive it through a capacity-1 bucket: the second request (spoofing a new XFF) is still limited.
        let rl = RateLimiter::new(0.0001, 1.0, 0, false, false);
        assert!(rl.allow(r1), "first from the peer");
        assert!(
            !rl.allow(r2),
            "spoofing a new XFF does NOT grant a fresh bucket — same peer is throttled"
        );
    }

    #[test]
    fn xff_default_zero_hops_ignores_xff_entirely() {
        // The SAFE DEFAULT (hops=0) is UNCHANGED by the off-by-one fix: XFF is ignored and the direct
        // peer is always used. (`client_ip_from_xff` also returns None at hops=0 so the `N-1` index
        // cannot underflow.)
        let peer = ipv4(203, 0, 113, 1);
        assert_eq!(resolve_client_ip(peer, Some("8.8.8.8"), 0), peer);
        assert_eq!(resolve_client_ip(peer, Some("8.8.8.8, 9.9.9.9"), 0), peer);
        assert_eq!(client_ip_from_xff("8.8.8.8", 0), None);
    }

    #[test]
    fn xff_one_proxy_client_is_the_single_rightmost_entry() {
        // THE CORRECTED COMMON CASE (the off-by-one fix). One trusted proxy receiving DIRECTLY from the
        // client: the proxy is our TCP peer (and writes nothing about itself), the header is just
        // `XFF: <client>`, so the client is the SINGLE rightmost entry = 0-based index N-1 = 0. The OLD
        // code took index N=1 ⇒ None ⇒ wrongly fell back to the proxy peer, collapsing all clients onto
        // one bucket.
        let proxy = ipv4(10, 0, 0, 7); // our reverse proxy = the direct peer
        let client = ipv4(198, 51, 100, 5);
        assert_eq!(
            resolve_client_ip(proxy, Some("198.51.100.5"), 1),
            client,
            "hops=1, XFF=<client> ⇒ client is the single rightmost entry"
        );
        assert_eq!(client_ip_from_xff("198.51.100.5", 1), Some(client));
        // Two distinct clients behind the same proxy each get their OWN bucket (keyed by client IP) —
        // the very behaviour the off-by-one broke (it had keyed them all on the shared proxy peer).
        let rl = RateLimiter::new(0.0001, 1.0, 1, false, false);
        let c1 = resolve_client_ip(proxy, Some("198.51.100.5"), 1);
        let c2 = resolve_client_ip(proxy, Some("198.51.100.6"), 1);
        assert!(rl.allow(c1), "client 1 first");
        assert!(
            rl.allow(c2),
            "client 2 unaffected (its OWN bucket — not the shared proxy bucket)"
        );
        assert!(
            !rl.allow(c1),
            "client 1 now limited (its own bucket exhausted)"
        );
    }

    #[test]
    fn xff_two_proxies_client_is_index_n_minus_one_from_right() {
        // hops=2: the request traversed TWO proxies, so the header carries two entries
        // `XFF: <client>, <proxy1>` (proxy1 appended its view = the client as IT saw it; the final proxy
        // is our peer and writes nothing of itself). The client is the N-th from the right, 0-based
        // index N-1 = 1 ⇒ the LEFTMOST entry here.
        let final_proxy = ipv4(10, 0, 0, 1); // our direct peer (proxy2)
        let client = ipv4(203, 0, 113, 200);
        assert_eq!(
            resolve_client_ip(final_proxy, Some("203.0.113.200, 192.0.2.50"), 2),
            client,
            "hops=2 ⇒ client = index N-1 = 1 from the right (the leftmost of the two entries)"
        );
        assert_eq!(
            client_ip_from_xff("203.0.113.200, 192.0.2.50", 2),
            Some(client)
        );
    }

    #[test]
    fn xff_too_short_falls_back_to_peer() {
        // A trusted-hops config but FEWER than N valid entries ⇒ fall back to the direct peer IP, never
        // invent a client (fail-safe). With the corrected index N-1, "too short" means < N entries.
        let peer = ipv4(10, 0, 0, 1);
        // hops=2 but only ONE entry present ⇒ index N-1 = 1 is out of range ⇒ fall back to peer.
        assert_eq!(
            resolve_client_ip(peer, Some("198.51.100.9"), 2),
            peer,
            "hops=2 with a single XFF entry is too short ⇒ fall back to peer"
        );
        // hops=1 but the only token is junk (no valid IP) ⇒ zero valid entries ⇒ fall back to peer.
        assert_eq!(
            resolve_client_ip(peer, Some("not-an-ip"), 1),
            peer,
            "hops=1 with no PARSEABLE entry ⇒ fall back to peer"
        );
        assert_eq!(
            resolve_client_ip(peer, Some(""), 1),
            peer,
            "empty XFF ⇒ fall back to peer"
        );
        assert_eq!(
            resolve_client_ip(peer, None, 1),
            peer,
            "absent XFF ⇒ fall back to peer"
        );
    }

    #[test]
    fn xff_skips_junk_tokens_from_the_right() {
        // A proxy that wrote a junk token must not shift the client index — non-IP tokens are skipped
        // before indexing. hops=2, junk between the two real entries ⇒ still resolves the client at the
        // (now junk-free) index N-1 = 1.
        let client = client_ip_from_xff("203.0.113.200, garbage, 192.0.2.50", 2);
        assert_eq!(client, Some(ipv4(203, 0, 113, 200)));
        // hops=1 with a trailing junk token after the client: junk skipped from the right ⇒ index 0 is
        // the real client entry.
        assert_eq!(
            client_ip_from_xff("198.51.100.5, garbage", 1),
            Some(ipv4(198, 51, 100, 5))
        );
    }

    // --- memory-cap (FIX 5) tests ---

    /// A distinct cold IP for the spray, derived from a counter so each lands somewhere in the shard
    /// space. Uses 198.18.0.0/15 (RFC 2544 benchmarking range) expanded across two octets, plus a third,
    /// to mint > cap*shards distinct addresses.
    fn churn_ip(n: u32) -> IpAddr {
        // 198.18.x.y over x,y gives 65k addresses — plenty above the test cap × shards.
        let b = ((n >> 8) & 0xff) as u8;
        let c = (n & 0xff) as u8;
        ipv4(198, 18, b, c)
    }

    #[test]
    fn resident_map_is_bounded_under_distinct_ip_churn() {
        // The MEMORY CEILING: spraying FAR more distinct IPs than the cap allows must NOT grow the
        // resident map without bound — the hard per-shard cap bounds total buckets to shards × cap.
        // (Without the cap, the IDLE_TTL retain alone would let the map grow to ~arrival × 600s.)
        let mut rl = RateLimiter::new(0.0001, 1.0, 0, false, false);
        let cap = 4usize;
        rl.set_max_buckets_per_shard_for_test(cap);

        // Spray 20_000 distinct IPs — vastly more than NUM_SHARDS × cap (= 256).
        for n in 0..20_000u32 {
            let _ = rl.allow(churn_ip(n));
        }

        let total = rl.bucket_count_for_test();
        let ceiling = NUM_SHARDS * cap;
        assert!(
            total <= ceiling,
            "resident buckets {total} must stay <= shards×cap = {ceiling} under churn (got {total})"
        );
    }

    #[test]
    fn hot_ip_throttle_survives_a_spray_of_cold_ips() {
        // THE VERIFIER'S EXACT PROBE: a HOT (actively-requesting) IP that is over its limit must STAY
        // throttled after a spray of cold distinct IPs — an attacker spraying cold IPs must NOT evict
        // the hot IP's exhausted bucket (which would reset it to full burst and dodge the limit). The
        // eviction policy is LRU-by-`last_seen`, so a continuously-warm IP is never the coldest, hence
        // never evicted. We model "actively requesting" by re-touching H after each cold IP.
        let mut rl = RateLimiter::new(0.0001, 1.0, 0, false, false); // capacity 1 ⇒ H exhausts after 1
        rl.set_max_buckets_per_shard_for_test(2); // a TIGHT cap so eviction fires constantly

        let hot = ipv4(203, 0, 113, 250);
        // Exhaust H's bucket: first request passes, H is now throttled.
        assert!(
            rl.allow(hot),
            "hot IP's first request (the single-capacity burst)"
        );
        assert!(
            !rl.allow(hot),
            "hot IP is now throttled (capacity 1 exhausted)"
        );

        // Spray cold IPs, keeping H WARM by re-touching it after each — so H is always the most-recently
        // seen in its shard and can never be the coldest (evicted) bucket.
        for n in 0..5_000u32 {
            let _ = rl.allow(churn_ip(n));
            // Re-touch H: it stays throttled (still no token) AND refreshes its last_seen so the churn
            // cannot evict it. The assertion is the load-bearing one — if H had been evicted, this would
            // recreate H's bucket at FULL burst and return `true` (a reset throttle = the bypass).
            assert!(
                !rl.allow(hot),
                "hot IP n={n}: must STAY throttled — its exhausted bucket survived the cold-IP spray \
                 (not evicted+reset). A reset would let it pass, the exact eviction bypass."
            );
        }

        // And the map stayed bounded throughout.
        assert!(
            rl.bucket_count_for_test() <= NUM_SHARDS * 2,
            "map stayed bounded under the spray"
        );
    }

    #[test]
    fn new_ip_at_cap_is_tracked_after_evicting_the_coldest() {
        // A brand-new IP arriving at a FULL shard is still TRACKED (it gets a bucket after the coldest
        // is evicted) — so the limiter keeps protecting against the newcomer; we chose eviction (a) over
        // fail-open-when-full (b) precisely so a not-tracked IP can't get a free pass. Verify a new IP
        // that lands in a full shard is throttled on its SECOND request (its bucket persists).
        let mut rl = RateLimiter::new(0.0001, 1.0, 0, false, false);
        rl.set_max_buckets_per_shard_for_test(2);
        // Fill the space with churn so shards reach the cap.
        for n in 0..2_000u32 {
            let _ = rl.allow(churn_ip(n));
        }
        // A fresh IP: first request passes (burst 1), second is throttled — proving it got + KEPT a
        // bucket (it was tracked, not given a fail-open pass).
        let fresh = ipv4(192, 0, 2, 200);
        assert!(rl.allow(fresh), "fresh IP first request passes (its burst)");
        assert!(
            !rl.allow(fresh),
            "fresh IP is tracked + throttled on its 2nd request even at the shard cap (eviction, not \
             fail-open)"
        );
    }

    #[test]
    fn retry_after_within_bounds() {
        for _ in 0..1000 {
            let v = jittered_retry_after_secs();
            assert!(
                (RETRY_AFTER_BASE_SECS..=RETRY_AFTER_BASE_SECS + RETRY_AFTER_JITTER_SECS)
                    .contains(&v),
                "retry-after {v} out of band"
            );
        }
    }

    #[test]
    fn limited_response_is_429_with_retry_after_and_no_store() {
        let resp = rate_limited_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(
            resp.headers().contains_key(header::RETRY_AFTER),
            "must carry Retry-After"
        );
        assert_eq!(
            resp.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-store"
        );
    }
}
