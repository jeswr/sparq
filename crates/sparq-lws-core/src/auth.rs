// AUTHORED-BY Claude Opus 4.8
//! DPoP-bound Solid-OIDC authentication middleware.
//!
//! Auth is **delegated** to the standalone
//! [`solid-oidc-verifier`](https://github.com/jeswr/solid-oidc-verifier) crate — this server does
//! **not** reimplement token/DPoP verification (the spike's load-bearing rule R1). This middleware
//! is the thin axum adapter: it reconstructs the verifier's [`AuthRequest`] from the HTTP request,
//! calls [`Verifier::verify`], and either injects the [`VerifiedToken`] into request extensions for
//! downstream handlers or returns the verifier's own status + `WWW-Authenticate` challenge unchanged.
//!
//! The error contract (401 invalid_token / 503 replay-store-unavailable / the challenge string) is
//! owned entirely by the verifier — this layer never re-derives it. An absent `Authorization` header
//! yields the verifier's public/unauthenticated [`VerifiedToken`] (the LDP layer then enforces that
//! public credentials reach only public resources — M2's WAC step).

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use solid_oidc_verifier::config::JwksProvider;
use solid_oidc_verifier::error::{ErrorKind, VerifyError};
use solid_oidc_verifier::replay::ReplayStore;
use solid_oidc_verifier::verifier::{AuthRequest, Verifier, X5tS256};

pub use solid_oidc_verifier::verifier::VerifiedToken;

use crate::auth_cache::{CacheDecision, VerifiedTokenCache};
use crate::error::ServerError;
use crate::ldp::target::parse_target;
use crate::pop::cert_bound::{CertBindingOutcome, CertThumbprint};
use crate::pop::conn::ConnPop;
use crate::pop::sk::verify::{verify_attested_request, SkDecision};
use crate::pop::sk::{ConnSk, SkSession, SkState};
use crate::pop::{dispatch as pop_dispatch, Confirmation, PopRoute};

/// Everything the auth layer needs: the verifier and the server's public base URL.
///
/// Generic over the verifier's [`JwksProvider`] + [`ReplayStore`] seams so M1 can use the in-memory
/// `StaticJwksProvider` + `InMemoryReplayStore` test doubles, and M2 can swap in the
/// network-backed JWKS (OIDC discovery) + a shared (Redis) replay store with no change here.
pub struct AuthContext<J: JwksProvider, R: ReplayStore> {
    pub verifier: Verifier<J, R>,
    /// The server's public origin (no trailing slash), used to reconstruct the DPoP `htu`.
    pub base_url: String,
    /// The round-3 verified-access-token cache (default-on; see [`crate::auth_cache`]). `None` =>
    /// every authenticated request runs the verifier's full `verify()` (the pre-round-3 behaviour).
    ///
    /// When `Some`, the cache + `replay` MUST share the SAME replay store the `verifier` holds -- the
    /// hit path marks the proof `jti` through `replay`, so it must be the verifier's store or a
    /// replay used on the miss path could be replayed on the hit path. The server wires this by
    /// building one `Arc<InMemoryReplayStore>`, giving the verifier `SharedReplay<_>` over it and the
    /// cache a clone of the same `Arc`. This `replay` handle is exactly that clone.
    cache: Option<TokenCache<R>>,
    /// PoP Tier-1b: whether the RFC 8705 mTLS cert-bound-token confirmation dispatch is ACTIVE (the
    /// `SOLID_SERVER_MTLS_BOUND_TOKENS` flag, wired in `main`). **Default `false`** — when off,
    /// [`authenticate`](Self::authenticate) runs the pre-Tier-1b path byte-for-byte (no confirmation
    /// dispatch; the presented certificate is ignored). When on, a *successfully verified* token's `cnf`
    /// confirmation is dispatched via [`crate::pop`]: a cert-bound token is matched against the
    /// connection's presented certificate (fail-closed), a malformed cert binding is rejected, a
    /// multi-binding token is refused, and a DPoP/public token is unchanged. See
    /// [`finalize_pop`](Self::finalize_pop).
    mtls_bound_tokens: bool,
    /// PoP Tier 2 (DPoP-SK, `SOLID_SERVER_DPOP_SK`): the shared session state. **Default `None`**
    /// — when unset, the middleware runs the pre-Tier-2 path byte-for-byte (any `Signature*`
    /// headers are ignored and DPoP remains the only accepted PoP). When set, a request bearing a
    /// `dpop-sk`-tagged RFC 9421 signature is processed under the DPoP-SK profile EXCLUSIVELY
    /// (see [`crate::pop::sk::verify`]); everything else is unchanged.
    sk: Option<Arc<SkState>>,
}

/// The token cache + the shared replay handle it marks `jti`s through (the SAME store the verifier
/// uses). Bundled so they cannot be wired independently (which would split the replay set).
struct TokenCache<R: ReplayStore> {
    cache: VerifiedTokenCache,
    replay: Arc<R>,
}

impl<J: JwksProvider, R: ReplayStore> AuthContext<J, R> {
    /// Construct WITHOUT the verified-token cache -- every authenticated request runs the full verifier
    /// (the pre-round-3 path). Used where no shared replay handle is available (e.g. unit harnesses).
    pub fn new(verifier: Verifier<J, R>, base_url: impl Into<String>) -> Self {
        Self {
            verifier,
            base_url: base_url.into(),
            cache: None,
            mtls_bound_tokens: false,
            sk: None,
        }
    }

    /// Construct WITH the round-3 verified-access-token cache. `replay` MUST be a clone of the `Arc`
    /// the `verifier`'s `SharedReplay` wraps (so the hit + miss paths mark the SAME jti set -- the
    /// replay-bypass guard). See [`crate::auth_cache`].
    pub fn with_cache(
        verifier: Verifier<J, R>,
        base_url: impl Into<String>,
        cache: VerifiedTokenCache,
        replay: Arc<R>,
    ) -> Self {
        Self {
            verifier,
            base_url: base_url.into(),
            cache: Some(TokenCache { cache, replay }),
            mtls_bound_tokens: false,
            sk: None,
        }
    }

    /// Enable (or disable) the PoP Tier-1b RFC 8705 mTLS cert-bound-token confirmation dispatch
    /// (`SOLID_SERVER_MTLS_BOUND_TOKENS`). Off by default; `main` turns it on only when the flag is set
    /// AND in-process TLS is terminating (so a client certificate can actually be presented). A
    /// builder-style setter so the two constructors above stay unchanged for every existing caller/test.
    pub fn with_mtls_bound_tokens(mut self, enabled: bool) -> Self {
        self.mtls_bound_tokens = enabled;
        self
    }

    /// Enable the PoP Tier-2 DPoP-SK fast path (`SOLID_SERVER_DPOP_SK`) by supplying the shared
    /// session state. `None` (the default) is byte-identical to the pre-Tier-2 middleware. A
    /// builder-style setter, mirroring [`with_mtls_bound_tokens`](Self::with_mtls_bound_tokens).
    pub fn with_dpop_sk(mut self, sk: Option<Arc<SkState>>) -> Self {
        self.sk = sk;
        self
    }

    /// The DPoP-SK state, when the tier is enabled (used by `crate::app` to mount the
    /// establishment routes + advertise the profile in the RFC 9728 metadata).
    pub fn sk(&self) -> Option<&Arc<SkState>> {
        self.sk.as_ref()
    }

    /// Whether the PoP Tier-1b mTLS cert-bound-token dispatch is enabled (used by `crate::app`
    /// for the RFC 9728 `tls_client_certificate_bound_access_tokens` metadata member).
    pub fn mtls_bound_tokens(&self) -> bool {
        self.mtls_bound_tokens
    }

    /// Verify the request and return the caller's [`VerifiedToken`] (possibly public), or the
    /// verifier's error mapped onto a [`ServerError::Unauthorized`] (carrying its status + challenge).
    ///
    /// ## Round-3 verified-access-token cache (when enabled via [`with_cache`](Self::with_cache))
    /// For a `DPoP <token>` request, the cache may already hold the verified result of THIS token.
    /// On a cache HIT the access-token signature + RFC-9068 claims are NOT re-verified (the saving),
    /// but the FRESH DPoP proof + `jti` replay + `cnf.jkt` binding ARE fully verified for this request
    /// (the cache cannot turn a failing proof into a success). On a MISS (or any non-DPoP request) the
    /// full verifier runs, and a successful DPoP-bound result is inserted for the token's `exp` window.
    /// Disabling the cache is byte-identical to the pre-round-3 path.
    ///
    /// ## PoP Tier-1b confirmation dispatch (when the mTLS flag is on)
    /// After the verifier (or the cache) yields a token, `finalize_pop` runs the
    /// RFC 8705 mTLS confirmation dispatch when [`with_mtls_bound_tokens`](Self::with_mtls_bound_tokens)
    /// is enabled: `presented_cert` is the thumbprint of the client certificate on THIS TLS connection
    /// (from the [`ConnPop`] request extension, `None` when no client cert / not on the TLS+mTLS path).
    /// When the flag is off, `presented_cert` is ignored and the result is byte-identical to before.
    pub fn authenticate(
        &self,
        authorization: Option<String>,
        dpop: Option<String>,
        method: &str,
        path: &str,
    ) -> Result<VerifiedToken, ServerError> {
        // No presented client certificate (the non-mTLS / no-TLS caller). With the mTLS flag off this is
        // byte-identical to the pre-Tier-1b path; with it on, a cert-bound token is denied fail-closed
        // (no cert to satisfy the binding). The mTLS serve path calls
        // [`authenticate_with_cert`](Self::authenticate_with_cert) with the connection's cert.
        self.authenticate_with_cert(authorization, dpop, method, path, None)
    }

    /// As [`authenticate`](Self::authenticate), but supplying the client-certificate thumbprint the peer
    /// presented on THIS TLS connection (PoP Tier-1b). Called by the auth middleware on the mTLS serve
    /// path from the [`ConnPop`] request extension; `presented_cert` is `None` when no client cert was
    /// presented / the mTLS path is inactive.
    pub fn authenticate_with_cert(
        &self,
        authorization: Option<String>,
        dpop: Option<String>,
        method: &str,
        path: &str,
        presented_cert: Option<&CertThumbprint>,
    ) -> Result<VerifiedToken, ServerError> {
        let token = self.authenticate_inner(authorization, dpop, method, path, presented_cert)?;
        self.finalize_pop(token, presented_cert)
    }

    /// The verify-or-cache core: returns the verifier's/cache's decision. Threads the RS-verified
    /// client-certificate thumbprint into the verifier's [`AuthRequest`] (PoP Tier-1 LIVE) so the
    /// verifier can itself ADMIT a cert-bound Bearer token under `require_dpop` and enforce the
    /// RFC 8705 §3.1 thumbprint match; [`authenticate`](Self::authenticate) then applies
    /// [`finalize_pop`](Self::finalize_pop) on top (dispatch-level match, defence-in-depth) when the
    /// mTLS flag is on.
    fn authenticate_inner(
        &self,
        authorization: Option<String>,
        dpop: Option<String>,
        method: &str,
        path: &str,
        presented_cert: Option<&CertThumbprint>,
    ) -> Result<VerifiedToken, ServerError> {
        // Reconstruct the htu the verifier checks the DPoP proof against. A bad target is a 400
        // before we even reach the verifier (it would otherwise reject on htu mismatch as a 401).
        let target = parse_target(&self.base_url, path)?;
        let method_uc = method.to_ascii_uppercase();

        // PoP Tier-1 LIVE: the RS-verified client-certificate thumbprint threaded into the verifier's
        // `AuthRequest.client_cert_x5t_s256` so the verifier can ADMIT a cert-bound Bearer token under
        // `require_dpop` — its proof-of-possession is the client certificate at the TLS layer (RFC 8705
        // §3), in place of a DPoP proof — and enforce the §3.1 thumbprint match itself (constant-time,
        // fail-closed). Env-gated + fail-closed: `None` whenever the mTLS flag is off OR the connection
        // presented no client certificate. With the flag OFF this is byte-identical to the pre-Tier-1
        // path (the field stays `None`, so the verifier keeps DPoP mandatory and rejects a cert-bound
        // Bearer exactly as before). Encoded base64url-no-pad to match how `cnf.x5t#S256` is encoded, or
        // the verifier's byte-compare would (fail-closed) reject. `finalize_pop`'s dispatch-level match
        // still runs afterwards as defence-in-depth.
        let client_cert_x5t_s256: Option<String> = if self.mtls_bound_tokens {
            presented_cert.map(CertThumbprint::to_base64url)
        } else {
            None
        };

        // Cache fast-path: ONLY for a `DPoP <token>` request (the production posture). Everything else
        // -- absent auth (public), Bearer, or an unparseable header -- goes straight to the verifier,
        // which owns those decisions. We extract the bearer token string purely as the cache key + the
        // `ath` input; the verifier remains the sole authority on a miss.
        //
        // Extract the access token as an OWNED `String` (not a borrow of `authorization`) so that on a
        // MISS we can still move `authorization` into the verifier's `AuthRequest` while using the
        // token string for the cache `insert`. The clone is one small string per cache-eligible request
        // -- negligible against the ES256 verify a hit saves, and only paid when the cache is enabled.
        let cache_token: Option<String> = self.cache.as_ref().and(
            authorization
                .as_deref()
                .and_then(dpop_scheme_access_token)
                .map(str::to_string),
        );
        if let (Some(tc), Some(access_token)) = (self.cache.as_ref(), cache_token.as_deref()) {
            match tc.cache.authenticate(
                access_token,
                dpop.as_deref(),
                &method_uc,
                &target.htu,
                now_secs(),
                tc.replay.as_ref(),
            ) {
                CacheDecision::Verified(token) => return Ok(token),
                CacheDecision::Reject(e) => {
                    return Err(ServerError::Unauthorized {
                        status: e.status(),
                        message: e.message().to_string(),
                        www_authenticate: self.verifier.www_authenticate(&e),
                    })
                }
                // Fall through to the full verifier; on success, populate the cache.
                CacheDecision::Miss => {
                    let req = AuthRequest {
                        authorization,
                        dpop,
                        method: method_uc,
                        url: target.htu,
                        client_cert_x5t_s256,
                    };
                    let token =
                        self.verifier
                            .verify(&req)
                            .map_err(|e| ServerError::Unauthorized {
                                status: e.status(),
                                message: e.message().to_string(),
                                www_authenticate: self.verifier.www_authenticate(&e),
                            })?;
                    // Only a SUCCESSFUL full verification reaches here => safe to cache. A non-DPoP-bound
                    // token (no cnf.jkt/exp) is silently not cached by `insert`.
                    //
                    // (roborev Medium) When the mTLS flag is on, `finalize_pop` (applied by the caller
                    // AFTER this returns) rejects a token that ALSO carries a cert binding — a
                    // multi-binding token is refused (a cert-bound-only token has no cnf.jkt so `insert`
                    // already skips it). Caching such a token would let a token that never completes
                    // authentication occupy an LRU slot and be served on later attempts. So skip the
                    // insert for ANY token carrying a cert binding when mTLS is on — only a PURELY
                    // DPoP-bound token (the sole thing `finalize_pop` accepts down the cache path) is
                    // cached. When mTLS is off this is unchanged (the condition is never true).
                    if !(self.mtls_bound_tokens && token.cnf_x5t_s256.is_some()) {
                        tc.cache.insert(access_token, &token, now_secs());
                    }
                    return Ok(token);
                }
            }
        }

        // No cache (or a non-DPoP request): the full verifier owns the decision.
        let req = AuthRequest {
            authorization,
            dpop,
            method: method_uc,
            url: target.htu,
            client_cert_x5t_s256,
        };
        self.verifier
            .verify(&req)
            .map_err(|e| ServerError::Unauthorized {
                status: e.status(),
                message: e.message().to_string(),
                www_authenticate: self.verifier.www_authenticate(&e),
            })
    }

    /// PoP Tier-1b — the RFC 8705 mTLS confirmation dispatch applied to an already-verified token.
    ///
    /// This is PURELY ADDITIVE hardening: it can only turn an otherwise-accepted token into a DENY (for
    /// a cert-bound / malformed-binding / multi-binding token that does not satisfy its confirmation on
    /// THIS connection); it NEVER turns a deny into an accept. The token reached here only via a full
    /// verify (signature + RFC 9068 + any DPoP proof) or a cache hit (fresh proof re-verified), so its
    /// `cnf` claims are trustworthy.
    ///
    /// - **Flag off** ⇒ returned unchanged (byte-identical to pre-Tier-1b; the presented cert is ignored).
    /// - **DPoP-bound** (`cnf.jkt` only) ⇒ [`PopRoute::Dpop`]: accepted (the verifier already ran the
    ///   full DPoP proof; a certificate can never satisfy — nor is it consulted for — a DPoP token).
    /// - **Public / unbound** (no `cnf`) ⇒ [`PopRoute::Unbound`]: returned unchanged (the verifier's
    ///   `require_dpop` policy already governed whether an unbound token was admitted at all).
    /// - **Cert-bound** (`cnf.x5t#S256` only) ⇒ [`PopRoute::CertBound`]: the presented certificate MUST
    ///   match; no cert / wrong cert ⇒ 401 (fail-closed — never a downgrade to bearer).
    /// - **Malformed cert binding** (`cnf.x5t#S256` present but not a valid thumbprint) ⇒ 401 (a broken
    ///   cert binding is never collapsed to "unbound" — the fail-closed choice, mirroring the verifier's
    ///   three-state [`X5tS256`]).
    /// - **Multiple bindings** (`cnf.jkt` AND `cnf.x5t#S256`) ⇒ [`PopRoute::MultipleBindings`]: refused
    ///   (combined both-must-hold verification is unimplemented; satisfying only one would bypass the
    ///   other — see [`Confirmation::MultipleBindings`]).
    fn finalize_pop(
        &self,
        token: VerifiedToken,
        presented_cert: Option<&CertThumbprint>,
    ) -> Result<VerifiedToken, ServerError> {
        if !self.mtls_bound_tokens {
            // mTLS path disabled — no confirmation dispatch, presented cert ignored. Byte-identical.
            return Ok(token);
        }

        // Parse the token's mTLS confirmation into a comparable thumbprint, failing CLOSED on a present
        // but malformed binding (never treated as unbound).
        let x5t: Option<CertThumbprint> = match &token.cnf_x5t_s256 {
            None => None,
            Some(X5tS256::Thumbprint(t)) => match CertThumbprint::from_base64url(t) {
                Ok(tp) => Some(tp),
                // The verifier already validates the base64url/length of a `Thumbprint`, so this is
                // belt-and-braces; a parse failure here still fails closed rather than silently unbinds.
                Err(_) => return Err(self.cert_bound_denied()),
            },
            Some(X5tS256::Malformed) => return Err(self.cert_bound_denied()),
        };

        let confirmation = Confirmation::select(token.cnf_jkt.clone(), x5t);
        match pop_dispatch(&confirmation, presented_cert) {
            // The verifier is authoritative for DPoP + unbound/public; return unchanged.
            PopRoute::Dpop | PopRoute::Unbound => Ok(token),
            PopRoute::CertBound(CertBindingOutcome::Confirmed) => Ok(token),
            PopRoute::CertBound(CertBindingOutcome::Denied(_)) => Err(self.cert_bound_denied()),
            PopRoute::MultipleBindings => Err(self.cert_bound_denied()),
        }
    }

    /// The 401 for a failed RFC 8705 mTLS cert-binding (no cert / wrong cert / malformed / multi-binding).
    /// Fail-closed: a cert-bound token that does not satisfy its binding on this connection is rejected,
    /// never accepted bare. The `WWW-Authenticate` challenge reuses the server's single-sourced DPoP
    /// challenge (the RFC 9728 `resource_metadata` param is a documented follow-up — see the design §6).
    fn cert_bound_denied(&self) -> ServerError {
        ServerError::Unauthorized {
            status: 401,
            message: "The access token's certificate binding was not satisfied on this connection."
                .to_string(),
            www_authenticate: self.unauthenticated_challenge(),
        }
    }

    /// Build the 401 + `WWW-Authenticate` challenge for a request that REQUIRES authentication but
    /// arrived without credentials (a public [`VerifiedToken`]).
    ///
    /// The verifier returns a *public* token (not an error) when no `Authorization` header is present
    /// — that is correct for the auth layer (an anonymous request is a valid, public identity). The
    /// LDP layer then decides whether the target needs auth; when it does and the caller is public, it
    /// must answer 401 with a challenge (Solid Protocol / RFC 6750), NOT a bare 403. This synthesises
    /// the SAME challenge string the verifier emits on a token failure (it names the trusted issuer(s)
    /// and DPoP `algs`), so a client knows where to obtain a token. We route it through the verifier's
    /// own [`Verifier::www_authenticate`] so the challenge format stays single-sourced in the verifier.
    pub fn unauthenticated_error(&self) -> ServerError {
        ServerError::Unauthorized {
            status: 401,
            message: "Authentication required for this resource.".to_string(),
            www_authenticate: self.unauthenticated_challenge(),
        }
    }

    /// The `WWW-Authenticate` challenge string for an anonymous request to a resource that requires
    /// authentication. Single-sourced through the verifier's own challenge builder so the format
    /// (scheme, `error=`, `issuer=`, `algs=`) matches every other challenge this server emits. The LDP
    /// layer caches this once (it does not vary per request) and attaches it to a 401.
    pub fn unauthenticated_challenge(&self) -> String {
        // An `invalid_token` DPoP-scheme error is the canonical "you need a (DPoP-bound) token" signal;
        // `www_authenticate` widens it to `DPoP` + `algs` per the verifier's require_dpop policy.
        let err = VerifyError::new(
            ErrorKind::InvalidToken,
            "Authentication required for this resource.",
        )
        .with_dpop(true);
        self.verifier.www_authenticate(&err)
    }
}

/// An axum middleware layer that authenticates the request and inserts the [`VerifiedToken`] into
/// request extensions. Handlers read it with `Extension<VerifiedToken>`.
///
/// `State` is an `Arc<AuthContext<_, _>>` so the verifier is shared across requests without cloning.
pub async fn auth_middleware<J, R>(
    State(ctx): State<Arc<AuthContext<J, R>>>,
    mut req: Request,
    next: Next,
) -> Response
where
    J: JwksProvider + Send + Sync + 'static,
    R: ReplayStore + Send + Sync + 'static,
{
    let method = req.method().as_str().to_string();
    let path = req.uri().path().to_string();

    // PoP Tier 2 (DPoP-SK) — the negotiated symmetric fast path, gated on `with_dpop_sk`. A
    // request bearing a `dpop-sk`-tagged RFC 9421 signature is processed under that profile
    // EXCLUSIVELY: on success the session's stored VerifiedToken is injected exactly as a DPoP
    // verification would inject it (downstream WAC/LDP unchanged); on ANY failure the response is
    // the standard 401 DPoP challenge (the client falls back to re-establishment or plain DPoP —
    // both full-strength PoP; never bearer). A request WITHOUT the tag — including every request
    // on a flag-off build — falls through to the unchanged DPoP path below, so stripping the
    // signature headers can only ever force full DPoP, not weaken anything.
    if let Some(sk) = ctx.sk.as_ref() {
        // The absolute target URI, reconstructed by the SERVER from its configured public origin
        // + the request's path-and-query (never from client-controlled Host/Forwarded headers).
        // The origin is normalized exactly as `parse_target` normalizes it for the DPoP `htu`
        // (trailing slash trimmed), so a trailing-slash-configured base_url cannot make every
        // attestation fail on a double slash (roborev Medium on the Tier-2 commit).
        let target_uri = format!(
            "{}{}",
            ctx.base_url.trim_end_matches('/'),
            req.uri()
                .path_and_query()
                .map(|pq| pq.as_str())
                .unwrap_or("/")
        );
        let conn_sk = req.extensions().get::<ConnSk>().cloned();
        match verify_attested_request(
            sk,
            req.headers(),
            &method,
            &target_uri,
            conn_sk.as_ref(),
            now_secs(),
        ) {
            SkDecision::NotApplicable => {} // not under the profile: the DPoP baseline gates it
            SkDecision::Verified { token, session_id } => {
                req.extensions_mut().insert(token);
                req.extensions_mut().insert(SkSession { session_id });
                return next.run(req).await;
            }
            SkDecision::Deny => {
                return ServerError::Unauthorized {
                    status: 401,
                    message: "Invalid DPoP-SK attestation.".to_string(),
                    www_authenticate: ctx.unauthenticated_challenge(),
                }
                .into_response()
            }
        }
    }

    // Distinguish an ABSENT auth header (⇒ public) from one that is PRESENT but unparseable
    // (non-UTF-8 bytes). A present-but-invalid credential must NOT be silently downgraded to public
    // access — that is a fail-open. Reject it as a 400.
    let authorization = match header_string(&req, axum::http::header::AUTHORIZATION) {
        Ok(v) => v,
        Err(()) => {
            return ServerError::BadRequest("malformed Authorization header".into()).into_response()
        }
    };
    // DPoP is a custom header; look it up by its lowercase name.
    let dpop = match header_string(&req, axum::http::HeaderName::from_static("dpop")) {
        Ok(v) => v,
        Err(()) => return ServerError::BadRequest("malformed DPoP header".into()).into_response(),
    };

    // PoP Tier-1b: the client-certificate thumbprint for THIS connection, injected once per connection
    // by the mTLS acceptor ([`crate::pop::conn::ConnPopService`]). Absent when the mTLS flag is off, on
    // the plain-HTTP path, or when the peer presented no client certificate — in all of which a
    // cert-bound token is denied fail-closed by `authenticate`. Cloned out (a 32-byte thumbprint) so we
    // can still move `req` into the handler chain below.
    let presented_cert = req
        .extensions()
        .get::<ConnPop>()
        .and_then(|p| p.thumbprint().cloned());

    match ctx.authenticate_with_cert(authorization, dpop, &method, &path, presented_cert.as_ref()) {
        Ok(token) => {
            req.extensions_mut().insert(token);
            next.run(req).await
        }
        Err(e) => e.into_response(),
    }
}

/// Read a header as a `String`. `Ok(None)` = absent; `Ok(Some(_))` = a valid value; `Err(())` =
/// present but not valid UTF-8 (a malformed value that must be rejected, never treated as absent).
fn header_string(req: &Request, name: axum::http::HeaderName) -> Result<Option<String>, ()> {
    match req.headers().get(&name) {
        None => Ok(None),
        Some(value) => value.to_str().map(|s| Some(s.to_string())).map_err(|_| ()),
    }
}

/// Extract the access-token string from a `DPoP <token>` Authorization header, returning `None` for
/// any other scheme (Bearer, etc.) or a malformed/empty header.
///
/// This MUST parse the header EXACTLY as the verifier's own `parse_authorization` does -- trim the
/// header, split on the FIRST space, lowercase the scheme, trim the token -- so the cache key is the
/// byte-identical token the verifier verifies on a miss (a divergent parse could key the cache by a
/// different string than the one verified, splitting the cache or, worse, reusing a verification for a
/// token that was never verified). It is consulted ONLY for the cache fast-path — plus the DPoP-SK
/// layer (`crate::pop::sk`), which needs the SAME byte-exact token string for its token-hash
/// binding; the verifier remains the sole authority on every miss, so this never makes a security
/// decision on its own.
pub(crate) fn dpop_scheme_access_token(header: &str) -> Option<&str> {
    let trimmed = header.trim();
    let sp = trimmed.find(' ')?;
    let scheme = &trimmed[..sp];
    if !scheme.eq_ignore_ascii_case("dpop") {
        return None;
    }
    let token = trimmed[sp + 1..].trim();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

/// Current UNIX time in seconds (the cache's `now` for token-`exp` + proof-`iat` checks). Matches the
/// verifier's internal clock.
fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::dpop_scheme_access_token;

    #[test]
    fn extracts_dpop_token_only() {
        assert_eq!(
            dpop_scheme_access_token("DPoP abc.def.ghi"),
            Some("abc.def.ghi")
        );
        // Case-insensitive scheme, trims surrounding + inter-token whitespace exactly like the verifier.
        assert_eq!(dpop_scheme_access_token("  dpop   tok  "), Some("tok"));
        // Non-DPoP schemes are not cache-eligible (verifier decides).
        assert_eq!(dpop_scheme_access_token("Bearer tok"), None);
        // Malformed / empty.
        assert_eq!(dpop_scheme_access_token("DPoP"), None);
        assert_eq!(dpop_scheme_access_token("DPoP "), None);
        assert_eq!(dpop_scheme_access_token(""), None);
    }
}
