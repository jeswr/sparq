//! Integration test for the inline `OVER(…)` window-clause SYNTAX (sq-h564),
//! exercised through the public `sparq_engine::query_over` entry point — a
//! NON-STANDARD, OPT-IN extension (the `window-functions` cargo feature). This is
//! the end-to-end "parse + eval an inline-OVER query" check the bead asks for:
//! a `ROW_NUMBER()` / `RANK()` over a known `PARTITION BY` / `ORDER BY` yields the
//! correct sequence. [OPUS-4.8]
#![cfg(feature = "window-functions")]

use oxrdf::Term;
use sparq_core::Graph;
use sparq_engine::{query_over, QueryResult};

const DATA: &str = r#"@prefix ex: <http://ex/> .
    ex:a ex:dept "eng"   ; ex:sales 30 .
    ex:b ex:dept "eng"   ; ex:sales 30 .
    ex:c ex:dept "eng"   ; ex:sales 20 .
    ex:d ex:dept "sales" ; ex:sales 10 .
    ex:e ex:dept "sales" ; ex:sales 40 ."#;

fn g() -> Graph {
    Graph::load_str(DATA, "turtle").unwrap()
}

fn int(cell: &Option<Term>) -> i64 {
    match cell {
        Some(Term::Literal(l)) => l.value().parse().expect("integer literal"),
        other => panic!("expected an integer literal, got {other:?}"),
    }
}

/// emp short name (`ex:a` → `a`).
fn name(cell: &Option<Term>) -> String {
    match cell {
        Some(Term::NamedNode(n)) => n.as_str().rsplit('/').next().unwrap().to_string(),
        other => panic!("expected an IRI, got {other:?}"),
    }
}

/// Find the value in `col` for the row whose `emp` (col 0) short name is `emp`.
fn for_emp(r: &QueryResult, emp: &str, col: usize) -> i64 {
    for row in &r.rows {
        if name(&row[0]) == emp {
            return int(&row[col]);
        }
    }
    panic!("emp {emp} not found in result");
}

#[test]
fn row_number_over_partition_by_order_by() {
    let q = "PREFIX ex: <http://ex/> \
        SELECT ?emp \
            (ROW_NUMBER() OVER (PARTITION BY ?dept ORDER BY DESC(?sales)) AS ?rn) \
        WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }";
    let r = query_over(&g(), q).unwrap();
    assert_eq!(r.vars.iter().map(|v| v.as_str()).collect::<Vec<_>>(), ["emp", "rn"]);
    // eng partition desc by sales: a=30,b=30 tie (broken stably by input order → a<b),
    // then c=20 → row numbers a=1, b=2, c=3.
    assert_eq!(for_emp(&r, "a", 1), 1);
    assert_eq!(for_emp(&r, "b", 1), 2);
    assert_eq!(for_emp(&r, "c", 1), 3);
    // sales partition desc: e=40, d=10 → e=1, d=2.
    assert_eq!(for_emp(&r, "e", 1), 1);
    assert_eq!(for_emp(&r, "d", 1), 2);
}

#[test]
fn rank_has_gaps_after_ties() {
    let q = "PREFIX ex: <http://ex/> \
        SELECT ?emp \
            (RANK() OVER (PARTITION BY ?dept ORDER BY DESC(?sales)) AS ?r) \
        WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }";
    let r = query_over(&g(), q).unwrap();
    // eng: a=30,b=30 → rank 1; c=20 → rank 3 (GAP after the 2-way tie).
    assert_eq!(for_emp(&r, "a", 1), 1);
    assert_eq!(for_emp(&r, "b", 1), 1);
    assert_eq!(for_emp(&r, "c", 1), 3);
    // sales: e=40 → 1, d=10 → 2.
    assert_eq!(for_emp(&r, "e", 1), 1);
    assert_eq!(for_emp(&r, "d", 1), 2);
}

#[test]
fn trailing_keyword_order_direction() {
    // `ORDER BY ?sales DESC` (the SQL trailing-keyword spelling) parses the same
    // as `ORDER BY DESC(?sales)`.
    let q = "PREFIX ex: <http://ex/> \
        SELECT ?emp \
            (DENSE_RANK() OVER (PARTITION BY ?dept ORDER BY ?sales DESC) AS ?dr) \
        WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }";
    let r = query_over(&g(), q).unwrap();
    // eng dense rank: 30→1, 30→1, 20→2 (no gap).
    assert_eq!(for_emp(&r, "a", 1), 1);
    assert_eq!(for_emp(&r, "c", 1), 2);
}

#[test]
fn order_by_computed_expression_in_over() {
    // sq-c1jv: an OVER ORDER BY key that is a computed SPARQL EXPRESSION rather
    // than a projected variable. `(?sales * -1)` ascending == `?sales` descending,
    // so ROW_NUMBER over it reproduces the descending-by-sales ranking. The helper
    // binding the expression must NOT leak into the output columns.
    let q = "PREFIX ex: <http://ex/> \
        SELECT ?emp \
            (ROW_NUMBER() OVER (PARTITION BY ?dept ORDER BY (?sales * -1)) AS ?rn) \
        WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }";
    let r = query_over(&g(), q).unwrap();
    // Output is exactly the named columns — the expression helper is dropped.
    assert_eq!(r.vars.iter().map(|v| v.as_str()).collect::<Vec<_>>(), ["emp", "rn"]);
    // eng partition, sales*-1 ascending == sales descending: a=30,b=30 tie (stable
    // by input order a<b), then c=20 → row numbers a=1, b=2, c=3.
    assert_eq!(for_emp(&r, "a", 1), 1);
    assert_eq!(for_emp(&r, "b", 1), 2);
    assert_eq!(for_emp(&r, "c", 1), 3);
    // sales partition, sales*-1 ascending == sales descending: e=40, d=10 → e=1, d=2.
    assert_eq!(for_emp(&r, "e", 1), 1);
    assert_eq!(for_emp(&r, "d", 1), 2);
}

#[test]
fn order_by_computed_expression_desc() {
    // The DESC()-wrapped expression form: `DESC(?sales + 0)` orders descending by
    // the computed key, so RANK gives the largest-sales row rank 1.
    let q = "PREFIX ex: <http://ex/> \
        SELECT ?emp \
            (RANK() OVER (ORDER BY DESC(?sales + 0)) AS ?r) \
        WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }";
    let r = query_over(&g(), q).unwrap();
    assert_eq!(r.vars.iter().map(|v| v.as_str()).collect::<Vec<_>>(), ["emp", "r"]);
    // Single partition, ?sales+0 descending: e=40→1, a=30,b=30→2 (tie), c=20→4, d=10→5.
    assert_eq!(for_emp(&r, "e", 1), 1);
    assert_eq!(for_emp(&r, "a", 1), 2);
    assert_eq!(for_emp(&r, "b", 1), 2);
    assert_eq!(for_emp(&r, "c", 1), 4);
    assert_eq!(for_emp(&r, "d", 1), 5);
}

#[test]
fn ordinary_query_passes_through_unchanged() {
    // No OVER clause → identical to sparq_engine::query.
    let q = "PREFIX ex: <http://ex/> SELECT ?emp WHERE { ?emp ex:dept ?d }";
    let viaq = query_over(&g(), q).unwrap();
    assert_eq!(viaq.rows.len(), 5);
    assert_eq!(viaq.vars.iter().map(|v| v.as_str()).collect::<Vec<_>>(), ["emp"]);
}

// ---- inline windowed AGGREGATES + frames (sq-imj8) [OPUS-4.8] ----

#[test]
fn inline_windowed_sum_per_partition() {
    // SUM(?sales) OVER (PARTITION BY ?dept): each emp sees its dept's sales total.
    let q = "PREFIX ex: <http://ex/> \
        SELECT ?emp \
            (SUM(?sales) OVER (PARTITION BY ?dept) AS ?dept_total) \
        WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }";
    let r = query_over(&g(), q).unwrap();
    assert_eq!(r.vars.iter().map(|v| v.as_str()).collect::<Vec<_>>(), ["emp", "dept_total"]);
    // eng = 30+30+20 = 80; sales = 10+40 = 50.
    for emp in ["a", "b", "c"] {
        assert_eq!(for_emp(&r, emp, 1), 80);
    }
    for emp in ["d", "e"] {
        assert_eq!(for_emp(&r, emp, 1), 50);
    }
}

#[test]
fn inline_running_total_rows_frame() {
    // ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW — a running total of sales
    // over the whole result ordered ascending by sales (10,20,30,30,40).
    let q = "PREFIX ex: <http://ex/> \
        SELECT ?emp ?sales \
            (SUM(?sales) OVER (ORDER BY ?sales ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS ?run) \
        WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }";
    let r = query_over(&g(), q).unwrap();
    assert_eq!(r.vars.iter().map(|v| v.as_str()).collect::<Vec<_>>(), ["emp", "sales", "run"]);
    // d (sales=10) is first → run 10. e (sales=40) is last → run = grand total 130
    // (10+20+30+30+40).
    assert_eq!(for_emp(&r, "d", 2), 10);
    assert_eq!(for_emp(&r, "e", 2), 130);
    // c (sales=20) → 10+20 = 30.
    assert_eq!(for_emp(&r, "c", 2), 30);
}

#[test]
fn inline_moving_average_rows_n_preceding() {
    // AVG(?sales) OVER (ORDER BY ?sales ROWS 1 PRECEDING) — a 2-row trailing moving
    // average. ordered: d=10, c=20, a=30, b=30, e=40. The first row's frame is just
    // itself (avg = its own value).
    let q = "PREFIX ex: <http://ex/> \
        SELECT ?emp ?sales \
            (AVG(?sales) OVER (ORDER BY ?sales ROWS 1 PRECEDING) AS ?mavg) \
        WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }";
    let r = query_over(&g(), q).unwrap();
    // d is the smallest (sales=10), its trailing 1-row frame is itself → avg 10.
    assert_eq!(for_emp(&r, "d", 2), 10);
}

#[test]
fn inline_range_frame_includes_peer_group() {
    // RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW — the two sales=30 rows are
    // PEERS, so both see the same running total (10+20+30+30 = 90), unlike ROWS.
    let q = "PREFIX ex: <http://ex/> \
        SELECT ?emp ?sales \
            (SUM(?sales) OVER (ORDER BY ?sales RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS ?run) \
        WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }";
    let r = query_over(&g(), q).unwrap();
    // a and b both have sales=30 → both run = 90 (their shared peer group included:
    // 10+20+30+30).
    assert_eq!(for_emp(&r, "a", 2), 90);
    assert_eq!(for_emp(&r, "b", 2), 90);
    // d=10 → 10; c=20 → 30; e=40 → 130 (the grand total 10+20+30+30+40).
    assert_eq!(for_emp(&r, "d", 2), 10);
    assert_eq!(for_emp(&r, "c", 2), 30);
    assert_eq!(for_emp(&r, "e", 2), 130);
}

#[test]
fn inline_frame_on_ranking_function_is_rejected() {
    // A frame is only valid on a windowed aggregate; on a ranking function it errors.
    let q = "PREFIX ex: <http://ex/> \
        SELECT ?emp (RANK() OVER (ORDER BY ?sales ROWS UNBOUNDED PRECEDING) AS ?r) \
        WHERE { ?emp ex:sales ?sales }";
    assert!(query_over(&g(), q).is_err());
}
