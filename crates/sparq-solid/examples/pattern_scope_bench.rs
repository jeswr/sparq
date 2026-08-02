//! [FABLE-5] sq-lrtc3.3 — the pattern-scope measured overhead envelope. Run:
//!     cargo run -p sparq-solid --features pattern-scope --release --example pattern_scope_bench
//!
//! Emits ONE JSON object to stdout (committed under `bench/pattern-scope/`; work-box
//! numbers are NON-canonical — see `research/odrl-pattern-scoped-targets-2026-07.md` §4).
//! Dimensions, per fixture size (`wac_fixture_sized(extra)`):
//!
//! 1. `cold_build_ms` — the cost this design pays INSTEAD of per-scan filtering: one
//!    `PodStore::scoped_dataset` materialization (decode → filter → rebuild of every
//!    scoped graph) for a scope masking the `title` predicate everywhere, on a FRESH
//!    store so the replica cache is empty. Sampled over fresh stores precisely because
//!    a repeat call on the same store would be a cache hit.
//! 2. `warm_build_ms` — the SAME call with the replica cache warm ([SONNET-4.6]
//!    sq-nc3c6, design record §6): what a repeat scoped query pays once the scope class
//!    has been built once. `cold_build_ms` vs `warm_build_ms` IS the amortization.
//! 3. `repeat10_build_query_ms` — ten consecutive (build + query) rounds on one store,
//!    the shape a session issuing several queries under one scope actually sees; and
//!    `first_build_query_ms`, the first such round alone (cold), to read the curve.
//! 4. `replica_query_ms` vs `view_query_ms` — the SAME query over the prebuilt
//!    masked replica vs the ordinary graph-granular `query_as` view path on the full
//!    store: the per-query overhead once assembly is done (expected ≈ 0 — after
//!    assembly the engine sees an ordinary dataset).
//! 5. `breakeven_queries` — cold build cost divided by any per-query saving (the
//!    replica is smaller), when a saving exists: how many queries amortize one build.

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

        // COLD build: a fresh store per sample, so the replica cache starts empty. This
        // is deliberately NOT `best_of` over one store — that would silently report the
        // WARM number, since the second and third calls would be cache hits.
        let mut cold_build_ms = f64::MAX;
        for _ in 0..3 {
            let mut fresh = PodStore::new(sparq_core::Graph::load_dataset(&nq, "nquads").unwrap());
            fresh.materialize_wac().unwrap();
            let t = Instant::now();
            let built = fresh.scoped_dataset(&alice, Mode::Read, &scopes);
            cold_build_ms = cold_build_ms.min(t.elapsed().as_secs_f64() * 1e3);
            std::hint::black_box(built.view());
        }

        // Ten (build + query) rounds on ONE store: round 1 pays the cold build, rounds
        // 2..10 hit the cache. `first` is the cold round on its own.
        let repeat_start = Instant::now();
        let mut first_build_query_ms = 0.0;
        for round in 0..10 {
            let t = Instant::now();
            let d = store.scoped_dataset(&alice, Mode::Read, &scopes);
            std::hint::black_box(d.query(Q).unwrap().rows.len());
            if round == 0 {
                first_build_query_ms = t.elapsed().as_secs_f64() * 1e3;
            }
        }
        let repeat10_build_query_ms = repeat_start.elapsed().as_secs_f64() * 1e3;

        // WARM build: same store, replicas already cached by the loop above.
        let (warm_build_ms, scoped) =
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

        let saving = view_scan_ms - replica_scan_ms;
        let breakeven = if saving > 0.0 { (cold_build_ms / saving).ceil() } else { -1.0 };
        rows.push(format!(
            "    {{\"extra\": {extra}, \"quads\": {quads}, \"cold_build_ms\": {cold_build_ms:.3}, \
             \"warm_build_ms\": {warm_build_ms:.3}, \
             \"first_build_query_ms\": {first_build_query_ms:.3}, \
             \"repeat10_build_query_ms\": {repeat10_build_query_ms:.3}, \
             \"view_query_ms\": {view_query_ms:.3}, \"replica_query_ms\": {replica_query_ms:.3}, \
             \"view_scan_ms\": {view_scan_ms:.3}, \"replica_scan_ms\": {replica_scan_ms:.3}, \
             \"view_title_rows\": {view_rows}, \"replica_title_rows\": {replica_rows}, \
             \"breakeven_queries\": {breakeven}}}"
        ));
    }

    println!(
        "{{\n  \"bead\": \"sq-nc3c6 (extends sq-lrtc3.3)\",\n  \"design\": \"masked-subgraph materialization \
         + bounded sharded replica cache (research/odrl-pattern-scoped-targets-2026-07.md §6)\",\n  \
         \"host\": \"work box — NON-canonical\",\n  \
         \"driver\": \"cargo run -p sparq-solid --features pattern-scope --release --example pattern_scope_bench\",\n  \
         \"note\": \"breakeven_queries = -1 means the replica query showed no measurable saving at this size; \
         cold_build_ms is sampled on FRESH stores (empty replica cache), warm_build_ms on a warm one\",\n  \
         \"sizes\": [\n{}\n  ]\n}}",
        rows.join(",\n")
    );
}
