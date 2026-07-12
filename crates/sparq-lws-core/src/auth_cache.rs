// AUTHORED-BY Claude Opus 4.8
//! A **verified-access-token cache** that removes the redundant per-request access-token verification
//! (round-3 optimisation) **without weakening DPoP auth**.
//!
//! ## The optimisation (and what it must NOT change)
//! An authenticated request pays *two* asymmetric (ES256) signature verifies: the **access token**
//! and the **DPoP proof**. The access token is *stable* across a client's requests -- re-verifying its
//! signature + RFC-9068 claims on every request is waste. The DPoP proof is *fresh per request* and is
//! the replay protection -- it MUST stay fully verified on every request.
//!
//! This cache stores the [`VerifiedToken`] produced by a *successful full verification*, keyed by a
//! SHA-256 of the **full access token**. On a cache HIT the server SKIPS only the token signature +
//! claims re-verification and STILL performs, per request, the COMPLETE DPoP-proof + replay path:
//! proof signature (embedded JWK), `htm`/`htu`/`iat`, `ath == H(token)`, the `jti` replay mark, and
//! the `cnf.jkt == thumbprint(proof JWK)` proof-of-possession binding. **Nothing the proof carries is
//! cached or skipped.**
//!
//! ## R1 (do-not-reimplement) posture
//! Every cryptographic + canonicalisation primitive used on the hit path is the verifier crate's own
//! **public** API -- `verify_proof_with_embedded_jwk` (the EmbeddedJWK signature check), `Jwk::
//! thumbprint_sha256` (RFC 7638), `proof_has_ath`, `peek_claims`, and the shared [`ReplayStore`]. This
//! module owns only the *orchestration* of those primitives (the same sequence the verifier's private
//! `validate_dpop_proof` runs). The cleaner long-term home for that orchestration is a public seam on
//! the verifier (`Verifier::verify_proof_for_cached_token`); that is a SEPARATE, owner-gated change to
//! `jeswr/solid-oidc-verifier`. Until that lands, this consumer-side orchestration is pinned by
//! exhaustive, adversarial unit tests so it cannot silently diverge.
//!
//! ## Replay-store sharing (the load-bearing correctness point)
//! The cache's hit-path proof verification marks the `jti` in the **SAME** replay store the verifier
//! uses -- otherwise a `jti` used on a cache-MISS request could be replayed on a cache-HIT request
//! (a replay bypass). The server therefore constructs ONE [`InMemoryReplayStore`](solid_oidc_verifier::replay::InMemoryReplayStore) behind an `Arc` and
//! hands a clone to both the verifier (via the [`SharedReplay`] newtype) and this cache.
//!
//! ## Bounds + lifetime
//! - **Per-instance only** (the stateless-core charter rule): a miss just re-verifies; correctness
//!   never depends on sharing across instances.
//! - **Bounded** (LRU + TTL): capacity-capped (default [`DEFAULT_CACHE_CAPACITY`]) so the cache can
//!   never grow unbounded (a DoS vector). LRU eviction on overflow; lazy TTL prune.
//! - **TTL <= the token's own `exp`**: an entry can never outlive the token. On every HIT, `exp > now`
//!   is RE-CHECKED and a since-expired entry is rejected (and evicted), so a cached entry can never
//!   turn a would-be-401 into a 200.
//! - **Bounded ALSO by a short validation TTL** (default = the JWKS cache TTL): an entry is force-re-
//!   verified at least every `max_entry_ttl`, so a JWKS key revocation / rotation propagates within one
//!   TTL window rather than being masked until `exp` (roborev round-1 Medium). The effective deadline is
//!   `min(exp, inserted_at + max_entry_ttl)`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use base64::Engine as _;
use serde_json::Value;
use sha2::{Digest, Sha256};
use solid_oidc_verifier::error::{ErrorKind, VerifyError};
use solid_oidc_verifier::jwt::{peek_claims, proof_has_ath, verify_proof_with_embedded_jwk};
use solid_oidc_verifier::replay::{MarkResult, ReplayBackendError, ReplayStore};
use solid_oidc_verifier::verifier::VerifiedToken;

/// The DPoP proof max age (RFC 9449) the verifier enforces. Mirrors the verifier's
/// `config::DPOP_PROOF_MAX_AGE_SECS` (not re-exported, so re-declared with the same value; the proof
/// `iat` window is `this + clock_tolerance`). Kept in sync with the verifier -- both are 300s.
const DPOP_PROOF_MAX_AGE_SECS: u64 = 300;

/// Default verified-token cache capacity (number of distinct live access tokens). A token is a whole
/// authenticated *client session*, so even a busy multi-tenant pod has far fewer live tokens than
/// requests; 4096 entries comfortably covers concurrent sessions while bounding memory (each entry is
/// a 32-byte key + a small `VerifiedToken`). Tunable via the constructor; conformance-neutral.
pub const DEFAULT_CACHE_CAPACITY: usize = 4096;

/// Default max lifetime of a cached verified-token entry, in seconds, INDEPENDENT of the token `exp`.
/// An entry is force-re-verified (treated as a miss) once it is older than this, so a JWKS signing-key
/// revocation / rotation is honoured within one TTL window instead of being masked until `exp`. The
/// default matches the verifier's default JWKS cache TTL (`SOLID_SERVER_JWKS_CACHE_TTL_SECS`, 300s):
/// after a key disappears from the JWKS, the miss path would start rejecting within that TTL, and this
/// bound makes the cache agree. The effective per-entry deadline is `min(exp, inserted_at + this)`.
pub const DEFAULT_MAX_ENTRY_TTL_SECS: i64 = 300;

/// A [`ReplayStore`] that forwards to a shared inner store behind an `Arc`, so the SAME replay state is
/// used by the verifier (cache-miss path) and the token cache (cache-hit path). Without this, the two
/// paths would have independent jti sets and a proof used on one path could be replayed on the other.
///
/// `ReplayStore` takes `&self`, so cloning the `Arc` shares one `Mutex<HashMap>` -- every `mark`
/// (whichever path) hits the one atomic check-and-set.
pub struct SharedReplay<R: ReplayStore>(Arc<R>);

// Manual `Clone` (NOT `#[derive]`): cloning only clones the inner `Arc`, so it must NOT require
// `R: Clone` (the derive would add that bound, which `InMemoryReplayStore` does not satisfy). Every
// clone shares the one underlying replay store.
impl<R: ReplayStore> Clone for SharedReplay<R> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<R: ReplayStore> SharedReplay<R> {
    pub fn new(inner: Arc<R>) -> Self {
        Self(inner)
    }

    /// The shared inner `Arc`, to hand a second handle to the cache.
    pub fn handle(&self) -> Arc<R> {
        Arc::clone(&self.0)
    }
}

impl<R: ReplayStore> ReplayStore for SharedReplay<R> {
    fn mark(&self, jti: &str, ttl: Duration) -> Result<MarkResult, ReplayBackendError> {
        self.0.mark(jti, ttl)
    }

    /// READ-ONLY existence probe — delegates verbatim to the shared inner store, so the verifier
    /// (cache-miss path) and the token cache (cache-hit path) probe the ONE shared jti set, exactly like
    /// `mark`. Non-mutating (the inner store's `contains` contract). NOTE: not yet wired into the auth
    /// path here — present to satisfy the trait + carry the opt-4 jti-precheck seam.
    fn contains(&self, jti: &str) -> Result<bool, ReplayBackendError> {
        self.0.contains(jti)
    }
}

/// The config the hit-path proof verification needs -- the SAME knobs the verifier was built with, so
/// the cached path enforces byte-identical proof semantics (clock tolerance, ath-compat). Captured at
/// server construction from the same `VerifierConfig`, BEFORE it is moved into the verifier.
#[derive(Debug, Clone)]
pub struct ProofPolicy {
    /// Clock skew tolerance (seconds) for the proof `iat` window -- same value the verifier uses.
    pub clock_tolerance_secs: u64,
    /// ADR-0007 opt-in: accept an otherwise-valid proof that OMITS `ath`. A present-but-wrong `ath` is
    /// still rejected. Mirrors `VerifierConfig.allow_missing_ath`.
    pub allow_missing_ath: bool,
    /// Fail-closed on a replay-store backend error (-> 503). Mirrors `VerifierConfig.replay_fail_closed`.
    pub replay_fail_closed: bool,
}

impl ProofPolicy {
    /// The `jti` replay TTL window: `max_age + tolerance` (the verifier's `replay_ttl()` invariant).
    /// An entry must live at least this long so the replay window cannot reopen.
    fn replay_ttl(&self) -> Duration {
        Duration::from_secs(DPOP_PROOF_MAX_AGE_SECS + self.clock_tolerance_secs)
    }
}

/// A cached, already-verified token result + the bookkeeping the hit path needs.
#[derive(Clone)]
struct CacheEntry {
    /// The verified credentials handed downstream verbatim on a hit (webid/iss/aud/cnf.jkt/exp/...).
    token: VerifiedToken,
    /// The token's `cnf.jkt` (the DPoP binding the fresh proof's key thumbprint must equal). Pulled
    /// out of `token` for clarity; a token reaching the cache is always DPoP-bound (it was verified
    /// under `require_dpop`/cnf-bound), so this is `Some`.
    cnf_jkt: String,
    /// The token's `exp` (epoch seconds). The entry is rejected + evicted once `now >= exp` even if
    /// the LRU/TTL has not yet pruned it -- a cached entry never outlives the token.
    exp: i64,
    /// Epoch seconds the entry was inserted. The entry is force-re-verified once `now >=
    /// inserted_at + max_entry_ttl_secs` (the validation-freshness bound), so a JWKS revocation is not
    /// masked until `exp`.
    inserted_at: i64,
    /// Monotonic last-access tick for LRU eviction (higher = more recently used).
    last_access: u64,
}

/// The verified-access-token cache. A bounded (LRU + TTL) map from `SHA-256(access_token)` to the
/// verified result. Thread-safe (one `Mutex`), per-instance.
pub struct VerifiedTokenCache {
    inner: Mutex<HashMap<[u8; 32], CacheEntry>>,
    capacity: usize,
    /// Monotonic access clock for LRU ordering (incremented on every get/insert).
    clock: AtomicU64,
    policy: ProofPolicy,
    /// Max entry lifetime (seconds) independent of `exp` -- the validation-freshness bound that makes
    /// a JWKS revocation propagate within one TTL window (see [`DEFAULT_MAX_ENTRY_TTL_SECS`]).
    max_entry_ttl_secs: i64,
}

/// The outcome of the cache-fronted authentication of an authenticated (token-bearing) request.
pub enum CacheDecision {
    /// A cache HIT whose fresh DPoP proof + replay + binding all re-verified: use this token.
    Verified(VerifiedToken),
    /// A cache MISS (no entry, or the entry expired/was evicted): the caller must run the verifier's
    /// FULL `verify()` and, on success, `insert` the result.
    Miss,
    /// A cache HIT whose fresh-proof re-check FAILED -- reject the request with this error. (A hit must
    /// never be able to turn a failing proof into a success; this is the path that enforces it.)
    Reject(VerifyError),
}

impl VerifiedTokenCache {
    /// Build a cache with the given capacity + proof policy. A `capacity` of 0 is clamped to 1 (a
    /// zero-capacity cache is pointless and would make every insert evict itself; never unbounded).
    pub fn new(capacity: usize, policy: ProofPolicy) -> Self {
        Self::with_max_entry_ttl(capacity, policy, DEFAULT_MAX_ENTRY_TTL_SECS)
    }

    /// Build a cache with an explicit max-entry validation TTL (seconds). A non-positive `ttl` is
    /// clamped to 1 (an entry is always re-verified at least every second; never an unbounded
    /// validation lifetime). See [`DEFAULT_MAX_ENTRY_TTL_SECS`].
    pub fn with_max_entry_ttl(
        capacity: usize,
        policy: ProofPolicy,
        max_entry_ttl_secs: i64,
    ) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            capacity: capacity.max(1),
            clock: AtomicU64::new(0),
            policy,
            max_entry_ttl_secs: max_entry_ttl_secs.max(1),
        }
    }

    /// The cache key: SHA-256 of the FULL access token. Collision-resistant -- a different token yields
    /// a different key. NEVER keyed by webid/jkt alone (that would let one token's verification be
    /// reused for a different token bearing the same webid).
    fn key(access_token: &str) -> [u8; 32] {
        Sha256::digest(access_token.as_bytes()).into()
    }

    /// The DPoP `ath` value for a token whose SHA-256 digest is ALREADY computed: `base64url(digest)`.
    /// Derived from the SAME 32-byte SHA-256 the cache key uses, so an authenticated hit hashes the
    /// access token exactly ONCE per request (the key + the `ath` comparison share the digest) rather
    /// than the two independent `Sha256::digest(token)` calls the old `key()` + `ath()` pair ran. The
    /// produced string is byte-identical to the standalone `ath()` (same digest, same base64url-no-pad
    /// encoding) so the security comparison `proof.ath == base64url(SHA-256(token))` is unchanged.
    fn ath_from_digest(digest: &[u8; 32]) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
    }

    /// Attempt to authenticate a token-bearing request from the cache.
    ///
    /// - On a MISS (no live, unexpired entry) returns [`CacheDecision::Miss`] -- the caller runs the
    ///   full verifier and then [`insert`](Self::insert)s the verified result.
    /// - On a HIT, re-checks the token `exp` (rejecting + evicting a since-expired entry) and then runs
    ///   the FULL fresh-proof + replay + `cnf.jkt` verification against THIS request, returning
    ///   [`CacheDecision::Verified`] only if all pass, else [`CacheDecision::Reject`].
    ///
    /// `now` is the current epoch seconds (injected for deterministic tests).
    #[allow(clippy::too_many_arguments)]
    pub fn authenticate(
        &self,
        access_token: &str,
        dpop_proof: Option<&str>,
        method: &str,
        url: &str,
        now: i64,
        replay: &dyn ReplayStore,
    ) -> CacheDecision {
        let key = Self::key(access_token);

        // Look up + bump LRU recency under the lock; clone the (small) entry out so the expensive proof
        // crypto runs OUTSIDE the lock (no cross-request lock contention on the hot path).
        let entry = {
            let mut map = match self.inner.lock() {
                Ok(m) => m,
                // A poisoned cache lock must NEVER fail open to a hit; treat as a miss (full re-verify).
                Err(_) => return CacheDecision::Miss,
            };
            match map.get(&key) {
                None => return CacheDecision::Miss,
                Some(e) => {
                    // Freshness gate, RE-CHECKED on every hit. An entry is NOT a valid hit if either:
                    //  (a) the token has expired (`now >= exp`) -- it can never outlive the token; or
                    //  (b) it is older than the validation TTL (`now >= inserted_at + max_entry_ttl`) --
                    //      so a JWKS revocation/rotation forces a full re-verify within one TTL window.
                    // Either way: evict + force a MISS (the verifier then makes the authoritative call).
                    let entry_deadline = e
                        .inserted_at
                        .saturating_add(self.max_entry_ttl_secs)
                        .min(e.exp);
                    if now >= entry_deadline {
                        map.remove(&key);
                        return CacheDecision::Miss;
                    }
                    let tick = self.clock.fetch_add(1, Ordering::Relaxed) + 1;
                    let mut e = e.clone();
                    e.last_access = tick;
                    // Persist the bumped recency.
                    if let Some(slot) = map.get_mut(&key) {
                        slot.last_access = tick;
                    }
                    e
                }
            }
        };

        // --- HIT: the access token is trusted (it was fully verified when inserted, and is not yet
        // expired). Now FULLY verify the FRESH DPoP proof + replay + binding for THIS request. Nothing
        // here is cached -- every check runs every request. The cache `key` already IS SHA-256(token);
        // pass it through so the `ath` comparison reuses that digest instead of hashing the token a
        // second time (the only `ath` input is `base64url` of this exact digest). ---
        match self.verify_fresh_proof(dpop_proof, &key, &entry.cnf_jkt, method, url, now, replay) {
            Ok(()) => CacheDecision::Verified(entry.token),
            Err(e) => CacheDecision::Reject(e),
        }
    }

    /// Insert a verified result. ONLY a SUCCESSFUL full verification calls this -- a malformed/invalid/
    /// expired token is never inserted (the caller only reaches here after the verifier returned an
    /// authenticated [`VerifiedToken`]). A token WITHOUT a `cnf.jkt` or `exp` is NOT cached (it cannot
    /// be a DPoP-bound token under this server's policy; not caching it is fail-safe -- it just re-
    /// verifies). The entry's lifetime is bounded by `min(exp, now + max_entry_ttl)` -- `now` is the
    /// insertion time, recorded so the validation-TTL bound can be enforced on later hits.
    pub fn insert(&self, access_token: &str, token: &VerifiedToken, now: i64) {
        let (Some(cnf_jkt), Some(exp)) = (token.cnf_jkt.clone(), token.expiry) else {
            // Not DPoP-bound / no exp => do not cache (every such request re-verifies -- never a hit).
            return;
        };
        let key = Self::key(access_token);
        let tick = self.clock.fetch_add(1, Ordering::Relaxed) + 1;
        let entry = CacheEntry {
            token: token.clone(),
            cnf_jkt,
            exp,
            inserted_at: now,
            last_access: tick,
        };

        let mut map = match self.inner.lock() {
            Ok(m) => m,
            Err(_) => return, // a poisoned lock simply means no caching -- correctness unaffected.
        };
        // Re-insert of the same key never grows the map. A NEW key at capacity evicts the LRU entry
        // first (bounded -- never unbounded growth).
        if !map.contains_key(&key) && map.len() >= self.capacity {
            Self::evict_one(&mut map);
        }
        map.insert(key, entry);
    }

    /// Evict exactly one entry to make room: the least-recently-used. Called only when inserting a NEW
    /// key at capacity. (Capacity is small, so an O(n) min-scan is cheap and avoids a second index
    /// structure that could desync from the map.)
    fn evict_one(map: &mut HashMap<[u8; 32], CacheEntry>) {
        if let Some(lru_key) = map
            .iter()
            .min_by_key(|(_, e)| e.last_access)
            .map(|(k, _)| *k)
        {
            map.remove(&lru_key);
        }
    }

    /// FULLY verify a fresh DPoP proof for a cache HIT -- the SAME orchestration the verifier's private
    /// `validate_dpop_proof` runs, composed from the verifier's PUBLIC primitives. Order + tolerances
    /// match the verifier exactly.
    ///
    /// Checks (all per request): proof present; signature self-verified by the embedded JWK with
    /// `typ=dpop+jwt` and an asymmetric alg; `htm == method`; `htu == url` (normalised); `jti` present;
    /// `iat` within `max_age + tolerance`; `ath == base64url(SHA-256(token))` (with the ath-compat
    /// routing); the `jti` replay mark (fail-closed); and `cnf.jkt == thumbprint(proof JWK)`.
    #[allow(clippy::too_many_arguments)]
    fn verify_fresh_proof(
        &self,
        dpop_proof: Option<&str>,
        token_digest: &[u8; 32],
        cnf_jkt: &str,
        method: &str,
        url: &str,
        now: i64,
        replay: &dyn ReplayStore,
    ) -> Result<(), VerifyError> {
        // A cached (DPoP-bound) token with NO proof on this request => reject (proof-of-possession
        // required every request).
        let proof = dpop_proof
            .ok_or_else(|| invalid_token_dpop("DPoP proof is required (no DPoP HTTP Header)."))?;

        // (sig) self-signed by the embedded public JWK; typ=dpop+jwt; asymmetric alg -- verifier crate.
        let (claims, jwk) = verify_proof_with_embedded_jwk(proof, "dpop+jwt").map_err(|e| {
            invalid_token_dpop(format!("DPoP proof verification failed: {}", e.message()))
        })?;

        // (htm) case-insensitive method match.
        let htm = claims.get("htm").and_then(Value::as_str).unwrap_or("");
        if !htm.eq_ignore_ascii_case(method) {
            return Err(invalid_token_dpop("DPoP proof htm mismatch."));
        }

        // (htu) normalised exact-URL match (query/fragment stripped, default ports normalised). Normalise
        // the REQUEST url ONCE (one `url::Url::parse`) and compare against the normalised proof `htu` —
        // the old code parsed `url` afresh inside the match arm, re-parsing the same string each call.
        // Semantics unchanged: both sides go through the same `normalize_htu` before the string compare.
        let normalized_url = normalize_htu(url);
        match claims.get("htu").and_then(Value::as_str) {
            Some(h) if normalize_htu(h) == normalized_url => {}
            _ => return Err(invalid_token_dpop("DPoP proof htu mismatch.")),
        }

        // (jti) present + non-empty (the replay mark consumes it below).
        match claims.get("jti").and_then(Value::as_str) {
            Some(s) if !s.is_empty() => {}
            _ => return Err(invalid_token_dpop("DPoP proof is missing a jti.")),
        }

        // (iat) freshness window -- EXACTLY the verifier's symmetric `|now - iat| > max_age + tolerance`
        // (`solid-oidc-verifier` verifier.rs). The hit path MUST enforce byte-identical proof semantics
        // to the miss path: if the cache were stricter (e.g. an asymmetric future bound), the SAME proof
        // could be accepted on a miss (verifier) then rejected on a later hit (cache) -- a state-
        // dependent inconsistency (roborev round-2 Medium). The window IS loose on the future axis
        // (it accepts far-future proofs), but tightening it belongs in the VERIFIER, applied to BOTH
        // paths together -- a separate verifier-crate follow-up (see bench/ROUND3.md). `checked_sub`
        // guards against overflow from a crafted huge `iat` (the overflow case falls to reject).
        let iat = claims
            .get("iat")
            .and_then(Value::as_i64)
            .ok_or_else(|| invalid_token_dpop("DPoP proof is missing iat."))?;
        let window = (DPOP_PROOF_MAX_AGE_SECS + self.policy.clock_tolerance_secs) as i64;
        match now.checked_sub(iat) {
            Some(age) if age.unsigned_abs() <= window as u64 => {}
            _ => return Err(invalid_token_dpop("DPoP proof iat is not recent enough.")),
        }

        // (ath) base64url(SHA-256(access_token)). ath-compat ONLY when opted-in AND the proof omits
        // ath; a present-but-wrong ath is ALWAYS rejected (only absence is tolerated, exactly as the
        // verifier routes). The expected value is base64url of the cache key's SHA-256 digest of the
        // token (`ath_from_digest`) — byte-identical to the old `ath(access_token)` but reusing the
        // already-computed digest, so the token is hashed once per request, not twice. Computed lazily
        // (only when an `ath` comparison is actually needed) so the ath-compat-absent path skips it.
        let ath_compat = self.policy.allow_missing_ath && !proof_has_ath(proof);
        let require_ath = !ath_compat;
        let proof_ath = claims.get("ath").and_then(Value::as_str);
        if require_ath {
            let expected = Self::ath_from_digest(token_digest);
            match proof_ath {
                Some(a) if a == expected => {}
                Some(_) => return Err(invalid_token_dpop("DPoP proof ath mismatch.")),
                None => return Err(invalid_token_dpop("DPoP proof is missing ath.")),
            }
        } else if let Some(a) = proof_ath {
            if a != Self::ath_from_digest(token_digest) {
                return Err(invalid_token_dpop("DPoP proof ath mismatch."));
            }
        }

        // (cnf.jkt) the proof-of-possession binding: the fresh proof's key thumbprint MUST equal the
        // CACHED token's cnf.jkt. A valid token presented with a DIFFERENT DPoP key => reject (no token
        // theft / proof swapping). Checked BEFORE the replay mark so the proof is FULLY validated before
        // its jti is consumed -- matching the verifier's order (full `validate_dpop_proof`, incl. this
        // binding, THEN `check_replay`): a proof that fails the binding must not burn a jti.
        let proof_jkt = jwk
            .thumbprint_sha256()
            .map_err(|e| invalid_token_dpop(format!("DPoP proof key is invalid: {e}.")))?;
        if proof_jkt != cnf_jkt {
            return Err(invalid_token_dpop(
                "JWT Access Token confirmation mismatch (cnf.jkt != proof jwk thumbprint).",
            ));
        }

        // (jti replay) mark in the SHARED store, fail-closed (-> 503) on a backend error, exactly as the
        // verifier's `check_replay`. LAST, only after the proof is fully validated -- so a malformed,
        // mis-bound, or otherwise-invalid proof never consumes a jti. A genuine replay of an otherwise-
        // valid proof is the case this catches.
        let jti = claims
            .get("jti")
            .and_then(Value::as_str)
            .map(str::to_string)
            // Unreachable (checked above), but keep the explicit guard rather than unwrap.
            .ok_or_else(|| invalid_token_dpop("DPoP proof is missing a jti."))?;
        match replay.mark(&jti, self.policy.replay_ttl()) {
            Ok(MarkResult::New) => {}
            Ok(MarkResult::Replay) => {
                return Err(invalid_token_dpop(
                    "DPoP proof has already been used (replay).",
                ))
            }
            Err(_e) => {
                if self.policy.replay_fail_closed {
                    return Err(VerifyError::new(
                        ErrorKind::ReplayStoreUnavailable,
                        "Replay protection backend is unavailable.",
                    )
                    .with_dpop(true));
                }
                // Dev fail-open (production config forbids this): treat as fresh.
            }
        }

        Ok(())
    }

    /// Test/diagnostic: current number of cached entries.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.inner.lock().map(|m| m.len()).unwrap_or(0)
    }
}

/// Read a non-empty `jti` from a (possibly malformed) proof without verifying it. Parity helper that
/// mirrors the verifier's `peek_claims`-based jti read; exercised by tests.
#[allow(dead_code)]
fn peek_jti(proof: &str) -> Option<String> {
    peek_claims(proof)
        .and_then(|c| c.get("jti").and_then(Value::as_str).map(str::to_string))
        .filter(|s| !s.is_empty())
}

/// Normalise an `htu` the way the verifier (and `oauth4webapi.validateDPoP`) does: strip query +
/// fragment and normalise default ports. Identical semantics to the verifier's private `normalize_htu`.
fn normalize_htu(htu: &str) -> String {
    match url::Url::parse(htu) {
        Ok(mut u) => {
            u.set_query(None);
            u.set_fragment(None);
            u.to_string()
        }
        Err(_) => htu.to_string(),
    }
}

/// Build a DPoP-scheme `invalid_token` error (status 401, `WWW-Authenticate: DPoP`). Mirrors the
/// verifier's `invalid_token_dpop` (not re-exported), so a hit-path rejection is indistinguishable
/// from a miss-path one to the client.
fn invalid_token_dpop(message: impl Into<String>) -> VerifyError {
    VerifyError::new(ErrorKind::InvalidToken, message).with_dpop(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use p256::ecdsa::{signature::Signer, Signature, SigningKey};
    use rand_core::OsRng;
    use solid_oidc_verifier::replay::InMemoryReplayStore;
    use std::time::Duration as StdDuration;

    // --- Test helpers: mint a client DPoP key + proofs, mirroring the verifier crate's test rig. -----

    struct ClientKey {
        signing: SigningKey,
        jwk: serde_json::Value,
        jkt: String,
    }

    fn new_client_key() -> ClientKey {
        let signing = SigningKey::random(&mut OsRng);
        let vk = signing.verifying_key();
        let point = vk.to_encoded_point(false);
        let x = URL_SAFE_NO_PAD.encode(point.x().unwrap());
        let y = URL_SAFE_NO_PAD.encode(point.y().unwrap());
        let jwk = serde_json::json!({"kty":"EC","crv":"P-256","x":x,"y":y});
        // RFC 7638 thumbprint over {"crv","kty","x","y"} (lexicographic, no whitespace).
        let canonical = format!(r#"{{"crv":"P-256","kty":"EC","x":"{x}","y":"{y}"}}"#);
        let jkt = URL_SAFE_NO_PAD.encode(Sha256::digest(canonical.as_bytes()));
        ClientKey { signing, jwk, jkt }
    }

    fn b64url_json(v: &serde_json::Value) -> String {
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(v).unwrap())
    }

    /// Mint an ES256 DPoP proof JWS with the embedded JWK header and the given claims.
    fn mint_proof(key: &ClientKey, claims: serde_json::Value) -> String {
        let header = serde_json::json!({
            "typ": "dpop+jwt",
            "alg": "ES256",
            "jwk": key.jwk,
        });
        let signing_input = format!("{}.{}", b64url_json(&header), b64url_json(&claims));
        let sig: Signature = key.signing.sign(signing_input.as_bytes());
        let sig_b64 = URL_SAFE_NO_PAD.encode(sig.to_bytes());
        format!("{signing_input}.{sig_b64}")
    }

    const TOKEN: &str = "header.payload.signature-opaque-access-token";
    const URL: &str = "https://localhost:3000/alice/test/doc";
    const METHOD: &str = "GET";

    fn ath_for(token: &str) -> String {
        URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))
    }

    /// A well-formed fresh proof for TOKEN/URL/METHOD with a unique jti.
    fn fresh_proof(key: &ClientKey, now: i64, jti: &str) -> String {
        fresh_proof_for(key, now, jti, TOKEN)
    }

    /// A well-formed fresh proof whose `ath` binds to the given access-token string (so it verifies on
    /// a hit when the cache was keyed by THAT token). Used by the LRU test, which keys by "A"/"C".
    fn fresh_proof_for(key: &ClientKey, now: i64, jti: &str, token: &str) -> String {
        mint_proof(
            key,
            serde_json::json!({
                "htu": URL,
                "htm": METHOD,
                "jti": jti,
                "iat": now,
                "ath": ath_for(token),
            }),
        )
    }

    fn policy() -> ProofPolicy {
        ProofPolicy {
            clock_tolerance_secs: 5,
            allow_missing_ath: false,
            replay_fail_closed: true,
        }
    }

    /// A verified token bound to `jkt`, expiring at `exp`.
    fn verified_token(jkt: &str, exp: i64) -> VerifiedToken {
        VerifiedToken {
            web_id: Some("https://localhost:3000/alice/profile/card#me".into()),
            issuer: Some("http://localhost:8080/realms/solid".into()),
            client_id: Some("conformance-alice".into()),
            scopes: vec![],
            cnf_jkt: Some(jkt.to_string()),
            cnf_x5t_s256: None,
            expiry: Some(exp),
        }
    }

    fn cache() -> (VerifiedTokenCache, InMemoryReplayStore) {
        (
            VerifiedTokenCache::new(DEFAULT_CACHE_CAPACITY, policy()),
            InMemoryReplayStore::with_window(StdDuration::from_secs(305)),
        )
    }

    // 1. A cache HIT reuses the verified result AND still verifies the fresh proof.
    #[test]
    fn hit_reuses_token_and_verifies_fresh_proof() {
        let key = new_client_key();
        let now = 1_000_000;
        let (cache, replay) = cache();
        let tok = verified_token(&key.jkt, now + 300);

        // Miss before insert.
        assert!(matches!(
            cache.authenticate(
                TOKEN,
                Some(&fresh_proof(&key, now, "j0")),
                METHOD,
                URL,
                now,
                &replay
            ),
            CacheDecision::Miss
        ));
        cache.insert(TOKEN, &tok, now);

        // Hit: a DIFFERENT fresh proof (new jti) verifies, returns the cached token.
        match cache.authenticate(
            TOKEN,
            Some(&fresh_proof(&key, now, "j1")),
            METHOD,
            URL,
            now,
            &replay,
        ) {
            CacheDecision::Verified(t) => {
                assert_eq!(t.web_id, tok.web_id);
                assert_eq!(t.cnf_jkt, tok.cnf_jkt);
            }
            _ => panic!("expected a verified hit"),
        }
    }

    // 2. An EXPIRED token (cached, then exp passes) => rejected on the next request (cache MISS forces a
    //    full re-verify, which will reject). The entry is evicted, never reused as valid.
    #[test]
    fn expired_token_is_miss_not_hit() {
        let key = new_client_key();
        let now = 1_000_000;
        let (cache, replay) = cache();
        let tok = verified_token(&key.jkt, now + 100); // expires at now+100
        cache.insert(TOKEN, &tok, now);

        // Still valid: a hit.
        assert!(matches!(
            cache.authenticate(
                TOKEN,
                Some(&fresh_proof(&key, now, "a")),
                METHOD,
                URL,
                now,
                &replay
            ),
            CacheDecision::Verified(_)
        ));
        // After exp: a MISS (the caller will then run the verifier, which rejects the expired token).
        let later = now + 101;
        assert!(matches!(
            cache.authenticate(
                TOKEN,
                Some(&fresh_proof(&key, later, "b")),
                METHOD,
                URL,
                later,
                &replay
            ),
            CacheDecision::Miss
        ));
        // And the expired entry was evicted.
        assert_eq!(cache.len(), 0);
    }

    // 3. A valid (cached) token + a DIFFERENT/swapped DPoP key => rejected (cnf.jkt enforced on the hit).
    #[test]
    fn wrong_dpop_key_rejected_on_hit() {
        let key = new_client_key();
        let attacker = new_client_key();
        let now = 1_000_000;
        let (cache, replay) = cache();
        let tok = verified_token(&key.jkt, now + 300);
        cache.insert(TOKEN, &tok, now);

        // The attacker holds the (cached) token but signs the proof with THEIR key -> cnf.jkt mismatch.
        match cache.authenticate(
            TOKEN,
            Some(&fresh_proof(&attacker, now, "x")),
            METHOD,
            URL,
            now,
            &replay,
        ) {
            CacheDecision::Reject(e) => {
                assert_eq!(e.status(), 401);
                assert!(e.message().contains("confirmation mismatch"));
            }
            _ => panic!("a swapped DPoP key must be rejected on a hit"),
        }
    }

    // 4. A replayed jti => rejected on the hit (proof replay still caught via the SHARED store).
    #[test]
    fn replayed_jti_rejected_on_hit() {
        let key = new_client_key();
        let now = 1_000_000;
        let (cache, replay) = cache();
        let tok = verified_token(&key.jkt, now + 300);
        cache.insert(TOKEN, &tok, now);

        let proof = fresh_proof(&key, now, "reused-jti");
        // First use: verified.
        assert!(matches!(
            cache.authenticate(TOKEN, Some(&proof), METHOD, URL, now, &replay),
            CacheDecision::Verified(_)
        ));
        // Replay the SAME proof: rejected.
        match cache.authenticate(TOKEN, Some(&proof), METHOD, URL, now, &replay) {
            CacheDecision::Reject(e) => {
                assert_eq!(e.status(), 401);
                assert!(e.message().contains("replay"));
            }
            _ => panic!("a replayed jti must be rejected on a hit"),
        }
    }

    // 4b. A jti used on the cache-MISS path (verifier) is replay-caught on the cache-HIT path because
    //     they share the replay store. Simulated here: mark via the SAME store, then hit.
    #[test]
    fn jti_shared_across_miss_and_hit_paths() {
        let key = new_client_key();
        let now = 1_000_000;
        let (cache, replay) = cache();
        let tok = verified_token(&key.jkt, now + 300);
        cache.insert(TOKEN, &tok, now);

        // The "miss path" (verifier) would mark this jti. Simulate by marking it directly.
        let proof = fresh_proof(&key, now, "cross-path-jti");
        replay
            .mark("cross-path-jti", StdDuration::from_secs(305))
            .unwrap();
        // Now the hit path sees the SAME jti -> replay.
        match cache.authenticate(TOKEN, Some(&proof), METHOD, URL, now, &replay) {
            CacheDecision::Reject(e) => assert!(e.message().contains("replay")),
            _ => panic!("a jti marked on the miss path must replay-fail on the hit path"),
        }
    }

    // 5. Two different tokens => two entries (no key collision; not keyed by webid/jkt).
    #[test]
    fn distinct_tokens_distinct_entries() {
        let key = new_client_key();
        let now = 1_000_000;
        let (cache, _replay) = cache();
        // SAME webid + SAME jkt, but different token strings.
        let tok = verified_token(&key.jkt, now + 300);
        cache.insert("token-A", &tok, now);
        cache.insert("token-B", &tok, now);
        assert_eq!(cache.len(), 2, "distinct tokens must not collide");
        // Re-inserting the same token string does not add a new entry.
        cache.insert("token-A", &tok, now);
        assert_eq!(cache.len(), 2);
    }

    // 6. An invalid token never populates the cache: `insert` is only called on success, and a token
    //    without cnf.jkt/exp is refused even if insert is (mis)called.
    #[test]
    fn token_without_cnf_or_exp_not_cached() {
        let (cache, _replay) = cache();
        let mut no_cnf = verified_token("jkt", 1_000_000);
        no_cnf.cnf_jkt = None;
        cache.insert("t", &no_cnf, 1_000_000);
        assert_eq!(cache.len(), 0, "a non-DPoP-bound token must not be cached");

        let mut no_exp = verified_token("jkt", 1_000_000);
        no_exp.expiry = None;
        cache.insert("t", &no_exp, 1_000_000);
        assert_eq!(cache.len(), 0, "a token without exp must not be cached");
    }

    // 7. Capacity bound holds (LRU eviction): the cache never exceeds capacity, and the least-recently-
    //    USED entry is the one evicted.
    #[test]
    fn capacity_bound_lru_eviction() {
        let key = new_client_key();
        let now = 1_000_000;
        let cache = VerifiedTokenCache::new(2, policy());
        let replay = InMemoryReplayStore::with_window(StdDuration::from_secs(305));
        let tok = verified_token(&key.jkt, now + 300);

        cache.insert("A", &tok, now);
        cache.insert("B", &tok, now);
        assert_eq!(cache.len(), 2);
        // Touch A (a hit, with a proof whose ath binds to "A" so it VERIFIES) so B becomes the LRU.
        assert!(matches!(
            cache.authenticate(
                "A",
                Some(&fresh_proof_for(&key, now, "ja", "A")),
                METHOD,
                URL,
                now,
                &replay
            ),
            CacheDecision::Verified(_)
        ));
        // Insert C at capacity -> evicts the LRU (B), keeps A + C.
        cache.insert("C", &tok, now);
        assert_eq!(cache.len(), 2, "capacity must never be exceeded");
        // A still present + verifies (its ath-matched proof passes -> a Verified hit, NOT a Miss).
        assert!(matches!(
            cache.authenticate(
                "A",
                Some(&fresh_proof_for(&key, now, "ja2", "A")),
                METHOD,
                URL,
                now,
                &replay
            ),
            CacheDecision::Verified(_)
        ));
        // C present + verifies.
        assert!(matches!(
            cache.authenticate(
                "C",
                Some(&fresh_proof_for(&key, now, "jc", "C")),
                METHOD,
                URL,
                now,
                &replay
            ),
            CacheDecision::Verified(_)
        ));
        // B was evicted -> a MISS (an evicted entry yields Miss; a present-but-failing one would
        // yield Reject, so Miss here proves eviction, not a proof failure).
        assert!(matches!(
            cache.authenticate(
                "B",
                Some(&fresh_proof_for(&key, now, "jb", "B")),
                METHOD,
                URL,
                now,
                &replay
            ),
            CacheDecision::Miss
        ));
    }

    // 8. A hit with a STALE proof iat => rejected (freshness still enforced per request).
    #[test]
    fn stale_proof_iat_rejected_on_hit() {
        let key = new_client_key();
        let now = 1_000_000;
        let (cache, replay) = cache();
        let tok = verified_token(&key.jkt, now + 100_000);
        cache.insert(TOKEN, &tok, now);
        // A proof iat far in the past (> 300 + 5) -> rejected.
        let stale = fresh_proof(&key, now - 1_000, "stale");
        match cache.authenticate(TOKEN, Some(&stale), METHOD, URL, now, &replay) {
            CacheDecision::Reject(e) => assert!(e.message().contains("iat")),
            _ => panic!("a stale proof iat must be rejected on a hit"),
        }
    }

    // 9. A hit with the WRONG htu / htm => rejected (binding to method+URL still enforced).
    #[test]
    fn wrong_htu_or_htm_rejected_on_hit() {
        let key = new_client_key();
        let now = 1_000_000;
        let (cache, replay) = cache();
        let tok = verified_token(&key.jkt, now + 300);
        cache.insert(TOKEN, &tok, now);

        // Wrong htu (different path).
        let wrong_htu = mint_proof(
            &key,
            serde_json::json!({"htu":"https://localhost:3000/bob/secret","htm":METHOD,"jti":"h1","iat":now,"ath":ath_for(TOKEN)}),
        );
        assert!(matches!(
            cache.authenticate(TOKEN, Some(&wrong_htu), METHOD, URL, now, &replay),
            CacheDecision::Reject(_)
        ));
        // Wrong htm.
        let wrong_htm = mint_proof(
            &key,
            serde_json::json!({"htu":URL,"htm":"DELETE","jti":"h2","iat":now,"ath":ath_for(TOKEN)}),
        );
        assert!(matches!(
            cache.authenticate(TOKEN, Some(&wrong_htm), METHOD, URL, now, &replay),
            CacheDecision::Reject(_)
        ));
    }

    // 10. A hit with a WRONG ath (proof's ath != H(token)) => rejected -- the ath binding is re-checked
    //     against the SAME token on every request (a captured proof for a different token cannot ride a
    //     cache hit).
    #[test]
    fn wrong_ath_rejected_on_hit() {
        let key = new_client_key();
        let now = 1_000_000;
        let (cache, replay) = cache();
        let tok = verified_token(&key.jkt, now + 300);
        cache.insert(TOKEN, &tok, now);
        let bad_ath = mint_proof(
            &key,
            serde_json::json!({"htu":URL,"htm":METHOD,"jti":"a1","iat":now,"ath":ath_for("a-different-token")}),
        );
        match cache.authenticate(TOKEN, Some(&bad_ath), METHOD, URL, now, &replay) {
            CacheDecision::Reject(e) => assert!(e.message().contains("ath")),
            _ => panic!("a wrong ath must be rejected on a hit"),
        }
    }

    // 11. A hit with NO proof at all => rejected (proof-of-possession required every request).
    #[test]
    fn missing_proof_rejected_on_hit() {
        let key = new_client_key();
        let now = 1_000_000;
        let (cache, replay) = cache();
        let tok = verified_token(&key.jkt, now + 300);
        cache.insert(TOKEN, &tok, now);
        match cache.authenticate(TOKEN, None, METHOD, URL, now, &replay) {
            CacheDecision::Reject(e) => assert_eq!(e.status(), 401),
            _ => panic!("a hit with no DPoP proof must be rejected"),
        }
    }

    // 12. A garbage / unsigned proof => rejected (signature still verified on the hit path).
    #[test]
    fn forged_proof_signature_rejected_on_hit() {
        let key = new_client_key();
        let now = 1_000_000;
        let (cache, replay) = cache();
        let tok = verified_token(&key.jkt, now + 300);
        cache.insert(TOKEN, &tok, now);
        // Tamper the signature of an otherwise-valid proof.
        let mut proof = fresh_proof(&key, now, "f1");
        proof.truncate(proof.len() - 4);
        proof.push_str("AAAA");
        assert!(matches!(
            cache.authenticate(TOKEN, Some(&proof), METHOD, URL, now, &replay),
            CacheDecision::Reject(_)
        ));
    }

    // 13. ath-compat: when allow_missing_ath, an ath-LESS proof is accepted on a hit; a present-but-
    //     wrong ath is STILL rejected (only absence tolerated).
    #[test]
    fn ath_compat_absent_ok_present_wrong_rejected() {
        let key = new_client_key();
        let now = 1_000_000;
        let compat_policy = ProofPolicy {
            clock_tolerance_secs: 5,
            allow_missing_ath: true,
            replay_fail_closed: true,
        };
        let cache = VerifiedTokenCache::new(16, compat_policy);
        let replay = InMemoryReplayStore::with_window(StdDuration::from_secs(305));
        let tok = verified_token(&key.jkt, now + 300);
        cache.insert(TOKEN, &tok, now);

        // ath ABSENT -> accepted under compat.
        let no_ath = mint_proof(
            &key,
            serde_json::json!({"htu":URL,"htm":METHOD,"jti":"c1","iat":now}),
        );
        assert!(matches!(
            cache.authenticate(TOKEN, Some(&no_ath), METHOD, URL, now, &replay),
            CacheDecision::Verified(_)
        ));
        // ath PRESENT-BUT-WRONG -> still rejected even under compat.
        let wrong_ath = mint_proof(
            &key,
            serde_json::json!({"htu":URL,"htm":METHOD,"jti":"c2","iat":now,"ath":ath_for("other")}),
        );
        assert!(matches!(
            cache.authenticate(TOKEN, Some(&wrong_ath), METHOD, URL, now, &replay),
            CacheDecision::Reject(_)
        ));
    }

    // 14. SharedReplay forwards to the inner store (one shared jti set).
    #[test]
    fn shared_replay_forwards() {
        let inner = Arc::new(InMemoryReplayStore::with_window(StdDuration::from_secs(
            305,
        )));
        let shared = SharedReplay::new(Arc::clone(&inner));
        assert_eq!(
            shared.mark("s1", StdDuration::from_secs(305)).unwrap(),
            MarkResult::New
        );
        // The SAME jti via the inner handle is a replay.
        assert_eq!(
            inner.mark("s1", StdDuration::from_secs(305)).unwrap(),
            MarkResult::Replay
        );
        // And the handle() accessor returns the same Arc.
        assert!(Arc::ptr_eq(&shared.handle(), &inner));
    }

    // 15. peek_jti reads a non-empty jti. Parity helper.
    #[test]
    fn peek_jti_reads_nonempty() {
        let key = new_client_key();
        let proof = fresh_proof(&key, 1_000_000, "pj");
        assert_eq!(peek_jti(&proof).as_deref(), Some("pj"));
    }

    // 18. VALIDATION-TTL: a still-unexpired token whose cache entry is older than the validation TTL
    //     is a MISS (force-re-verify), so a JWKS revocation propagates within one TTL window instead of
    //     being masked until exp (roborev round-1 Medium, finding #2).
    #[test]
    fn entry_older_than_validation_ttl_is_miss() {
        let key = new_client_key();
        let now = 1_000_000;
        // exp is far away (token still valid), but the validation TTL is short (10s).
        let cache = VerifiedTokenCache::with_max_entry_ttl(64, policy(), 10);
        let replay = InMemoryReplayStore::with_window(StdDuration::from_secs(305));
        let tok = verified_token(&key.jkt, now + 100_000);
        cache.insert(TOKEN, &tok, now);

        // Within the TTL: a hit.
        assert!(matches!(
            cache.authenticate(
                TOKEN,
                Some(&fresh_proof(&key, now, "v1")),
                METHOD,
                URL,
                now,
                &replay
            ),
            CacheDecision::Verified(_)
        ));
        // After the TTL (but BEFORE exp): a MISS -- forces the verifier to re-check the (possibly now
        // revoked) signing key. The entry is evicted.
        let later = now + 11;
        assert!(matches!(
            cache.authenticate(
                TOKEN,
                Some(&fresh_proof(&key, later, "v2")),
                METHOD,
                URL,
                later,
                &replay
            ),
            CacheDecision::Miss
        ));
        assert_eq!(
            cache.len(),
            0,
            "the stale entry must be evicted on the validation-TTL miss"
        );
    }

    // 19. iat WINDOW PARITY with the verifier (symmetric |now - iat| <= max_age + tolerance). A proof
    //     within the window (incl. a future skew up to the window) is accepted on a hit exactly as the
    //     verifier accepts it on a miss; one beyond the window is rejected. (Future-axis tightening is a
    //     verifier follow-up applied to BOTH paths -- roborev round-2: the hit path must not diverge.)
    #[test]
    fn iat_window_matches_verifier() {
        let key = new_client_key();
        let now = 1_000_000;
        let (cache, replay) = cache(); // clock_tolerance_secs = 5, max_age = 300 -> window 305
        let tok = verified_token(&key.jkt, now + 100_000);
        cache.insert(TOKEN, &tok, now);

        // Within the window (a +60s future skew, < 305) -> accepted, matching the verifier.
        let in_window = fresh_proof(&key, now + 60, "w1");
        assert!(matches!(
            cache.authenticate(TOKEN, Some(&in_window), METHOD, URL, now, &replay),
            CacheDecision::Verified(_)
        ));
        // Beyond the window (a +1000s future skew, > 305) -> rejected, matching the verifier.
        let out_window = fresh_proof(&key, now + 1_000, "w2");
        match cache.authenticate(TOKEN, Some(&out_window), METHOD, URL, now, &replay) {
            CacheDecision::Reject(e) => assert!(e.message().contains("iat")),
            _ => panic!("a proof beyond the iat window must be rejected on a hit"),
        }
    }

    // 17. ORDER: a wrong-cnf.jkt proof is rejected BEFORE the jti is marked, so it does NOT consume the
    //     jti -- the SAME jti re-presented with the CORRECT key still verifies (matches the verifier's
    //     full-proof-validate-then-replay order; a mis-bound proof must not burn a victim's jti).
    #[test]
    fn wrong_cnf_does_not_consume_jti() {
        let key = new_client_key();
        let attacker = new_client_key();
        let now = 1_000_000;
        let (cache, replay) = cache();
        let tok = verified_token(&key.jkt, now + 300);
        cache.insert(TOKEN, &tok, now);

        // Attacker's proof carries jti "shared" but the WRONG key -> cnf.jkt mismatch, rejected BEFORE
        // the replay mark, so "shared" is NOT consumed.
        let bad = mint_proof(
            &attacker,
            serde_json::json!({"htu":URL,"htm":METHOD,"jti":"shared","iat":now,"ath":ath_for(TOKEN)}),
        );
        assert!(matches!(
            cache.authenticate(TOKEN, Some(&bad), METHOD, URL, now, &replay),
            CacheDecision::Reject(_)
        ));
        // The legitimate holder uses the SAME jti "shared" with the CORRECT key -> still New -> verifies
        // (proving the mis-bound attempt did not mark/consume "shared").
        let good = mint_proof(
            &key,
            serde_json::json!({"htu":URL,"htm":METHOD,"jti":"shared","iat":now,"ath":ath_for(TOKEN)}),
        );
        assert!(matches!(
            cache.authenticate(TOKEN, Some(&good), METHOD, URL, now, &replay),
            CacheDecision::Verified(_)
        ));
    }

    // 16. A FUTURE proof iat beyond the window => rejected (the |now - iat| bound is symmetric).
    #[test]
    fn future_proof_iat_rejected_on_hit() {
        let key = new_client_key();
        let now = 1_000_000;
        let (cache, replay) = cache();
        let tok = verified_token(&key.jkt, now + 100_000);
        cache.insert(TOKEN, &tok, now);
        let future = fresh_proof(&key, now + 1_000, "future");
        match cache.authenticate(TOKEN, Some(&future), METHOD, URL, now, &replay) {
            CacheDecision::Reject(e) => assert!(e.message().contains("iat")),
            _ => panic!("a far-future proof iat must be rejected on a hit"),
        }
    }
}
