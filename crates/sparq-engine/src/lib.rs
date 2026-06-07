//! sparq-engine: a SPARQL query engine over [`sparq_core::Graph`].
//!
//! M1 scope: SELECT with Basic Graph Patterns, FILTER (a useful expression
//! subset), DISTINCT/LIMIT/OFFSET, and projection. SPARQL syntax is parsed to
//! the algebra by `spargebra`; we build the physical plan (greedy join order by
//! cardinality, index-scan leaves, hash joins) and execute it over the
//! dictionary-encoded permutation indexes. Later milestones add merge joins,
//! worst-case-optimal joins, OPTIONAL/UNION, aggregation and property paths.

mod exec;

use oxrdf::{Term, Variable};
use sparq_core::Graph;
use spargebra::Query;

/// Executes a SPARQL query string against a graph.
pub fn query(graph: &Graph, sparql: &str) -> Result<QueryResult, String> {
    let q = Query::parse(sparql, None).map_err(|e| e.to_string())?;
    match q {
        Query::Select { pattern, .. } => exec::eval_select(graph, &pattern),
        _ => Err("M1 supports SELECT queries only".into()),
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
        for &(a, b) in e {
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
