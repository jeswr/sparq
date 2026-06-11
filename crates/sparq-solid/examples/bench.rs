//! The v1 measured baseline (design doc §6). Run:
//!     cargo run -p sparq-solid --example bench --release
//!
//! Measures, on the ~1.1k-graph WAC fixture (+ ACP variant):
//!   1. auth-view materialization (and re-materialization after an ACL change — v1's
//!      incremental-maintenance story IS a full re-run);
//!   2. per-query overhead of the v1 FROM-NAMED-copy path vs the engine on the full
//!      dataset directly (the copy the L1 zero-copy view must beat);
//!   3. session graph-set computation, cold vs cached.

use sparq_core::Graph;
use sparq_solid::fixture::{acp_fixture, wac_fixture, ALICE};
use sparq_solid::{Mode, PodStore, Session};
use std::time::Instant;

const TITLES: &str = "SELECT ?title WHERE { ?s <https://ex.dev/ns#title> ?title }";

fn timed<T>(label: &str, mut f: impl FnMut() -> T) -> T {
    // best-of-3 for the multi-ms operations
    let mut best = f64::MAX;
    let mut out = None;
    for _ in 0..3 {
        let t = Instant::now();
        out = Some(f());
        best = best.min(t.elapsed().as_secs_f64() * 1e3);
    }
    println!("{label:64} {best:10.2} ms");
    out.unwrap()
}

fn main() {
    let nq = wac_fixture();
    let graph = Graph::load_dataset(&nq, "nquads").unwrap();
    let n_graphs = graph.named.len();
    let n_quads: usize = graph.named.iter().map(|(_, g)| g.len()).sum();
    println!("WAC fixture: {n_graphs} named graphs, {n_quads} quads\n");

    let mut store = PodStore::new(graph);
    // 1. materialization + re-materialization (≈ the ACL-change cost)
    let stats = timed("WAC auth-view materialization (full pipeline)", || {
        store.materialize_wac().unwrap()
    });
    println!("    auth view: {} triples, closure {:?}\n", stats.auth_triples, stats.strata_facts);

    let acp_graph = Graph::load_dataset(&acp_fixture(), "nquads").unwrap();
    let mut acp_store = PodStore::new(acp_graph);
    let astats = timed("ACP auth-view materialization (3 strata)", || {
        acp_store.materialize_acp().unwrap()
    });
    println!("    auth view: {} triples, closure per stratum {:?}\n", astats.auth_triples, astats.strata_facts);

    // 2. per-query: v1 rewrite+copy vs full-dataset direct query
    let alice = Session { agent: Some(ALICE), client: None };
    let allowed = store.accessible(&alice, Mode::Read);
    println!("alice readable graphs: {}", allowed.len());
    let direct = timed("engine::query, FULL dataset (GRAPH ?g over all graphs)", || {
        sparq_engine::query(
            &store.graph,
            "SELECT ?title WHERE { GRAPH ?g { ?s <https://ex.dev/ns#title> ?title } }",
        )
        .unwrap()
    });
    println!("    rows: {}", direct.rows.len());
    let v1 = timed("query_as (v1: rewrite + FROM NAMED build_active copy)", || {
        store.query_as(&alice, Mode::Read, TITLES).unwrap()
    });
    println!("    rows: {} (authorized subset)", v1.rows.len());
    // the copy alone, isolated: same rewritten query, pattern matched nothing
    let nothing = timed("v1 copy cost isolated (rewritten query, empty pattern)", || {
        store
            .query_as(&alice, Mode::Read, "SELECT ?x WHERE { ?x <urn:none> <urn:none> }")
            .unwrap()
    });
    assert_eq!(nothing.rows.len(), 0);

    // 2b. copy-cost SCALING: same tree, 50 filler triples per document (~46k quads).
    //     build_active is O(authorized data) PER QUERY — this is the number the L1
    //     zero-copy view deletes.
    let fat = Graph::load_dataset(&sparq_solid::fixture::wac_fixture_sized(50), "nquads").unwrap();
    let fat_quads: usize = fat.named.iter().map(|(_, g)| g.len()).sum();
    let mut fat_store = PodStore::new(fat);
    fat_store.materialize_wac().unwrap();
    println!("\nfat fixture: {fat_quads} quads");
    let fat_direct = timed("FULL dataset query, fat fixture", || {
        sparq_engine::query(
            &fat_store.graph,
            "SELECT ?title WHERE { GRAPH ?g { ?s <https://ex.dev/ns#title> ?title } }",
        )
        .unwrap()
    });
    println!("    rows: {}", fat_direct.rows.len());
    let fat_v1 = timed("query_as (v1 copy path), fat fixture", || {
        fat_store.query_as(&alice, Mode::Read, TITLES).unwrap()
    });
    println!("    rows: {}", fat_v1.rows.len());
    let fat_nothing = timed("v1 copy cost isolated, fat fixture", || {
        fat_store
            .query_as(&alice, Mode::Read, "SELECT ?x WHERE { ?x <urn:none> <urn:none> }")
            .unwrap()
    });
    assert_eq!(fat_nothing.rows.len(), 0);

    // 3. session graph-set: cold vs cached
    let bob = Session { agent: Some(sparq_solid::fixture::BOB), client: None };
    let t = Instant::now();
    let cold = store.accessible(&bob, Mode::Read);
    println!("{:64} {:10.3} ms", "session graph-set, cold (AuthIndex walk)", t.elapsed().as_secs_f64() * 1e3);
    let t = Instant::now();
    let cached = store.accessible(&bob, Mode::Read);
    println!("{:64} {:10.4} ms", "session graph-set, cached", t.elapsed().as_secs_f64() * 1e3);
    assert_eq!(cold.len(), cached.len());
}
