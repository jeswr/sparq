// AUTHORED-BY Claude Fable 5
//! DPoP-SK HTTP surface — the session establishment/termination endpoint (`/.pop/session`) and
//! the RFC 9728 protected-resource-metadata document that advertises the profile.
//!
//! The establishment route is mounted BEHIND the standard DPoP auth middleware (`crate::app`), so
//! by the time [`establish_handler`] runs, the request has passed the complete, UNMODIFIED
//! RFC 9449 §4.3 verification (proof signature, `htm`/`htu` against this endpoint, `iat` window,
//! `jti` replay, `ath`, `cnf.jkt`) plus every access-token and WebID/issuer check — establishment
//! is exactly as strong as today's per-request check (spec §"Session establishment"). The handler
//! then only has to check it was *ordinary DPoP* (not an SK attestation, not a cert-bound Bearer,
//! not anonymous), bind the token, and mint the session.

use axum::body::Body;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use serde_json::json;
use sha2::{Digest as _, Sha256};

use crate::auth::VerifiedToken;
use crate::error::ServerError;

use super::derive::{confirm_value, derive_session_key, SessionKey};
use super::store::{CbFlavour, Session};
use super::{now_secs, ConnSk, SkSession, SkState, SESSION_ENDPOINT_PATH};

/// Router state for the `/.pop/session` routes: the shared session state + the server's standard
/// DPoP challenge (single-sourced from the verifier by `crate::app`).
#[derive(Clone)]
pub struct SkRouteState {
    pub sk: std::sync::Arc<SkState>,
    pub challenge: String,
}

impl SkRouteState {
    /// The uniform 401 for any establishment/termination authentication failure — the standard
    /// DPoP challenge, no profile-specific error code (spec §"Failure signalling").
    fn challenge_401(&self, message: &str) -> Response {
        ServerError::Unauthorized {
            status: 401,
            message: message.to_string(),
            www_authenticate: self.challenge.clone(),
        }
        .into_response()
    }
}

/// The establishment request body ceiling — the body is a tiny JSON object; anything bigger is
/// hostile or broken.
const MAX_ESTABLISH_BODY_BYTES: usize = 4096;

/// `POST /.pop/session` — establish a DPoP-SK session (spec §"Session establishment").
pub async fn establish_handler(
    State(st): State<SkRouteState>,
    req: axum::extract::Request,
) -> Response {
    let now = now_secs();
    let (parts, body) = req.into_parts();

    // The auth middleware ALWAYS inserts a VerifiedToken (public when anonymous). Establishment
    // requires a full-strength, ordinary-DPoP identity:
    let Some(token) = parts.extensions.get::<VerifiedToken>().cloned() else {
        return st.challenge_401("Authentication required.");
    };
    // NOT via an SK attestation (a session must not mint a session — establishment is "an
    // ordinary RFC 9449 request"; re-establishment likewise costs one real DPoP proof).
    if parts.extensions.get::<SkSession>().is_some() {
        return st.challenge_401("Session establishment requires an ordinary DPoP request.");
    }
    // NOT anonymous, and DPoP-bound (an RFC 8705 cert-bound Bearer has no cnf.jkt and is not the
    // establishment mechanism this profile defines).
    if token.web_id.is_none() {
        return st.challenge_401("Authentication required.");
    }
    let Some(jkt) = token.cnf_jkt.clone() else {
        return st.challenge_401("Session establishment requires a DPoP-bound access token.");
    };
    // The bound token's exp caps the session lifetime; a token without exp cannot bound it
    // (RFC 9068 requires exp, so this is unreachable for a verified at+jwt — fail closed anyway).
    let Some(token_exp) = token.expiry else {
        return st.challenge_401("Session establishment requires a token with an expiry.");
    };

    // The access token's bytes (for token_hash + ath). The verifier already authenticated this
    // exact header; we re-read it only to hash the token string.
    let Some(access_token) = parts
        .headers
        .get(http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(crate::auth::dpop_scheme_access_token)
        .map(str::to_string)
    else {
        return st.challenge_401("Session establishment requires a DPoP-scheme access token.");
    };
    let token_hash: [u8; 32] = Sha256::digest(access_token.as_bytes()).into();
    let ath = URL_SAFE_NO_PAD.encode(token_hash);

    let ttl = st.sk.max_ttl_secs().min(token_exp - now);
    if ttl <= 0 {
        return st.challenge_401("The access token is expired.");
    }

    // Body: `{"cb": "<flavour>"}` — the single flavour the client REQUIRES (a demand, not a
    // menu). Unknown JSON members are ignored (forward-compat); a missing/unknown `cb`, or one
    // the server cannot satisfy EXACTLY, is a 400 — never a substituted flavour (downgrade rule 4).
    let Ok(bytes) = axum::body::to_bytes(body, MAX_ESTABLISH_BODY_BYTES).await else {
        return ServerError::BadRequest("establishment body too large".into()).into_response();
    };
    let cb = match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(v) => match v.get("cb").and_then(|c| c.as_str()) {
            Some("none") => CbFlavour::None,
            Some("tls-exporter") => CbFlavour::TlsExporter,
            _ => {
                return ServerError::BadRequest("cb must be \"tls-exporter\" or \"none\"".into())
                    .into_response()
            }
        },
        Err(_) => {
            return ServerError::BadRequest("malformed establishment body".into()).into_response()
        }
    };

    // Mint the session id FIRST — it is the HKDF salt for the tls-exporter flavour.
    let Ok(session_id) = mint_session_id() else {
        // Maps to a non-leaky 500 (the OS entropy source failing is an operational fault).
        return ServerError::Storage("entropy source unavailable".into()).into_response();
    };

    let (key, conn_id, confirm): (SessionKey, Option<u64>, Option<String>) = match cb {
        CbFlavour::None => {
            let Ok(k) = SessionKey::random() else {
                return ServerError::Storage("entropy source unavailable".into()).into_response();
            };
            (k, None, None)
        }
        CbFlavour::TlsExporter => {
            // Offered only where this process terminates TLS 1.3 itself and the acceptor exported
            // keying material for THIS connection. Anything else is a 400 — never a silent
            // downgrade to cb=none (downgrade rule 4), never a derivation from other material.
            if !st.sk.exporter_available() {
                return ServerError::BadRequest(
                    "cb=tls-exporter is not available on this deployment".into(),
                )
                .into_response();
            }
            let Some(conn) = parts.extensions.get::<ConnSk>() else {
                return ServerError::BadRequest("cb=tls-exporter requires in-process TLS".into())
                    .into_response();
            };
            let Some(ekm) = conn.ekm.as_ref() else {
                // No EKM: not TLS 1.3, exporter failure, or early data — fail closed.
                return ServerError::BadRequest(
                    "cb=tls-exporter requires a TLS 1.3 connection".into(),
                )
                .into_response();
            };
            let k = derive_session_key(ekm, &session_id, &ath, &jkt);
            let confirm = confirm_value(&k, &session_id);
            (k, Some(conn.conn_id), Some(confirm))
        }
    };

    // cb=none: the one wire transfer of K (TLS-protected, no-store) — encode BEFORE the key moves
    // into the session record.
    let key_member = match cb {
        CbFlavour::None => Some(key.to_base64url()),
        CbFlavour::TlsExporter => None,
    };

    let session = Session::new(
        session_id.clone(),
        key,
        token_hash,
        jkt,
        token,
        token_exp,
        now + ttl,
        cb,
        conn_id,
    );
    if !st.sk.store().insert(session, now) {
        // A 128-bit CSPRNG collision — practically unreachable; refuse rather than overwrite
        // (spec: a session_id is never reused). Non-leaky 500.
        return ServerError::Storage("session id collision".into()).into_response();
    }

    let mut body = json!({
        "session_id": session_id,
        "cb": cb.as_str(),
        "alg": super::sig::ALG,
        "expires_in": ttl,
    });
    if let Some(k) = key_member {
        body["key"] = json!(k);
    }
    if let Some(c) = confirm {
        body["confirm"] = json!(c);
    }

    Response::builder()
        .status(http::StatusCode::CREATED)
        .header(http::header::CONTENT_TYPE, "application/json")
        // Normative: the establishment response MUST NOT be cached (it may carry K).
        .header(http::header::CACHE_CONTROL, "no-store")
        .body(Body::from(body.to_string()))
        .expect("static establishment response")
}

/// `DELETE /.pop/session` — client-requested session termination (spec §"Lifetime"). The request
/// MUST be attested UNDER THE SESSION BEING TERMINATED (the `keyid` names it); the auth
/// middleware has already verified that attestation like any other request, leaving an
/// [`SkSession`] extension. A non-attested DELETE (e.g. plain DPoP) names no session — 401.
pub async fn terminate_handler(
    State(st): State<SkRouteState>,
    req: axum::extract::Request,
) -> Response {
    let Some(sk_session) = req.extensions().get::<SkSession>() else {
        return st.challenge_401("Session termination must be attested under the session.");
    };
    st.sk.store().remove(&sk_session.session_id);
    http::StatusCode::NO_CONTENT.into_response()
}

/// Mint an opaque session identifier: 128 bits of CSPRNG output, base64url-no-pad (spec
/// §"Response": "MUST contain at least 128 bits of CSPRNG output").
fn mint_session_id() -> Result<String, getrandom::Error> {
    let mut id = [0u8; 16];
    getrandom::getrandom(&mut id)?;
    Ok(URL_SAFE_NO_PAD.encode(id))
}

/// Build the RFC 9728 protected-resource-metadata document (`/.well-known/oauth-protected-resource`).
///
/// Standard members: `dpop_bound_access_tokens_required: true` IS the Solid-OIDC DPoP mandate,
/// machine-readably; `dpop_signing_alg_values_supported` mirrors the verifier's policy allowlist
/// (the same set its `WWW-Authenticate` challenge advertises);
/// `tls_client_certificate_bound_access_tokens` advertises PoP Tier 1 (RFC 8705) when the mTLS
/// flag is on. The `pop_session` extension member (legal per RFC 9728 §2, ignored by
/// non-implementers per §3.2) advertises DPoP-SK when the tier is enabled; `tls-exporter` is
/// offered ONLY when this process terminates TLS itself (spec §"Discovery": a server "MUST NOT
/// advertise tls-exporter" behind a TLS-terminating intermediary).
pub fn protected_resource_metadata_json(
    base_url: &str,
    mtls_bound_tokens: bool,
    sk: Option<&SkState>,
) -> String {
    let base = base_url.trim_end_matches('/');
    let mut doc = json!({
        "resource": base,
        "dpop_bound_access_tokens_required": true,
        "dpop_signing_alg_values_supported": solid_oidc_verifier::SIGNING_ALGS,
        "tls_client_certificate_bound_access_tokens": mtls_bound_tokens,
    });
    if let Some(sk) = sk {
        let channel_bindings: Vec<&str> = if sk.exporter_available() {
            vec!["tls-exporter", "none"]
        } else {
            vec!["none"]
        };
        doc["pop_session"] = json!({
            "endpoint": format!("{base}{SESSION_ENDPOINT_PATH}"),
            "algs": [super::sig::ALG],
            "channel_bindings": channel_bindings,
            "profile": "https://w3id.org/jeswr/dpop-sk/v1",
        });
    }
    doc.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pop::sk::SkConfig;

    #[test]
    fn metadata_advertises_tiers_only_when_enabled() {
        // Neither tier: the baseline document still states the DPoP mandate.
        let doc: serde_json::Value = serde_json::from_str(&protected_resource_metadata_json(
            "https://pod.example/",
            false,
            None,
        ))
        .unwrap();
        assert_eq!(doc["resource"], "https://pod.example");
        assert_eq!(doc["dpop_bound_access_tokens_required"], true);
        assert_eq!(doc["tls_client_certificate_bound_access_tokens"], false);
        assert!(doc.get("pop_session").is_none());
        assert!(doc["dpop_signing_alg_values_supported"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a == "ES256"));

        // SK enabled without in-process TLS: only cb=none is offered (never tls-exporter across a
        // TLS-terminating proxy — the spec's MUST NOT).
        let sk = SkState::new(SkConfig::default());
        let doc: serde_json::Value = serde_json::from_str(&protected_resource_metadata_json(
            "https://pod.example",
            false,
            Some(&sk),
        ))
        .unwrap();
        let ps = &doc["pop_session"];
        assert_eq!(ps["endpoint"], "https://pod.example/.pop/session");
        assert_eq!(ps["algs"], json!(["hmac-sha256"]));
        assert_eq!(ps["channel_bindings"], json!(["none"]));
        assert_eq!(ps["profile"], "https://w3id.org/jeswr/dpop-sk/v1");

        // SK + in-process TLS: both flavours; mTLS flag surfaces the RFC 8705 member.
        let sk = SkState::new(SkConfig {
            exporter_available: true,
            ..SkConfig::default()
        });
        let doc: serde_json::Value = serde_json::from_str(&protected_resource_metadata_json(
            "https://pod.example",
            true,
            Some(&sk),
        ))
        .unwrap();
        assert_eq!(doc["tls_client_certificate_bound_access_tokens"], true);
        assert_eq!(
            doc["pop_session"]["channel_bindings"],
            json!(["tls-exporter", "none"])
        );
    }

    #[test]
    fn session_ids_are_distinct_and_urlsafe() {
        let a = mint_session_id().unwrap();
        let b = mint_session_id().unwrap();
        assert_ne!(a, b);
        assert_eq!(a.len(), 22); // 16 bytes, base64url-no-pad
        assert!(a
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'));
    }
}
