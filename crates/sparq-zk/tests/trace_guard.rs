//! Integration tests of the zk-trace consumer and the Q6 blank-node
//! anti-correlation machinery (plan §2.4): traced execution + leaf
//! resolution, the prover-side cross-graph bnode-join rejection (layer 2),
//! the verifier-side static re-check (layer 3), and the prover↔verifier
//! handshake over public attributions.

use oxrdf::NamedNode;
use sparq_zk::encode::salt_from_bytes;
use sparq_zk::trace::{TraceError, ZkDataset};
use sparq_zk::verify;
use std::collections::BTreeSet;

/// Well-behaved pair of credentials: an employment credential
/// (bnode-structured address) and a public org registry whose own bnode uses
/// a non-colliding label. Cross-graph joins happen on IRIs only.
const CLEAN: &str = r#"
<http://ex/alice> <http://ex/worksAt> <http://ex/acme> <http://ex/g/employment> .
<http://ex/alice> <http://ex/address> _:addr <http://ex/g/employment> .
_:addr <http://ex/city> "Springfield" <http://ex/g/employment> .
<http://ex/acme> <http://ex/sector> "robotics" <http://ex/g/registry> .
<http://ex/acme> <http://ex/hq> _:hq <http://ex/g/registry> .
_:hq <http://ex/city> "Shelbyville" <http://ex/g/registry> .
"#;

/// The attack case: the SAME bnode label `_:addr` occurs in both graphs
/// (RDFC10 canonical labels recur across graphs too — `c14n0` is
/// everywhere). RDF merge semantics make these two distinct nodes; the
/// engine's label-identifying union would correlate them.
const COLLIDING: &str = r#"
<http://ex/alice> <http://ex/address> _:addr <http://ex/g/employment> .
_:addr <http://ex/city> "Springfield" <http://ex/g/employment> .
<http://ex/acme> <http://ex/hq> _:addr <http://ex/g/registry> .
_:addr <http://ex/city> "Shelbyville" <http://ex/g/registry> .
"#;

fn dataset(nquads: &str) -> ZkDataset {
    ZkDataset::from_dataset_str(
        nquads,
        "n-quads",
        &[
            NamedNode::new("http://ex/g/employment").unwrap(),
            NamedNode::new("http://ex/g/registry").unwrap(),
        ],
        &[salt_from_bytes(&[1u8; 32]), salt_from_bytes(&[2u8; 32])],
    )
    .unwrap()
}

#[test]
fn iri_cross_graph_join_is_traced_and_resolved() {
    let ds = dataset(CLEAN);
    // ?org joins across graphs on an IRI — semantically correct merge
    // behaviour, must succeed.
    let exec = ds
        .query_traced(
            "SELECT ?sector WHERE { <http://ex/alice> <http://ex/worksAt> ?org . ?org <http://ex/sector> ?sector }",
        )
        .unwrap();
    assert_eq!(exec.result.rows.len(), 1);
    assert_eq!(exec.patterns.len(), 2);
    // The two patterns are attributed across the two different graphs and
    // every traced triple resolved to at least one (graph, leaf) reference.
    let attrs = exec.attributions();
    let all: BTreeSet<usize> = attrs.iter().flat_map(|(_, s)| s.iter().copied()).collect();
    assert_eq!(all, BTreeSet::from([0, 1]));
    for (_, set) in &attrs {
        assert!(!set.is_empty());
    }
    // Leaf indices are in range of each graph's commitment.
    for rp in &exec.patterns {
        for refs in &rp.leaves {
            for r in refs {
                assert!(r.leaf < ds.graphs[r.graph].commitment.leaves.len());
            }
        }
    }
}

#[test]
fn traced_input_set_is_sufficient() {
    // Differential sufficiency: re-running the query over ONLY the traced
    // triples must reproduce the result rows.
    let ds = dataset(CLEAN);
    let q = "SELECT ?city WHERE { <http://ex/alice> <http://ex/address> ?a . ?a <http://ex/city> ?city }";
    let exec = ds.query_traced(q).unwrap();
    let replay = sparq_zk::trace::traced_triples_store(&exec.trace);
    let replayed = sparq_engine::query(&replay, q).unwrap();
    let rows = |r: &sparq_engine::QueryResult| -> BTreeSet<String> {
        r.rows.iter().map(|row| format!("{row:?}")).collect()
    };
    assert_eq!(rows(&exec.result), rows(&replayed));
    assert!(!exec.result.rows.is_empty());
}

#[test]
fn same_graph_bnode_join_is_allowed() {
    let ds = dataset(CLEAN);
    // ?a is bnode-valued, but every join pair is supported within a single
    // graph (labels don't collide across graphs here): the guard must NOT
    // fire, and the result is employment's city only.
    let exec = ds
        .query_traced(
            "SELECT ?city WHERE { <http://ex/alice> <http://ex/address> ?a . ?a <http://ex/city> ?city }",
        )
        .unwrap();
    assert_eq!(exec.result.rows.len(), 1);
    assert_eq!(
        exec.result.rows[0][0].as_ref().map(ToString::to_string),
        Some("\"Springfield\"".to_string())
    );
}

#[test]
fn cross_graph_bnode_join_rejected_by_prover() {
    let ds = dataset(COLLIDING);
    // ?h would equate employment's _:addr with registry's _:addr — same
    // label, distinct nodes under RDF merge. Must fail closed at
    // witness-build time.
    let err = ds
        .query_traced(
            "SELECT ?h WHERE { <http://ex/alice> <http://ex/address> ?h . <http://ex/acme> <http://ex/hq> ?h }",
        )
        .unwrap_err();
    match err {
        TraceError::CrossGraphBnodeJoin { variable, .. } => assert_eq!(variable, "h"),
        other => panic!("expected CrossGraphBnodeJoin, got: {other}"),
    }
}

#[test]
fn label_identified_union_leak_is_caught() {
    let ds = dataset(COLLIDING);
    // Subtler leak: the engine's union identifies the colliding labels, so
    // "?a city ?city" picks up registry's "Shelbyville" through employment's
    // _:addr. The supporting pair (employment address-triple, registry
    // city-triple) has disjoint attributions — the guard must reject the
    // whole plan rather than let the correlated row into a witness.
    let err = ds
        .query_traced(
            "SELECT ?city WHERE { <http://ex/alice> <http://ex/address> ?a . ?a <http://ex/city> ?city }",
        )
        .unwrap_err();
    assert!(
        matches!(err, TraceError::CrossGraphBnodeJoin { ref variable, .. } if variable == "a"),
        "expected CrossGraphBnodeJoin on ?a, got: {err}"
    );
}

#[test]
fn verifier_independently_requires_obligation_prover_handshake() {
    let ds = dataset(CLEAN);
    let q = "SELECT ?sector WHERE { <http://ex/alice> <http://ex/worksAt> ?org . ?org <http://ex/sector> ?sector }";
    let exec = ds.query_traced(q).unwrap();

    // Manifest construction (prover side): patterns in QUERY order, each
    // attributed by PatternKey lookup into the trace's public attributions —
    // the verifier-derived keys must match the engine-recorded keys exactly.
    let query_patterns = verify::fragment_patterns(q).unwrap();
    let traced = exec.attributions();
    let attributions: Vec<BTreeSet<usize>> = query_patterns
        .iter()
        .map(|k| {
            traced
                .iter()
                .find(|(tk, _)| tk == k)
                .map(|(_, s)| s.clone())
                .unwrap_or_else(|| panic!("query pattern not traced: {:?}", k))
        })
        .collect();

    // Verifier side: the ?org edge crosses graphs, so an obligation is
    // required; a manifest declaring none is rejected…
    let err = verify::recheck(q, &attributions, &[]).unwrap_err();
    assert!(matches!(err, verify::VerifyError::MissingObligation(_)));
    // …and one declaring it is accepted.
    let edge = verify::JoinEdge {
        variable: "org".into(),
        patterns: (0, 1),
    };
    let required = verify::recheck(q, &attributions, std::slice::from_ref(&edge)).unwrap();
    assert_eq!(required, vec![edge]);
}

#[test]
fn verifier_rejects_graph_keyword() {
    // GRAPH is contradictory with graph-set privacy (plan §2.4) — the
    // verifier must refuse to derive obligations for it at all.
    let err =
        verify::fragment_patterns("SELECT * WHERE { GRAPH <http://ex/g/employment> { ?s ?p ?o } }")
            .unwrap_err();
    assert!(matches!(err, verify::VerifyError::UnsupportedFragment(_)));
}
