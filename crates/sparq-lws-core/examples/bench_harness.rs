// AUTHORED-BY Claude Opus 4.8
//! LOCAL performance benchmark harness for the EXPERIMENTAL solid-server-rs.
//!
//! Boots the assembled router over the IN-MEMORY test-double backends (no docker / Keycloak / S3 /
//! SPARQ) with the production auth posture (verified-token cache ON), then drives concurrent load
//! over the hot paths through the FULL request stack (auth verify → WAC ACL resolve → store →
//! content handling), via an in-process `tower::Service` `oneshot` driver. It measures, PER SCENARIO
//! at MULTIPLE concurrency levels:
//!  - DETERMINISTIC (strict/comparable): HTTP status, response byte length, and — through a counting
//!    global allocator — allocations + bytes allocated for ONE request in isolation; and
//!  - TIMING (ADVISORY): throughput (req/s) + p50/p90/p99/p999/max latency under concurrency. Marked
//!    advisory in the report — wall-clock variance makes them un-gateable (PSS charter perf-gate rule).
//!
//! Client-side ES256 proof signing happens OUTSIDE the timed window (requests are pre-built), so the
//! timing reflects the SERVER, not the load client's crypto.
//!
//! ## Scenarios (the hot paths the conformance+perf plan calls out)
//!  - `authed_get_cached`  — DPoP-authed GET of a small private resource, verified-token cache HIT
//!    (the production steady state: token sig NOT re-verified, fresh proof + jti + cnf.jkt ARE).
//!  - `authed_get_cold`    — same, but a DISTINCT access token per request (cache MISS: full verify).
//!  - `public_get`         — anonymous GET of a public resource (the pre-crypto public-read fast path).
//!  - `container_listing`  — authed GET of a container with N children (the ldp:contains listing).
//!  - `put_create`         — authed PUT creating a fresh resource (unique path per request).
//!  - `conditional_put_412`— authed PUT with `If-None-Match: *` on an existing resource → 412 (the
//!    write-path precondition; non-mutating + race-free). This server applies conditional
//!    preconditions to MUTATIONS only — a GET carrying `If-None-Match` is NOT turned into a 304 (see
//!    `src/ldp/conditional.rs`), so benchmarking a "GET-304" would misrepresent server behaviour; the
//!    GET-conditional-304 optimization is a documented server gap / follow-up.
//!
//! Run: `cargo run --release --example bench_harness -- \
//!         --requests 600 --concurrencies 1,8,32,64 --out bench/results/harness/bench-report.json`
//! (see `bench/run-bench.sh`). The report is machine-readable JSON; no perf numbers live in markdown.

use std::sync::Arc;

use axum::body::Bytes;

#[path = "support/mod.rs"]
mod support;
use support::*;

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

const SMALL_TURTLE: &str =
    "<https://pod.example/alice/small#me> <http://xmlns.com/foaf/0.1/name> \"Alice\" .";
const PUT_BODY: &str = "<#it> <http://xmlns.com/foaf/0.1/name> \"created\" .";
const CHILD_COUNT: usize = 25;

fn parse_concurrencies(s: &str) -> Vec<usize> {
    let v: Vec<usize> = s
        .split(',')
        .filter_map(|p| p.trim().parse::<usize>().ok())
        .filter(|&c| c > 0)
        .collect();
    if v.is_empty() {
        vec![1, 8, 32]
    } else {
        v
    }
}

fn main() {
    let args = parse_args();
    let requests: usize = args
        .get("requests")
        .and_then(|s| s.parse().ok())
        .unwrap_or(600);
    let probe_iters: u32 = args
        .get("probe-iters")
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);
    let concurrencies = parse_concurrencies(
        args.get("concurrencies")
            .map(String::as_str)
            .unwrap_or("1,8,32,64"),
    );
    let out = args
        .get("out")
        .cloned()
        .unwrap_or_else(|| "bench/results/harness/bench-report.json".to_string());

    // Multi-thread runtime for setup + the timing sweep. It stays IDLE (workers parked, not
    // allocating) during the deterministic probes, which each spin their own single-threaded runtime.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("mt runtime");

    let issuer_key = BenchKey::generate();
    let client_key = BenchKey::generate();

    // --- Seed fixtures on the store, then assemble the app. ---
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
        // public resource + a public-read ACL
        seed_resource(
            &store,
            "https://pod.example/alice/pub",
            SMALL_TURTLE,
            "text/turtle",
        )
        .await;
        seed_public_read_acl(&store, "https://pod.example/alice/pub").await;
        // a container with children (for the listing scenario). The children must be seeded through
        // the CONTAINMENT path (create_in_container) so `ldp:contains` membership is recorded — a
        // plain write would leave the container empty and the listing scenario would benchmark an
        // empty listing.
        seed_resource(&store, "https://pod.example/alice/box/", "", "text/turtle").await;
        for i in 0..CHILD_COUNT {
            seed_child(
                &store,
                "https://pod.example/alice/box/",
                &format!("https://pod.example/alice/box/c{i}"),
                SMALL_TURTLE,
                "text/turtle",
            )
            .await;
        }
    });
    let app = assemble_app(store, &issuer_key, /* cache_capacity */ 512);

    // A single owner token reused for the cached / listing / conditional / put scenarios.
    let owner_token = mint_access_token(&issuer_key, &client_key.thumbprint);

    let mut put_counter: usize = 0;

    let scenarios = vec![
        // ---- authed_get_cached ----
        run_scenario(
            &rt,
            &app,
            "authed_get_cached",
            "DPoP-authed GET of a small private resource; verified-token cache HIT (steady state).",
            &concurrencies,
            requests,
            probe_iters,
            200,
            &mut || {
                authed_prereq(
                    &client_key,
                    &owner_token,
                    "GET",
                    "/alice/small",
                    None,
                    &[],
                    Bytes::new(),
                )
            },
        ),
        // ---- authed_get_cold ----
        run_scenario(
            &rt,
            &app,
            "authed_get_cold",
            "DPoP-authed GET, DISTINCT access token per request (cache MISS: full token verify).",
            &concurrencies,
            requests,
            probe_iters,
            200,
            &mut || {
                let fresh = mint_access_token(&issuer_key, &client_key.thumbprint);
                authed_prereq(
                    &client_key,
                    &fresh,
                    "GET",
                    "/alice/small",
                    None,
                    &[],
                    Bytes::new(),
                )
            },
        ),
        // ---- public_get ----
        run_scenario(
            &rt,
            &app,
            "public_get",
            "Anonymous GET of a public resource (exercises the pre-crypto public-read fast path).",
            &concurrencies,
            requests,
            probe_iters,
            200,
            &mut || anon_prereq("GET", "/alice/pub"),
        ),
        // ---- container_listing ----
        run_scenario(
            &rt,
            &app,
            "container_listing",
            &format!(
                "Authed GET of a container with {CHILD_COUNT} children (ldp:contains listing)."
            ),
            &concurrencies,
            requests,
            probe_iters,
            200,
            &mut || {
                authed_prereq(
                    &client_key,
                    &owner_token,
                    "GET",
                    "/alice/box/",
                    None,
                    &[],
                    Bytes::new(),
                )
            },
        ),
        // ---- put_create ----
        run_scenario(
            &rt,
            &app,
            "put_create",
            "Authed PUT creating a fresh resource at a unique path per request (201 Created).",
            &concurrencies,
            requests,
            probe_iters,
            201,
            &mut || {
                let path = format!("/alice/load/put-{put_counter}");
                put_counter += 1;
                authed_prereq(
                    &client_key,
                    &owner_token,
                    "PUT",
                    &path,
                    Some("text/turtle"),
                    &[],
                    Bytes::from_static(PUT_BODY.as_bytes()),
                )
            },
        ),
        // ---- conditional_put_412 ----
        // The server applies conditional preconditions to MUTATIONS only (a GET carrying
        // If-None-Match is NOT turned into a 304 — see src/ldp/conditional.rs; that GET-304
        // optimization is a documented server gap → a follow-up bead). So the honest conditional
        // hot-path scenario is the WRITE precondition: `If-None-Match: *` (the no-overwrite create
        // guard) against an EXISTING resource → 412. It is non-mutating (the precondition
        // short-circuits before any write) and therefore race-free and idempotent under any
        // concurrency, exercising the etag/existence precondition-evaluation path.
        run_scenario(
            &rt,
            &app,
            "conditional_put_412",
            "Authed PUT with If-None-Match:* on an existing resource → 412 (write-path precondition).",
            &concurrencies,
            requests,
            probe_iters,
            412,
            &mut || {
                authed_prereq(
                    &client_key,
                    &owner_token,
                    "PUT",
                    "/alice/small",
                    Some("text/turtle"),
                    &[("if-none-match", "*")],
                    Bytes::from_static(PUT_BODY.as_bytes()),
                )
            },
        ),
    ];

    let report = Report {
        harness: "solid-server-rs bench_harness".to_string(),
        generated_unix: generated_unix(),
        build_profile: build_profile(),
        driver: "in-process tower::Service oneshot (no socket/TLS); in-memory test-double backends"
            .to_string(),
        notes:
            "Deterministic metrics (status/response_bytes/alloc_*) are strict/comparable; timing \
                metrics are ADVISORY (wall-clock; never a merge gate). See bench/HARNESS.md."
                .to_string(),
        scenarios,
    };

    print_summary(&report);
    write_json(&out, &report);
    println!("\nwrote report -> {out}");
    if report.build_profile == "debug" {
        eprintln!(
            "note: built in DEBUG. Timing is meaningless in debug — run with --release for the \
             advisory numbers (the deterministic block is still valid)."
        );
    }
}

/// Build one scenario: the deterministic probe (one isolated request) + the timing sweep across all
/// concurrency levels. `mk` produces a fresh pre-built request each call (unique jti/path/token as the
/// scenario needs).
#[allow(clippy::too_many_arguments)]
fn run_scenario(
    rt: &tokio::runtime::Runtime,
    app: &axum::Router,
    name: &str,
    description: &str,
    concurrencies: &[usize],
    requests: usize,
    probe_iters: u32,
    expected_status: u16,
    mk: &mut dyn FnMut() -> PreReq,
) -> Scenario {
    // Deterministic probe: `mk` mints a FRESH request per iteration (unique jti/path), so the measured
    // allocation path is the real hot path — not a replay-reject.
    let deterministic = deterministic_probe(app, mk, probe_iters);
    if deterministic.status != expected_status {
        eprintln!(
            "warning: scenario '{name}' deterministic probe returned status {} (expected {expected_status}) \
             — the measured allocation figures reflect that path",
            deterministic.status
        );
    }

    // Timing sweep: a fresh pool of pre-built requests per concurrency level (so put_create paths and
    // per-request jtis never repeat within or across levels).
    let mut levels = Vec::with_capacity(concurrencies.len());
    for &c in concurrencies {
        let pool: Vec<PreReq> = (0..requests).map(|_| mk()).collect();
        let level = timing_sweep(rt, app, Arc::new(pool), c, expected_status);
        levels.push(level);
    }

    Scenario {
        name: name.to_string(),
        description: description.to_string(),
        deterministic,
        timing_advisory: TimingBlock::new(levels),
    }
}

fn print_summary(report: &Report) {
    println!(
        "solid-server-rs bench_harness  ({} build, {} driver)",
        report.build_profile, report.driver
    );
    println!("{}", "-".repeat(96));
    for s in &report.scenarios {
        let d = &s.deterministic;
        println!(
            "{:<22} DETERMINISTIC status={} resp_bytes={} allocs/op={} alloc_bytes/op={}",
            s.name, d.status, d.response_bytes, d.alloc_count_per_op, d.alloc_bytes_per_op
        );
        for l in &s.timing_advisory.levels {
            println!(
                "  {:<20} c={:<4} ADVISORY rps={:<10} p50={}us p99={}us p999={}us max={}us succ={:.4}",
                "", l.concurrency, l.throughput_rps, l.latency_us.p50, l.latency_us.p99,
                l.latency_us.p999, l.latency_us.max, l.success_rate
            );
        }
    }
    println!("{}", "-".repeat(96));
    println!("(timing = ADVISORY, non-gating; deterministic = strict)");
}
