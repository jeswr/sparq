//! [FABLE-5] sq-lrtc3.3 — the pattern-scope measured overhead envelope. Run:
//!     cargo run -p sparq-solid --features pattern-scope --release --example pattern_scope_bench
//!
//! Emits ONE JSON object to stdout (committed under `bench/pattern-scope/`; work-box
//! numbers are NON-canonical — see `research/odrl-pattern-scoped-targets-2026-07.md` §4).
//! Dimensions, per fixture size (`wac_fixture_sized(extra)`):
//!
//! 1. `scoped_build_cold_ms` — the cost this design pays INSTEAD of per-scan filtering:
//!    one `PodStore::scoped_dataset` materialization (decode → filter → rebuild of
//!    every accessible graph) for a scope masking the `title` predicate everywhere,
//!    with the replica cache dropped before each timed run so every one pays in full.
//! 2. `replica_query_ms` vs `view_query_ms` — the SAME query over the prebuilt
//!    masked replica vs the ordinary graph-granular `query_as` view path on the full
//!    store: the per-query overhead once assembly is done (expected ≈ 0 — after
//!    assembly the engine sees an ordinary dataset).
//! 3. `breakeven_queries` — build cost divided by any per-query saving (the replica
//!    is smaller), when a saving exists: how many queries amortize one assembly.
//! 4. [OPUS-5] sq-nc3c6 — the REPLICA-CACHE dimension the acceptance criterion asks
//!    for: `scoped_build_warm_ms` (a `scoped_dataset` call that hits the cache) and
//!    `repeat_scoped_query_*_ms` (the per-iteration cost of a repeat scoped-query loop
//!    with the cache live vs. with it dropped every iteration — i.e. the pre-cache
//!    behaviour). The gap between the two is the amortization the cache buys.

use oxrdf::{NamedNode, Term};
use rustc_hash::FxHashMap;
use sparq_solid::fixture::{wac_fixture_sized, ALICE};
use sparq_solid::{GraphScope, Mode, PodStore, ScopePattern, Session};
use std::time::Instant;

const TITLE: &str = "https://ex.dev/ns#title";
const Q: &str = "SELECT ?s WHERE { GRAPH ?g { ?s <https://ex.dev/ns#title> ?title } }";
const Q_ALL: &str = "SELECT (COUNT(*) AS ?c) WHERE { GRAPH ?g { ?s ?p ?o } }";

fn best_of<T>(n: usize, mut f: impl FnMut() -> T) -> (f64, T) {
    let mut best = f64::MAX;
    let mut out = None;
    for _ in 0..n {
        let t = Instant::now();
        out = Some(f());
        best = best.min(t.elapsed().as_secs_f64() * 1e3);
    }
    (best, out.unwrap())
}

fn main() {
    let alice = Session { agent: Some(ALICE), client: None, issuer: None, now: None };
    let mut rows = Vec::new();

    for extra in [0usize, 20, 50] {
        let nq = wac_fixture_sized(extra);
        let quads = nq.lines().filter(|l| !l.trim().is_empty()).count();
        let mut store = PodStore::new(sparq_core::Graph::load_dataset(&nq, "nquads").unwrap());
        store.materialize_wac().unwrap();

        // The scope: mask every `title` triple in every accessible graph (a
        // store-wide prohibition — worst case: every accessible graph is scoped).
        let deny = GraphScope::deny_within(vec![ScopePattern::new(
            None,
            Some(Term::NamedNode(NamedNode::new(TITLE).unwrap())),
            None,
        )]);
        let scopes: FxHashMap<Term, GraphScope> = store
            .graph
            .named
            .iter()
            .map(|(name, _)| (name.clone(), deny.clone()))
            .collect();

        // [OPUS-5] sq-nc3c6: COLD build — drop the replica cache (outside the timer) so
        // every timed run pays the full decode → filter → rebuild, as it did before the
        // cache landed. `best_of` alone would now measure a cache HIT after the first run.
        let mut scoped_build_cold_ms = f64::MAX;
        for _ in 0..3 {
            store.invalidate_scoped_replicas();
            let t = Instant::now();
            let cold = store.scoped_dataset(&alice, Mode::Read, &scopes);
            scoped_build_cold_ms = scoped_build_cold_ms.min(t.elapsed().as_secs_f64() * 1e3);
            drop(cold);
        }
        // WARM: the same call served from the replica cache.
        let (scoped_build_warm_ms, scoped) =
            best_of(5, || store.scoped_dataset(&alice, Mode::Read, &scopes));

        let (view_query_ms, view_rows) =
            best_of(5, || store.query_as(&alice, Mode::Read, Q).unwrap().rows.len());
        let (replica_query_ms, replica_rows) =
            best_of(5, || scoped.query(Q).unwrap().rows.len());
        let (view_scan_ms, _) =
            best_of(5, || store.query_as(&alice, Mode::Read, Q_ALL).unwrap().rows.len());
        let (replica_scan_ms, _) = best_of(5, || scoped.query(Q_ALL).unwrap().rows.len());

        assert_eq!(replica_rows, 0, "masked replica must hold no title triples");
        assert!(view_rows > 0, "the unmasked view path must see title triples");

        // [OPUS-5] sq-nc3c6 — the repeat-scoped-query loop the acceptance criterion names:
        // REPS iterations of (obtain the scoped dataset, run the query) with the cache
        // live, versus the same loop with the cache dropped each iteration (the pre-cache
        // behaviour: one full rebuild per scoped query).
        const REPS: usize = 10;
        let t = Instant::now();
        for _ in 0..REPS {
            let s = store.scoped_dataset(&alice, Mode::Read, &scopes);
            assert_eq!(s.query(Q).unwrap().rows.len(), 0);
        }
        let repeat_scoped_query_cached_ms = t.elapsed().as_secs_f64() * 1e3 / REPS as f64;
        let t = Instant::now();
        for _ in 0..REPS {
            store.invalidate_scoped_replicas();
            let s = store.scoped_dataset(&alice, Mode::Read, &scopes);
            assert_eq!(s.query(Q).unwrap().rows.len(), 0);
        }
        let repeat_scoped_query_rebuilt_ms = t.elapsed().as_secs_f64() * 1e3 / REPS as f64;

        let saving = view_scan_ms - replica_scan_ms;
        let breakeven = if saving > 0.0 { (scoped_build_cold_ms / saving).ceil() } else { -1.0 };
        rows.push(format!(
            "    {{\"extra\": {extra}, \"quads\": {quads}, \
             \"scoped_build_cold_ms\": {scoped_build_cold_ms:.3}, \
             \"scoped_build_warm_ms\": {scoped_build_warm_ms:.3}, \
             \"repeat_scoped_query_cached_ms\": {repeat_scoped_query_cached_ms:.3}, \
             \"repeat_scoped_query_rebuilt_ms\": {repeat_scoped_query_rebuilt_ms:.3}, \
             \"repeat_reps\": {REPS}, \
             \"view_query_ms\": {view_query_ms:.3}, \"replica_query_ms\": {replica_query_ms:.3}, \
             \"view_scan_ms\": {view_scan_ms:.3}, \"replica_scan_ms\": {replica_scan_ms:.3}, \
             \"view_title_rows\": {view_rows}, \"replica_title_rows\": {replica_rows}, \
             \"breakeven_queries\": {breakeven}}}"
        ));
    }

    println!(
        "{{\n  \"bead\": \"sq-nc3c6 (replica cache) over sq-lrtc3.3\",\n  \
         \"design\": \"masked-subgraph materialization \
         (research/odrl-pattern-scoped-targets-2026-07.md)\",\n  \"host\": \"work box — NON-canonical\",\n  \
         \"driver\": \"cargo run -p sparq-solid --features pattern-scope --release --example pattern_scope_bench\",\n  \
         \"note\": \"breakeven_queries = -1 means the replica query showed no measurable saving at this size\",\n  \
         \"sizes\": [\n{}\n  ]\n}}",
        rows.join(",\n")
    );
}
