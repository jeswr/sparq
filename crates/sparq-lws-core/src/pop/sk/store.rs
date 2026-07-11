// AUTHORED-BY Claude Fable 5
//! The DPoP-SK session store — bounded, expiring, per-session replay window.
//!
//! Server sessions are SOFT state (spec §"Server session state"): the store MUST be
//! capacity-bounded, eviction is always safe (an evicted session costs the client one extra full
//! DPoP round), and losing everything on restart is correct behaviour. Nothing here is persisted;
//! `K` MUST NOT reach durable storage (spec §"Lifetime": "Implementations MUST NOT persist K").
//!
//! Locking layout (the per-request hot path):
//! - the MAP is behind one `RwLock` — per-request lookup is a read lock + `Arc` clone;
//! - each session's immutable half (`K`, `token_hash`, `jkt`, the stored [`VerifiedToken`],
//!   expiries, `conn_id`) is lock-free after creation, so the HMAC recompute runs without holding
//!   any lock;
//! - the REPLAY WINDOW is the one mutable per-session piece, behind its own `Mutex`. The
//!   spec-normative atomicity ("the signature-verify + window check-and-mark pair … MUST be
//!   atomic") is satisfied because the *check-and-mark itself* is a single guarded operation and
//!   is only invoked after HMAC success (verify-then-mark): two concurrent copies of the same
//!   counter serialize on the window mutex and the second is a [`WindowVerdict::Duplicate`] —
//!   they cannot both pass.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::auth::VerifiedToken;

use super::derive::SessionKey;
use super::window::{ReplayWindow, WindowVerdict};

/// The negotiated channel-binding flavour of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CbFlavour {
    /// Server-generated key, TLS-delivered once; not connection-bound (the browser flavour).
    None,
    /// Key derived from TLS Exported Keying Material; session pinned to its TLS connection.
    TlsExporter,
}

impl CbFlavour {
    pub fn as_str(&self) -> &'static str {
        match self {
            CbFlavour::None => "none",
            CbFlavour::TlsExporter => "tls-exporter",
        }
    }
}

/// One established DPoP-SK session.
pub struct Session {
    /// The opaque session identifier (≥128 bits CSPRNG, base64url-no-pad).
    pub session_id: String,
    /// The symmetric session key — private; use [`Session::hmac_verify`] so `K` never leaves.
    key: SessionKey,
    /// SHA-256 of the ASCII access token bound at establishment (the `ath`-preimage check).
    pub token_hash: [u8; 32],
    /// The token's `cnf.jkt` at establishment (an HKDF input; kept for diagnostics/invariants).
    pub jkt: String,
    /// The verified identity injected downstream on every accepted attested request — exactly the
    /// [`VerifiedToken`] the establishment DPoP verification produced.
    pub token: VerifiedToken,
    /// The bound access token's `exp` (enforced per request alongside the session expiry).
    pub token_exp: i64,
    /// Session expiry (epoch seconds): `established_at + min(300, token_exp − now)`.
    pub expires_at: i64,
    /// The negotiated flavour.
    pub cb: CbFlavour,
    /// The pinned TLS connection id — `Some` iff `cb=tls-exporter` (verification step 3).
    pub conn_id: Option<u64>,
    /// The RFC 4303 receive window (spec §"Anti-replay").
    window: Mutex<ReplayWindow>,
}

impl Session {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_id: String,
        key: SessionKey,
        token_hash: [u8; 32],
        jkt: String,
        token: VerifiedToken,
        token_exp: i64,
        expires_at: i64,
        cb: CbFlavour,
        conn_id: Option<u64>,
    ) -> Self {
        Self {
            session_id,
            key,
            token_hash,
            jkt,
            token,
            token_exp,
            expires_at,
            cb,
            conn_id,
            window: Mutex::new(ReplayWindow::new()),
        }
    }

    /// Constant-time HMAC-SHA256 verification under this session's key (the key never leaves).
    pub fn hmac_verify(&self, message: &[u8], tag: &[u8]) -> bool {
        super::derive::hmac_verify(&self.key, message, tag)
    }

    /// Check-and-mark the replay counter — MUST be called only after [`Session::hmac_verify`]
    /// succeeded (verify-then-mark; see the module docs for the atomicity argument).
    pub fn window_check_and_mark(&self, nonce: u64) -> WindowVerdict {
        self.window
            .lock()
            .expect("window mutex poisoned")
            .check_and_mark(nonce)
    }
}

/// The bounded session store.
pub struct SessionStore {
    inner: RwLock<HashMap<String, Arc<Session>>>,
    capacity: usize,
}

impl SessionStore {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
            capacity: capacity.max(1),
        }
    }

    /// Insert a fresh session. At capacity, expired sessions are dropped first; if still full, the
    /// soonest-expiring live session is evicted (always safe — one extra DPoP round for its
    /// client). Returns `false` (refusing the insert) only on a session-id collision, which the
    /// caller retries with a fresh id (spec: "servers MUST NOT reuse one").
    pub fn insert(&self, session: Session, now: i64) -> bool {
        let mut map = self.inner.write().expect("session map poisoned");
        if map.contains_key(&session.session_id) {
            return false;
        }
        if map.len() >= self.capacity {
            map.retain(|_, s| s.expires_at > now);
        }
        if map.len() >= self.capacity {
            if let Some(oldest) = map
                .values()
                .min_by_key(|s| s.expires_at)
                .map(|s| s.session_id.clone())
            {
                map.remove(&oldest);
            }
        }
        map.insert(session.session_id.clone(), Arc::new(session));
        true
    }

    /// Look up a LIVE session (session expiry enforced here; an expired entry is removed and
    /// treated as a miss — fail-closed). Token expiry is enforced separately by the verifier
    /// (both are per-request checks per the spec's lifetime rules).
    pub fn get(&self, session_id: &str, now: i64) -> Option<Arc<Session>> {
        let found = self
            .inner
            .read()
            .expect("session map poisoned")
            .get(session_id)
            .cloned();
        match found {
            Some(s) if s.expires_at > now => Some(s),
            Some(_) => {
                // Expired: drop it (lazily, under the write lock).
                self.inner
                    .write()
                    .expect("session map poisoned")
                    .remove(session_id);
                None
            }
            None => None,
        }
    }

    /// Remove a session (client-requested termination, or the server's forced-rekey right).
    pub fn remove(&self, session_id: &str) -> bool {
        self.inner
            .write()
            .expect("session map poisoned")
            .remove(session_id)
            .is_some()
    }

    /// Number of stored sessions (tests/diagnostics).
    pub fn len(&self) -> usize {
        self.inner.read().expect("session map poisoned").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(id: &str, expires_at: i64) -> Session {
        Session::new(
            id.to_string(),
            SessionKey::new([7u8; 32]),
            [0u8; 32],
            "jkt".into(),
            VerifiedToken::public(),
            i64::MAX,
            expires_at,
            CbFlavour::None,
            None,
        )
    }

    #[test]
    fn get_enforces_session_expiry_fail_closed() {
        let store = SessionStore::new(4);
        assert!(store.insert(session("a", 100), 0));
        assert!(store.get("a", 99).is_some());
        // At/after expiry ⇒ miss, and the entry is dropped.
        assert!(store.get("a", 100).is_none());
        assert!(store.is_empty());
    }

    #[test]
    fn capacity_is_bounded_with_soonest_expiry_eviction() {
        let store = SessionStore::new(2);
        assert!(store.insert(session("a", 50), 0));
        assert!(store.insert(session("b", 200), 0));
        // Full; "a" expires soonest ⇒ evicted to admit "c".
        assert!(store.insert(session("c", 300), 0));
        assert_eq!(store.len(), 2);
        assert!(store.get("a", 0).is_none());
        assert!(store.get("b", 0).is_some());
        assert!(store.get("c", 0).is_some());
    }

    #[test]
    fn expired_entries_are_reaped_before_evicting_live_ones() {
        let store = SessionStore::new(2);
        assert!(store.insert(session("a", 50), 0));
        assert!(store.insert(session("b", 200), 0));
        // At now=60, "a" is expired: the insert reaps it instead of evicting live "b".
        assert!(store.insert(session("c", 300), 60));
        assert!(store.get("b", 60).is_some());
        assert!(store.get("c", 60).is_some());
    }

    #[test]
    fn id_collision_refused() {
        let store = SessionStore::new(4);
        assert!(store.insert(session("a", 100), 0));
        assert!(!store.insert(session("a", 200), 0));
        // Original untouched.
        assert_eq!(store.get("a", 0).unwrap().expires_at, 100);
    }

    #[test]
    fn remove_terminates() {
        let store = SessionStore::new(4);
        assert!(store.insert(session("a", 100), 0));
        assert!(store.remove("a"));
        assert!(!store.remove("a"));
        assert!(store.get("a", 0).is_none());
    }
}
