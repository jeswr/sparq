// AUTHORED-BY Claude Fable 5
//! DPoP-SK per-request attestation verification — the spec's §"Verification" steps, in order,
//! all fail-closed.
//!
//! A request bearing a `dpop-sk`-tagged RFC 9421 signature is processed under this profile
//! EXCLUSIVELY: the server never falls back to evaluating a `DPoP` proof header also present on
//! the same request, and never treats the access token as bearer under any outcome (spec
//! §"Fallback and no-downgrade rules" 3). Every failure maps to the SAME decision ([`SkDecision::Deny`]
//! ⇒ the caller's single generic `WWW-Authenticate: DPoP` challenge) — deliberately withholding
//! which check failed (spec §"Failure signalling"; adversarial-review entry A18).
//!
//! Ordering is normative and mirrored here exactly:
//! 1. parse → 2. session lookup → 3. connection binding → 4. token binding → 5. freshness →
//! 6. HMAC over the server-reconstructed base (BEFORE any state mutation) → 7. window
//!    check-and-mark (verify-then-mark) → 8. admit (inject the stored verified identity).

use sha2::{Digest as _, Sha256};
use subtle::ConstantTimeEq as _;

use crate::auth::VerifiedToken;

use super::sig::{mentions_dpop_sk, parse_attestation, signature_base};
use super::window::WindowVerdict;
use super::{ConnSk, SkState};

/// The outcome of DPoP-SK request processing.
#[derive(Debug)]
pub enum SkDecision {
    /// The request carries no `dpop-sk`-tagged signature — it is NOT under this profile. The
    /// caller proceeds with the unchanged DPoP baseline (which never accepts a bare token).
    NotApplicable,
    /// The attestation verified: inject `token` downstream exactly as a successful DPoP
    /// verification would; `session_id` names the authenticating session (used by termination).
    Verified {
        token: VerifiedToken,
        session_id: String,
    },
    /// Any failure of any step: the caller answers 401 with the standard DPoP challenge. The
    /// client falls back to re-establishment or plain per-request DPoP — both full-strength PoP.
    Deny,
}

/// Process a request under the DPoP-SK profile (or decide it is not applicable).
///
/// `method` is the request method; `target_uri` is the absolute target URI reconstructed by the
/// SERVER from its configured base origin + the request's path-and-query (spec §"MITM and the
/// transport": never from unauthenticated client-controlled headers). `conn` is the
/// per-connection context injected by the TLS acceptor (`None` off the in-process-TLS path).
pub fn verify_attested_request(
    state: &SkState,
    headers: &http::HeaderMap,
    method: &str,
    target_uri: &str,
    conn: Option<&ConnSk>,
    now: i64,
) -> SkDecision {
    // ---- Applicability routing (cheap, byte-level) --------------------------------------------
    let sig_input_headers: Vec<&http::HeaderValue> =
        headers.get_all("signature-input").iter().collect();
    if sig_input_headers.is_empty() {
        return SkDecision::NotApplicable;
    }
    // The substring pre-check runs on raw bytes so an undecodable foreign header cannot dodge it.
    if !sig_input_headers
        .iter()
        .any(|v| mentions_dpop_sk(&String::from_utf8_lossy(v.as_bytes())))
    {
        return SkDecision::NotApplicable;
    }

    // From here on the request claims this profile: every failure is a Deny (fail-closed).
    // (1) Parse — strict RFC 8941 + profile validation.
    let Ok(sig_input) = join_field(&sig_input_headers) else {
        return SkDecision::Deny;
    };
    let signature_headers: Vec<&http::HeaderValue> = headers.get_all("signature").iter().collect();
    let Ok(signature_field) = join_field(&signature_headers) else {
        return SkDecision::Deny;
    };
    let attestation = match parse_attestation(&sig_input, &signature_field) {
        Ok(Some(a)) => a,
        // No string-typed dpop-sk tag after strict parsing: not ours (the pre-check was a
        // substring heuristic). The DPoP baseline still fully gates the request.
        Ok(None) => return SkDecision::NotApplicable,
        Err(_) => return SkDecision::Deny,
    };

    // (2) Session lookup — live, unexpired (expiry enforced inside `get`).
    let Some(session) = state.store().get(&attestation.keyid, now) else {
        return SkDecision::Deny;
    };

    // (3) Connection binding (cb=tls-exporter sessions only): the request must arrive on the
    // session's pinned TLS connection. A missing ConnSk (plain path / different listener) denies.
    if let Some(pinned) = session.conn_id {
        match conn {
            Some(c) if c.conn_id == pinned => {}
            _ => return SkDecision::Deny,
        }
    }

    // (4) Token binding: the token still travels under the DPoP scheme; its hash must equal the
    // session's bound token hash (constant-time), and the token must be unexpired.
    let auth_headers: Vec<&http::HeaderValue> = headers.get_all("authorization").iter().collect();
    if auth_headers.len() != 1 {
        return SkDecision::Deny; // absent or ambiguous credentials
    }
    let Ok(auth_value) = auth_headers[0].to_str() else {
        return SkDecision::Deny;
    };
    let Some(access_token) = crate::auth::dpop_scheme_access_token(auth_value) else {
        return SkDecision::Deny;
    };
    let presented_hash: [u8; 32] = Sha256::digest(access_token.as_bytes()).into();
    if presented_hash.ct_ne(&session.token_hash).into() {
        return SkDecision::Deny;
    }
    if session.token_exp <= now {
        return SkDecision::Deny;
    }

    // (5) Freshness: `created` within the same acceptance window as DPoP `iat` (symmetric), and
    // `expires` (when the client sent one) honoured.
    if (now - attestation.created).abs() > state.created_window_secs() {
        return SkDecision::Deny;
    }
    if let Some(expires) = attestation.expires {
        if expires <= now {
            return SkDecision::Deny;
        }
    }

    // (6) Signature: recompute the base from the ACTUAL request + the received Signature-Input
    // serialization; constant-time HMAC compare. Runs BEFORE any session-state mutation.
    let Ok(base) = signature_base(&attestation, method, target_uri, headers) else {
        return SkDecision::Deny;
    };
    if !session.hmac_verify(base.as_bytes(), &attestation.signature) {
        return SkDecision::Deny;
    }

    // (7) Anti-replay — verify-then-mark: only an AUTHENTICATED request reaches the window, so
    // forged traffic can neither burn a counter nor advance the window (spec A7). The
    // check-and-mark is atomic per session (the window mutex), so two concurrent copies of the
    // same counter cannot both pass (spec A8).
    if session.window_check_and_mark(attestation.nonce) != WindowVerdict::Accept {
        return SkDecision::Deny;
    }

    // (8) Admit: the session's stored verified identity, exactly as DPoP verification would.
    SkDecision::Verified {
        token: session.token.clone(),
        session_id: session.session_id.clone(),
    }
}

/// Join multiple instances of a structured field per RFC 8941 (list-based fields may be split
/// across field lines; combining with `", "` restores the single field value).
fn join_field(values: &[&http::HeaderValue]) -> Result<String, ()> {
    let mut parts = Vec::with_capacity(values.len());
    for v in values {
        parts.push(v.to_str().map_err(|_| ())?);
    }
    Ok(parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pop::sk::store::{CbFlavour, Session};
    use crate::pop::sk::test_vectors::{
        example_k_none, EXAMPLE_JKT, EXAMPLE_SESSION_ID, EXAMPLE_TOKEN,
    };
    use crate::pop::sk::{SkConfig, SkState};

    const NOW: i64 = 1_720_000_000;
    const TARGET: &str = "https://pod.example/alice/private/doc";

    fn state() -> SkState {
        SkState::new(SkConfig::default())
    }

    fn insert_example_session(state: &SkState, conn_id: Option<u64>) {
        let token_hash: [u8; 32] = sha2::Sha256::digest(EXAMPLE_TOKEN.as_bytes()).into();
        let mut token = VerifiedToken::public();
        token.web_id = Some("https://id.example/alice#me".to_string());
        token.cnf_jkt = Some(EXAMPLE_JKT.to_string());
        token.expiry = Some(NOW + 300);
        let session = Session::new(
            EXAMPLE_SESSION_ID.to_string(),
            example_k_none(),
            token_hash,
            EXAMPLE_JKT.to_string(),
            token,
            NOW + 300,
            NOW + 300,
            if conn_id.is_some() {
                CbFlavour::TlsExporter
            } else {
                CbFlavour::None
            },
            conn_id,
        );
        assert!(state.store().insert(session, NOW));
    }

    /// The spec's Appendix-A attested request, as exact header values.
    fn appendix_a_headers() -> http::HeaderMap {
        let mut h = http::HeaderMap::new();
        h.insert(
            "authorization",
            format!("DPoP {EXAMPLE_TOKEN}").parse().unwrap(),
        );
        h.insert(
            "signature-input",
            format!(
                "sig=(\"@method\" \"@target-uri\" \"authorization\");created=1720000000;keyid=\"{EXAMPLE_SESSION_ID}\";alg=\"hmac-sha256\";nonce=\"1\";tag=\"dpop-sk\""
            )
            .parse()
            .unwrap(),
        );
        h.insert(
            "signature",
            "sig=:hPkuPd32xbb4hUjR/hjbj0Cp445ZfWoOgrcq8+f3I3g=:"
                .parse()
                .unwrap(),
        );
        h
    }

    #[test]
    fn appendix_a_request_verifies_byte_for_byte() {
        // Reproduces the spec's worked example end-to-end through the REAL verification path:
        // published headers + published key ⇒ accept, high-water = 1.
        let st = state();
        insert_example_session(&st, None);
        let d = verify_attested_request(&st, &appendix_a_headers(), "GET", TARGET, None, NOW);
        match d {
            SkDecision::Verified { session_id, token } => {
                assert_eq!(session_id, EXAMPLE_SESSION_ID);
                assert_eq!(token.web_id.as_deref(), Some("https://id.example/alice#me"));
            }
            other => panic!("expected Verified, got {other:?}"),
        }
        // The counter is now consumed: an exact replay is denied (window duplicate).
        let d2 = verify_attested_request(&st, &appendix_a_headers(), "GET", TARGET, None, NOW);
        assert!(matches!(d2, SkDecision::Deny), "replay must be denied");
    }

    #[test]
    fn no_signature_headers_is_not_applicable() {
        let st = state();
        let mut h = http::HeaderMap::new();
        h.insert("authorization", "DPoP whatever".parse().unwrap());
        assert!(matches!(
            verify_attested_request(&st, &h, "GET", TARGET, None, NOW),
            SkDecision::NotApplicable
        ));
    }

    #[test]
    fn foreign_signature_tag_is_not_applicable() {
        let st = state();
        let mut h = http::HeaderMap::new();
        h.insert(
            "signature-input",
            "sig=(\"@method\");created=1;keyid=\"x\";tag=\"other-app\""
                .parse()
                .unwrap(),
        );
        h.insert(
            "signature",
            "sig=:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=:"
                .parse()
                .unwrap(),
        );
        assert!(matches!(
            verify_attested_request(&st, &h, "GET", TARGET, None, NOW),
            SkDecision::NotApplicable
        ));
    }

    #[test]
    fn unknown_session_denies() {
        let st = state(); // nothing inserted
        assert!(matches!(
            verify_attested_request(&st, &appendix_a_headers(), "GET", TARGET, None, NOW),
            SkDecision::Deny
        ));
    }

    #[test]
    fn expired_session_denies_fail_closed() {
        let st = state();
        insert_example_session(&st, None);
        // 301 s later the session (expires_at = NOW+300) is gone.
        assert!(matches!(
            verify_attested_request(&st, &appendix_a_headers(), "GET", TARGET, None, NOW + 301),
            SkDecision::Deny
        ));
    }

    #[test]
    fn expired_bound_token_denies_even_with_live_session() {
        let st = state();
        // Session outlives the token (mis-established on purpose): token expiry must still gate.
        let token_hash: [u8; 32] = sha2::Sha256::digest(EXAMPLE_TOKEN.as_bytes()).into();
        let session = Session::new(
            EXAMPLE_SESSION_ID.to_string(),
            example_k_none(),
            token_hash,
            EXAMPLE_JKT.to_string(),
            VerifiedToken::public(),
            NOW - 1, // token already expired
            NOW + 300,
            CbFlavour::None,
            None,
        );
        st.store().insert(session, NOW);
        assert!(matches!(
            verify_attested_request(&st, &appendix_a_headers(), "GET", TARGET, None, NOW),
            SkDecision::Deny
        ));
    }

    #[test]
    fn mutated_components_fail_the_hmac() {
        // HMAC over mutated components: the signature was computed for
        // GET https://pod.example/alice/private/doc — any server-observed divergence must deny.
        let st = state();
        insert_example_session(&st, None);
        // (a) Different target (attestation transplant, spec A10).
        assert!(matches!(
            verify_attested_request(
                &st,
                &appendix_a_headers(),
                "GET",
                "https://pod.example/alice/private/other",
                None,
                NOW
            ),
            SkDecision::Deny
        ));
        // (b) Different method.
        assert!(matches!(
            verify_attested_request(&st, &appendix_a_headers(), "DELETE", TARGET, None, NOW),
            SkDecision::Deny
        ));
        // (c) Tampered signature bytes.
        let mut h = appendix_a_headers();
        h.insert(
            "signature",
            "sig=:hPkuPd32xbb4hUjR/hjbj0Cp445ZfWoOgrcq8+f3I3h=:"
                .parse()
                .unwrap(),
        );
        assert!(matches!(
            verify_attested_request(&st, &h, "GET", TARGET, None, NOW),
            SkDecision::Deny
        ));
        // (d) A different presented token (token substitution, spec A9) — fails token binding.
        let mut h = appendix_a_headers();
        h.insert("authorization", "DPoP some.other.token".parse().unwrap());
        assert!(matches!(
            verify_attested_request(&st, &h, "GET", TARGET, None, NOW),
            SkDecision::Deny
        ));
        // NONE of the failures consumed the counter (verify-then-mark): the genuine request
        // still verifies afterwards.
        assert!(matches!(
            verify_attested_request(&st, &appendix_a_headers(), "GET", TARGET, None, NOW),
            SkDecision::Verified { .. }
        ));
    }

    #[test]
    fn bearer_scheme_denies_token_binding() {
        let st = state();
        insert_example_session(&st, None);
        let mut h = appendix_a_headers();
        h.insert(
            "authorization",
            format!("Bearer {EXAMPLE_TOKEN}").parse().unwrap(),
        );
        assert!(matches!(
            verify_attested_request(&st, &h, "GET", TARGET, None, NOW),
            SkDecision::Deny
        ));
    }

    #[test]
    fn created_outside_window_denies() {
        let st = state();
        insert_example_session(&st, None);
        // The example's created is fixed at NOW; move the clock past the acceptance window.
        let skewed_now = NOW + st.created_window_secs() + 1;
        // (Session would also be expired at that point; rebuild with a long-lived session.)
        let st = state();
        let token_hash: [u8; 32] = sha2::Sha256::digest(EXAMPLE_TOKEN.as_bytes()).into();
        let session = Session::new(
            EXAMPLE_SESSION_ID.to_string(),
            example_k_none(),
            token_hash,
            EXAMPLE_JKT.to_string(),
            VerifiedToken::public(),
            skewed_now + 100,
            skewed_now + 100,
            CbFlavour::None,
            None,
        );
        st.store().insert(session, NOW);
        assert!(matches!(
            verify_attested_request(&st, &appendix_a_headers(), "GET", TARGET, None, skewed_now),
            SkDecision::Deny
        ));
    }

    #[test]
    fn connection_pinned_session_requires_the_same_connection() {
        // Session-key reuse across connections is forbidden for cb=tls-exporter: the session is
        // pinned to conn 7; any other (or missing) connection context denies (spec step 3 / A4).
        let st = state();
        insert_example_session(&st, Some(7));
        let ok = verify_attested_request(
            &st,
            &appendix_a_headers(),
            "GET",
            TARGET,
            Some(&ConnSk::new(7, None)),
            NOW,
        );
        assert!(matches!(ok, SkDecision::Verified { .. }));
        // A different connection: deny (fresh state so the counter is untouched).
        let st = state();
        insert_example_session(&st, Some(7));
        assert!(matches!(
            verify_attested_request(
                &st,
                &appendix_a_headers(),
                "GET",
                TARGET,
                Some(&ConnSk::new(8, None)),
                NOW
            ),
            SkDecision::Deny
        ));
        // No connection context at all (e.g. the plain-HTTP listener): deny.
        assert!(matches!(
            verify_attested_request(&st, &appendix_a_headers(), "GET", TARGET, None, NOW),
            SkDecision::Deny
        ));
    }

    #[test]
    fn duplicate_authorization_headers_deny() {
        let st = state();
        insert_example_session(&st, None);
        let mut h = appendix_a_headers();
        h.append(
            "authorization",
            format!("DPoP {EXAMPLE_TOKEN}").parse().unwrap(),
        );
        assert!(matches!(
            verify_attested_request(&st, &h, "GET", TARGET, None, NOW),
            SkDecision::Deny
        ));
    }
}
