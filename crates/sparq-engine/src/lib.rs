//! sparq-engine: a SPARQL query engine over [`sparq_core::Graph`].
//!
//! Supported (M2): SELECT with Basic Graph Patterns evaluated by greedy
//! cardinality-ordered sort-merge / hash joins over the permutation indexes;
//! FILTER (a useful expression subset with XSD-numeric-aware comparisons);
//! OPTIONAL, UNION, MINUS, BIND, VALUES; aggregation (COUNT/SUM/AVG/MIN/MAX/
//! GROUP_CONCAT) with GROUP BY and HAVING (as a post-group FILTER); ORDER BY;
//! DISTINCT/REDUCED/LIMIT/OFFSET; projection and sub-SELECT. SPARQL is parsed
//! to algebra by `spargebra`. Values computed at query time (BIND, aggregates)
//! are interned in a per-query local vocabulary. Later milestones add
//! worst-case-optimal joins, a DP planner and property paths.

mod exec;
pub mod json;

use oxrdf::{Term, Variable};
use sparq_core::Graph;
use spargebra::{Query, SparqlParser};

/// Executes a SPARQL query string against a graph, materialising the solutions.
pub fn query(graph: &Graph, sparql: &str) -> Result<QueryResult, String> {
    let q = SparqlParser::new().parse_query(sparql).map_err(|e| e.to_string())?;
    match q {
        Query::Select { pattern, .. } => exec::eval_select(graph, &pattern),
        _ => Err("only SELECT queries are supported".into()),
    }
}

/// Counts the solutions of a SELECT query *without* materialising the result
/// terms (the id-level row count equals the solution count). Used to measure
/// engine compute in isolation from result serialisation.
pub fn count(graph: &Graph, sparql: &str) -> Result<usize, String> {
    let q = SparqlParser::new().parse_query(sparql).map_err(|e| e.to_string())?;
    match q {
        Query::Select { pattern, .. } => exec::count_select(graph, &pattern),
        _ => Err("only SELECT queries are supported".into()),
    }
}

pub struct QueryResult {
    pub vars: Vec<Variable>,
    /// Each row has one entry per `vars` position; `None` is unbound.
    pub rows: Vec<Vec<Option<Term>>>,
}

impl QueryResult {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: &str = r#"
        @prefix ex: <http://ex/> .
        ex:alice ex:knows ex:bob ; ex:age 30 ; ex:name "Alice" .
        ex:bob   ex:knows ex:carol ; ex:age 25 ; ex:name "Bob" .
        ex:carol ex:age 35 ; ex:name "Carol" .
    "#;

    fn g() -> Graph {
        Graph::load_str(DATA, "turtle").unwrap()
    }

    fn count(q: &str) -> usize {
        query(&g(), q).unwrap().len()
    }

    #[test]
    fn single_pattern() {
        assert_eq!(count("SELECT ?s WHERE { ?s <http://ex/age> ?a }"), 3);
    }

    #[test]
    fn two_pattern_join() {
        // who does someone know, and what is that person's age
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?a ?b ?age WHERE { ?a ex:knows ?b . ?b ex:age ?age }",
        )
        .unwrap();
        // alice->bob(25), bob->carol(35)
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn filter_numeric() {
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a . FILTER(?a > 28) }",
        )
        .unwrap();
        assert_eq!(r.len(), 2); // alice(30), carol(35)
    }

    #[test]
    fn distinct_and_limit() {
        assert_eq!(count("SELECT DISTINCT ?p WHERE { ?s ?p ?o }"), 3); // knows, age, name
        assert_eq!(count("SELECT ?s WHERE { ?s ?p ?o } LIMIT 2"), 2);
    }

    #[test]
    fn unsatisfiable_absent_term() {
        assert_eq!(count("SELECT ?s WHERE { ?s <http://ex/nope> ?o }"), 0);
    }

    #[test]
    fn blank_node_is_existential_variable() {
        // _:x acts as a variable: matches any subject with ex:age. SELECT *
        // must not expose the blank-node variable.
        let r = query(&g(), "SELECT * WHERE { _:x <http://ex/age> ?a }").unwrap();
        assert_eq!(r.len(), 3);
        assert_eq!(r.vars.len(), 1); // only ?a, not _:x
        // repeated blank label behaves like a repeated variable (no self-knows here)
        assert_eq!(
            count("PREFIX ex: <http://ex/> SELECT * WHERE { _:x ex:knows _:x }"),
            0
        );
    }

    #[test]
    fn chain_join_three_patterns() {
        // ?a knows ?b, ?b knows ?c : alice->bob->carol
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:knows ?b . ?b ex:knows ?c }",
        )
        .unwrap();
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn star_join_merge() {
        // star on ?s: all three patterns share ?s -> merge joins on ?s
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a . ?s ex:name ?n . ?s ex:knows ?k }",
        )
        .unwrap();
        // only alice and bob have knows+age+name
        assert_eq!(r.len(), 2);
    }

    // Differential check: merge-join path must agree with a forced hash-only path
    // across many random small graphs would be ideal; here we cross-check that
    // join result counts are order-independent on a few shapes.
    #[test]
    fn join_two_shared_vars() {
        // ?x ex:knows ?y . ?y ex:knows ?x  (symmetric knows? none here) -> 0
        assert_eq!(
            count("PREFIX ex: <http://ex/> SELECT * WHERE { ?x ex:knows ?y . ?y ex:knows ?x }"),
            0
        );
    }

    #[test]
    fn filter_ebv_boolean_and_string() {
        // FILTER(true) keeps all; numeric/string EBV
        assert_eq!(count("SELECT ?s WHERE { ?s <http://ex/age> ?a . FILTER(true) }"), 3);
        // string EBV: ?n is a non-empty string for all
        assert_eq!(
            count("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:name ?n . FILTER(?n) }"),
            3
        );
    }

    #[test]
    fn optional() {
        // carol has no ex:knows -> OPTIONAL leaves ?k unbound for carol
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s ?k WHERE { ?s ex:age ?a . OPTIONAL { ?s ex:knows ?k } }",
        )
        .unwrap();
        assert_eq!(r.len(), 3);
        let unbound = r.rows.iter().filter(|row| row[1].is_none()).count();
        assert_eq!(unbound, 1); // carol
    }

    #[test]
    fn union() {
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?x WHERE { { ?x ex:age 30 } UNION { ?x ex:age 25 } }",
        )
        .unwrap();
        assert_eq!(r.len(), 2); // alice, bob
    }

    #[test]
    fn bind_and_arithmetic() {
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s ?next WHERE { ?s ex:age ?a . BIND(?a + 1 AS ?next) FILTER(?next > 31) }",
        )
        .unwrap();
        // ages 30,25,35 -> next 31,26,36 -> >31 keeps carol(36)
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn minus() {
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a . MINUS { ?s ex:knows ?k } }",
        )
        .unwrap();
        assert_eq!(r.len(), 1); // only carol has no knows
    }

    #[test]
    fn aggregate_count_and_group() {
        // total count
        let r = query(&g(), "SELECT (COUNT(*) AS ?c) WHERE { ?s ?p ?o }").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r.rows[0][0].as_ref().unwrap().to_string(), "\"8\"^^<http://www.w3.org/2001/XMLSchema#integer>");

        // group by predicate, count
        let r = query(
            &g(),
            "SELECT ?p (COUNT(*) AS ?c) WHERE { ?s ?p ?o } GROUP BY ?p",
        )
        .unwrap();
        assert_eq!(r.len(), 3); // knows, age, name
    }

    #[test]
    fn aggregate_sum_avg_min_max() {
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT (SUM(?a) AS ?s)(AVG(?a) AS ?av)(MIN(?a) AS ?mn)(MAX(?a) AS ?mx) WHERE { ?x ex:age ?a }",
        )
        .unwrap();
        assert_eq!(r.len(), 1);
        // sum 90, min 25, max 35
        assert!(r.rows[0][0].as_ref().unwrap().to_string().contains("90"));
        assert!(r.rows[0][2].as_ref().unwrap().to_string().contains("25"));
        assert!(r.rows[0][3].as_ref().unwrap().to_string().contains("35"));
    }

    #[test]
    fn order_by() {
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a } ORDER BY DESC(?a)",
        )
        .unwrap();
        let ages: Vec<String> = r.rows.iter().map(|row| row[1].as_ref().unwrap().to_string()).collect();
        // 35, 30, 25 descending
        assert!(ages[0].contains("35") && ages[2].contains("25"));
    }

    #[test]
    fn sameterm_numeric_identity() {
        // sameTerm on a numeric variable with itself is true — the numeric fast
        // path must not discard term identity (regression for roborev 1271).
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a . FILTER(sameTerm(?a, ?a)) }",
        )
        .unwrap();
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn bind_preserves_numeric_lexical_form() {
        // BIND(?x AS ?y) must re-emit the ORIGINAL term, not a canonicalised number
        // (regression for roborev 1271): "1.0"^^decimal must stay "1.0"^^decimal.
        let g = Graph::load_str(
            "@prefix ex: <http://ex/> . ex:a ex:score \"1.0\"^^<http://www.w3.org/2001/XMLSchema#decimal> .",
            "turtle",
        )
        .unwrap();
        let r = query(
            &g,
            "PREFIX ex: <http://ex/> SELECT ?d WHERE { ?s ex:score ?sc . BIND(?sc AS ?d) }",
        )
        .unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(
            r.rows[0][0].as_ref().unwrap().to_string(),
            "\"1.0\"^^<http://www.w3.org/2001/XMLSchema#decimal>"
        );
    }

    #[test]
    fn filter_numeric_still_fast_path() {
        // The numeric comparison fast path still gives correct results.
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a . FILTER(?a + 1 > 31) }",
        )
        .unwrap();
        // ages 30,25,35 -> +1 = 31,26,36 -> >31 keeps only carol(36)
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn filter_pushdown_with_join() {
        // A numeric FILTER on a variable that is also a join variable: the filter
        // is pushed into that pattern's scan, then joined. Must stay correct.
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s ?k WHERE { ?s ex:age ?a . ?s ex:knows ?k . FILTER(?a > 28) }",
        )
        .unwrap();
        // ages: alice 30, bob 25, carol 35. >28 keeps alice, carol. knows: alice->bob,
        // bob->carol. After filter+join on ?s: only alice(30)->bob (carol has no knows).
        assert_eq!(r.len(), 1);
        assert!(r.rows[0][0].as_ref().unwrap().to_string().contains("alice"));
    }

    #[test]
    fn filter_pushdown_boundaries() {
        // >=, <=, = pushed-down comparisons.
        assert_eq!(count("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a . FILTER(?a >= 30) }"), 2);
        assert_eq!(count("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a . FILTER(?a <= 30) }"), 2);
        assert_eq!(count("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a . FILTER(?a = 25) }"), 1);
    }

    #[test]
    fn planner_four_pattern_multi_predicate() {
        // 4 patterns over predicates of different selectivity (knows/age/name) —
        // exercises the cost-based GOO planner's candidate scoring. Result must be
        // correct regardless of the order it picks.
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s ?k ?ka WHERE { \
               ?s ex:knows ?k . ?s ex:age ?a . ?s ex:name ?n . ?k ex:age ?ka }",
        )
        .unwrap();
        // alice->bob(25), bob->carol(35); carol has no knows
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn planner_path_vs_naive() {
        // 3-hop path ?a e ?b . ?b e ?c . ?c e ?d over a random graph, checked
        // against a brute-force count — the planner orders 3 candidates and the
        // result must match a naive evaluator.
        let mut seed = 0x00C0FFEEu64;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) as u32
        };
        let n = 22u32;
        let mut edges: Vec<(u32, u32)> = Vec::new();
        let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
        for _ in 0..130 {
            let (a, b) = (next() % n, next() % n);
            edges.push((a, b));
            ttl.push_str(&format!("ex:n{a} ex:e ex:n{b} .\n"));
        }
        edges.sort_unstable();
        edges.dedup();
        let graph = Graph::load_str(&ttl, "turtle").unwrap();

        // naive 3-hop path count: a->b, b->c, c->d
        let mut naive = 0usize;
        for &(_, b) in &edges {
            for &(b2, c) in &edges {
                if b2 != b {
                    continue;
                }
                for &(c2, _) in &edges {
                    if c2 == c {
                        naive += 1;
                    }
                }
            }
        }
        let q = query(
            &graph,
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c . ?c ex:e ?d }",
        )
        .unwrap();
        assert_eq!(q.len(), naive);
    }

    #[test]
    fn lazy_count_matches_materialized() {
        // The lazy count() (single-pattern range size, two-pattern group-count
        // join) must equal the materialised result length.
        let cases = [
            "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a }", // single pattern
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:knows ?b . ?b ex:age ?age }", // chain join
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:age ?a . ?s ex:name ?n }", // subject-shared
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?x ex:knows ?x }",               // repeated var
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:knows ?b . ?b ex:knows ?c }", // chain
        ];
        for q in cases {
            assert_eq!(super::count(&g(), q).unwrap(), query(&g(), q).unwrap().len(), "count mismatch: {q}");
        }
    }

    #[test]
    fn lazy_count_join_vs_naive_random() {
        // Two-pattern group-count join vs the materialised count over a random graph.
        let mut seed = 0xBEEF_1234u64;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) as u32
        };
        let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
        for _ in 0..140 {
            ttl.push_str(&format!("ex:n{} ex:e ex:n{} .\n", next() % 18, next() % 18));
        }
        let g = Graph::load_str(&ttl, "turtle").unwrap();
        let q = "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c }";
        assert_eq!(super::count(&g, q).unwrap(), query(&g, q).unwrap().len());
    }

    #[test]
    fn limit_early_termination() {
        // LIMIT over a single-pattern scan (early-terminating path).
        assert_eq!(count("SELECT * WHERE { ?s ?p ?o } LIMIT 5"), 5);
        assert_eq!(count("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a } LIMIT 2"), 2);
        // OFFSET + LIMIT.
        assert_eq!(count("SELECT * WHERE { ?s ?p ?o } LIMIT 3 OFFSET 2"), 3);
        // LIMIT larger than the result is fine.
        assert_eq!(count("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a } LIMIT 100"), 3);
        // LIMIT with a pushed-down sargable filter.
        assert_eq!(count("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a . FILTER(?a > 20) } LIMIT 1"), 1);
    }

    #[test]
    fn limit_with_order_by_is_correct() {
        // ORDER BY before LIMIT must NOT early-terminate — it returns the globally
        // smallest, not just the first scanned.
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a } ORDER BY ?a LIMIT 1",
        )
        .unwrap();
        assert_eq!(r.len(), 1);
        assert!(r.rows[0][1].as_ref().unwrap().to_string().contains("25")); // bob, youngest
    }

    #[test]
    fn having() {
        // group ?s, count its triples, keep groups with >= 3 triples
        let r = query(
            &g(),
            "SELECT ?s (COUNT(*) AS ?c) WHERE { ?s ?p ?o } GROUP BY ?s HAVING (COUNT(*) >= 3)",
        )
        .unwrap();
        // alice & bob have 3 triples each (knows,age,name); carol has 2
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn values_clause() {
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a . VALUES ?s { ex:alice ex:carol } }",
        )
        .unwrap();
        assert_eq!(r.len(), 2); // alice(30), carol(35)
    }

    #[test]
    fn sub_select() {
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s WHERE { { SELECT ?s WHERE { ?s ex:age ?a } ORDER BY DESC(?a) LIMIT 1 } }",
        )
        .unwrap();
        assert_eq!(r.len(), 1); // carol (oldest)
    }

    #[test]
    fn values_undef_is_wildcard() {
        // VALUES with UNDEF for ?s must join with every binding, not just an
        // (impossible) "unbound" id. Expect all three people.
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a . VALUES ?s { UNDEF } }",
        )
        .unwrap();
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn optional_then_join_unbound_compatible() {
        // "Non-well-designed" nested OPTIONAL (Pérez–Arenas–Gutierrez): ?k is
        // introduced in the first OPTIONAL and reused in the second. Under the
        // SPARQL bottom-up algebra, carol's UNBOUND ?k is compatible with EVERY
        // `?k ex:name ?kn` row, so the second LeftJoin pairs carol with all three
        // names. This (surprising) count of 5 is the spec-correct answer and
        // exercises the unbound-as-wildcard path in the compatibility join.
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s ?kn WHERE { \
               ?s ex:age ?a . \
               OPTIONAL { ?s ex:knows ?k } \
               OPTIONAL { ?k ex:name ?kn } }",
        )
        .unwrap();
        // alice->\"Bob\", bob->\"Carol\", carol-><\"Alice\",\"Bob\",\"Carol\"> = 5
        assert_eq!(r.len(), 5);
        assert!(r.rows.iter().all(|row| row[1].is_some())); // every ?kn ends up bound
    }

    #[test]
    fn computed_values_dedup() {
        // Two distinct ages (30,35,25) all map via floor()-like BIND to a constant
        // computed literal; DISTINCT must collapse them to one row, proving equal
        // computed terms share a local id.
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT DISTINCT ?c WHERE { ?s ex:age ?a . BIND(1 AS ?c) }",
        )
        .unwrap();
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn minus_undef_domain_overlap() {
        // Left has ?k possibly UNBOUND (via OPTIONAL); right binds ?k. MINUS must
        // exercise the general compatibility+domain-overlap path:
        //   alice(k=bob)   -> right has k=bob   -> compatible & overlap -> REMOVED
        //   bob(k=carol)   -> right has k=carol -> compatible & overlap -> REMOVED
        //   carol(k=UNDEF) -> compatible with all, but NO bound overlap -> KEPT
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s WHERE { \
               ?s ex:age ?a . \
               OPTIONAL { ?s ex:knows ?k } \
               MINUS { ?x ex:knows ?k } }",
        )
        .unwrap();
        assert_eq!(r.len(), 1); // only carol survives
        assert!(r.rows[0][0].as_ref().unwrap().to_string().contains("carol"));
    }

    #[test]
    fn range_pruning_boundaries_and_mixed_columns() {
        // Inline-integer range-pruning: an all-canonical-integer column binary-searches
        // to the passing value range. Check every operator at exact boundaries, plus
        // out-of-range and empty cases. Ages 10,20,30,40,50.
        let g = Graph::load_str(
            "@prefix ex: <http://ex/> . ex:a ex:age 10 . ex:b ex:age 20 . ex:c ex:age 30 . ex:d ex:age 40 . ex:e ex:age 50 .",
            "turtle",
        )
        .unwrap();
        let c = |q: &str| query(&g, &format!("PREFIX ex: <http://ex/> SELECT ?s WHERE {{ ?s ex:age ?a . FILTER({q}) }}")).unwrap().len();
        assert_eq!(c("?a > 30"), 2); // 40,50
        assert_eq!(c("?a >= 30"), 3); // 30,40,50
        assert_eq!(c("?a < 30"), 2); // 10,20
        assert_eq!(c("?a <= 30"), 3);
        assert_eq!(c("?a = 30"), 1);
        assert_eq!(c("?a != 30"), 4);
        assert_eq!(c("?a > 50"), 0); // above max -> empty
        assert_eq!(c("?a < 10"), 0); // below min -> empty
        assert_eq!(c("?a >= 0"), 5); // all
        assert_eq!(c("?a > -100"), 5); // negative threshold (non-sargable) -> all

        // MIXED column: the range-pruning guard must fall back to a full scan so a
        // non-inline numeric (xsd:int) that passes the filter is NOT dropped.
        let gm = Graph::load_str(
            "@prefix ex: <http://ex/> . @prefix xsd: <http://www.w3.org/2001/XMLSchema#> . \
             ex:a ex:v 50 . ex:b ex:v \"200\"^^xsd:int . ex:c ex:v \"95\"^^xsd:integer . ex:d ex:v \"-10\"^^xsd:integer .",
            "turtle",
        )
        .unwrap();
        let cm = |q: &str| query(&gm, &format!("PREFIX ex: <http://ex/> SELECT ?s WHERE {{ ?s ex:v ?v . FILTER({q}) }}")).unwrap().len();
        assert_eq!(cm("?v > 90"), 2); // 95 (inline) AND 200 (xsd:int, non-inline) — both kept
        assert_eq!(cm("?v < 60"), 2); // 50 and -10
    }

    #[test]
    fn relational_type_error_semantics() {
        // A numeric ordering comparison against a NON-numeric term is a SPARQL type
        // error -> the row is excluded (NOT a string comparison). Regression for the
        // adversarial-audit finding: `?v > -1` with ?v a string wrongly passed via a
        // lexical-byte comparison. A negative threshold is non-sargable, so this runs
        // the residual compare path. Cross-checks several non-numeric term kinds.
        let g = Graph::load_str(
            r#"@prefix ex: <http://ex/> . @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
               ex:s1 ex:v 50 .
               ex:s2 ex:v "hi" .
               ex:s3 ex:v ex:thing .
               ex:s4 ex:v "true"^^xsd:boolean .
               ex:s5 ex:v "bonjour"@fr ."#,
            "turtle",
        )
        .unwrap();
        let cnt = |q: &str| query(&g, q).unwrap().len();
        // Only the number 50 satisfies an ordering comparison; every non-numeric is a
        // type error -> excluded.
        assert_eq!(cnt("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v FILTER(?v > -1) }"), 1);
        assert_eq!(cnt("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v FILTER(?v < 100) }"), 1);
        // `=` / `!=` are NOT type errors across recognised types: term-distinct values
        // are KNOWN different, so `!= 5` is true for all four non-numerics + (50!=5).
        assert_eq!(cnt("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v FILTER(?v != 5) }"), 5);
        assert_eq!(cnt("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v FILTER(?v = 5) }"), 0);
        // Value equality across integer datatypes still holds.
        let g2 = Graph::load_str(
            "@prefix ex: <http://ex/> . ex:a ex:v \"05\"^^<http://www.w3.org/2001/XMLSchema#integer> .",
            "turtle",
        )
        .unwrap();
        assert_eq!(query(&g2, "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v FILTER(?v = 5) }").unwrap().len(), 1);
        // 3-valued OR: `(?v > -1) || (?v = "hi")` — s1 true via >, s2 ("hi") true via =,
        // the rest error||false = false. So 2 rows.
        assert_eq!(
            cnt("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v FILTER((?v > -1) || (?v = \"hi\")) }"),
            2
        );
    }

    #[test]
    fn logical_error_propagation_and_short_circuit() {
        // Roborev follow-ups to the type-error fix: IF must propagate a condition
        // error (not select the else branch); && / || must short-circuit on the
        // dominating value; IN must reuse `=` semantics. Two rows: 50 and "hi".
        let g = Graph::load_str(
            "@prefix ex: <http://ex/> . ex:s1 ex:v 50 . ex:s2 ex:v \"hi\" .",
            "turtle",
        )
        .unwrap();
        let cnt = |q: &str| query(&g, q).unwrap().len();
        // IF(error, _, _) is an error -> the "hi" row is excluded (NOT the else branch).
        assert_eq!(cnt("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v FILTER(IF(?v > -1, true, true)) }"), 1);
        // `false && error` = false (short-circuit) -> excludes both rows.
        assert_eq!(cnt("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v FILTER((?v = \"no\") && (?v > -1)) }"), 0);
        // `true || error` = true (short-circuit) -> keeps both rows.
        assert_eq!(cnt("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v FILTER((?v != \"no\") || (?v > -1)) }"), 2);
        // IN reuses `=`: only the numeric matches the numeric list entry.
        assert_eq!(cnt("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v FILTER(?v IN (50)) }"), 1);
    }

    #[test]
    fn named_graph_unsupported() {
        let e = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s WHERE { GRAPH ?g { ?s ex:age ?a } }",
        );
        assert!(e.is_err());
    }

    #[test]
    fn bnode_prefix_does_not_collide() {
        // A user variable that looks like the old synthetic prefix must be a real,
        // projected SELECT * variable now that synthetic vars use an illegal char.
        let r = query(&g(), "SELECT * WHERE { ?__bn_x <http://ex/age> ?a }").unwrap();
        assert_eq!(r.len(), 3);
        assert_eq!(r.vars.len(), 2); // both ?__bn_x and ?a are visible
    }

    // Differential: the engine's join machinery (merge/hash/greedy ordering)
    // must agree with a brute-force nested-loop evaluator over a random graph,
    // for a chain query and a triangle (cyclic) query.
    #[test]
    fn differential_vs_naive() {
        // Deterministic pseudo-random graph (seeded LCG, no external randomness).
        let mut seed: u64 = 0x1234_5678;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) as u32
        };
        let n_nodes = 40u32;
        let mut edges: Vec<(u32, u32)> = Vec::new();
        let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
        for _ in 0..200 {
            let a = next() % n_nodes;
            let b = next() % n_nodes;
            edges.push((a, b));
            ttl.push_str(&format!("ex:n{a} ex:e ex:n{b} .\n"));
        }
        edges.sort_unstable();
        edges.dedup();

        let graph = Graph::load_str(&ttl, "turtle").unwrap();

        // Chain: ?a e ?b . ?b e ?c
        let chain = naive_count_chain(&edges);
        let q = query(&graph, "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c }").unwrap();
        assert_eq!(q.len(), chain, "chain join count mismatch");

        // Triangle: ?a e ?b . ?b e ?c . ?c e ?a  (cyclic)
        let tri = naive_count_triangle(&edges);
        let q = query(
            &graph,
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?b ex:e ?c . ?c ex:e ?a }",
        )
        .unwrap();
        assert_eq!(q.len(), tri, "triangle join count mismatch");
    }

    fn naive_count_chain(e: &[(u32, u32)]) -> usize {
        let mut c = 0;
        for &(_, b) in e {
            for &(b2, _) in e {
                if b == b2 {
                    c += 1;
                }
            }
        }
        c
    }

    fn naive_count_triangle(e: &[(u32, u32)]) -> usize {
        use std::collections::HashSet;
        let set: HashSet<(u32, u32)> = e.iter().copied().collect();
        let mut c = 0;
        for &(a, b) in e {
            for &(b2, cc) in e {
                if b == b2 && set.contains(&(cc, a)) {
                    c += 1;
                }
            }
        }
        c
    }
}
