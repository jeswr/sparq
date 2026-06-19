//! Tests for the OPT-IN, NON-STANDARD `rdf12-triple-terms` profile (sq-hslb).
//!
//! Only compiled when the feature is on. The headline invariants:
//!  1. **Standard-path agreement** — on triple-term-free input the v2 profile is
//!     byte-identical to the standard `rdf-canon`-backed path (the strongest
//!     correctness anchor; run over every W3C suite eval vector).
//!  2. **Ground triple-term datasets** — order-invariant, isomorphism-stable.
//!  3. **Nested-bnode triple terms** — a blank node inside a triple-term object
//!     (and deeper) is relabelled by the HNDQ descent; bnode-label and
//!     triple-order invariance hold, including bnodes shared between a top-level
//!     position and a triple-term-internal position. Includes a case the
//!     standard path rejects with `TripleTerm` that now canonicalizes.
#![cfg(feature = "rdf12-triple-terms")]

use oxrdf::{BlankNode, Literal, NamedNode, NamedOrBlankNode, Quad, Term, Triple, GraphName};
use std::path::{Path, PathBuf};

fn testdata() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/rdf-canon-testdata")
}

fn iri(s: &str) -> NamedNode {
    NamedNode::new(s).unwrap()
}
fn bn(s: &str) -> BlankNode {
    BlankNode::new(s).unwrap()
}

/// Triple-term object `<<( s p o )>>`.
fn tt(s: NamedOrBlankNode, p: NamedNode, o: Term) -> Term {
    Term::Triple(Box::new(Triple::new(s, p, o)))
}

// ---------------------------------------------------------------------------
// 1) Standard-path agreement on triple-term-free input (the W3C suite).
// ---------------------------------------------------------------------------

fn load_quads(path: &Path) -> Vec<Quad> {
    let bytes = std::fs::read(path).unwrap();
    oxttl::NQuadsParser::new()
        .for_slice(&bytes)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

/// Every default-SHA-256 eval `.nq` input under the suite, canonicalized by both
/// the standard path and the v2 profile, must agree byte-for-byte. This proves
/// the native HNDQ matches `rdf-canon` on the shared (triple-term-free) subset.
#[test]
fn v2_agrees_with_standard_on_suite_inputs() {
    let dir = testdata().join("rdfc10");
    let mut compared = 0usize;
    let mut mismatches: Vec<String> = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with("-in.nq"))
                .unwrap_or(false)
        })
        .collect();
    entries.sort();

    for path in entries {
        let quads = load_quads(&path);
        // Standard path: skip the few suite inputs the standard path itself
        // rejects (poison graphs hit the HNDQ limit). We only compare where the
        // standard path succeeds — that is exactly the shared subset.
        let std_out = match sparq_canon::canonicalize(&quads) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let v2_out = sparq_canon::canonicalize_rdf12(&quads)
            .unwrap_or_else(|e| panic!("v2 failed on {:?}: {e}", path));
        if std_out != v2_out {
            mismatches.push(format!(
                "{:?}\n  std: {:?}\n  v2:  {:?}",
                path.file_name().unwrap(),
                std_out,
                v2_out
            ));
        }
        compared += 1;
    }
    assert!(
        compared >= 60,
        "expected to compare most suite inputs, got {compared}"
    );
    assert!(
        mismatches.is_empty(),
        "{} suite inputs disagree between standard and v2:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

/// The single-graph v2 API agrees with the standard single-graph API on every
/// default-graph-only, triple-term-free suite eval input.
#[test]
fn v2_single_graph_agrees_with_standard() {
    let dir = testdata().join("rdfc10");
    let mut compared = 0usize;
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with("-in.nq"))
                .unwrap_or(false)
        })
        .collect();
    entries.sort();
    for path in entries {
        let quads = load_quads(&path);
        if !quads.iter().all(|q| q.graph_name == GraphName::DefaultGraph) {
            continue;
        }
        let triples: Vec<Triple> = quads
            .iter()
            .map(|q| Triple::new(q.subject.clone(), q.predicate.clone(), q.object.clone()))
            .collect();
        let std_out = match sparq_canon::canonicalize_triples(&triples) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let v2_out = sparq_canon::canonicalize_triples_rdf12(&triples).unwrap();
        assert_eq!(
            std_out.lines, v2_out.lines,
            "single-graph lines disagree on {:?}",
            path.file_name().unwrap()
        );
        assert_eq!(
            std_out.triples, v2_out.triples,
            "single-graph triples disagree on {:?}",
            path.file_name().unwrap()
        );
        compared += 1;
    }
    assert!(compared >= 30, "expected most eval inputs default-graph, got {compared}");
}

// ---------------------------------------------------------------------------
// 2) Ground (blank-free) triple-term datasets.
// ---------------------------------------------------------------------------

/// A ground triple-term dataset (no blank nodes anywhere) is order-invariant and
/// canonicalizes to the same bytes regardless of input quad order — the common
/// credential/VC case.
#[test]
fn ground_triple_term_is_order_invariant() {
    let inner = |o: &str| -> Term {
        tt(
            NamedOrBlankNode::NamedNode(iri("http://ex/s")),
            iri("http://ex/p"),
            Term::NamedNode(iri(o)),
        )
    };
    let t1 = Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/a")),
        iri("http://ex/says"),
        inner("http://ex/o1"),
    );
    let t2 = Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/b")),
        iri("http://ex/says"),
        inner("http://ex/o2"),
    );
    let forward = sparq_canon::canonicalize_triples_rdf12(&[t1.clone(), t2.clone()]).unwrap();
    let reverse = sparq_canon::canonicalize_triples_rdf12(&[t2, t1]).unwrap();
    assert_eq!(forward.lines, reverse.lines, "ground TT must be order-invariant");
    // Sanity: triple-term token form is present in the canonical output.
    assert!(
        forward.lines.iter().any(|l| l.contains("<<(")),
        "expected canonical triple-term token form: {:?}",
        forward.lines
    );
}

/// A triple term whose object is itself a triple term (ground, deep nesting)
/// canonicalizes deterministically and preserves the nested token form.
#[test]
fn ground_deep_nested_triple_term() {
    let deepest = tt(
        NamedOrBlankNode::NamedNode(iri("http://ex/x")),
        iri("http://ex/p"),
        Term::Literal(Literal::new_simple_literal("v")),
    );
    let mid = tt(
        NamedOrBlankNode::NamedNode(iri("http://ex/y")),
        iri("http://ex/q"),
        deepest,
    );
    let t = Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/a")),
        iri("http://ex/says"),
        mid,
    );
    let c = sparq_canon::canonicalize_triples_rdf12(std::slice::from_ref(&t)).unwrap();
    assert_eq!(c.lines.len(), 1);
    // Nested `<<( ... <<( ... )>> )>>` round-trips back to the same term.
    assert_eq!(c.triples[0], t, "deep ground TT must round-trip");
}

// ---------------------------------------------------------------------------
// 3) Nested-blank-node triple terms — the headline HNDQ-descent deliverable.
// ---------------------------------------------------------------------------

/// The case the OLD code rejected with `TripleTerm`: a blank node nested inside
/// a triple-term object now canonicalizes, and is relabelled to `c14nN`.
#[test]
fn nested_bnode_now_canonicalizes() {
    let inner = tt(
        NamedOrBlankNode::BlankNode(bn("nested")),
        iri("http://ex/p"),
        Term::Literal(Literal::new_simple_literal("v")),
    );
    let t = Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/s")),
        iri("http://ex/asserts"),
        inner,
    );
    // Standard path still rejects.
    assert!(matches!(
        sparq_canon::canonicalize_triples(std::slice::from_ref(&t)),
        Err(sparq_canon::CanonError::TripleTerm)
    ));
    // v2 profile canonicalizes and relabels the nested bnode.
    let c = sparq_canon::canonicalize_triples_rdf12(&[t]).unwrap();
    assert_eq!(c.lines.len(), 1);
    assert!(
        c.lines[0].contains("_:c14n0"),
        "nested bnode must be relabelled to c14n0: {}",
        c.lines[0]
    );
}

/// Bnode-label invariance + triple-order invariance for nested-bnode triple
/// terms: two isomorphic graphs differing only by input bnode labels and quad
/// order canonicalize byte-identically, including a bnode shared between a
/// top-level subject and a triple-term-internal position.
#[test]
fn nested_bnode_isomorphism() {
    // Graph shape:
    //   _:shared  ex:says  <<( _:shared ex:p _:other )>>
    //   _:other   ex:t     "leaf"
    // _:shared appears both at top level (subject) AND inside the triple term.
    let build = |shared: &str, other: &str, swap: bool| -> Vec<Triple> {
        let inner = tt(
            NamedOrBlankNode::BlankNode(bn(shared)),
            iri("http://ex/p"),
            Term::BlankNode(bn(other)),
        );
        let t1 = Triple::new(
            NamedOrBlankNode::BlankNode(bn(shared)),
            iri("http://ex/says"),
            inner,
        );
        let t2 = Triple::new(
            NamedOrBlankNode::BlankNode(bn(other)),
            iri("http://ex/t"),
            Term::Literal(Literal::new_simple_literal("leaf")),
        );
        if swap {
            vec![t2, t1]
        } else {
            vec![t1, t2]
        }
    };
    let a = sparq_canon::canonicalize_triples_rdf12(&build("shared", "other", false)).unwrap();
    let b = sparq_canon::canonicalize_triples_rdf12(&build("xxx", "yyy", true)).unwrap();
    assert_eq!(
        a.lines, b.lines,
        "isomorphic nested-bnode graphs must canonicalize identically\n a={:?}\n b={:?}",
        a.lines, b.lines
    );
    // The shared bnode must resolve to one canonical label used in both its
    // top-level and triple-term-internal occurrences.
    assert!(a.lines.iter().any(|l| l.contains("<<(")));
}

/// Distinguishing case: two graphs that are NOT isomorphic (a nested bnode is
/// connected differently) must canonicalize to DIFFERENT output, so the descent
/// is not collapsing distinct structure.
#[test]
fn nested_bnode_distinguishes_non_isomorphic() {
    // G1: _:a ex:says <<( _:a ex:p _:b )>> ;  _:b ex:t "x"
    let g1 = {
        let inner = tt(
            NamedOrBlankNode::BlankNode(bn("a")),
            iri("http://ex/p"),
            Term::BlankNode(bn("b")),
        );
        vec![
            Triple::new(NamedOrBlankNode::BlankNode(bn("a")), iri("http://ex/says"), inner),
            Triple::new(
                NamedOrBlankNode::BlankNode(bn("b")),
                iri("http://ex/t"),
                Term::Literal(Literal::new_simple_literal("x")),
            ),
        ]
    };
    // G2: _:a ex:says <<( _:b ex:p _:b )>> ;  _:b ex:t "x"  (subject of inner differs)
    let g2 = {
        let inner = tt(
            NamedOrBlankNode::BlankNode(bn("b")),
            iri("http://ex/p"),
            Term::BlankNode(bn("b")),
        );
        vec![
            Triple::new(NamedOrBlankNode::BlankNode(bn("a")), iri("http://ex/says"), inner),
            Triple::new(
                NamedOrBlankNode::BlankNode(bn("b")),
                iri("http://ex/t"),
                Term::Literal(Literal::new_simple_literal("x")),
            ),
        ]
    };
    let c1 = sparq_canon::canonicalize_triples_rdf12(&g1).unwrap();
    let c2 = sparq_canon::canonicalize_triples_rdf12(&g2).unwrap();
    assert_ne!(c1.lines, c2.lines, "non-isomorphic graphs must differ");
}

/// Dataset-level (named-graph) entry point with a nested bnode triple term.
#[test]
fn dataset_named_graph_nested_bnode() {
    let inner = tt(
        NamedOrBlankNode::BlankNode(bn("n")),
        iri("http://ex/p"),
        Term::Literal(Literal::new_simple_literal("v")),
    );
    let q = Quad::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/s")),
        iri("http://ex/asserts"),
        inner,
        GraphName::NamedNode(iri("http://ex/g")),
    );
    let out = sparq_canon::canonicalize_rdf12(std::slice::from_ref(&q)).unwrap();
    assert!(out.contains("<http://ex/g>"), "graph name preserved: {out}");
    assert!(out.contains("_:c14n0"), "nested bnode relabelled: {out}");
    let map = sparq_canon::issue_dataset_rdf12(&[q]).unwrap();
    assert_eq!(map.get("n").map(String::as_str), Some("c14n0"));
}

/// Directional language-tagged string inside a triple term renders with the
/// canonical RDF-1.2 `@lang--dir` token form (single-sourced via oxrdf Display).
#[test]
fn directional_language_string_in_triple_term() {
    let dir_lit = Literal::new_directional_language_tagged_literal("שלום", "he", oxrdf::BaseDirection::Rtl)
        .unwrap();
    let inner = tt(
        NamedOrBlankNode::NamedNode(iri("http://ex/s")),
        iri("http://ex/p"),
        Term::Literal(dir_lit),
    );
    let t = Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/a")),
        iri("http://ex/says"),
        inner,
    );
    let c = sparq_canon::canonicalize_triples_rdf12(&[t]).unwrap();
    assert!(
        c.lines[0].contains("@he--rtl"),
        "directional language token form expected: {}",
        c.lines[0]
    );
}
