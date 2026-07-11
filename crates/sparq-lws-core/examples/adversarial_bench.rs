// AUTHORED-BY Claude Opus 4.8
//! ADVERSARIAL benchmark suite — validates that the server's PROTECTIONS hold UNDER LOAD, and
//! measures the (advisory) timing side-channels the charter's perf-gate note flags.
//!
//! Drives the assembled router over the in-memory doubles (production auth posture: verified-token
//! cache ON) with hostile traffic and asserts the security INVARIANTS deterministically, while
//! reporting the timing distributions ADVISORY-only. Arms:
//!  - `existence_nondisclosure` — an authenticated FOREIGN reader (no ACL grant) GETs an EXISTING
//!    forbidden resource vs a NON-EXISTENT one. Deterministic invariant: the two denial STATUSES are
//!    identical (existence is not disclosed). Advisory: the two latency distributions (a gross
//!    divergence is a timing side-channel — measured, reported, never gated).
//!  - `replay_storm` — the SAME DPoP proof (fixed jti) replayed. Deterministic: at most ONE accept,
//!    the rest replay-REJECTED (401). Advisory: reject latency.
//!  - `jti_churn` — many FRESH jtis against one token. Deterministic: all accepted (the replay store
//!    keeps accepting fresh). Advisory: per-op latency as the store grows (detects pathological cost).
//!  - `cache_bust` — a DISTINCT valid token per request (every request a cache MISS → full verify).
//!    Deterministic: all valid tokens still authorize AND a distinct forged-ISSUER token per request
//!    is still rejected (busting the cache cannot leak a forgery). Advisory: the miss-vs-hit
//!    throughput ratio (the amplification the pre-crypto rate-limiter defends — per-verify cost, not
//!    hard-coded).
//!  - `bogus_proof` / `bogus_token` — garbage credentials. Deterministic: NEVER a 200 (always 401).
//!  - `post_attack_invariants` — after the flood, re-exec WAC on the LIVE server: the foreign reader
//!    is STILL denied, the owner STILL authorized (the attack corrupted nothing).
//!
//! Run: `cargo run --release --example adversarial_bench -- \
//!         --requests 500 --concurrency 32 --out bench/results/adversarial/adversarial-report.json`
//! (see `bench/run-adversarial.sh`). The strict invariants also run as `cargo test --test
//! adversarial_invariants`; this binary adds the under-load timing measurement + the JSON report.

use std::sync::Arc;

use axum::body::Bytes;
use serde_json::{json, Value};

#[path = "support/mod.rs"]
mod support;
use support::*;

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

const SMALL_TURTLE: &str =
    "<https://pod.example/alice/small#me> <http://xmlns.com/foaf/0.1/name> \"Alice\" .";

fn status_map_json(m: &std::collections::BTreeMap<u16, u64>) -> Value {
    let obj: serde_json::Map<String, Value> =
        m.iter().map(|(k, v)| (k.to_string(), json!(v))).collect();
    Value::Object(obj)
}

fn percentiles_json(sorted: &[u64]) -> Value {
    let p = percentiles(sorted);
    json!({ "p50": p.p50, "p90": p.p90, "p99": p.p99, "p999": p.p999, "max": p.max })
}

fn main() {
    let args = parse_args();
    let requests: usize = args
        .get("requests")
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);
    let concurrency: usize = args
        .get("concurrency")
        .and_then(|s| s.parse().ok())
        .unwrap_or(32);
    let out = args
        .get("out")
        .cloned()
        .unwrap_or_else(|| "bench/results/adversarial/adversarial-report.json".to_string());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("mt runtime");

    let issuer_key = BenchKey::generate();
    let client_key = BenchKey::generate();

    let store = make_store();
    rt.block_on(async {
        seed_owner_root_acl(&store, WEBID).await;
        seed_resource(
            &store,
            "https://pod.example/alice/small",
            SMALL_TURTLE,
            "text/turtle",
        )
        .await;
    });
    let app = assemble_app(store, &issuer_key, 512);

    let owner_token = mint_access_token(&issuer_key, &client_key.thumbprint);
    let foreign_token = mint_access_token_webid(&issuer_key, &client_key.thumbprint, FOREIGN_WEBID);

    let mut arms: Vec<Value> = Vec::new();

    // ---------------------------------------------------------------------------------------------
    // 1) existence non-disclosure — foreign reader, existing-forbidden vs nonexistent-forbidden.
    // ---------------------------------------------------------------------------------------------
    {
        let existing: Vec<PreReq> = (0..requests)
            .map(|_| {
                authed_prereq(
                    &client_key,
                    &foreign_token,
                    "GET",
                    "/alice/small",
                    None,
                    &[],
                    Bytes::new(),
                )
            })
            .collect();
        let ghost: Vec<PreReq> = (0..requests)
            .map(|_| {
                authed_prereq(
                    &client_key,
                    &foreign_token,
                    "GET",
                    "/alice/ghost-does-not-exist",
                    None,
                    &[],
                    Bytes::new(),
                )
            })
            .collect();
        let r_ex = run_pool_detailed(&rt, &app, Arc::new(existing), concurrency);
        let r_gh = run_pool_detailed(&rt, &app, Arc::new(ghost), concurrency);

        // Report the sole status ONLY when the path returned exactly one status; otherwise `0` (a
        // "mixed — see histogram" sentinel) so we never present an arbitrary lowest key as "the"
        // status. The full histograms are emitted below regardless.
        let single_ex = r_ex.status_counts.len() == 1;
        let single_gh = r_gh.status_counts.len() == 1;
        let status_ex = if single_ex {
            r_ex.status_counts.keys().next().copied().unwrap_or(0)
        } else {
            0
        };
        let status_gh = if single_gh {
            r_gh.status_counts.keys().next().copied().unwrap_or(0)
        } else {
            0
        };
        let consistent = single_ex && single_gh && status_ex == status_gh;
        // Never a 200 — a foreign reader must NEVER get the resource.
        let never_disclosed = !r_ex.status_counts.contains_key(&200);

        // Advisory: relative median-latency divergence between the two paths.
        let med_ex = percentiles(&r_ex.latencies_us).p50 as f64;
        let med_gh = percentiles(&r_gh.latencies_us).p50 as f64;
        let ratio = if med_gh > 0.0 {
            round2_pub(med_ex / med_gh)
        } else {
            0.0
        };

        arms.push(json!({
            "name": "existence_nondisclosure",
            "description": "Foreign authenticated reader: existing-forbidden vs nonexistent path.",
            "deterministic": {
                "mode": "deterministic",
                "existing_status": status_ex,
                "nonexistent_status": status_gh,
                "statuses_consistent": consistent,
                "never_disclosed_200": never_disclosed,
                "existing_status_histogram": status_map_json(&r_ex.status_counts),
                "nonexistent_status_histogram": status_map_json(&r_gh.status_counts),
                "invariant_holds": consistent && never_disclosed
            },
            "timing_advisory": {
                "mode": "timing_advisory",
                "disclaimer": TIMING_DISCLAIMER,
                "concurrency": concurrency,
                "existing_latency_us": percentiles_json(&r_ex.latencies_us),
                "nonexistent_latency_us": percentiles_json(&r_gh.latencies_us),
                "median_ratio_existing_over_nonexistent": ratio,
                "note": "A ratio grossly != 1.0 hints at a timing side-channel; advisory only."
            }
        }));
    }

    // ---------------------------------------------------------------------------------------------
    // 2) replay_storm — one fixed-jti proof replayed (concurrency 1 for a clean at-most-one-accept).
    // ---------------------------------------------------------------------------------------------
    {
        let n = requests.max(2);
        let fixed_jti = next_jti();
        let htu = format!("{BASE_URL}/alice/small");
        let proof = mint_dpop_proof_fixed_jti(&client_key, "GET", &htu, &owner_token, &fixed_jti);
        let one = PreReq {
            method: "GET".to_string(),
            path: "/alice/small".to_string(),
            authz: Some(format!("DPoP {owner_token}")),
            dpop: Some(proof),
            content_type: None,
            extra: Vec::new(),
            body: Bytes::new(),
        };
        let pool: Vec<PreReq> = (0..n).map(|_| one.clone()).collect();
        let r = run_pool_detailed(&rt, &app, Arc::new(pool), 1);
        let accepted = *r.status_counts.get(&200).unwrap_or(&0);
        let rejected_401 = *r.status_counts.get(&401).unwrap_or(&0);
        let invariant = accepted <= 1 && rejected_401 >= (n as u64 - 1);

        arms.push(json!({
            "name": "replay_storm",
            "description": "Same DPoP proof (fixed jti) replayed N times; expect at most 1 accept.",
            "deterministic": {
                "mode": "deterministic",
                "replays": n,
                "accepted_200": accepted,
                "rejected_401": rejected_401,
                "status_histogram": status_map_json(&r.status_counts),
                "invariant_holds": invariant
            },
            "timing_advisory": {
                "mode": "timing_advisory",
                "disclaimer": TIMING_DISCLAIMER,
                "concurrency": 1,
                "reject_latency_us": percentiles_json(&r.latencies_us),
                "throughput_rps": r.throughput_rps
            }
        }));
    }

    // ---------------------------------------------------------------------------------------------
    // 3) jti_churn — many FRESH jtis, one token; all accepted, store grows.
    // ---------------------------------------------------------------------------------------------
    {
        let pool: Vec<PreReq> = (0..requests)
            .map(|_| {
                authed_prereq(
                    &client_key,
                    &owner_token,
                    "GET",
                    "/alice/small",
                    None,
                    &[],
                    Bytes::new(),
                )
            })
            .collect();
        let r = run_pool_detailed(&rt, &app, Arc::new(pool), concurrency);
        let accepted = *r.status_counts.get(&200).unwrap_or(&0);
        let invariant = accepted == r.requests && r.status_counts.len() == 1;
        arms.push(json!({
            "name": "jti_churn",
            "description": "Distinct fresh jtis against one token; replay store must keep accepting.",
            "deterministic": {
                "mode": "deterministic",
                "fresh_jtis": r.requests,
                "accepted_200": accepted,
                "status_histogram": status_map_json(&r.status_counts),
                "invariant_holds": invariant
            },
            "timing_advisory": {
                "mode": "timing_advisory",
                "disclaimer": TIMING_DISCLAIMER,
                "concurrency": concurrency,
                "latency_us": percentiles_json(&r.latencies_us),
                "throughput_rps": r.throughput_rps
            }
        }));
    }

    // ---------------------------------------------------------------------------------------------
    // 4) cache_bust — a DISTINCT valid token per request (all misses) vs the cached steady state.
    // ---------------------------------------------------------------------------------------------
    {
        let cold: Vec<PreReq> = (0..requests)
            .map(|_| {
                let t = mint_access_token(&issuer_key, &client_key.thumbprint);
                authed_prereq(
                    &client_key,
                    &t,
                    "GET",
                    "/alice/small",
                    None,
                    &[],
                    Bytes::new(),
                )
            })
            .collect();
        let warm: Vec<PreReq> = (0..requests)
            .map(|_| {
                authed_prereq(
                    &client_key,
                    &owner_token,
                    "GET",
                    "/alice/small",
                    None,
                    &[],
                    Bytes::new(),
                )
            })
            .collect();
        // A well-formed token signed by an UNTRUSTED issuer key (iss=ISSUER but the signature does not
        // verify against the JWKS) — every request a distinct cache-miss, all of which MUST be rejected.
        // This proves busting the cache cannot make a forged token slip through under load.
        let attacker_issuer = BenchKey::generate();
        let forged: Vec<PreReq> = (0..requests)
            .map(|_| {
                let t = mint_access_token(&attacker_issuer, &client_key.thumbprint);
                authed_prereq(
                    &client_key,
                    &t,
                    "GET",
                    "/alice/small",
                    None,
                    &[],
                    Bytes::new(),
                )
            })
            .collect();
        let r_cold = run_pool_detailed(&rt, &app, Arc::new(cold), concurrency);
        let r_warm = run_pool_detailed(&rt, &app, Arc::new(warm), concurrency);
        let r_forged = run_pool_detailed(&rt, &app, Arc::new(forged), concurrency);
        let cold_ok = *r_cold.status_counts.get(&200).unwrap_or(&0) == r_cold.requests;
        let warm_ok = *r_warm.status_counts.get(&200).unwrap_or(&0) == r_warm.requests;
        let forged_rejected = !r_forged.status_counts.contains_key(&200);
        let amplification = if r_cold.throughput_rps > 0.0 {
            round2_pub(r_warm.throughput_rps / r_cold.throughput_rps)
        } else {
            0.0
        };
        arms.push(json!({
            "name": "cache_bust",
            "description": "Distinct valid token per request (cache miss) vs reused token (hit); a \
                            forged-issuer token flood must still be rejected.",
            "deterministic": {
                "mode": "deterministic",
                "cold_all_authorized": cold_ok,
                "warm_all_authorized": warm_ok,
                "forged_issuer_rejected": forged_rejected,
                "forged_status_histogram": status_map_json(&r_forged.status_counts),
                "invariant_holds": cold_ok && warm_ok && forged_rejected
            },
            "timing_advisory": {
                "mode": "timing_advisory",
                "disclaimer": TIMING_DISCLAIMER,
                "concurrency": concurrency,
                "cold_miss_throughput_rps": r_cold.throughput_rps,
                "warm_hit_throughput_rps": r_warm.throughput_rps,
                "hit_over_miss_speedup": amplification,
                "cold_latency_us": percentiles_json(&r_cold.latencies_us),
                "warm_latency_us": percentiles_json(&r_warm.latencies_us),
                "note": "The hit/miss gap is the amplification the pre-crypto rate-limiter defends; \
                         per-verify cost is measured here, not hard-coded."
            }
        }));
    }

    // ---------------------------------------------------------------------------------------------
    // 5) bogus_proof / bogus_token — garbage credentials must NEVER yield a 200.
    // ---------------------------------------------------------------------------------------------
    for (name, authz, dpop) in [
        (
            "bogus_proof",
            Some(format!("DPoP {owner_token}")),
            Some("this.is.not-a-valid-dpop-proof".to_string()),
        ),
        (
            "bogus_token",
            Some("DPoP not-a-real-jwt".to_string()),
            Some("this.is.not-a-valid-dpop-proof".to_string()),
        ),
    ] {
        let pool: Vec<PreReq> = (0..requests)
            .map(|_| PreReq {
                method: "GET".to_string(),
                path: "/alice/small".to_string(),
                authz: authz.clone(),
                dpop: dpop.clone(),
                content_type: None,
                extra: Vec::new(),
                body: Bytes::new(),
            })
            .collect();
        let r = run_pool_detailed(&rt, &app, Arc::new(pool), concurrency);
        let never_200 = !r.status_counts.contains_key(&200);
        arms.push(json!({
            "name": name,
            "description": "Malformed credentials flood; must be rejected, never authorized.",
            "deterministic": {
                "mode": "deterministic",
                "never_authorized_200": never_200,
                "status_histogram": status_map_json(&r.status_counts),
                "invariant_holds": never_200
            },
            "timing_advisory": {
                "mode": "timing_advisory",
                "disclaimer": TIMING_DISCLAIMER,
                "concurrency": concurrency,
                "reject_latency_us": percentiles_json(&r.latencies_us),
                "throughput_rps": r.throughput_rps
            }
        }));
    }

    // ---------------------------------------------------------------------------------------------
    // 6) post_attack_invariants — re-exec WAC on the LIVE server after the flood.
    // ---------------------------------------------------------------------------------------------
    let post = rt.block_on(async {
        use tower::ServiceExt;
        let owner_req = authed_prereq(
            &client_key,
            &owner_token,
            "GET",
            "/alice/small",
            None,
            &[],
            Bytes::new(),
        );
        let foreign_req = authed_prereq(
            &client_key,
            &foreign_token,
            "GET",
            "/alice/small",
            None,
            &[],
            Bytes::new(),
        );
        let owner_status = app
            .clone()
            .oneshot(owner_req.to_request())
            .await
            .unwrap()
            .status()
            .as_u16();
        let foreign_status = app
            .clone()
            .oneshot(foreign_req.to_request())
            .await
            .unwrap()
            .status()
            .as_u16();
        (owner_status, foreign_status)
    });
    let post_ok = post.0 == 200 && post.1 != 200;
    arms.push(json!({
        "name": "post_attack_invariants",
        "description": "After the adversarial flood, WAC still holds on the live server.",
        "deterministic": {
            "mode": "deterministic",
            "owner_read_status": post.0,
            "foreign_read_status": post.1,
            "owner_still_authorized": post.0 == 200,
            "foreign_still_denied": post.1 != 200,
            "invariant_holds": post_ok
        }
    }));

    let all_hold = arms.iter().all(|a| {
        a.get("deterministic")
            .and_then(|d| d.get("invariant_holds"))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    });

    let report = json!({
        "harness": "solid-server-rs adversarial_bench",
        "generated_unix": generated_unix(),
        "build_profile": build_profile(),
        "driver": "in-process tower::Service oneshot (no socket/TLS); in-memory test-double backends",
        "notes": "Deterministic invariant_holds flags are strict; timing blocks are ADVISORY \
                  (never a merge gate). Strict invariants also run as `cargo test --test \
                  adversarial_invariants`. See bench/ADVERSARIAL-BENCH.md.",
        "all_invariants_hold": all_hold,
        "arms": arms
    });

    // Human summary.
    println!(
        "solid-server-rs adversarial_bench  ({} build)",
        build_profile()
    );
    println!("{}", "-".repeat(88));
    for a in report["arms"].as_array().unwrap() {
        let holds = a["deterministic"]["invariant_holds"]
            .as_bool()
            .unwrap_or(false);
        println!(
            "  [{}] {:<26} invariant_holds={}",
            if holds { "PASS" } else { "FAIL" },
            a["name"].as_str().unwrap_or("?"),
            holds
        );
    }
    println!("{}", "-".repeat(88));
    println!("all_invariants_hold = {all_hold}");

    write_json(&out, &report);
    println!("wrote report -> {out}");

    if !all_hold {
        eprintln!("FAIL: at least one adversarial invariant did NOT hold — see the report.");
        std::process::exit(1);
    }
}

fn round2_pub(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}
