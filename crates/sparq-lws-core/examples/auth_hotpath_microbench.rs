// AUTHORED-BY Claude Opus 4.8
//! DETERMINISTIC in-process micro-benchmark of the AUTHENTICATED (DPoP) GET hot path, component by
//! component. It reproduces the per-component crypto/parse work `src/auth_cache.rs::verify_fresh_proof`
//! does on a cache HIT under the default policy (the production steady state) plus the cache-MISS
//! access-token verify, using REAL ES256 keys + REAL DPoP proofs minted the same way the verifier's
//! own tests + `examples/auth_load.rs` do. The summed budget is ADDITIVE (each step timed in isolation),
//! NOT the literal execution order of `verify_fresh_proof`.
//!
//! Why this and not only the HTTP sweep: on a contended box (load 25-33 here) the end-to-end RPS
//! wall-clock is noise. This times each crypto/parse step back-to-back in the SAME process on the SAME
//! loaded box, so the RELATIVE cost share between steps is robust to load (every step pays the same
//! contention tax). Each step is timed over N iterations and the per-op median + total are printed; the
//! relative shares are the trustworthy figure.
//!
//! Run: `cargo run --release --example auth_hotpath_microbench [-- <iters>]`.

use std::hint::black_box;
use std::time::Instant;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use p256::ecdsa::{signature::Signer, Signature, SigningKey};
use rand_core::OsRng;
use sha2::{Digest, Sha256};

use solid_oidc_verifier::jwk::Jwk;
use solid_oidc_verifier::jwt::{
    peek_claims, proof_has_ath, verify_proof_with_embedded_jwk, verify_signature,
};

struct ClientKey {
    signing: SigningKey,
    jwk_value: serde_json::Value,
    jkt: String,
}

fn new_client_key() -> ClientKey {
    let signing = SigningKey::random(&mut OsRng);
    let vk = signing.verifying_key();
    let point = vk.to_encoded_point(false);
    let x = URL_SAFE_NO_PAD.encode(point.x().unwrap());
    let y = URL_SAFE_NO_PAD.encode(point.y().unwrap());
    let jwk_value = serde_json::json!({"kty":"EC","crv":"P-256","x":x,"y":y});
    let canonical = format!(r#"{{"crv":"P-256","kty":"EC","x":"{x}","y":"{y}"}}"#);
    let jkt = URL_SAFE_NO_PAD.encode(Sha256::digest(canonical.as_bytes()));
    ClientKey {
        signing,
        jwk_value,
        jkt,
    }
}

fn b64url_json(v: &serde_json::Value) -> String {
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(v).unwrap())
}

/// Mint an ES256 compact JWS with the given header + claims, signed by `key`.
fn mint_jws(key: &SigningKey, header: serde_json::Value, claims: serde_json::Value) -> String {
    let signing_input = format!("{}.{}", b64url_json(&header), b64url_json(&claims));
    let sig: Signature = key.sign(signing_input.as_bytes());
    let sig_b64 = URL_SAFE_NO_PAD.encode(sig.to_bytes());
    format!("{signing_input}.{sig_b64}")
}

const URL: &str = "https://localhost:3000/alice/test/bench-private";
const METHOD: &str = "GET";

fn ath(token: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))
}

/// Mirror `auth_cache::ath_from_digest` — base64url of the ALREADY-computed SHA-256 token digest (the
/// cache key). The hit path reuses the cache-key digest here rather than re-hashing the token.
fn ath_from_digest(digest: &[u8; 32]) -> String {
    URL_SAFE_NO_PAD.encode(digest)
}

/// Time a closure `iters` times; returns (total_ns, per_op_ns).
fn time_it<F: FnMut()>(iters: u64, mut f: F) -> (u128, f64) {
    let start = Instant::now();
    for _ in 0..iters {
        f();
    }
    let total = start.elapsed().as_nanos();
    (total, total as f64 / iters as f64)
}

fn main() {
    let iters: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(200_000);

    // --- Build a realistic access token (the IdP-signed at+jwt) + its issuer JWK. ---
    let issuer_key = SigningKey::random(&mut OsRng);
    let ik_vk = issuer_key.verifying_key();
    let ik_point = ik_vk.to_encoded_point(false);
    let ik_x = URL_SAFE_NO_PAD.encode(ik_point.x().unwrap());
    let ik_y = URL_SAFE_NO_PAD.encode(ik_point.y().unwrap());
    let issuer_jwk: Jwk = serde_json::from_value(serde_json::json!({
        "kty":"EC","crv":"P-256","x":ik_x,"y":ik_y
    }))
    .unwrap();

    let client = new_client_key();
    let now = 1_000_000i64;

    // The at+jwt access token: RFC 9068 claims + cnf.jkt bound to the client key. This is the token a
    // cache MISS verifies (signature against the issuer JWK).
    let token_claims = serde_json::json!({
        "iss":"http://localhost:8080/realms/solid",
        "aud":["solid","https://localhost:3000"],
        "sub":"alice","jti":"tok-jti","client_id":"conformance-alice",
        "webid":"https://localhost:3000/alice/profile/card#me",
        "iat": now, "exp": now + 300,
        "cnf": {"jkt": client.jkt},
        "scope":"webid",
    });
    let access_token = mint_jws(
        &issuer_key,
        serde_json::json!({"typ":"at+jwt","alg":"ES256"}),
        token_claims,
    );
    let issuer_keys = vec![issuer_jwk];

    // A representative fresh DPoP proof (the hit path mints one per request; here we use a fixed one
    // and time the verify components — minting cost is the client's, not the server's).
    let proof = mint_jws(
        &client.signing,
        serde_json::json!({"typ":"dpop+jwt","alg":"ES256","jwk": client.jwk_value}),
        serde_json::json!({
            "htu": URL, "htm": METHOD, "jti": "proof-jti-fixed",
            "iat": now, "ath": ath(&access_token),
        }),
    );

    // The cache key is SHA-256(access_token), computed ONCE per request to look up the verified-token
    // cache. The real hit path (`auth_cache::verify_fresh_proof`) REUSES that same digest for the `ath`
    // comparison (`ath_from_digest` = base64url of the digest) — it does NOT re-hash the token. So the
    // bench hashes once (H7) and reuses the digest for ath (H3).
    let token_digest: [u8; 32] = Sha256::digest(access_token.as_bytes()).into();

    println!(
        "# auth hot-path microbench — iters={iters}, load={}",
        load_avg()
    );
    println!("# (per-op nanoseconds; relative %% of the summed hit-path crypto/parse budget)\n");

    // === Off-hit-path context: access-token signature verify (ES256 against issuer JWKS) ===
    // This is the work the verified-token CACHE removes on a hit. Timed for context (it is NOT on the
    // steady-state hit path).
    let (at_total, at_op) = time_it(iters, || {
        let claims = verify_signature(
            black_box(&access_token),
            black_box(&issuer_keys),
            Some("at+jwt"),
        )
        .expect("access token must verify");
        black_box(claims);
    });

    // === HIT-path components (run EVERY authenticated request, in order) ===

    // (H1) DPoP proof signature verify with embedded JWK (ES256). The one verify the cache can't remove.
    let (proof_total, proof_op) = time_it(iters, || {
        let (claims, jwk) =
            verify_proof_with_embedded_jwk(black_box(&proof), "dpop+jwt").expect("proof verifies");
        black_box((claims, jwk));
    });

    // (H2) RFC 7638 SHA-256 thumbprint of the proof's embedded JWK (cnf.jkt binding).
    let (claims, proof_jwk) = verify_proof_with_embedded_jwk(&proof, "dpop+jwt").unwrap();
    let (tp_total, tp_op) = time_it(iters, || {
        let t = black_box(&proof_jwk).thumbprint_sha256().unwrap();
        black_box(t);
    });

    // (H3) ath = base64url of the cache-key digest (`ath_from_digest`). The hit path reuses the digest
    //      computed for the cache key (H7) — NOT a second SHA-256 of the token — so this is just a
    //      base64url encode.
    let (ath_total, ath_op) = time_it(iters, || {
        let a = ath_from_digest(black_box(&token_digest));
        black_box(a);
    });

    // (C1) proof_has_ath peek (base64 + serde_json parse). OFF the default hit path: the cache calls it
    //      only in ath-compat mode (`allow_missing_ath && !proof_has_ath(proof)` short-circuits, default
    //      off), so it is excluded from the hit budget and timed for context only.
    let (peekath_total, peekath_op) = time_it(iters, || {
        let b = proof_has_ath(black_box(&proof));
        black_box(b);
    });

    // (C2) peek_claims of the proof (base64 + serde_json parse). OFF the hit path: the hit path reads
    //      jti/htm/htu/iat from the already-decoded `claims` returned by H1 and never re-peeks. Timed
    //      for context only (sizes a redundant JSON-parse), excluded from the hit budget.
    let (peek_total, peek_op) = time_it(iters, || {
        let c = peek_claims(black_box(&proof));
        black_box(c);
    });

    // (H6) the claim field-extraction the hit path does over the already-decoded `claims` map
    //      (htm/htu/jti/iat/ath gets + the normalize_htu URL parse). Times the non-crypto orchestration.
    let (fields_total, fields_op) = time_it(iters, || {
        let htm = claims.get("htm").and_then(|v| v.as_str()).unwrap_or("");
        let htu = claims.get("htu").and_then(|v| v.as_str()).unwrap_or("");
        let jti = claims.get("jti").and_then(|v| v.as_str()).unwrap_or("");
        let iat = claims.get("iat").and_then(|v| v.as_i64()).unwrap_or(0);
        let a = claims.get("ath").and_then(|v| v.as_str()).unwrap_or("");
        // the htu normalize (url::Url::parse) the hit path runs on BOTH the proof htu and the req url.
        let nh1 = normalize_htu(black_box(htu));
        let nh2 = normalize_htu(black_box(URL));
        black_box((htm, jti, iat, a, nh1, nh2));
    });

    // (H7) SHA-256 of the access token for the CACHE KEY (auth_cache::key, per request to look up).
    let (key_total, key_op) = time_it(iters, || {
        let k: [u8; 32] = Sha256::digest(black_box(access_token.as_bytes())).into();
        black_box(k);
    });

    // The summed HIT-path crypto+parse budget — what a cache hit ACTUALLY pays under the DEFAULT policy
    // (`allow_missing_ath = false`), as an ADDITIVE total (NOT execution order — the real order is
    // H1 → H6 field/htu checks → H3 ath → H2 thumbprint): H1 proof verify, H2 cnf.jkt thumbprint, H3 ath
    // (b64url of the reused digest), H6 claim-field gets + htu normalise, H7 the one token hash for the
    // cache key. C1/C2 are EXCLUDED — C1 (proof_has_ath) runs only when `allow_missing_ath` is enabled,
    // and C2 (peek_claims) is a redundant re-parse the hit path never does.
    let hit_budget = proof_op + tp_op + ath_op + fields_op + key_op;

    let row = |name: &str, op: f64, total: u128| {
        println!(
            "{:<46} {:>10.1} ns/op   {:>6.2}%   (total {:.2} ms)",
            name,
            op,
            100.0 * op / hit_budget,
            total as f64 / 1e6
        );
    };

    println!("--- HIT-path components (paid EVERY authed request) ---");
    row(
        "H1 DPoP proof ES256 verify (embedded JWK)",
        proof_op,
        proof_total,
    );
    row("H2 RFC7638 thumbprint SHA-256 (cnf.jkt)", tp_op, tp_total);
    row("H3 ath = b64url(cache-key digest)", ath_op, ath_total);
    row(
        "H6 claim field gets + normalize_htu (2x url parse)",
        fields_op,
        fields_total,
    );
    row("H7 SHA-256 token -> cache key", key_op, key_total);
    println!(
        "{:<46} {:>10.1} ns/op   100.00%",
        "= HIT crypto+parse budget (H1+H2+H3+H6+H7)", hit_budget
    );

    println!("\n--- Off-hit-path context (NOT in the hit budget) ---");
    println!(
        "{:<46} {:>10.1} ns/op   (ath-compat-only; total {:.2} ms)",
        "C1 proof_has_ath peek (b64+json parse)",
        peekath_op,
        peekath_total as f64 / 1e6
    );
    println!(
        "{:<46} {:>10.1} ns/op   (redundant re-parse; total {:.2} ms)",
        "C2 peek_claims proof (b64+json parse)",
        peek_op,
        peek_total as f64 / 1e6
    );
    println!(
        "{:<46} {:>10.1} ns/op   ({:.2}x the proof verify; total {:.2} ms)",
        "M  access-token ES256 verify (miss-only)",
        at_op,
        at_op / proof_op,
        at_total as f64 / 1e6
    );

    println!(
        "\n# Interpretation: on a cache HIT the server pays ~{:.1} ns of crypto+parse (H1+H2+H3+H6+H7);\n\
         # the two ES256 verifies (H1 proof + M token) are {:.1} + {:.1} ns. The token verify (M) is the\n\
         # part the cache removes on a hit, so a hit roughly HALVES the asymmetric crypto. The thumbprint\n\
         # (H2) is a second SHA-256-based step on the hit path; the token is hashed once (H7) and its\n\
         # digest is reused for ath (H3), not re-hashed. C1/C2 are off the default hit path.",
        hit_budget, proof_op, at_op
    );
}

/// Mirror the verifier/auth_cache normalize_htu (url parse + strip query/fragment).
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

fn load_avg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next().map(str::to_string))
        .unwrap_or_else(|| "n/a(macos)".to_string())
}
