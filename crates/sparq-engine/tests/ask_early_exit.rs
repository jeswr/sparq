//! ASK first-solution early termination through joins / unions / optionals
//! (sq-7d3dj.30.8). [FABLE-5]
//!
//! INVARIANT UNDER TEST: `ask()` returns the identical boolean the unoptimized
//! path computes — for every query, ASK == non-emptiness of its SELECT twin
//! (the twin has no LIMIT, so it never takes a capped path and is the oracle).
//!
//! REVERT-WITNESSES (each fails if the early exit is broken or removed):
//!
//! * `ask_join_early_exit_fires_under_budget` — an ASK over a join whose FULL
//!   materialisation exceeds a row budget must still answer `true`, because the
//!   capped block-driven chain stops at the first solution long before the
//!   budget trips. The SELECT twin under the same budget errors — asserted in
//!   the same test — so the witness is non-vacuous: remove the capped join path
//!   and the ASK errors exactly like the twin (RED).
//! * `ask_late_filter_never_fires_on_partial_solutions` — the first seed block's
//!   candidate rows are all eliminated by a residual (non-sargable) FILTER; the
//!   only passing row lives beyond the first block. Early exit on a PARTIAL
//!   (pre-FILTER) solution would return a wrong boolean in the false-case twin
//!   test; stopping the seed sweep after an empty first block would return a
//!   wrong `false` here.
//! * `ask_distinct_under_offset_is_not_stripped` — `DISTINCT` below an `OFFSET`
//!   changes the visible COUNT, so `ask_simplify` must NOT strip it there:
//!   2 distinct values under `OFFSET 2` is empty (ASK false); the stripped plan
//!   would see 3 rows and answer `true` (RED).
//! * aggregation / HAVING — `Group` is never stripped or capped: a COUNT over an
//!   EMPTY pattern still yields one solution (ASK true), and HAVING can remove
//!   it (ASK false).
//!
//! Plus a q12a-shaped acceptance test (the SP2Bench evidence shape: ASK form of
//! q05a — two components bridged by a name-equality FILTER) against the
//! SELECT-nonemptiness oracle, and a LIMIT-path contract check (the capped rows
//! are a subset of the full result; a LIMIT past the result size returns the
//! complete multiset).

use sparq_core::Graph;
use sparq_engine::{ask, ask_with_budget, query, query_with_budget, QueryBudget};

const PFX: &str = "PREFIX : <http://ex/>\nPREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n";

fn load(ttl: &str) -> Graph {
    Graph::load_str(&format!("@prefix : <http://ex/> .\n@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n{ttl}"), "turtle")
        .expect("test graph")
}

/// Oracle assertion: ASK over `body` must equal non-emptiness of the SELECT twin
/// (which has no LIMIT and therefore never takes a capped path).
fn assert_ask_matches_select(g: &Graph, body: &str, expected: bool) {
    let a = ask(g, &format!("{PFX}ASK {{ {body} }}")).expect("ask");
    let s = query(g, &format!("{PFX}SELECT * WHERE {{ {body} }}")).expect("select twin");
    assert_eq!(a, !s.rows.is_empty(), "ASK != SELECT-nonemptiness for: {body}");
    assert_eq!(a, expected, "unexpected boolean for: {body}");
}

// ---- joins ---------------------------------------------------------------------

#[test]
fn ask_through_joins_true_and_false() {
    let g = load(":a :p :x . :x :q :y . :b :p :z .");
    // Two-hop join with a solution.
    assert_ask_matches_select(&g, "?s :p ?m . ?m :q ?o", true);
    // Join-compatible patterns exist but never join (:z has no :q edge beyond... it has none).
    assert_ask_matches_select(&g, "?s :p ?m . ?m :r ?o", false);
    // Three patterns, star shape.
    assert_ask_matches_select(&g, "?s :p ?m . ?m :q ?o . ?s :p :z", false);
    assert_ask_matches_select(&g, "?s :p ?m . ?m :q ?o . ?b :p :z", true);
}

/// THE core revert-witness: the full join materialises 80 000 rows, the budget
/// allows 60 000 — the SELECT twin trips the budget, the ASK must not, because
/// the capped chain answers from the first ~1024-row seed block.
#[test]
fn ask_join_early_exit_fires_under_budget() {
    let mut ttl = String::new();
    for i in 0..3000 {
        ttl.push_str(&format!(":s{i} :p :a{i}, :b{i} ; :q :c{i}, :d{i} .\n"));
    }
    let g = load(&ttl);
    // Sized so that every capped block stays comfortably below the budget (first
    // block 1024 seed rows -> 2048 joined rows; even a 4096-row block -> 8192)
    // while the FULL join exceeds it.
    let budget = QueryBudget { max_rows: Some(9_500), ..Default::default() };
    let body = "?s :p ?x . ?s :q ?y";
    // Non-vacuity: the FULL evaluation genuinely exceeds the budget (join output
    // is 3000 subjects x 2 x 2 = 12 000 rows > 9 500).
    let twin = query_with_budget(&g, &format!("{PFX}SELECT * WHERE {{ {body} }}"), &budget);
    assert!(
        twin.as_ref().is_err_and(|e| e.contains("budget")),
        "the SELECT twin must trip the row budget (got {:?} rows / err)",
        twin.as_ref().map(|r| r.rows.len())
    );
    // The witness: ASK stops at the first solution, far below the budget.
    let a = ask_with_budget(&g, &format!("{PFX}ASK {{ {body} }}"), &budget);
    assert_eq!(a, Ok(true), "ASK must early-exit through the join instead of materialising it");
    // And the boolean is right (unlimited oracle).
    assert_ask_matches_select(&g, body, true);
}

// ---- late residual FILTER ------------------------------------------------------

/// 2000 subjects; `?v = ?w` only holds for :s1500 — beyond the first seed block
/// (1024 rows). Rows must count toward the LIMIT-1 cap only AFTER the residual
/// FILTER, and the seed sweep must continue past an empty first block.
#[test]
fn ask_late_filter_never_fires_on_partial_solutions() {
    let mut ttl = String::new();
    for i in 0..2000 {
        let w = if i == 1500 { i } else { i + 1 };
        ttl.push_str(&format!(":s{i} :p {i} ; :q {w} .\n"));
    }
    let g = load(&ttl);
    // Plain equality (may become a value join once equality-FILTER planning lands
    // — the boolean is invariant either way).
    assert_ask_matches_select(&g, "?s :p ?v . ?s :q ?w . FILTER(?v = ?w)", true);
    // Arithmetic form: never a value join, guaranteed residual.
    assert_ask_matches_select(&g, "?s :p ?v . ?s :q ?w . FILTER(?v + 0 = ?w)", true);
    // False case: no subject satisfies it.
    assert_ask_matches_select(&g, "?s :p ?v . ?s :q ?w . FILTER(?v = ?w + 1000)", false);
}

#[test]
fn ask_filter_eliminates_everything() {
    let g = load(":a :p 1 . :b :p 2 . :a :q 5 . :b :q 6 .");
    // Sargable single-pattern filter to empty.
    assert_ask_matches_select(&g, "?s :p ?v FILTER(?v > 1000)", false);
    // Residual cross-pattern filter to empty (all v < w).
    assert_ask_matches_select(&g, "?s :p ?v . ?s :q ?w . FILTER(?v > ?w)", false);
    // Same shapes, satisfiable.
    assert_ask_matches_select(&g, "?s :p ?v FILTER(?v > 1)", true);
    assert_ask_matches_select(&g, "?s :p ?v . ?s :q ?w . FILTER(?v < ?w)", true);
}

// ---- OPTIONAL ------------------------------------------------------------------

#[test]
fn ask_optional_true_and_false() {
    let g = load(":a :p 1 . :a :q 2 .");
    // OPTIONAL never affects existence of the left side.
    assert_ask_matches_select(&g, "?s :p ?v OPTIONAL { ?s :q ?w }", true);
    assert_ask_matches_select(&g, "?s :p ?v OPTIONAL { ?s :missing ?w }", true);
    assert_ask_matches_select(&g, "?s :r ?v OPTIONAL { ?s :q ?w }", false);
    // OPTIONAL with an inner condition that filters the right side away.
    assert_ask_matches_select(&g, "?s :p ?v OPTIONAL { ?s :q ?w FILTER(?w > 100) }", true);
    // Multi-pattern (capped-chain) left side under OPTIONAL.
    assert_ask_matches_select(&g, "?s :p ?v . ?s :q ?w OPTIONAL { ?s :r ?z }", true);
}

// ---- UNION ---------------------------------------------------------------------

#[test]
fn ask_union_branch_short_circuit() {
    let g = load(":a :p 1 . :b :q 2 .");
    assert_ask_matches_select(&g, "{ ?s :p ?v } UNION { ?s :q ?v }", true);
    // Only the RIGHT branch matches: the left-first capped pass must continue.
    assert_ask_matches_select(&g, "{ ?s :r ?v } UNION { ?s :q ?v }", true);
    // Only the LEFT matches.
    assert_ask_matches_select(&g, "{ ?s :p ?v } UNION { ?s :r ?v }", true);
    // Neither.
    assert_ask_matches_select(&g, "{ ?s :r ?v } UNION { ?s :t ?v }", false);
    // Branches that are themselves joins.
    assert_ask_matches_select(&g, "{ ?s :p ?v . ?s :q ?w } UNION { ?s :q ?v }", true);
}

/// A LIMIT over a UNION whose left branch satisfies the cap: the right branch is
/// never evaluated, but the result HEADER must still carry the right branch's
/// in-scope variables (unbound), exactly as the full evaluation lays them out.
#[test]
fn union_early_exit_keeps_right_branch_header() {
    let g = load(":a :p 1 . :b :q 2 .");
    let r = query(&g, &format!("{PFX}SELECT ?v ?w WHERE {{ {{ ?x :p ?v }} UNION {{ ?y :q ?w }} }} LIMIT 1")).expect("query");
    assert_eq!(r.vars.len(), 2);
    assert_eq!(r.rows.len(), 1);
    let row = &r.rows[0];
    assert!(row[0].is_some(), "?v bound by the left branch");
    assert!(row[1].is_none(), "?w is a right-branch variable: unbound in a left-branch solution");
}

// ---- Join of non-conjunctive children (the q12b / q08 shape) --------------------

#[test]
fn ask_join_with_union_child() {
    let g = load(
        ":paul :name \"Paul\" . :doc1 :creator :paul . :doc1 :creator :alice . \
         :doc2 :editor :paul . :doc2 :editor :bob .",
    );
    // Selective left + UNION right (the SP2Bench q12b shape).
    assert_ask_matches_select(
        &g,
        "?p :name \"Paul\" . { ?d :creator ?p . ?d :creator ?other } UNION { ?d :editor ?p . ?d :editor ?other }",
        true,
    );
    assert_ask_matches_select(
        &g,
        "?p :name \"Nobody\" . { ?d :creator ?p . ?d :creator ?other } UNION { ?d :editor ?p . ?d :editor ?other }",
        false,
    );
    // Left matches, right never joins.
    assert_ask_matches_select(&g, "?p :name \"Paul\" . { ?d :reviewer ?p } UNION { ?d :translator ?p }", false);
}

// ---- BIND ----------------------------------------------------------------------

#[test]
fn ask_bind_over_capped_inner() {
    let g = load(":a :p 1 . :b :p 2 .");
    assert_ask_matches_select(&g, "?s :p ?v . BIND(?v * 2 AS ?d)", true);
    assert_ask_matches_select(&g, "?s :r ?v . BIND(?v * 2 AS ?d)", false);
}

// ---- ORDER BY / DISTINCT under ASK ----------------------------------------------

#[test]
fn ask_order_by_is_emptiness_neutral() {
    let g = load(":a :p 3 . :b :p 1 . :c :p 2 .");
    let t = ask(&g, &format!("{PFX}ASK WHERE {{ ?s :p ?v }} ORDER BY ?v")).expect("ask");
    assert!(t);
    let f = ask(&g, &format!("{PFX}ASK WHERE {{ ?s :r ?v }} ORDER BY ?v")).expect("ask");
    assert!(!f);
}

#[test]
fn ask_order_by_limit_offset_subselect() {
    let g = load(":a :p 3 . :b :p 1 . :c :p 2 .");
    // ORDER BY + LIMIT below ASK: the Slice stays exact.
    assert_ask_matches_select(&g, "{ SELECT ?v WHERE { ?s :p ?v } ORDER BY ?v LIMIT 1 }", true);
    assert_ask_matches_select(&g, "{ SELECT ?v WHERE { ?s :p ?v } ORDER BY ?v LIMIT 0 }", false);
    // OFFSET beyond the result size: empty.
    assert_ask_matches_select(&g, "{ SELECT ?v WHERE { ?s :p ?v } ORDER BY ?v OFFSET 10 }", false);
    assert_ask_matches_select(&g, "{ SELECT ?v WHERE { ?s :p ?v } ORDER BY ?v LIMIT 2 OFFSET 1 }", true);
}

/// DISTINCT below an OFFSET observes the deduplicated COUNT: 3 rows / 2 distinct
/// values under `OFFSET 2` is EMPTY. Stripping the DISTINCT (an unsound
/// `ask_simplify` recursion through `Slice`) would leave 3 - 2 = 1 row and flip
/// the boolean — this is the mutation witness for the Slice guard.
#[test]
fn ask_distinct_under_offset_is_not_stripped() {
    let g = load(":a :p 1 . :b :p 1 . :c :p 2 .");
    assert_ask_matches_select(&g, "{ SELECT DISTINCT ?v WHERE { ?s :p ?v } OFFSET 2 }", false);
    // Control: without DISTINCT the same OFFSET leaves a row.
    assert_ask_matches_select(&g, "{ SELECT ?v WHERE { ?s :p ?v } OFFSET 2 }", true);
    // DISTINCT with no OFFSET above it is emptiness-neutral (strippable).
    assert_ask_matches_select(&g, "{ SELECT DISTINCT ?v WHERE { ?s :p ?v } }", true);
    assert_ask_matches_select(&g, "{ SELECT DISTINCT ?v WHERE { ?s :r ?v } }", false);
}

// ---- aggregation / HAVING (never stripped, never capped) -------------------------

#[test]
fn ask_aggregation_and_having() {
    let g = load(":a :p 1 . :b :p 2 . :c :p 3 .");
    // COUNT over an EMPTY pattern is still one solution (the empty group).
    assert_ask_matches_select(&g, "{ SELECT (COUNT(*) AS ?c) WHERE { ?s :nothing ?v } }", true);
    // ... and HAVING can remove that solution.
    assert_ask_matches_select(&g, "{ SELECT (COUNT(*) AS ?c) WHERE { ?s :nothing ?v } HAVING (COUNT(*) > 0) }", false);
    // Non-empty groups, HAVING keeps / removes.
    assert_ask_matches_select(&g, "{ SELECT (COUNT(*) AS ?c) WHERE { ?s :p ?v } HAVING (COUNT(*) >= 3) }", true);
    assert_ask_matches_select(&g, "{ SELECT (COUNT(*) AS ?c) WHERE { ?s :p ?v } HAVING (COUNT(*) > 100) }", false);
    // GROUP BY over an empty pattern: ZERO groups, ASK false (contrast with the
    // implicit single empty group above).
    assert_ask_matches_select(&g, "{ SELECT ?s (COUNT(*) AS ?c) WHERE { ?s :nothing ?v } GROUP BY ?s }", false);
}

// ---- q12a-shaped acceptance (the SP2Bench evidence shape) ------------------------

fn q12a_graph(shared_names: bool) -> Graph {
    let mut ttl = String::new();
    for i in 0..300 {
        ttl.push_str(&format!(":art{i} rdf:type :Article ; :creator :ap{i} .\n:ap{i} :name \"n{}\" .\n", i % 37));
        let name = if shared_names { format!("n{}", i % 37) } else { format!("m{}", i % 41) };
        ttl.push_str(&format!(":inp{i} rdf:type :Inproceedings ; :creator :ip{i} .\n:ip{i} :name \"{name}\" .\n"));
    }
    load(&ttl)
}

const Q12A_BODY: &str = "?article rdf:type :Article . ?article :creator ?person . ?person :name ?name . \
                         ?inproc rdf:type :Inproceedings . ?inproc :creator ?person2 . ?person2 :name ?name2 . \
                         FILTER(?name = ?name2)";

#[test]
fn ask_q12a_shape_matches_select_oracle() {
    assert_ask_matches_select(&q12a_graph(true), Q12A_BODY, true);
    assert_ask_matches_select(&q12a_graph(false), Q12A_BODY, false);
}

/// NON-canonical local probe (work-box timings are never authoritative): prints
/// the SELECT-twin time (== the pre-fix ASK cost, since the old ASK evaluated
/// the full solution set) next to the ASK time on a larger synthetic q12a.
/// Run manually: `cargo test -p sparq-engine --release --test ask_early_exit -- --ignored --nocapture`
#[test]
#[ignore = "timing probe, not an assertion (run manually with --release --nocapture)"]
fn q12a_shape_timing_probe() {
    let mut ttl = String::new();
    for i in 0..20_000 {
        ttl.push_str(&format!(":art{i} rdf:type :Article ; :creator :ap{i} .\n:ap{i} :name \"n{}\" .\n", i % 371));
        ttl.push_str(&format!(":inp{i} rdf:type :Inproceedings ; :creator :ip{i} .\n:ip{i} :name \"n{}\" .\n", i % 373));
    }
    let g = load(&ttl);
    let t0 = std::time::Instant::now();
    let s = query(&g, &format!("{PFX}SELECT * WHERE {{ {Q12A_BODY} }}")).expect("select twin");
    let select_ms = t0.elapsed().as_millis();
    let t1 = std::time::Instant::now();
    let a = ask(&g, &format!("{PFX}ASK {{ {Q12A_BODY} }}")).expect("ask");
    let ask_ms = t1.elapsed().as_millis();
    assert_eq!(a, !s.rows.is_empty());
    println!("q12a-shape (synthetic, 20k+20k, NON-canonical): SELECT twin {select_ms} ms ({} rows) vs ASK {ask_ms} ms", s.rows.len());
}

// ---- LIMIT-path contract through the capped chain --------------------------------

/// The capped conjunctive chain also serves plain LIMIT: the returned rows must be
/// a sub-multiset of the full result, exactly `min(limit, N)` of them.
#[test]
fn limit_through_join_returns_a_subset_of_the_full_result() {
    let mut ttl = String::new();
    for i in 0..1500 {
        ttl.push_str(&format!(":s{i} :p :x{i}, :y{i} ; :q :u{i}, :v{i} .\n"));
    }
    let g = load(&ttl);
    let body = "?s :p ?a . ?s :q ?b";
    let full = query(&g, &format!("{PFX}SELECT ?s ?a ?b WHERE {{ {body} }}")).expect("full");
    assert_eq!(full.rows.len(), 6000);
    let key = |row: &Vec<Option<oxrdf::Term>>| -> String {
        row.iter().map(|c| c.as_ref().map_or(String::new(), |t| t.to_string())).collect::<Vec<_>>().join("|")
    };
    let full_set: std::collections::HashSet<String> = full.rows.iter().map(key).collect();
    for (limit, offset) in [(7usize, 0usize), (7, 5), (1, 0), (6000, 0), (9999, 0)] {
        let q = format!("{PFX}SELECT ?s ?a ?b WHERE {{ {body} }} LIMIT {limit} OFFSET {offset}");
        let r = query(&g, &q).expect("limited");
        let expected = limit.min(6000usize.saturating_sub(offset));
        assert_eq!(r.rows.len(), expected, "row count for LIMIT {limit} OFFSET {offset}");
        for row in &r.rows {
            assert!(full_set.contains(&key(row)), "capped row not in the full result (LIMIT {limit} OFFSET {offset})");
        }
    }
    // LIMIT past the result size returns the COMPLETE multiset.
    let all = query(&g, &format!("{PFX}SELECT ?s ?a ?b WHERE {{ {body} }} LIMIT 9999")).expect("all");
    let mut got: Vec<String> = all.rows.iter().map(key).collect();
    let mut want: Vec<String> = full.rows.iter().map(key).collect();
    got.sort();
    want.sort();
    assert_eq!(got, want, "LIMIT >= N must be the complete result");
}

/// LIMIT 0 through the capped shapes stays empty and never panics.
#[test]
fn limit_zero_through_capped_shapes() {
    let g = load(":a :p 1 . :a :q 2 .");
    for q in [
        "SELECT * WHERE { ?s :p ?v . ?s :q ?w } LIMIT 0",
        "SELECT * WHERE { { ?s :p ?v } UNION { ?s :q ?v } } LIMIT 0",
        "SELECT * WHERE { ?s :p ?v OPTIONAL { ?s :q ?w } } LIMIT 0",
    ] {
        let r = query(&g, &format!("{PFX}{q}")).expect("limit 0");
        assert!(r.rows.is_empty(), "LIMIT 0 must be empty for: {q}");
    }
}
