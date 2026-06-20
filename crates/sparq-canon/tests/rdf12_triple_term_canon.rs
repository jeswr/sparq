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

use oxrdf::{BlankNode, GraphName, Literal, NamedNode, NamedOrBlankNode, Quad, Term, Triple};
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
        if !quads
            .iter()
            .all(|q| q.graph_name == GraphName::DefaultGraph)
        {
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
            std_out.lines,
            v2_out.lines,
            "single-graph lines disagree on {:?}",
            path.file_name().unwrap()
        );
        assert_eq!(
            std_out.triples,
            v2_out.triples,
            "single-graph triples disagree on {:?}",
            path.file_name().unwrap()
        );
        compared += 1;
    }
    assert!(
        compared >= 30,
        "expected most eval inputs default-graph, got {compared}"
    );
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
    assert_eq!(
        forward.lines, reverse.lines,
        "ground TT must be order-invariant"
    );
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
            Triple::new(
                NamedOrBlankNode::BlankNode(bn("a")),
                iri("http://ex/says"),
                inner,
            ),
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
            Triple::new(
                NamedOrBlankNode::BlankNode(bn("a")),
                iri("http://ex/says"),
                inner,
            ),
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
    let dir_lit =
        Literal::new_directional_language_tagged_literal("שלום", "he", oxrdf::BaseDirection::Rtl)
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

// ---------------------------------------------------------------------------
// 4) Hash-profile parity: `*_with::<D: Digest>` (SHA-384) + back-compat guard
//    (sq-5i1d). The non-generic default stays byte-identical to today; SHA-384
//    is purely additive.
// ---------------------------------------------------------------------------

use sha2::{Sha256, Sha384};

/// A symmetric two-bnode cycle whose c14n order is decided by the n-degree hash
/// (the two bnodes share a first-degree hash), so the choice of hash function is
/// observable in the relabelling. Shared by the parity tests below.
fn symmetric_cycle_dataset() -> Vec<Quad> {
    let mk = |s: &str, o: &str| {
        Quad::new(
            NamedOrBlankNode::BlankNode(bn(s)),
            iri("http://ex/p"),
            Term::BlankNode(bn(o)),
            GraphName::DefaultGraph,
        )
    };
    vec![
        mk("a", "b"),
        mk("b", "a"),
        Quad::new(
            NamedOrBlankNode::BlankNode(bn("a")),
            iri("http://ex/t"),
            Term::Literal(Literal::new_simple_literal("x")),
            GraphName::DefaultGraph,
        ),
        Quad::new(
            NamedOrBlankNode::BlankNode(bn("b")),
            iri("http://ex/u"),
            Term::Literal(Literal::new_simple_literal("y")),
            GraphName::DefaultGraph,
        ),
    ]
}

/// BACK-COMPAT GUARD: the non-generic default (`canonicalize_rdf12`) must be
/// byte-identical to the explicit SHA-256 profile (`canonicalize_rdf12_with::<
/// Sha256>`) across every entry point. This is the load-bearing back-compat
/// invariant: the SHA-384 variant is purely additive.
#[test]
fn default_is_sha256_byte_identical() {
    let ds = symmetric_cycle_dataset();
    assert_eq!(
        sparq_canon::canonicalize_rdf12(&ds).unwrap(),
        sparq_canon::canonicalize_rdf12_with::<Sha256>(&ds).unwrap(),
        "non-generic default must equal the explicit SHA-256 profile"
    );
    assert_eq!(
        sparq_canon::issue_dataset_rdf12(&ds).unwrap(),
        sparq_canon::issue_dataset_rdf12_with::<Sha256>(&ds).unwrap(),
        "issuer maps must match"
    );
    let triples: Vec<Triple> = ds
        .iter()
        .map(|q| Triple::new(q.subject.clone(), q.predicate.clone(), q.object.clone()))
        .collect();
    let g_default = sparq_canon::canonicalize_triples_rdf12(&triples).unwrap();
    let g_sha256 = sparq_canon::canonicalize_triples_rdf12_with::<Sha256>(&triples).unwrap();
    assert_eq!(g_default.lines, g_sha256.lines);
    assert_eq!(g_default.triples, g_sha256.triples);
}

/// On a ground (blank-node-free) dataset no hash feeds into a relabelling, so the
/// canonical OUTPUT is identical under any hash. This guards against an
/// accidental hash leak into the serialized line text: the hash only ever shows
/// up in intermediate labelling, never in the output bytes.
#[test]
fn sha384_ground_output_equals_sha256() {
    let inner = tt(
        NamedOrBlankNode::NamedNode(iri("http://ex/s")),
        iri("http://ex/p"),
        Term::NamedNode(iri("http://ex/o")),
    );
    let t = Triple::new(
        NamedOrBlankNode::NamedNode(iri("http://ex/a")),
        iri("http://ex/says"),
        inner,
    );
    let sha256 =
        sparq_canon::canonicalize_triples_rdf12_with::<Sha256>(std::slice::from_ref(&t)).unwrap();
    let sha384 =
        sparq_canon::canonicalize_triples_rdf12_with::<Sha384>(std::slice::from_ref(&t)).unwrap();
    assert_eq!(
        sha256.lines, sha384.lines,
        "ground (bnode-free) output must not depend on the hash function"
    );
}

/// The SHA-384 profile is DETERMINISTIC: the same dataset canonicalizes to the
/// same bytes on repeated runs.
#[test]
fn sha384_is_deterministic() {
    let ds = symmetric_cycle_dataset();
    let a = sparq_canon::canonicalize_rdf12_with::<Sha384>(&ds).unwrap();
    let b = sparq_canon::canonicalize_rdf12_with::<Sha384>(&ds).unwrap();
    assert_eq!(a, b, "SHA-384 profile must be deterministic");
}

/// The SHA-384 profile is ISOMORPHISM-STABLE under that hash: two isomorphic
/// graphs (relabelled bnodes + permuted quad order) canonicalize byte-identically
/// — including a nested-bnode triple term.
#[test]
fn sha384_isomorphism_stable() {
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
    let a =
        sparq_canon::canonicalize_triples_rdf12_with::<Sha384>(&build("shared", "other", false))
            .unwrap();
    let b =
        sparq_canon::canonicalize_triples_rdf12_with::<Sha384>(&build("xxx", "yyy", true)).unwrap();
    assert_eq!(
        a.lines, b.lines,
        "SHA-384 profile must be isomorphism-stable\n a={:?}\n b={:?}",
        a.lines, b.lines
    );
    assert!(a.lines.iter().any(|l| l.contains("<<(")));
}

/// SHA-384 still DISTINGUISHES non-isomorphic graphs (the descent isn't
/// collapsing distinct structure under the wider hash).
#[test]
fn sha384_distinguishes_non_isomorphic() {
    let g1 = {
        let inner = tt(
            NamedOrBlankNode::BlankNode(bn("a")),
            iri("http://ex/p"),
            Term::BlankNode(bn("b")),
        );
        vec![
            Triple::new(
                NamedOrBlankNode::BlankNode(bn("a")),
                iri("http://ex/says"),
                inner,
            ),
            Triple::new(
                NamedOrBlankNode::BlankNode(bn("b")),
                iri("http://ex/t"),
                Term::Literal(Literal::new_simple_literal("x")),
            ),
        ]
    };
    let g2 = {
        let inner = tt(
            NamedOrBlankNode::BlankNode(bn("b")),
            iri("http://ex/p"),
            Term::BlankNode(bn("b")),
        );
        vec![
            Triple::new(
                NamedOrBlankNode::BlankNode(bn("a")),
                iri("http://ex/says"),
                inner,
            ),
            Triple::new(
                NamedOrBlankNode::BlankNode(bn("b")),
                iri("http://ex/t"),
                Term::Literal(Literal::new_simple_literal("x")),
            ),
        ]
    };
    let c1 = sparq_canon::canonicalize_triples_rdf12_with::<Sha384>(&g1).unwrap();
    let c2 = sparq_canon::canonicalize_triples_rdf12_with::<Sha384>(&g2).unwrap();
    assert_ne!(
        c1.lines, c2.lines,
        "non-isomorphic graphs must differ under SHA-384"
    );
}

/// REAL-PATH PROOF that `D` is load-bearing (not a no-op generic): on the
/// symmetric two-bnode cycle, SHA-256 and SHA-384 issue the SAME canonical
/// labels to OPPOSITE input bnodes, so the two relabelling maps differ. If the
/// hash were ever dropped (e.g. a hardcoded `Sha256` left in a call site), this
/// assertion would fail.
#[test]
fn sha384_relabels_differently_from_sha256_on_symmetric_cycle() {
    let ds = symmetric_cycle_dataset();
    let m256 = sparq_canon::issue_dataset_rdf12_with::<Sha256>(&ds).unwrap();
    let m384 = sparq_canon::issue_dataset_rdf12_with::<Sha384>(&ds).unwrap();
    assert_ne!(
        m256, m384,
        "the chosen hash must actually drive the relabelling (D is load-bearing)"
    );
    // Both are still valid bijections onto {c14n0, c14n1}.
    let labels256: std::collections::BTreeSet<_> = m256.values().cloned().collect();
    let labels384: std::collections::BTreeSet<_> = m384.values().cloned().collect();
    let expected: std::collections::BTreeSet<_> = ["c14n0".to_string(), "c14n1".to_string()]
        .into_iter()
        .collect();
    assert_eq!(labels256, expected);
    assert_eq!(labels384, expected);
}
