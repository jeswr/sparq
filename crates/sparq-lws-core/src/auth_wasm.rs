//! Host-provided identity for the WebAssembly request-handling core.
//!
//! Solid OIDC verification is deliberately not compiled for wasm: the pinned verifier selects an
//! AWS-LC backend that cannot target `wasm32-unknown-unknown`. The JavaScript host verifies OIDC and
//! inserts a `VerifiedToken` into each routed request. A request without that extension is public.

/// Identity already authenticated by the JavaScript host.
///
/// The wasm core consumes only the verified WebID because WAC decisions are the only portable auth
/// concern. Token validation, issuer checks, proof-of-possession, and network discovery stay in the
/// host and are not represented as successful work inside wasm.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VerifiedToken {
    /// The authenticated agent WebID, or `None` for a public request.
    pub web_id: Option<String>,
}

impl VerifiedToken {
    /// Construct credentials from a WebID already authenticated by the JavaScript host.
    #[must_use]
    pub fn from_authenticated_webid(web_id: impl Into<String>) -> Self {
        Self {
            web_id: Some(web_id.into()),
        }
    }

    /// Public, unauthenticated credentials.
    #[must_use]
    pub fn public() -> Self {
        Self::default()
    }

    /// Whether these credentials represent a public request.
    #[must_use]
    pub fn is_public(&self) -> bool {
        self.web_id.is_none()
    }
}
