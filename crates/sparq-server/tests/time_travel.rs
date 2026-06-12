//! Time-travel query integration tests, both feature states (compile the suite
//! twice: `cargo test -p sparq-server` and `… --features time-travel`):
//!
//! * feature ON (`with_feature`): `?generation=N` end-to-end through the real
//!   server — a pinned query returns OLD data after subsequent updates; the
//!   `Sparq-Generation` header exposes the produced-against generation on query
//!   responses and the read-your-writes token on update 204s; aged-out → 410;
//!   never-published → 400; unparsable → 400; pinning an update → 400; and the
//!   retention-bound composition (configured time-travel window SMALLER than the
//!   ring's K — the K floor wins, documented in sparq-serve).
//! * feature OFF (`without_feature`): the parameter handling is compiled out —
//!   `?generation=` is an ignored unknown parameter (current data, 200) and no
//!   `Sparq-Generation` header exists.
//!
//! Aging out is driven by the deterministic COUNT bound (publish past the
//! window), never by wall-clock `max_age` — the clock-sensitive age semantics
//! are unit-tested in sparq-serve with an injected clock (the recorded
//! determinism concern).

use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig};
use tokio::net::TcpListener;

/// Boots the real server over `graph` on an ephemeral port; returns its base URL.
async fn spawn_with(graph: Graph, config: ServerConfig) -> String {
    let app = router(AppState::with_config(graph, config));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

async fn spawn_empty(config: ServerConfig) -> String {
    spawn_with(Graph::load_str("", "turtle").unwrap(), config).await
}

async fn post_update(cl: &reqwest::Client, base: &str, update: &str) -> reqwest::Response {
    cl.post(format!("{base}/sparql"))
        .header("content-type", "application/sparql-update")
        .body(update.to_string())
        .send()
        .await
        .unwrap()
}

const PROBE: &str = "SELECT ?s WHERE { ?s <http://ex/seen> ?o }";

/// GETs `query` (optionally pinned to `generation`), returning the response.
async fn query_response(cl: &reqwest::Client, base: &str, query: &str, generation: Option<&str>) -> reqwest::Response {
    let mut params = vec![("query", query)];
    if let Some(g) = generation {
        params.push(("generation", g));
    }
    cl.get(format!("{base}/sparql"))
        .header("accept", "application/sparql-results+json")
        .query(&params)
        .send()
        .await
        .unwrap()
}

/// Solution count of a 200 JSON response.
async fn rows(resp: reqwest::Response) -> usize {
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    body["results"]["bindings"].as_array().unwrap().len()
}

fn generation_header(resp: &reqwest::Response) -> Option<u64> {
    resp.headers()
        .get("sparq-generation")
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

#[cfg(feature = "time-travel")]
mod with_feature {
    use super::*;

    fn insert(i: usize) -> String {
        format!("INSERT DATA {{ <http://ex/u{i}> <http://ex/seen> <http://ex/yes> }}")
    }

    /// The headline property: a query pinned to a captured generation returns the
    /// OLD data after subsequent updates — end to end through the real server —
    /// while unpinned queries see the new state. Headers carry the tokens: each
    /// update 204 exposes the generation containing it (read-your-writes), each
    /// query response the generation it was produced against.
    #[tokio::test]
    async fn pinned_query_returns_old_data_after_updates() {
        let base = spawn_empty(ServerConfig::default()).await;
        let cl = reqwest::Client::new();

        // Generation 0: empty. The response header exposes the current generation.
        let r0 = query_response(&cl, &base, PROBE, None).await;
        assert_eq!(generation_header(&r0), Some(0), "query response exposes its generation");
        assert_eq!(rows(r0).await, 0);

        // Three sequential updates → generations 1, 2, 3; each 204 carries its token.
        for i in 1..=3usize {
            let resp = post_update(&cl, &base, &insert(i)).await;
            assert_eq!(resp.status(), 204);
            assert_eq!(
                generation_header(&resp),
                Some(i as u64),
                "update ack exposes the generation containing the update"
            );
        }

        // Unpinned: current state (3 rows), header says generation 3.
        let now = query_response(&cl, &base, PROBE, None).await;
        assert_eq!(generation_header(&now), Some(3));
        assert_eq!(rows(now).await, 3);

        // Pinned to each retained generation: exactly the state as of that point.
        for (g, expect) in [(0u64, 0usize), (1, 1), (2, 2), (3, 3)] {
            let resp = query_response(&cl, &base, PROBE, Some(&g.to_string())).await;
            assert_eq!(generation_header(&resp), Some(g), "pinned response confirms its pin");
            assert_eq!(rows(resp).await, expect, "generation {g} must serve its own state");
        }
    }

    /// The pin also flows through the url-encoded POST body (same precedence
    /// contract as `explain`: body wins).
    #[tokio::test]
    async fn generation_in_urlencoded_post_body_pins() {
        let base = spawn_empty(ServerConfig::default()).await;
        let cl = reqwest::Client::new();
        assert_eq!(post_update(&cl, &base, &insert(1)).await.status(), 204);

        let resp = cl
            .post(format!("{base}/sparql"))
            .header("content-type", "application/x-www-form-urlencoded")
            .header("accept", "application/sparql-results+json")
            .body(format!("query={}&generation=0", urlencoded(PROBE)))
            .send()
            .await
            .unwrap();
        assert_eq!(generation_header(&resp), Some(0));
        assert_eq!(rows(resp).await, 0, "POST-form pin must serve generation 0's empty state");
    }

    fn urlencoded(s: &str) -> String {
        s.bytes()
            .map(|b| match b {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => (b as char).to_string(),
                b' ' => "+".into(),
                _ => format!("%{b:02X}"),
            })
            .collect()
    }

    /// Aged-out generations are a 410 Gone (not a silent fallback) — and the
    /// retention bounds compose as documented: configure a time-travel window
    /// (2) SMALLER than the ring's concurrency bound K (4) and the K floor wins,
    /// so exactly the K newest older-than-current generations are servable.
    #[tokio::test]
    async fn aged_out_generation_is_410_and_k_floor_wins() {
        let config = ServerConfig { time_travel_generations: 2, ..ServerConfig::default() };
        let base = spawn_empty(config).await;
        let cl = reqwest::Client::new();
        for i in 1..=6usize {
            assert_eq!(post_update(&cl, &base, &insert(i)).await.status(), 204);
        }
        // Current = 6; retained older = max(K = 4, configured 2) = 4 → gens 2..=6.
        let ok = query_response(&cl, &base, PROBE, Some("2")).await;
        assert_eq!(rows(ok).await, 2, "oldest retained generation still serves its state");

        let gone = query_response(&cl, &base, PROBE, Some("1")).await;
        assert_eq!(gone.status(), 410, "an aged-out generation is Gone, never substituted");
        let body: serde_json::Value = gone.json().await.unwrap();
        let msg = body["error"].as_str().unwrap();
        assert!(msg.contains("aged out"), "error explains the 410: {msg}");
        assert!(msg.contains("oldest retained: 2"), "error names the oldest retained: {msg}");
    }

    /// Token misuse is a 400 with a precise message: a generation that was never
    /// published, an unparsable number, and pinning an update.
    #[tokio::test]
    async fn bad_generation_tokens_are_400() {
        let base = spawn_empty(ServerConfig::default()).await;
        let cl = reqwest::Client::new();

        let future = query_response(&cl, &base, PROBE, Some("999")).await;
        assert_eq!(future.status(), 400);
        let body: serde_json::Value = future.json().await.unwrap();
        assert!(body["error"].as_str().unwrap().contains("not been published"));

        let junk = query_response(&cl, &base, PROBE, Some("yesterday")).await;
        assert_eq!(junk.status(), 400);
        let body: serde_json::Value = junk.json().await.unwrap();
        assert!(body["error"].as_str().unwrap().contains("invalid 'generation' parameter"));

        // Updates always apply to the current generation: pinning one is refused,
        // not silently ignored.
        let resp = cl
            .post(format!("{base}/sparql?generation=0"))
            .header("content-type", "application/sparql-update")
            .body(insert(1))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 400);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body["error"].as_str().unwrap().contains("updates always apply to the current generation"));
        // …and nothing was applied.
        assert_eq!(rows(query_response(&cl, &base, PROBE, None).await).await, 0);
    }
}

#[cfg(not(feature = "time-travel"))]
mod without_feature {
    use super::*;

    /// With the feature off the parameter handling is compiled out: `?generation=`
    /// is just an ignored unknown parameter — the query runs against the CURRENT
    /// generation (even though the number names an older one) and no
    /// `Sparq-Generation` header exists anywhere.
    #[tokio::test]
    async fn generation_parameter_is_compiled_out() {
        let base = spawn_empty(ServerConfig::default()).await;
        let cl = reqwest::Client::new();

        let upd = post_update(&cl, &base, "INSERT DATA { <http://ex/u1> <http://ex/seen> <http://ex/yes> }").await;
        assert_eq!(upd.status(), 204);
        assert_eq!(generation_header(&upd), None, "no generation header without the feature");

        // `?generation=0` names the pre-update state; without the feature it is
        // ignored and the response is the CURRENT data with no header.
        let resp = query_response(&cl, &base, PROBE, Some("0")).await;
        assert_eq!(resp.status(), 200);
        assert_eq!(generation_header(&resp), None);
        assert_eq!(rows(resp).await, 1, "feature off: ?generation is ignored, current state served");

        // Garbage values are equally ignored — no 400s from compiled-out handling.
        let resp = query_response(&cl, &base, PROBE, Some("yesterday")).await;
        assert_eq!(resp.status(), 200);
        assert_eq!(rows(resp).await, 1);
    }
}
