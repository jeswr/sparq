//! [SONNET-4.6] (sq-7d3dj.30.15) Compile-time tripwire for the server's
//! `dp-planner` forwarding edge.
//!
//! This file is gated on the server feature so deliberate lean builds remain valid.
//! Removing `dp-planner` from `default` therefore silently skips it; it does not prove
//! default-set membership. The per-crate default lanes, which do not pass
//! `--no-default-features`, are where the forwarding witness is authoritative. A
//! whole-workspace build may also unify the engine feature through another crate.
#![cfg(feature = "dp-planner")]

fn result_bag(graph: &sparq_core::Graph, query: &str) -> Vec<String> {
    let mut rows: Vec<String> = sparq_engine::query(graph, query)
        .expect("query")
        .rows
        .iter()
        .map(|row| format!("{:?}", row))
        .collect();
    rows.sort();
    rows
}

/// Compile-time forwarding witness plus a smoke-level query sanity check.
///
/// `without_dp_planner` only exists when `sparq-engine/dp-planner` is enabled,
/// so a broken forwarding edge fails this test at compile time. The query comparison
/// checks result equivalence but does not prove that DPccp accepted this particular BGP.
#[test]
fn dp_planner_is_forwarded_and_default_query_is_result_equivalent() {
    let graph = sparq_core::Graph::load_str(
        "<http://ex/a> <http://ex/p> <http://ex/b> .\n\
         <http://ex/b> <http://ex/q> <http://ex/c> .\n\
         <http://ex/c> <http://ex/r> <http://ex/d> .\n",
        "ntriples",
    )
    .expect("load fixture");
    let query = "SELECT ?a ?d WHERE {
        ?a <http://ex/p> ?b .
        ?b <http://ex/q> ?c .
        ?c <http://ex/r> ?d .
    }";

    let planned = result_bag(&graph, query);
    let greedy = sparq_engine::without_dp_planner(|| result_bag(&graph, query));

    assert_eq!(planned, greedy, "DPccp must preserve the result bag");
    assert!(!planned.is_empty(), "fixture must exercise a non-empty query path");
}
