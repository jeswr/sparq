//! [OPUS-5] (sq-lsp7k.7) e2e integration tests for the OPT-IN `?at=<instant>` point-in-time pin
//! on `/sparql` — SPARQL against the store as it stood at an ARBITRARY past moment, reconstructed
//! from the durable CDC change stream.
//!
//! These tests run ONLY with the `point-in-time` cargo feature (the whole file is gated). They
//! spin the real axum server pointed at a durable log directory and drive the REAL HTTP path:
//!   * the load-bearing invariant — write, capture the instant, write again, and assert `?at=`
//!     answers with the OLD state while the unpinned query answers with the new one;
//!   * the reach beyond the in-memory ring — an instant far enough back that `?generation=N`
//!     would be `410 Gone` is still served, which is the whole point of the feature over
//!     `time-travel` alone;
//!   * the `Sparq-Generation` header names the RECONSTRUCTED historical generation, not the
//!     meaningless number of the rebuilt snapshot's throwaway ring;
//!   * both accepted `at=` forms (`xsd:dateTime` and the `commitTimestampNanos` integer);
//!   * the refusals, all fail-closed: unparsable `at` → `400`; `at` + `generation` together →
//!     `400`; an instant before the retained horizon → `410`; no `--change-stream` dir → `400`.
//!
//! Instants come from the change stream's OWN recorded timestamps (read back off `GET /streams`),
//! never from the test's wall clock — the recorded timestamp is the authority the resolution
//! compares against, so asserting through it keeps these tests free of clock races.
#![cfg(feature = "point-in-time")]

use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig};
use tokio::net::TcpListener;

/// A unique scratch directory for one test's durable change-stream log.
fn scratch(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "sparq-pit-{}-{}-{:?}",
        tag,
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Boots the real server with the change-stream log at `dir` (or none, the unconfigured case),
/// with a deliberately TINY time-travel window so a test can prove the point-in-time reach
/// exceeds the ring's.
async fn spawn(dir: Option<std::path::PathBuf>, generations: usize) -> String {
    let graph = Graph::load_str("", "turtle").unwrap();
    let config = ServerConfig {
        change_stream_dir: dir,
        time_travel_generations: generations,
        ..ServerConfig::default()
    };
    let app = router(AppState::with_config(graph, config));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

async fn post_update(cl: &reqwest::Client, base: &str, update: &str) {
    let status = cl
        .post(format!("{base}/sparql"))
        .header("content-type", "application/sparql-update")
        .body(update.to_string())
        .send()
        .await
        .unwrap()
        .status();
    assert!(status.is_success(), "update failed: {status}: {update}");
}

/// GETs `/sparql` with the given extra parameters.
async fn query(
    cl: &reqwest::Client,
    base: &str,
    sparql: &str,
    extra: &[(&str, &str)],
) -> reqwest::Response {
    let mut params = vec![("query", sparql)];
    params.extend_from_slice(extra);
    cl.get(format!("{base}/sparql"))
        .header("accept", "application/sparql-results+json")
        .query(&params)
        .send()
        .await
        .unwrap()
}

/// The subjects a SELECT returned, sorted — the state assertion these tests make.
async fn subjects(resp: reqwest::Response) -> Vec<String> {
    assert_eq!(resp.status(), 200, "expected a 200 result");
    let body: serde_json::Value = resp.json().await.unwrap();
    let mut out: Vec<String> = body["results"]["bindings"]
        .as_array()
        .expect("SELECT bindings")
        .iter()
        .map(|b| b["s"]["value"].as_str().unwrap().to_string())
        .collect();
    out.sort();
    out
}

/// The recorded commit timestamps (nanoseconds since the Unix epoch), one per generation, read
/// back off the CDC feed the reconstruction itself resolves against.
async fn commit_instants(cl: &reqwest::Client, base: &str) -> Vec<(u64, u128)> {
    let body: serde_json::Value = cl
        .get(format!("{base}/streams"))
        .query(&[("iteratorType", "TRIM_HORIZON"), ("limit", "1000")])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let mut seen: Vec<(u64, u128)> = Vec::new();
    for record in body["records"].as_array().expect("records array") {
        let generation = record["generation"].as_u64().expect("generation");
        let nanos: u128 = record["commitTimestampNanos"]
            .as_str()
            .expect("commitTimestampNanos is a string")
            .parse()
            .expect("nanos parse");
        if !seen.iter().any(|(g, _)| *g == generation) {
            seen.push((generation, nanos));
        }
    }
    seen
}

const PROBE: &str = "SELECT ?s WHERE { ?s <http://ex/p> ?o }";

fn insert(subject: &str) -> String {
    format!("INSERT DATA {{ <http://ex/{subject}> <http://ex/p> <http://ex/o> }}")
}

fn delete(subject: &str) -> String {
    format!("DELETE DATA {{ <http://ex/{subject}> <http://ex/p> <http://ex/o> }}")
}

/// THE headline guard: a query pinned to a past instant answers with the state AT that instant,
/// while the unpinned query answers with the present. Covers both an insert and a delete after
/// the pinned moment, so the rewind has to undo both directions.
#[tokio::test]
async fn at_serves_the_state_as_of_a_past_instant() {
    let dir = scratch("as-of");
    let base = spawn(Some(dir.clone()), 16).await;
    let cl = client();

    post_update(&cl, &base, &insert("a")).await; // generation 1: {a}
    post_update(&cl, &base, &insert("b")).await; // generation 2: {a, b}
    post_update(&cl, &base, &delete("a")).await; // generation 3: {b}
    post_update(&cl, &base, &insert("c")).await; // generation 4: {b, c}

    let instants = commit_instants(&cl, &base).await;
    assert_eq!(instants.len(), 4, "one record per commit: {instants:?}");
    let (gen2, at_gen2) = instants[1];
    assert_eq!(gen2, 2);

    // Now: {b, c}. At generation 2's instant: {a, b} — the delete of `a` and the insert of `c`
    // are both after it and must both be undone.
    assert_eq!(
        subjects(query(&cl, &base, PROBE, &[]).await).await,
        vec!["http://ex/b", "http://ex/c"],
        "the unpinned query sees the present"
    );
    let pinned = query(&cl, &base, PROBE, &[("at", &at_gen2.to_string())]).await;
    assert_eq!(
        pinned
            .headers()
            .get("sparq-generation")
            .and_then(|v| v.to_str().ok()),
        Some("2"),
        "the header names the RECONSTRUCTED historical generation"
    );
    assert_eq!(
        subjects(pinned).await,
        vec!["http://ex/a", "http://ex/b"],
        "the pinned query sees the state at that instant"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// The reason this feature exists on top of `time-travel`: the durable log reaches PAST the
/// in-memory ring. With a window far smaller than the number of commits, `?generation=N` is
/// `410 Gone` for the old generation while `?at=` still reconstructs it.
#[tokio::test]
async fn at_reaches_past_the_in_memory_retention_window() {
    let dir = scratch("beyond-ring");
    // A window of 1 composes with the ring's concurrency floor (K = 4), so ~5 generations of
    // history are retained in memory; 20 commits leaves generation 1 far outside it.
    let base = spawn(Some(dir.clone()), 1).await;
    let cl = client();

    post_update(&cl, &base, &insert("first")).await; // generation 1
    for i in 0..20 {
        post_update(&cl, &base, &insert(&format!("filler{i}"))).await;
    }

    let instants = commit_instants(&cl, &base).await;
    let (gen1, at_gen1) = instants[0];
    assert_eq!(gen1, 1);

    // The in-memory pin for that generation has aged out.
    let by_generation = query(&cl, &base, PROBE, &[("generation", "1")]).await;
    assert_eq!(
        by_generation.status(),
        410,
        "the ring no longer retains generation 1"
    );

    // The durable log still does.
    let by_instant = query(&cl, &base, PROBE, &[("at", &at_gen1.to_string())]).await;
    assert_eq!(
        by_instant
            .headers()
            .get("sparq-generation")
            .and_then(|v| v.to_str().ok()),
        Some("1")
    );
    assert_eq!(
        subjects(by_instant).await,
        vec!["http://ex/first"],
        "reconstructed from the change stream, past the ring's window"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// `at=` accepts an `xsd:dateTime` lexical, resolving to the same state as the equivalent
/// nanosecond integer.
#[tokio::test]
async fn at_accepts_an_xsd_datetime_lexical() {
    let dir = scratch("datetime");
    let base = spawn(Some(dir.clone()), 16).await;
    let cl = client();

    post_update(&cl, &base, &insert("a")).await;
    post_update(&cl, &base, &insert("b")).await;

    let instants = commit_instants(&cl, &base).await;
    let (_, at_gen1) = instants[0];
    // Render generation 1's recorded instant as an RFC-3339 UTC lexical, truncated to whole
    // seconds and stepped forward so the truncation cannot land before the commit. Generation 2
    // follows within the same run, so this still resolves to generation 1 unless a whole second
    // elapsed between the two commits — hence the tolerance below.
    let secs = (at_gen1 / 1_000_000_000) as i64;
    let lexical = rfc3339_utc(secs + 1);

    let by_lexical = query(&cl, &base, PROBE, &[("at", &lexical)]).await;
    assert_eq!(by_lexical.status(), 200, "a dateTime lexical is accepted");
    let generation = by_lexical
        .headers()
        .get("sparq-generation")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .expect("generation header");
    let seen = subjects(by_lexical).await;
    // Both commits land in the same wall-clock second in practice; either resolution is a
    // CORRECT answer for that lexical, and both are asserted exactly.
    if generation == 1 {
        assert_eq!(seen, vec!["http://ex/a"]);
    } else {
        assert_eq!(generation, 2, "resolved to a commit that exists");
        assert_eq!(seen, vec!["http://ex/a", "http://ex/b"]);
    }

    let _ = std::fs::remove_dir_all(&dir);
}

/// Fail-closed refusals: every one is an explicit status, never an answer at a different instant.
#[tokio::test]
async fn at_refuses_rather_than_approximating() {
    let dir = scratch("refusals");
    let base = spawn(Some(dir.clone()), 16).await;
    let cl = client();
    post_update(&cl, &base, &insert("a")).await;

    let instants = commit_instants(&cl, &base).await;
    let (_, at_gen1) = instants[0];

    // Unparsable.
    assert_eq!(
        query(&cl, &base, PROBE, &[("at", "yesterday")])
            .await
            .status(),
        400
    );
    // Two different pins in one request.
    assert_eq!(
        query(
            &cl,
            &base,
            PROBE,
            &[("at", &at_gen1.to_string()), ("generation", "1")]
        )
        .await
        .status(),
        400,
        "'at' and 'generation' are mutually exclusive"
    );
    // Before the retained horizon — the log starts at the first commit, so anything earlier is
    // unreconstructable and must be `410`, not the oldest retained state.
    let before = query(&cl, &base, PROBE, &[("at", &(at_gen1 - 1).to_string())]).await;
    assert_eq!(
        before.status(),
        410,
        "an instant before the first retained record is Gone, never clamped"
    );
    // The refusal quotes the earliest reconstructable instant, so the caller can retry AT the
    // horizon instead of bisecting for it.
    let body = before.text().await.unwrap();
    assert!(
        body.contains(&at_gen1.to_string()),
        "the 410 names the horizon it refused past: {body}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// The single-slot reconstruction cache is KEYED, not blind: two different instants in a row
/// must each get their own state. A cache that ignored its key would serve the first
/// reconstruction for both — which is the silent-substitution failure this surface exists to
/// refuse, so it is worth an explicit guard rather than trusting the lookup.
#[tokio::test]
async fn repeat_and_alternating_instants_each_get_their_own_state() {
    let dir = scratch("cache-keying");
    let base = spawn(Some(dir.clone()), 16).await;
    let cl = client();

    post_update(&cl, &base, &insert("a")).await; // generation 1: {a}
    post_update(&cl, &base, &insert("b")).await; // generation 2: {a, b}
    post_update(&cl, &base, &insert("c")).await; // generation 3: {a, b, c}

    let instants = commit_instants(&cl, &base).await;
    let at_gen1 = instants[0].1.to_string();
    let at_gen2 = instants[1].1.to_string();

    let one = vec!["http://ex/a".to_string()];
    let two = vec!["http://ex/a".to_string(), "http://ex/b".to_string()];

    // Alternate, so every request but the first finds a cache entry for the OTHER instant.
    for _ in 0..3 {
        assert_eq!(
            subjects(query(&cl, &base, PROBE, &[("at", &at_gen1)]).await).await,
            one
        );
        assert_eq!(
            subjects(query(&cl, &base, PROBE, &[("at", &at_gen2)]).await).await,
            two
        );
    }
    // And a repeat of the same instant (the cache-HIT path) is still that instant's state.
    assert_eq!(
        subjects(query(&cl, &base, PROBE, &[("at", &at_gen2)]).await).await,
        two
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// The double opt-in: compiling the feature is not enough — without a configured log directory
/// there is no history, and `at=` says so instead of silently answering at the present.
#[tokio::test]
async fn at_requires_a_configured_change_stream() {
    let base = spawn(None, 16).await;
    let cl = client();
    post_update(&cl, &base, &insert("a")).await;

    let resp = query(&cl, &base, PROBE, &[("at", "2026-07-28T12:00:00Z")]).await;
    assert_eq!(
        resp.status(),
        400,
        "no --change-stream dir: an explicit refusal, not a present-state answer"
    );
    // The unpinned query is completely unaffected.
    assert_eq!(
        subjects(query(&cl, &base, PROBE, &[]).await).await,
        vec!["http://ex/a"]
    );
}

/// Formats whole seconds since the Unix epoch as an RFC-3339 UTC lexical. Local to the test so
/// the suite needs no date dependency (Howard Hinnant's `civil_from_days`).
fn rfc3339_utc(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (hh, mm, ss) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if m <= 2 { y + 1 } else { y };
    format!("{year:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}
