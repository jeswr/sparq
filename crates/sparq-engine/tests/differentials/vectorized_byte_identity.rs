//! [OPUS-4.8] (sq-pntvh.1) M4 **Phase 1**: the vectorized differential BYTE-IDENTITY
//! harness — the acceptance gate every later vector-at-a-time (M4) phase leans on
//! (`research/vector-at-a-time-m4.md` §3.2, §5.1, §6 Phase 1).
//!
//! 🤖 SPARQ agent. The M4 roadmap's load-bearing merge-blocking property is that a
//! columnar (vector-at-a-time) operator is an *equivalence-checked accelerator*: it
//! must return the byte-for-byte same SPARQL-1.1-JSON result the feature-OFF row
//! evaluator returns. This file lands that harness FIRST — before any columnar seam
//! exists in the evaluator — so every later phase (decode primitive → residual FILTER
//! → reducer aggregate → morsel pipeline) has a ready differential to gate against.
//!
//! ## The invariant is OPERATOR-level, not cross-query (a Phase-1 finding)
//!
//! The intuitive reading of §3.2 — "run the query with the feature ON, run the
//! equivalent scalar `FILTER` query with it OFF, assert byte-identical" — is **unsound
//! as a Phase-1 differential** and this harness deliberately does not do it. A sargable
//! numeric `FILTER` (`?o > 30`) is *pushed into the scan* by the planner
//! (`split_sargable`, `research/vector-at-a-time-m4.md` §0 refinement 1), which makes
//! the filtered query iterate a numeric-range scan in *value* order while the unfiltered
//! scan iterates in dictionary-id order — so the two enumerate the same bag in a
//! DIFFERENT order and are (correctly) not byte-identical. That difference is a PLANNER
//! choice (which physical scan), orthogonal to what M4 changes (the FILTER/aggregate
//! *operator*'s inner loop). The sound invariant, and the one this harness asserts, is
//! **operator-level**: given the *same input batch*, the columnar operator's serialised
//! survivors are byte-identical to the row operator's — order-exact. That is exactly the
//! shape a later phase reuses: the columnar seam receives the plan's batch and must
//! reproduce the row operator's output on it (at the full-query level the byte-identity
//! then holds because the SAME query fixes the SAME plan, toggling only the operator).
//!
//! ## What Phase 1 proves
//!
//! * **Operator equivalence + row order** — the REAL `DataChunk::select_numeric` kernel,
//!   over a batch a REAL SPARQL scan produced (`query`), keeps exactly the rows (in the
//!   same scan order) the row `FILTER` semantics keep, and the two serialise
//!   byte-identically through the engine's own writer.
//! * **NaN / type-error** — a non-numeric column selects nothing, exactly as a scalar
//!   numeric `FILTER` over a non-numeric operand excludes every row (type error → out).
//! * **Serialiser fidelity** — the columnar path's serialisation (`to_sparql_json_rows`)
//!   is anchored to the REAL `query_json` for the plan-stable select-all and the empty
//!   boundaries, so byte-identity is against the true end-use output, not just a helper.
//! * **Unbound handling** — the Phase-1 corpus filters only on BGP-bound columns
//!   (OPTIONAL / `NO_ID` sentinels stay row-only until a later phase, §4).
//!
//! Only built under the opt-in `vectorized` feature (the whole file is empty otherwise,
//! so the default `cargo test` is unaffected); the `sparq-engine (vectorized)` leg in
//! `.github/feature-matrix.d/sparq-engine.yml` runs it in CI. Sibling of
//! `tests/vectorized_exec_differential.rs` (which proves *multiset* equality of the
//! kernels); this file adds the stronger *byte-identity of the serialised result*
//! differential the M4 acceptance gate requires.
#![cfg(feature = "vectorized")]

use sparq_core::dict::Id;
use sparq_core::Graph;
use sparq_engine::json::to_sparql_json_rows;
use sparq_engine::{query, query_json, DataChunk, QueryResult, VecCmp};

const XSD_INT: &str = "http://www.w3.org/2001/XMLSchema#integer";
const PFX: &str = "PREFIX ex: <http://ex/> PREFIX xsd: <http://www.w3.org/2001/XMLSchema#> ";

/// A graph of `n` `(ex:s{i}, ex:age, i)` integer-literal triples, plus two NON-numeric
/// `ex:name` string triples so a column built over the names exercises the kernel's
/// `numeric_value(...) == None` reject path (mirroring the row evaluator excluding a
/// non-numeric `FILTER` operand).
fn ages_graph(n: usize) -> Graph {
    let mut nt = String::new();
    for i in 0..n {
        nt.push_str(&format!(
            "<http://ex/s{}> <http://ex/age> \"{}\"^^<{}> .\n",
            i, i, XSD_INT
        ));
    }
    nt.push_str("<http://ex/p1> <http://ex/name> \"alice\" .\n");
    nt.push_str("<http://ex/p2> <http://ex/name> \"bob\" .\n");
    Graph::load_str(&nt, "ntriples").unwrap()
}

/// The materialised batch a SELECT produces (the input a columnar operator would receive)
/// plus the ascending selection a REAL columnar numeric FILTER keeps over `filter_col` of
/// it — driving the actual `chunk` kernel. Panics if the filter column is unbound in any
/// row (the Phase-1 corpus filters only on BGP-bound columns).
fn columnar_selection(
    g: &Graph,
    proj: &str,
    body: &str,
    filter_col: usize,
    cmp: VecCmp,
) -> (QueryResult, Vec<usize>) {
    let qr = query(g, &format!("{}SELECT {} WHERE {{ {} }}", PFX, proj, body)).unwrap();
    let col: Vec<Id> = qr
        .rows
        .iter()
        .map(|row| {
            let term = row[filter_col].as_ref().expect("filter column is bound");
            g.id_of(term).expect("a queried term is interned")
        })
        .collect();
    let n = qr.rows.len();
    let chunk = DataChunk::from_columns(vec![col], n).expect("uniform single column");
    let sel = chunk.select_numeric(g, 0, cmp);
    (qr, sel)
}

/// Serialise the FULL rows named by `sel` (ascending, order-preserving) out of a
/// materialised result, using the engine's own SPARQL-JSON writer — exactly as the row
/// path serialises its filtered result.
fn selection_json(qr: &QueryResult, sel: &[usize]) -> String {
    // Type inferred from `to_sparql_json_rows`'s `&[Vec<Option<Term>>]` parameter — no
    // need to name `oxrdf::Term` (and thus no direct oxrdf import in the test).
    let rows: Vec<_> = sel.iter().map(|&r| qr.rows[r].clone()).collect();
    to_sparql_json_rows(&qr.vars, &rows)
}

/// CORE DIFFERENTIAL (operator-level, order-exact): over the *same* materialised batch,
/// the REAL `select_numeric` kernel's serialised survivors are BYTE-IDENTICAL to the row
/// `FILTER` semantics' survivors — for every comparison, both boundaries, and single- and
/// two-column projections (the two-column ?s case pins prefix-factored IRI serialisation,
/// so byte-identity there also exercises the writer, not just the row set).
#[test]
fn columnar_filter_is_byte_identical_over_the_batch() {
    let g = ages_graph(200);

    type Case = (&'static str, &'static str, usize, VecCmp);
    let cases: &[Case] = &[
        ("?o", "?s ex:age ?o", 0, VecCmp::Gt(30.0)),
        ("?o", "?s ex:age ?o", 0, VecCmp::Ge(30.0)),
        ("?o", "?s ex:age ?o", 0, VecCmp::Lt(10.0)),
        ("?o", "?s ex:age ?o", 0, VecCmp::Le(10.0)),
        ("?o", "?s ex:age ?o", 0, VecCmp::Eq(42.0)),
        // Boundary: nothing passes (threshold above the max value).
        ("?o", "?s ex:age ?o", 0, VecCmp::Gt(1_000.0)),
        // Boundary: everything passes (threshold at/below the min value).
        ("?o", "?s ex:age ?o", 0, VecCmp::Ge(0.0)),
        // Two columns: filter on ?o (index 1), keep the ?s IRI aligned + in order.
        ("?s ?o", "?s ex:age ?o", 1, VecCmp::Ge(25.0)),
        ("?s ?o", "?s ex:age ?o", 1, VecCmp::Lt(3.0)),
    ];

    for (proj, body, filter_col, cmp) in cases {
        let (qr, columnar_sel) = columnar_selection(&g, proj, body, *filter_col, *cmp);

        // Faithful row-`FILTER` reference: keep row r iff its filter cell's numeric value
        // passes, IN SCAN ORDER — the engine's own `numeric_value` + the same comparison,
        // i.e. the residual-FILTER operator's per-row test (`ScanCmp`/`apply_filter`).
        let row_sel: Vec<usize> = (0..qr.rows.len())
            .filter(|&r| {
                let id = g.id_of(qr.rows[r][*filter_col].as_ref().unwrap()).unwrap();
                g.numeric_value(id).is_some_and(|x| cmp.test(x))
            })
            .collect();

        assert_eq!(
            columnar_sel, row_sel,
            "columnar selection must equal the row selection ORDER-EXACT for {:?}",
            cmp
        );
        assert_eq!(
            selection_json(&qr, &columnar_sel).as_bytes(),
            selection_json(&qr, &row_sel).as_bytes(),
            "columnar vs row FILTER JSON must be BYTE-IDENTICAL over the same batch for {:?}",
            cmp
        );
    }
}

/// SERIALISER ANCHOR: a select-all columnar filter over the unfiltered batch serialises
/// byte-identically to the REAL `query_json` of that same (unfiltered) query. Because it
/// is the SAME query on both sides there is no plan change, so this soundly ties the
/// columnar path + `to_sparql_json_rows` to the true end-use `query_json` output.
#[test]
fn columnar_select_all_is_byte_identical_to_query_json() {
    let g = ages_graph(120);
    for (proj, body) in [("?o", "?s ex:age ?o"), ("?s ?o", "?s ex:age ?o")] {
        // `Ge(0.0)` keeps every (non-negative) age → the columnar survivors ARE the whole
        // unfiltered batch, in scan order.
        let (qr, sel) = columnar_selection(&g, proj, body, if proj == "?o" { 0 } else { 1 }, VecCmp::Ge(0.0));
        assert_eq!(sel.len(), qr.rows.len(), "select-all keeps every row");
        let query_json_out =
            query_json(&g, &format!("{}SELECT {} WHERE {{ {} }}", PFX, proj, body)).unwrap();
        assert_eq!(
            selection_json(&qr, &sel).as_bytes(),
            query_json_out.as_bytes(),
            "columnar select-all must be byte-identical to query_json for `{}`",
            body
        );
    }
}

/// A non-numeric column yields byte-identical EMPTY documents on both paths: the kernel
/// rejects every non-numeric cell exactly as a scalar numeric `FILTER` over string
/// operands does (per-row type error → empty result). Anchored to the REAL `query_json`
/// of a scalar filter that returns empty (order is irrelevant for an empty result).
#[test]
fn non_numeric_column_is_byte_identical_empty_to_query_json() {
    let g = ages_graph(8);
    for (cmp, tail) in [
        (VecCmp::Gt(-1.0), "?s ex:name ?o FILTER(?o > -1)"),
        (VecCmp::Ge(-1.0), "?s ex:name ?o FILTER(?o >= -1)"),
        (VecCmp::Lt(1e9), "?s ex:name ?o FILTER(?o < 1000000000)"),
        (VecCmp::Eq(0.0), "?s ex:name ?o FILTER(?o = 0)"),
    ] {
        let (qr, sel) = columnar_selection(&g, "?o", "?s ex:name ?o", 0, cmp);
        assert!(sel.is_empty(), "kernel rejects every non-numeric cell for {:?}", cmp);
        let row_json = query_json(&g, &format!("{}SELECT ?o WHERE {{ {} }}", PFX, tail)).unwrap();
        assert_eq!(
            selection_json(&qr, &sel).as_bytes(),
            row_json.as_bytes(),
            "non-numeric column must produce a byte-identical EMPTY result for `{}`",
            tail
        );
        // Independent oracle: the REAL engine returns an empty bindings array for a string
        // FILTER, anchoring that the byte-identical pair above is specifically the EMPTY
        // document (byte-identity alone would not catch an equal-but-non-empty regression).
        // `contains` rather than `ends_with` so a future trailing-whitespace tweak in the JSON
        // writer does not make this brittle while still asserting emptiness.
        assert!(row_json.contains("\"bindings\":[]"), "the row result is empty too");
    }
}

/// The gather kernel (`apply_selection`) keeps the id column row-count-consistent with
/// the selection and aligned to the survivor set — covering `apply_selection` inside the
/// acceptance harness, not only in the sibling multiset suite.
#[test]
fn apply_selection_is_consistent_with_the_survivor_set() {
    let g = ages_graph(60);
    let (qr, sel) = columnar_selection(&g, "?o", "?s ex:age ?o", 0, VecCmp::Ge(25.0));
    let col: Vec<Id> = qr
        .rows
        .iter()
        .map(|row| g.id_of(row[0].as_ref().unwrap()).unwrap())
        .collect();
    let chunk = DataChunk::from_columns(vec![col.clone()], qr.rows.len()).unwrap();
    let gathered = chunk.apply_selection(&sel);
    assert_eq!(gathered.len(), sel.len(), "gather row count == selection length");
    let gathered_ids: Vec<Id> = gathered.column(0).unwrap().to_vec();
    let survivor_ids: Vec<Id> = sel.iter().map(|&r| col[r]).collect();
    assert_eq!(gathered_ids, survivor_ids, "gathered ids align with the survivor set");
}

/// GUARD for the serialiser anchor: the row evaluator's scan order for a single-pattern
/// SELECT is stable across runs, so the plan-stable `query_json` anchor above is comparing
/// like with like (a flake here would diagnose engine nondeterminism, not a harness bug).
#[test]
fn unfiltered_scan_order_is_deterministic() {
    let g = ages_graph(150);
    let q = "SELECT ?s ?o WHERE { ?s ex:age ?o }";
    let a = query_json(&g, &format!("{}{}", PFX, q)).unwrap();
    let b = query_json(&g, &format!("{}{}", PFX, q)).unwrap();
    assert_eq!(a.as_bytes(), b.as_bytes(), "single-pattern scan order must be deterministic");
}
