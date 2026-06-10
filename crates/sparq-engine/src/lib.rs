//! sparq-engine: a SPARQL query engine over [`sparq_core::Graph`].
//!
//! Supported (M2): SELECT with Basic Graph Patterns evaluated by greedy
//! cardinality-ordered sort-merge / hash joins over the permutation indexes;
//! FILTER (a useful expression subset with XSD-numeric-aware comparisons);
//! OPTIONAL, UNION, MINUS, BIND, VALUES; aggregation (COUNT/SUM/AVG/MIN/MAX/
//! GROUP_CONCAT) with GROUP BY and HAVING (as a post-group FILTER); ORDER BY;
//! DISTINCT/REDUCED/LIMIT/OFFSET; projection and sub-SELECT. SPARQL is parsed
//! to algebra by `spargebra`. Values computed at query time (BIND, aggregates)
//! are interned in a per-query local vocabulary. ASK runs natively; CONSTRUCT
//! and DESCRIBE (T16) return RDF graphs — see [`construct`] / [`describe`].
//! Later milestones add worst-case-optimal joins, a DP planner and property
//! paths.

mod construct;
mod exec;
pub mod json;
mod update;
pub use construct::{
    construct, construct_ntriples, construct_ntriples_with_budget, construct_with_budget, describe,
    describe_with_budget, triples_to_ntriples,
};
pub use update::update;

use oxrdf::{Term, Variable};
use sparq_core::Graph;
use spargebra::{Query, SparqlParser};

/// A cooperative resource budget for one query evaluation (T15 server hardening).
///
/// The executor checks it at coarse sites only (operator entry, once per outer
/// iteration of the big scan/join loops), so enforcement is approximate but cheap:
/// an unlimited budget (the default) costs nothing on the hot paths. When a limit
/// trips, evaluation stops and the query fails with
/// `"query budget exceeded (timeout)"` / `"query budget exceeded (max-rows)"`.
#[derive(Debug, Clone, Copy, Default)]
pub struct QueryBudget {
    /// Wall-clock deadline. Native only: `std::time::Instant` is unusable on
    /// `wasm32-unknown-unknown` (it panics), so the field does not exist there —
    /// the row budget below stays fully portable.
    #[cfg(not(target_arch = "wasm32"))]
    pub deadline: Option<std::time::Instant>,
    /// Upper bound on the rows of any materialised (intermediate or final) result.
    /// This is a *working-set* bound: a query whose intermediate result exceeds it
    /// is refused even if a later operator (e.g. LIMIT) would have shrunk it.
    pub max_rows: Option<usize>,
}

impl QueryBudget {
    /// The do-nothing budget every non-budgeted entry point uses.
    pub fn unlimited() -> Self {
        Self::default()
    }
}

/// Executes a SPARQL query string against a graph, materialising the solutions.
pub fn query(graph: &Graph, sparql: &str) -> Result<QueryResult, String> {
    query_with_budget(graph, sparql, &QueryBudget::unlimited())
}

/// [`query`] under a cooperative [`QueryBudget`] (deadline / max result rows).
pub fn query_with_budget(graph: &Graph, sparql: &str, budget: &QueryBudget) -> Result<QueryResult, String> {
    let q = SparqlParser::new().parse_query(sparql).map_err(|e| e.to_string())?;
    let _guard = exec::budget::install(budget);
    exec::set_query_base(q.base_iri().map(|b| b.as_str()));
    match q {
        Query::Select { pattern, .. } => exec::eval_select(graph, &pattern),
        // ASK as a QueryResult: zero variables, and one (empty) row iff the pattern
        // is satisfiable — the standard "unit row" encoding of a boolean result.
        Query::Ask { pattern, .. } => Ok(QueryResult {
            vars: Vec::new(),
            rows: if exec::eval_ask(graph, &pattern)? { vec![Vec::new()] } else { Vec::new() },
        }),
        _ => Err("only SELECT and ASK queries are supported".into()),
    }
}

/// Executes an ASK query: `true` iff the pattern has at least one solution.
/// Evaluation early-exits where the engine has a streaming path (the pattern is
/// evaluated under a `LIMIT 1`).
pub fn ask(graph: &Graph, sparql: &str) -> Result<bool, String> {
    ask_with_budget(graph, sparql, &QueryBudget::unlimited())
}

/// [`ask`] under a cooperative [`QueryBudget`] (deadline / max result rows).
pub fn ask_with_budget(graph: &Graph, sparql: &str, budget: &QueryBudget) -> Result<bool, String> {
    let q = SparqlParser::new().parse_query(sparql).map_err(|e| e.to_string())?;
    let _guard = exec::budget::install(budget);
    exec::set_query_base(q.base_iri().map(|b| b.as_str()));
    match q {
        Query::Ask { pattern, .. } => exec::eval_ask(graph, &pattern),
        _ => Err("ask() requires an ASK query".into()),
    }
}

/// Executes a SELECT and serialises it directly to a SPARQL 1.1 JSON results string,
/// skipping the intermediate `QueryResult` and its per-cell `oxrdf::Term` allocation
/// (the dictionary case is formatted straight from the stored prefix/suffix). This is
/// the fast path for the actual end-use — returning results to the CLI / browser.
pub fn query_json(graph: &Graph, sparql: &str) -> Result<String, String> {
    query_json_with_budget(graph, sparql, &QueryBudget::unlimited())
}

/// [`query_json`] under a cooperative [`QueryBudget`] (deadline / max result rows).
pub fn query_json_with_budget(graph: &Graph, sparql: &str, budget: &QueryBudget) -> Result<String, String> {
    let q = SparqlParser::new().parse_query(sparql).map_err(|e| e.to_string())?;
    let _guard = exec::budget::install(budget);
    exec::set_query_base(q.base_iri().map(|b| b.as_str()));
    match q {
        Query::Select { pattern, .. } => exec::eval_select_json(graph, &pattern),
        // The SPARQL 1.1 JSON results boolean form.
        Query::Ask { pattern, .. } => Ok(format!("{{\"head\":{{}},\"boolean\":{}}}", exec::eval_ask(graph, &pattern)?)),
        _ => Err("only SELECT and ASK queries are supported".into()),
    }
}

/// Flush threshold for [`query_json_chunks_with_budget`]: large enough that the
/// per-chunk overhead (stream item, HTTP write) is negligible, small enough that a
/// streamed body never holds a second whole-result copy in memory.
const JSON_CHUNK_BYTES: usize = 64 * 1024;

/// [`query_json_with_budget`] as an ordered sequence of chunks whose concatenation is
/// **byte-identical** to the single-string result — the server streams these as one
/// HTTP body instead of concatenating a giant `String` (T16), which removes the
/// second whole-result copy from peak memory on large SELECTs.
pub fn query_json_chunks_with_budget(graph: &Graph, sparql: &str, budget: &QueryBudget) -> Result<Vec<String>, String> {
    let q = SparqlParser::new().parse_query(sparql).map_err(|e| e.to_string())?;
    let _guard = exec::budget::install(budget);
    exec::set_query_base(q.base_iri().map(|b| b.as_str()));
    match q {
        Query::Select { pattern, .. } => exec::eval_select_json_chunks(graph, &pattern, Some(JSON_CHUNK_BYTES)),
        Query::Ask { pattern, .. } => {
            Ok(vec![format!("{{\"head\":{{}},\"boolean\":{}}}", exec::eval_ask(graph, &pattern)?)])
        }
        _ => Err("only SELECT and ASK queries are supported".into()),
    }
}

/// Counts the solutions of a SELECT query *without* materialising the result
/// terms (the id-level row count equals the solution count). Used to measure
/// engine compute in isolation from result serialisation.
pub fn count(graph: &Graph, sparql: &str) -> Result<usize, String> {
    count_with_budget(graph, sparql, &QueryBudget::unlimited())
}

/// [`count`] under a cooperative [`QueryBudget`] (the server's budgeted ASK path).
pub fn count_with_budget(graph: &Graph, sparql: &str, budget: &QueryBudget) -> Result<usize, String> {
    let q = SparqlParser::new().parse_query(sparql).map_err(|e| e.to_string())?;
    let _guard = exec::budget::install(budget);
    exec::set_query_base(q.base_iri().map(|b| b.as_str()));
    match q {
        Query::Select { pattern, .. } => exec::count_select(graph, &pattern),
        // An ASK counts its unit row: 1 when satisfiable, 0 otherwise.
        Query::Ask { pattern, .. } => Ok(usize::from(exec::eval_ask(graph, &pattern)?)),
        _ => Err("only SELECT and ASK queries are supported".into()),
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
    fn exact_decimal_arithmetic_filter() {
        // Arithmetic on integer/decimal operands must be EXACT (not f64): `0.1 + 0.2`
        // is 0.3 exactly, `0.3 - 0.1` is 0.2, integers past 2^53 stay distinct. The f64
        // arithmetic path gets all of these wrong.
        let data = r#"@prefix ex: <http://ex/> . @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
            ex:a ex:v "0.1"^^xsd:decimal .
            ex:b ex:v "0.2"^^xsd:decimal .
            ex:c ex:v "0.3"^^xsd:decimal .
            ex:d ex:v "9007199254740993"^^xsd:integer ."#;
        let gg = Graph::load_str(data, "turtle").unwrap();
        let n = |q: &str| query(&gg, q).unwrap().len();
        let pfx = "PREFIX ex: <http://ex/> PREFIX xsd: <http://www.w3.org/2001/XMLSchema#> ";
        // 0.2 + 0.1 = 0.3 exactly -> the 0.2 row passes.
        assert_eq!(n(&format!("{pfx}SELECT ?s WHERE {{ ?s ex:v ?v FILTER((?v + \"0.1\"^^xsd:decimal) = \"0.3\"^^xsd:decimal) }}")), 1);
        // 0.3 - 0.1 = 0.2 exactly.
        assert_eq!(n(&format!("{pfx}SELECT ?s WHERE {{ ?s ex:v ?v FILTER((?v - \"0.1\"^^xsd:decimal) = \"0.2\"^^xsd:decimal) }}")), 1);
        // 0.1 * 0.1 = 0.01 (none equal, but ordering exact): values < 0.25 are a,b (0.1,0.2).
        assert_eq!(n(&format!("{pfx}SELECT ?s WHERE {{ ?s ex:v ?v FILTER((?v + ?v) <= \"0.4\"^^xsd:decimal) }}")), 2);
        // Integer arithmetic beyond 2^53 stays exact: (n - 1) = 2^53+2 is false for n=2^53+3.
        assert_eq!(n(&format!("{pfx}SELECT ?s WHERE {{ ?s ex:v ?v FILTER((?v - 1) = \"9007199254740992\"^^xsd:integer) }}")), 1);
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
            // 3-pattern STAR on ?s — the N-star count path (Σ_s Πc_i(s)).
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:age ?a . ?s ex:name ?n . ?s ex:knows ?k }",
            // star with a constant-object pattern mixed in.
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:age ?a . ?s ex:name ?n . ?s ex:knows ex:bob }",
            // 3-pattern CHAIN — NOT a star; must fall back to materialised count (still correct).
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:knows ?b . ?b ex:knows ?c . ?c ex:age ?x }",
            // OPTIONAL — lazy left-join count Σ_s c_left(s)·max(1, c_right(s)).
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:name ?n OPTIONAL { ?s ex:age ?a } }",
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:name ?n OPTIONAL { ?s ex:knows ?k } }",
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:age ?a OPTIONAL { ?s ex:knows ?k } }",
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
    fn query_json_matches_materialized_json() {
        // The direct id->JSON path must produce byte-identical output to building the
        // QueryResult (Terms) then serialising — across IRIs (prefix-factored), inline
        // integers, language tags, OPTIONAL-unbound cells, and computed aggregates.
        let queries = [
            "SELECT * WHERE { ?s ?p ?o }",
            "PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a }",
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:name ?n OPTIONAL { ?s ex:knows ?k } }",
            "PREFIX ex: <http://ex/> SELECT (AVG(?a) AS ?avg) WHERE { ?s ex:age ?a }",
            "PREFIX ex: <http://ex/> SELECT ?n WHERE { ?s ex:name ?n } ORDER BY ?n",
            "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a . FILTER(?a > 28) }",
        ];
        for q in queries {
            let direct = query_json(&g(), q).unwrap();
            let via_result = json::to_sparql_json(&query(&g(), q).unwrap());
            assert_eq!(direct, via_result, "json mismatch for: {q}");
        }
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
        // OPEN-WORLD `=` / `!=` (W3C open-world suite): an IRI and a language-tagged
        // literal are KNOWN different from a number (`!=` true), but a plain string or
        // boolean against a number is a cross-family TYPE ERROR -> excluded. So 50,
        // ex:thing and "bonjour"@fr pass; "hi" and true error out.
        assert_eq!(cnt("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v FILTER(?v != 5) }"), 3);
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
        // IN reuses `=`: only the numeric matches the numeric list entry.
        assert_eq!(cnt("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v FILTER(?v IN (50)) }"), 1);

        // Short-circuit, proven rigorously: STRLEN is UNSUPPORTED, so evaluating the
        // right operand returns Err and `query` would fail. The query SUCCEEDING is
        // proof the dominating left operand skipped the right (not just that the
        // truth table tolerates an error).
        let g1 = Graph::load_str("@prefix ex: <http://ex/> . ex:s1 ex:v 50 .", "turtle").unwrap();
        let r1 = query(&g1, "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v FILTER((?v = 99) && (STRLEN(?v) > 0)) }");
        assert_eq!(r1.expect("false && _ must short-circuit, not error").len(), 0);
        let r2 = query(&g1, "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v FILTER((?v = 50) || (STRLEN(?v) > 0)) }");
        assert_eq!(r2.expect("true || _ must short-circuit, not error").len(), 1);

        // IN preserves a type error (unbound operand): `!(?k IN (ex:x))` with ?k
        // unbound is error -> false -> excluded, NOT `!(false)` = true -> included.
        assert_eq!(
            cnt("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:v ?v OPTIONAL { ?s ex:nomatch ?k } FILTER(!(?k IN (<http://ex/x>))) }"),
            0
        );
    }

    #[test]
    fn named_graph_query_over_default_only_is_empty() {
        // A graph loaded without named graphs: GRAPH ?g matches nothing (no error). Named-graph
        // querying over an actual dataset is covered in exec::path_tests::named_graphs.
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s WHERE { GRAPH ?g { ?s ex:age ?a } }",
        )
        .unwrap();
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn bnode_prefix_does_not_collide() {
        // A user variable that looks like the old synthetic prefix must be a real,
        // projected SELECT * variable now that synthetic vars use an illegal char.
        let r = query(&g(), "SELECT * WHERE { ?__bn_x <http://ex/age> ?a }").unwrap();
        assert_eq!(r.len(), 3);
        assert_eq!(r.vars.len(), 2); // both ?__bn_x and ?a are visible
    }

    #[test]
    fn ask_true_false_and_result_forms() {
        // Satisfiable and unsatisfiable patterns.
        assert!(ask(&g(), "PREFIX ex: <http://ex/> ASK { ?s ex:age ?a }").unwrap());
        assert!(!ask(&g(), "PREFIX ex: <http://ex/> ASK { ?s ex:nope ?o }").unwrap());
        // ASK with FILTER / join.
        assert!(ask(&g(), "PREFIX ex: <http://ex/> ASK { ?s ex:age ?a FILTER(?a > 34) }").unwrap());
        assert!(!ask(&g(), "PREFIX ex: <http://ex/> ASK { ?s ex:age ?a FILTER(?a > 100) }").unwrap());
        // query(): unit-row encoding (zero vars, 0/1 rows).
        let r = query(&g(), "PREFIX ex: <http://ex/> ASK { ?s ex:age 30 }").unwrap();
        assert_eq!((r.vars.len(), r.rows.len()), (0, 1));
        let r = query(&g(), "PREFIX ex: <http://ex/> ASK { ?s ex:age 31 }").unwrap();
        assert_eq!((r.vars.len(), r.rows.len()), (0, 0));
        // count(): 1 / 0.
        assert_eq!(super::count(&g(), "PREFIX ex: <http://ex/> ASK { ?s ex:knows ?o }").unwrap(), 1);
        assert_eq!(super::count(&g(), "PREFIX ex: <http://ex/> ASK { ?s ex:nope ?o }").unwrap(), 0);
        // query_json(): the SPARQL 1.1 JSON boolean form.
        assert_eq!(
            query_json(&g(), "PREFIX ex: <http://ex/> ASK { ?s ex:knows ?o }").unwrap(),
            "{\"head\":{},\"boolean\":true}"
        );
        assert_eq!(
            query_json(&g(), "PREFIX ex: <http://ex/> ASK { ?s ex:nope ?o }").unwrap(),
            "{\"head\":{},\"boolean\":false}"
        );
        // ask() on a non-ASK query is a clear error.
        assert!(ask(&g(), "SELECT * WHERE { ?s ?p ?o }").is_err());
    }

    #[test]
    fn exists_and_not_exists() {
        // EXISTS correlated on the outer row: people who know someone.
        assert_eq!(count("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a FILTER EXISTS { ?s ex:knows ?o } }"), 2);
        // NOT EXISTS: people who know no-one (carol).
        let r = query(&g(), "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a FILTER NOT EXISTS { ?s ex:knows ?o } }").unwrap();
        assert_eq!(r.rows.len(), 1);
        assert_eq!(r.rows[0][0].as_ref().unwrap().to_string(), "<http://ex/carol>");
        // Uncorrelated EXISTS: a satisfiable / unsatisfiable constant pattern keeps / drops all rows.
        assert_eq!(count("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a FILTER EXISTS { ex:alice ex:knows ex:bob } }"), 3);
        assert_eq!(count("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a FILTER EXISTS { ex:alice ex:knows ex:carol } }"), 0);
        // Nested EXISTS (exists04 shape): knows someone who is known by someone.
        assert_eq!(
            count(
                "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a \
                 FILTER EXISTS { ?s ex:knows ?o FILTER EXISTS { ?o ex:knows ?p } } }"
            ),
            1 // alice: knows bob, and bob knows carol
        );
        // NOT EXISTS in ASK.
        assert!(ask(&g(), "PREFIX ex: <http://ex/> ASK { ?s ex:age 35 FILTER NOT EXISTS { ?s ex:knows ?o } }").unwrap());
    }

    #[test]
    #[cfg(feature = "digest")]
    fn hash_builtins() {
        let one = |q: &str| {
            let r = query(&g(), q).unwrap();
            r.rows[0][0].as_ref().unwrap().to_string()
        };
        // RFC / spec vectors for "abc".
        assert_eq!(one("SELECT (MD5(\"abc\") AS ?h) {}"), "\"900150983cd24fb0d6963f7d28e17f72\"");
        assert_eq!(one("SELECT (SHA1(\"abc\") AS ?h) {}"), "\"a9993e364706816aba3e25717850c26c9cd0d89d\"");
        assert_eq!(
            one("SELECT (SHA256(\"abc\") AS ?h) {}"),
            "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\""
        );
        // A language-tagged operand is a type error -> unbound.
        let r = query(&g(), "SELECT (MD5(\"abc\"@en) AS ?h) {}").unwrap();
        assert!(r.rows[0][0].is_none());
    }

    #[test]
    fn timezone_tz_bnode_uuid_builtins() {
        let one = |q: &str| {
            let r = query(&g(), q).unwrap();
            r.rows[0][0].as_ref().map(|t| t.to_string())
        };
        let dt = "\"2010-12-21T15:38:02-08:00\"^^<http://www.w3.org/2001/XMLSchema#dateTime>";
        assert_eq!(one(&format!("SELECT (TZ({dt}) AS ?x) {{}}")).unwrap(), "\"-08:00\"");
        assert_eq!(
            one(&format!("SELECT (TIMEZONE({dt}) AS ?x) {{}}")).unwrap(),
            "\"-PT8H\"^^<http://www.w3.org/2001/XMLSchema#dayTimeDuration>"
        );
        let dtz = "\"2010-12-21T15:38:02Z\"^^<http://www.w3.org/2001/XMLSchema#dateTime>";
        assert_eq!(one(&format!("SELECT (TZ({dtz}) AS ?x) {{}}")).unwrap(), "\"Z\"");
        assert_eq!(
            one(&format!("SELECT (TIMEZONE({dtz}) AS ?x) {{}}")).unwrap(),
            "\"PT0S\"^^<http://www.w3.org/2001/XMLSchema#dayTimeDuration>"
        );
        // No timezone: TZ -> "", TIMEZONE -> type error (unbound).
        let dn = "\"2011-02-01T01:02:03\"^^<http://www.w3.org/2001/XMLSchema#dateTime>";
        assert_eq!(one(&format!("SELECT (TZ({dn}) AS ?x) {{}}")).unwrap(), "\"\"");
        assert_eq!(one(&format!("SELECT (TIMEZONE({dn}) AS ?x) {{}}")), None);

        // BNODE(): two calls in one row give two distinct fresh blank nodes.
        let r = query(&g(), "SELECT (BNODE() AS ?a) (BNODE() AS ?b) {}").unwrap();
        let (a, b) = (r.rows[0][0].as_ref().unwrap(), r.rows[0][1].as_ref().unwrap());
        assert_ne!(a, b);
        // BNODE(str): same argument in the same solution -> same bnode; different
        // rows -> different bnodes.
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT (BNODE(\"x\") AS ?a) (BNODE(\"x\") AS ?b) WHERE { ?s ex:age ?n }",
        )
        .unwrap();
        assert_eq!(r.rows.len(), 3);
        for row in &r.rows {
            assert_eq!(row[0], row[1]);
        }
        assert_ne!(r.rows[0][0], r.rows[1][0]);

        // UUID()/STRUUID() (native targets).
        let u = one("SELECT (UUID() AS ?u) {}").unwrap();
        assert!(u.starts_with("<urn:uuid:") && u.len() == 47, "got: {u}");
        let s = one("SELECT (STRUUID() AS ?s) {}").unwrap();
        assert_eq!(s.len(), 38); // quoted 36-char UUID
        assert_ne!(one("SELECT (STRUUID() AS ?s) {}").unwrap(), s);
    }

    #[test]
    fn string_functions_preserve_language_tags() {
        let one = |q: &str| {
            let r = query(&g(), q).unwrap();
            r.rows[0][0].as_ref().map(|t| t.to_string())
        };
        assert_eq!(one("SELECT (UCASE(\"bar\"@en) AS ?x) {}").unwrap(), "\"BAR\"@en");
        assert_eq!(one("SELECT (LCASE(\"BAR\"@en) AS ?x) {}").unwrap(), "\"bar\"@en");
        assert_eq!(one("SELECT (SUBSTR(\"bar\"@en, 2) AS ?x) {}").unwrap(), "\"ar\"@en");
        assert_eq!(one("SELECT (SUBSTR(\"bar\"@en, 1, 1) AS ?x) {}").unwrap(), "\"b\"@en");
        // CONCAT: same tag everywhere -> tagged; mixed -> simple; non-string -> error.
        assert_eq!(one("SELECT (CONCAT(\"a\"@en, \"b\"@en) AS ?x) {}").unwrap(), "\"ab\"@en");
        assert_eq!(one("SELECT (CONCAT(\"a\"@en, \"b\") AS ?x) {}").unwrap(), "\"ab\"");
        assert_eq!(one("SELECT (CONCAT(\"a\", 1) AS ?x) {}"), None);
        // STRBEFORE/STRAFTER: result carries arg1's tag on a match, plain "" on no
        // match, and incompatible language tags are a type error.
        assert_eq!(one("SELECT (STRBEFORE(\"abc\"@en, \"b\") AS ?x) {}").unwrap(), "\"a\"@en");
        assert_eq!(one("SELECT (STRAFTER(\"abc\"@en, \"b\"@en) AS ?x) {}").unwrap(), "\"c\"@en");
        assert_eq!(one("SELECT (STRBEFORE(\"abc\"@en, \"z\") AS ?x) {}").unwrap(), "\"\"");
        assert_eq!(one("SELECT (STRBEFORE(\"abc\"@en, \"b\"@cy) AS ?x) {}"), None);
        assert_eq!(one("SELECT (STRAFTER(\"abc\", \"b\"@en) AS ?x) {}"), None);
        // REPLACE keeps the text's tag.
        #[cfg(feature = "regex")]
        assert_eq!(one("SELECT (REPLACE(\"abc\"@en, \"b\", \"-\") AS ?x) {}").unwrap(), "\"a-c\"@en");
        // STRDT/STRLANG require a simple-literal first argument.
        assert_eq!(
            one("PREFIX xsd: <http://www.w3.org/2001/XMLSchema#> SELECT (STRDT(\"1\", xsd:integer) AS ?x) {}").unwrap(),
            "\"1\"^^<http://www.w3.org/2001/XMLSchema#integer>"
        );
        assert_eq!(one("PREFIX xsd: <http://www.w3.org/2001/XMLSchema#> SELECT (STRDT(\"1\"@en, xsd:integer) AS ?x) {}"), None);
        assert_eq!(one("SELECT (STRLANG(\"a\", \"en\") AS ?x) {}").unwrap(), "\"a\"@en");
        assert_eq!(one("SELECT (STRLANG(\"a\"@en, \"en\") AS ?x) {}"), None);
    }

    #[test]
    fn query_json_chunks_concat_is_byte_identical() {
        // The streamed chunk sequence must concatenate to EXACTLY the single-string
        // JSON — across the single-pattern fast path, the general path, OPTIONAL
        // unbounds, aggregates and ASK.
        let queries = [
            "SELECT * WHERE { ?s ?p ?o }",                                                  // fast path
            "PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a }",                  // fast path, projected
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:name ?n OPTIONAL { ?s ex:knows ?k } }", // general
            "PREFIX ex: <http://ex/> SELECT (AVG(?a) AS ?avg) WHERE { ?s ex:age ?a }",      // aggregate
            "PREFIX ex: <http://ex/> ASK { ?s ex:age ?a }",                                 // boolean form
        ];
        let b = QueryBudget::unlimited();
        for q in queries {
            let single = query_json(&g(), q).unwrap();
            let chunks = query_json_chunks_with_budget(&g(), q, &b).unwrap();
            assert_eq!(chunks.concat(), single, "chunk concat mismatch for: {q}");
        }

        // A result big enough to actually split (>64 KiB of JSON): every chunk
        // boundary must fall so that the concatenation is still byte-identical.
        let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
        for i in 0..3000 {
            ttl.push_str(&format!("ex:subject{i} ex:somePredicate \"value-{i}-padding-padding\" .\n"));
        }
        let big = Graph::load_str(&ttl, "turtle").unwrap();
        for q in ["SELECT * WHERE { ?s ?p ?o }", "SELECT ?s ?o WHERE { ?s ?p ?o . ?s ?p2 ?o }"] {
            let single = query_json(&big, q).unwrap();
            let chunks = query_json_chunks_with_budget(&big, q, &b).unwrap();
            assert!(chunks.len() > 1, "expected a multi-chunk stream for: {q}");
            assert_eq!(chunks.concat(), single, "chunk concat mismatch for: {q}");
        }
    }

    #[test]
    fn query_json_chunks_respects_budget() {
        let b = QueryBudget { max_rows: Some(3), ..QueryBudget::unlimited() };
        let e = query_json_chunks_with_budget(&g(), "SELECT * WHERE { ?s ?p ?o }", &b).unwrap_err();
        assert!(e.contains("query budget exceeded (max-rows)"), "got: {e}");
    }

    #[test]
    fn budget_unlimited_matches_query() {
        let q = "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:knows ?b . ?b ex:age ?age }";
        let plain = query(&g(), q).unwrap();
        let budgeted = query_with_budget(&g(), q, &QueryBudget::unlimited()).unwrap();
        assert_eq!(plain.len(), budgeted.len());
        assert_eq!(
            query_json(&g(), q).unwrap(),
            query_json_with_budget(&g(), q, &QueryBudget::unlimited()).unwrap()
        );
    }

    #[test]
    fn budget_max_rows_refuses_not_truncates() {
        // 8 triples; max_rows 3 must REFUSE (error), never return a truncated result.
        let b = QueryBudget { max_rows: Some(3), ..QueryBudget::unlimited() };
        let e = query_with_budget(&g(), "SELECT * WHERE { ?s ?p ?o }", &b).map(|r| r.len()).unwrap_err();
        assert!(e.contains("query budget exceeded (max-rows)"), "got: {e}");
        let e = query_json_with_budget(&g(), "SELECT * WHERE { ?s ?p ?o }", &b).unwrap_err();
        assert!(e.contains("query budget exceeded (max-rows)"), "got: {e}");
        // A generous row budget changes nothing.
        let b = QueryBudget { max_rows: Some(1000), ..QueryBudget::unlimited() };
        assert_eq!(query_with_budget(&g(), "SELECT * WHERE { ?s ?p ?o }", &b).unwrap().len(), 8);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn budget_deadline_times_out() {
        // A deadline already in the past trips the first cooperative check.
        let b = QueryBudget {
            deadline: Some(std::time::Instant::now() - std::time::Duration::from_millis(1)),
            ..QueryBudget::unlimited()
        };
        let q = "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:knows ?b . ?b ex:age ?age }";
        let e = query_with_budget(&g(), q, &b).map(|r| r.len()).unwrap_err();
        assert!(e.contains("query budget exceeded (timeout)"), "got: {e}");
        // …and the budget never leaks into the next (unbudgeted) query on this thread.
        assert_eq!(query(&g(), q).unwrap().len(), 2);
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
