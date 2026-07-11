// AUTHORED-BY Claude Opus 4.8
//! Explicit request-body size limit — an audited, configurable ceiling on the number of bytes any
//! single request body may buffer, enforced with a **413 Payload Too Large**.
//!
//! ## Why (the memory-exhaustion gap this closes)
//! The LDP handlers read a request body via axum's [`Bytes`](axum::body::Bytes) extractor, which
//! buffers the WHOLE body into memory before the handler runs. axum applies an *implicit* 2 MiB
//! default limit to that extractor — but that default is (a) not an explicit, audited contract of THIS
//! crate (a future axum bump could change it silently), and (b) not configurable, and (c) invisible in
//! the router assembly. More importantly, the AGGREGATE is unbounded relative to concurrency: up to
//! `max_concurrency` in-flight requests could each buffer a body, so the real resident-memory ceiling
//! for body buffering is `max_concurrency × body_limit`. Making the per-request limit an EXPLICIT,
//! env-tunable [`axum::extract::DefaultBodyLimit`] layer ([`layer`]) lets an operator size that product
//! to their box's RAM budget, and pins the value as a tested invariant.
//!
//! ## Where it sits
//! The limit is applied as a router layer over the application routes (see [`crate::app`]). It only
//! bounds the buffered body of a request that reaches a body-reading handler — auth still runs first,
//! so an ANONYMOUS oversized body is rejected by auth (401) before any body is read; the limit's job is
//! to bound an AUTHENTICATED oversized upload (and the aggregate across concurrency). A request whose
//! `Content-Length` (or streamed length) exceeds the limit gets a **413** from the extractor.
//!
//! 🔒 Security framing: like the other admission layers, this can only REJECT a request (413) — it
//! never grants access, so it can never be an authorization bypass. It is a pure memory-safety bound.
//!
//! ## Sizing + the deliberate absence of an "unlimited" sentinel
//! The default ([`DEFAULT_MAX_BODY_BYTES`]) is **2 MiB** — identical to axum's implicit default, so the
//! default build behaves exactly as before, only now the value is explicit + owned. There is
//! INTENTIONALLY no `off`/`unlimited` sentinel: an unbounded body is precisely the memory-exhaustion
//! vector this exists to close, so the limit is always finite (a `0`/invalid env value falls back to
//! the default rather than disabling the bound — never silently remove the ceiling on a typo). An
//! operator serving large uploads RAISES the number; they cannot remove the ceiling.

use axum::extract::DefaultBodyLimit;

/// Env var: the maximum request-body size in BYTES. Unset / empty / non-numeric / `0` ⇒
/// [`DEFAULT_MAX_BODY_BYTES`] (never disable the ceiling on a typo — there is deliberately no
/// "unlimited" option; see the module docs). A positive integer ⇒ that many bytes.
pub const ENV_MAX_BODY_BYTES: &str = "SOLID_SERVER_MAX_BODY_BYTES";

/// The default maximum request-body size (bytes). **2 MiB** — identical to axum's own implicit
/// `DefaultBodyLimit`, so making the limit explicit changes NOTHING about the default behaviour; it
/// only makes the value owned, audited, tested, and configurable. An operator raises it for a
/// large-upload workload (mindful that the resident-memory ceiling for body buffering is
/// `max_concurrency × this`).
pub const DEFAULT_MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

/// Resolve the max body size (bytes) from the environment.
pub fn max_body_bytes_from_env() -> usize {
    parse_max_body_bytes(std::env::var(ENV_MAX_BODY_BYTES).ok())
}

/// Testable core of [`max_body_bytes_from_env`].
/// - absent / empty / non-numeric / `0` ⇒ [`DEFAULT_MAX_BODY_BYTES`] (never silently remove the
///   ceiling — a `0` body limit would 413 EVERY body-bearing request, and there is no unlimited mode);
/// - a positive integer ⇒ that many bytes.
pub fn parse_max_body_bytes(raw: Option<String>) -> usize {
    match raw.as_deref().map(str::trim) {
        None | Some("") => DEFAULT_MAX_BODY_BYTES,
        Some(s) => match s.parse::<usize>() {
            Ok(0) | Err(_) => DEFAULT_MAX_BODY_BYTES,
            Ok(n) => n,
        },
    }
}

/// The [`DefaultBodyLimit`] router layer enforcing `max_bytes` (a request body over the limit ⇒ 413).
/// A thin, single-sourced constructor so both router builders (see [`crate::app`]) apply the limit the
/// same way and the value is never duplicated as a bare `DefaultBodyLimit::max(..)` call.
pub fn layer(max_bytes: usize) -> DefaultBodyLimit {
    DefaultBodyLimit::max(max_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_on_absent_empty_invalid_or_zero() {
        // Never silently remove the ceiling on a typo — there is no "unlimited" sentinel.
        assert_eq!(parse_max_body_bytes(None), DEFAULT_MAX_BODY_BYTES);
        assert_eq!(
            parse_max_body_bytes(Some("".into())),
            DEFAULT_MAX_BODY_BYTES
        );
        assert_eq!(
            parse_max_body_bytes(Some("   ".into())),
            DEFAULT_MAX_BODY_BYTES
        );
        assert_eq!(
            parse_max_body_bytes(Some("abc".into())),
            DEFAULT_MAX_BODY_BYTES
        );
        assert_eq!(
            parse_max_body_bytes(Some("0".into())),
            DEFAULT_MAX_BODY_BYTES
        );
        assert_eq!(
            parse_max_body_bytes(Some("-5".into())),
            DEFAULT_MAX_BODY_BYTES
        );
    }

    #[test]
    fn explicit_positive_is_honoured() {
        assert_eq!(parse_max_body_bytes(Some("1".into())), 1);
        assert_eq!(parse_max_body_bytes(Some("16".into())), 16);
        assert_eq!(
            parse_max_body_bytes(Some("  1048576  ".into())),
            1024 * 1024
        );
        assert_eq!(
            parse_max_body_bytes(Some("10485760".into())),
            10 * 1024 * 1024
        );
    }

    #[test]
    fn default_matches_axum_implicit_two_mib() {
        // The default is deliberately 2 MiB — axum's own implicit DefaultBodyLimit — so making the
        // limit explicit is behaviour-preserving by default.
        assert_eq!(DEFAULT_MAX_BODY_BYTES, 2 * 1024 * 1024);
    }
}
