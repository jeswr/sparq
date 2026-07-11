// AUTHORED-BY Claude Opus 4.8
//! RFC 8705 §3 — mutual-TLS **certificate-bound access token** verification core.
//!
//! This is the RS-side token-binding check mandated by
//! [RFC 8705 §3](https://www.rfc-editor.org/rfc/rfc8705.html#section-3): the RS "**MUST obtain,
//! from its TLS implementation layer, the client certificate used for mutual TLS and MUST verify
//! that the certificate matches the certificate associated with the access token**". The AS
//! embeds `"cnf":{"x5t#S256":"<base64url(SHA-256(cert DER))>"}` (§3.1); the RS hashes the presented
//! certificate and compares it, in constant time, to that value. The check is *flavour-agnostic*
//! (identical for the PKI `tls_client_auth` and the self-signed `self_signed_tls_client_auth`
//! variants — §2.1/§2.2): trust is the thumbprint match, **not** the certificate chain.
//!
//! ## Scope (T1a — the verification core only)
//! This module is deliberately transport-free. It takes two INJECTED inputs:
//! - `expected` — the token's `cnf.x5t#S256`, supplied by the OIDC verifier once it exposes a
//!   cert-bound [`Confirmation`](super::Confirmation) (the owner-gated `solid-oidc-verifier`
//!   follow-up); and
//! - `presented` — the SHA-256 thumbprint of the peer certificate the TLS acceptor read from the
//!   live connection (the T1b acceptor-wiring follow-up).
//!
//! Neither seam is wired here, so the core is unit-testable in isolation. [`CertThumbprint::from_cert_der`]
//! is the exact entry point the T1b acceptor will call once per connection.
//!
//! ## Fail-closed invariants (all tested below)
//! - A cert-bound token presented on a connection with **no** client certificate ⇒
//!   [`CertBindingDenial::NoCertPresented`] (a cert-bound token is NEVER accepted bare — RFC 8705 §3).
//! - A **wrong** thumbprint ⇒ [`CertBindingDenial::ThumbprintMismatch`].
//! - The compare is **constant-time** ([`subtle::ConstantTimeEq`]) so it cannot leak, via timing, how
//!   many leading bytes of a forged certificate's hash matched.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// A SHA-256 certificate thumbprint — RFC 8705 §3.1 `x5t#S256`: the SHA-256 digest of the DER
/// encoding of an X.509 certificate. Exactly 32 bytes.
///
/// `PartialEq`/`Eq` are **hand-implemented over the constant-time compare** rather than derived: a
/// derived `==` would be a variable-time byte compare a future caller could reach for by mistake on
/// this secret-adjacent value. Implementing it via [`subtle::ConstantTimeEq`] means every comparison
/// of a thumbprint — the operator `==` and [`verify_cert_binding`] alike — is constant-time.
#[derive(Clone, Copy)]
pub struct CertThumbprint([u8; 32]);

impl PartialEq for CertThumbprint {
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.0.ct_eq(&other.0))
    }
}

impl Eq for CertThumbprint {}

impl std::fmt::Debug for CertThumbprint {
    /// Renders the base64url thumbprint (a public certificate hash, not a secret) so logs/test
    /// output are legible without exposing anything sensitive.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CertThumbprint({})", self.to_base64url())
    }
}

/// Why a base64url `cnf.x5t#S256` value could not be parsed into a [`CertThumbprint`].
///
/// Any parse failure is fail-closed at the call site: without a well-formed expected thumbprint the
/// token cannot be treated as cert-bound, so it can never be *accepted* via this path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbprintParseError {
    /// The value was not valid base64url (unpadded, per the JOSE `x5t#S256` convention).
    InvalidBase64,
    /// The value decoded, but not to exactly 32 bytes (a SHA-256 digest is 32 bytes).
    WrongLength(usize),
}

impl std::fmt::Display for ThumbprintParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThumbprintParseError::InvalidBase64 => {
                write!(f, "cnf.x5t#S256 is not valid base64url")
            }
            ThumbprintParseError::WrongLength(n) => {
                write!(
                    f,
                    "cnf.x5t#S256 decoded to {n} bytes, expected 32 (SHA-256)"
                )
            }
        }
    }
}

impl std::error::Error for ThumbprintParseError {}

impl CertThumbprint {
    /// Compute the thumbprint of a certificate from its DER bytes — `SHA-256(DER)`.
    ///
    /// This is the entry point the T1b TLS acceptor will call **once per connection** on the peer
    /// certificate (`rustls ServerConnection::peer_certificates()[0].der()`), caching the result for
    /// every request on that connection (RFC 8705's per-request cost is then a 32-byte compare).
    pub fn from_cert_der(der: &[u8]) -> Self {
        Self(Sha256::digest(der).into())
    }

    /// Parse a token's `cnf.x5t#S256` claim value — base64url (unpadded) of a 32-byte SHA-256 digest.
    ///
    /// Unpadded base64url matches the JOSE `x5t#S256` convention (RFC 7515) and Keycloak's output,
    /// and is consistent with how the rest of this server encodes thumbprints (`cnf.jkt`,
    /// [`crate::auth_cache`]).
    pub fn from_base64url(value: &str) -> Result<Self, ThumbprintParseError> {
        let bytes = URL_SAFE_NO_PAD
            .decode(value.as_bytes())
            .map_err(|_| ThumbprintParseError::InvalidBase64)?;
        let arr: [u8; 32] = bytes
            .as_slice()
            .try_into()
            .map_err(|_| ThumbprintParseError::WrongLength(bytes.len()))?;
        Ok(Self(arr))
    }

    /// Construct directly from 32 raw digest bytes (e.g. from a rustls-computed hash).
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// The raw 32 digest bytes. For serialization / tests only — never for a security comparison
    /// (use [`verify_cert_binding`], which compares in constant time).
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// The base64url (unpadded) rendering — the on-the-wire `cnf.x5t#S256` form. Round-trips with
    /// [`from_base64url`](Self::from_base64url).
    pub fn to_base64url(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0)
    }
}

/// The outcome of the RFC 8705 §3 token-binding check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertBindingOutcome {
    /// The presented certificate's thumbprint matched the token's `cnf.x5t#S256`.
    Confirmed,
    /// The binding did not hold — see [`CertBindingDenial`]. **Fail closed**: the caller MUST reject
    /// (401), never fall back to accepting the token bare.
    Denied(CertBindingDenial),
}

/// Why an RFC 8705 cert binding was denied. Both are fail-closed rejects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertBindingDenial {
    /// The token is cert-bound (`cnf.x5t#S256`) but the connection presented no client certificate.
    /// A cert-bound token is never accepted without a matching certificate (RFC 8705 §3).
    NoCertPresented,
    /// A client certificate was presented, but its thumbprint did not match the token's `cnf.x5t#S256`.
    ThumbprintMismatch,
}

/// Verify an RFC 8705 §3 certificate-bound access token: does the certificate presented on the TLS
/// connection match the thumbprint the token is bound to?
///
/// - `expected` — the token's `cnf.x5t#S256` thumbprint.
/// - `presented` — the thumbprint of the peer certificate from the live TLS connection, or `None`
///   when the connection presented no client certificate.
///
/// Fail-closed: `None` presented ⇒ [`CertBindingDenial::NoCertPresented`]; a non-matching
/// certificate ⇒ [`CertBindingDenial::ThumbprintMismatch`]. The match itself is a constant-time
/// 32-byte compare ([`subtle::ConstantTimeEq`]).
pub fn verify_cert_binding(
    expected: &CertThumbprint,
    presented: Option<&CertThumbprint>,
) -> CertBindingOutcome {
    match presented {
        None => CertBindingOutcome::Denied(CertBindingDenial::NoCertPresented),
        Some(p) => {
            // Constant-time compare — do not short-circuit on the first differing byte.
            if bool::from(p.0.ct_eq(&expected.0)) {
                CertBindingOutcome::Confirmed
            } else {
                CertBindingOutcome::Denied(CertBindingDenial::ThumbprintMismatch)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A stable, arbitrary certificate stand-in. The binding check only ever sees DER *bytes*; it
    // never parses X.509, so any byte string exercises the hash+compare path exactly as a real cert
    // would (the DER structure is the acceptor's/rustls's concern, not this core's).
    const CERT_A: &[u8] = b"-----fake-der-cert-A-----";
    const CERT_B: &[u8] = b"-----fake-der-cert-B-----";

    #[test]
    fn from_cert_der_matches_known_sha256() {
        // Anchor the digest to a known SHA-256 so a future refactor of the hashing can't silently
        // change what a certificate hashes to. SHA-256("") = e3b0c442...
        let empty = CertThumbprint::from_cert_der(b"");
        assert_eq!(
            empty.to_base64url(),
            // base64url(no-pad) of SHA-256("")
            "47DEQpj8HBSa-_TImW-5JCeuQeRkm5NMpJWZG3hSuFU"
        );
    }

    #[test]
    fn confirmed_when_presented_matches_expected() {
        let expected = CertThumbprint::from_cert_der(CERT_A);
        let presented = CertThumbprint::from_cert_der(CERT_A);
        assert_eq!(
            verify_cert_binding(&expected, Some(&presented)),
            CertBindingOutcome::Confirmed
        );
    }

    #[test]
    fn fail_closed_no_cert_presented() {
        // A cert-bound token but the connection carried no client certificate ⇒ DENY.
        let expected = CertThumbprint::from_cert_der(CERT_A);
        assert_eq!(
            verify_cert_binding(&expected, None),
            CertBindingOutcome::Denied(CertBindingDenial::NoCertPresented)
        );
    }

    #[test]
    fn fail_closed_wrong_thumbprint() {
        let expected = CertThumbprint::from_cert_der(CERT_A);
        let presented = CertThumbprint::from_cert_der(CERT_B);
        assert_eq!(
            verify_cert_binding(&expected, Some(&presented)),
            CertBindingOutcome::Denied(CertBindingDenial::ThumbprintMismatch)
        );
    }

    #[test]
    fn base64url_roundtrip() {
        let t = CertThumbprint::from_cert_der(CERT_A);
        let encoded = t.to_base64url();
        let decoded = CertThumbprint::from_base64url(&encoded).expect("valid");
        assert_eq!(decoded.as_bytes(), t.as_bytes());
    }

    #[test]
    fn from_base64url_rejects_wrong_length() {
        // base64url(no-pad) of 3 bytes ("abc") ⇒ decodes fine but is not 32 bytes.
        let short = URL_SAFE_NO_PAD.encode(b"abc");
        assert_eq!(
            CertThumbprint::from_base64url(&short),
            Err(ThumbprintParseError::WrongLength(3))
        );
    }

    #[test]
    fn from_base64url_rejects_invalid_base64() {
        // '*' is not in the base64url alphabet.
        assert_eq!(
            CertThumbprint::from_base64url("not*valid*base64url"),
            Err(ThumbprintParseError::InvalidBase64)
        );
        // Standard-base64 padding is not accepted by the URL_SAFE_NO_PAD engine.
        assert_eq!(
            CertThumbprint::from_base64url("=="),
            Err(ThumbprintParseError::InvalidBase64)
        );
    }

    #[test]
    fn verify_accepts_thumbprint_decoded_from_the_wire() {
        // The realistic path: the token's cnf.x5t#S256 arrives as base64url; the acceptor computes
        // the presented cert's thumbprint from DER. They must agree for the SAME certificate.
        let on_the_wire = CertThumbprint::from_cert_der(CERT_A).to_base64url();
        let expected = CertThumbprint::from_base64url(&on_the_wire).expect("valid");
        let presented = CertThumbprint::from_cert_der(CERT_A);
        assert_eq!(
            verify_cert_binding(&expected, Some(&presented)),
            CertBindingOutcome::Confirmed
        );
    }
}
