//! The parse-once / execute-many seam ([`sparq_engine::PreparedQuery`], recorded by
//! sparq-rsp): every prepared entry point must behave byte-identically to its
//! string twin, across query forms, repeated executions, different graphs, dataset
//! clauses, and budgets.

use sparq_core::Graph;
use sparq_engine::{
    ask, ask_prepared, construct_ntriples, construct_prepared, count, count_prepared,
    describe_prepared, query, query_json, query_json_prepared, query_prepared,
    query_prepared_with_budget, PreparedQuery, QueryBudget,
};

fn g(n: usize) -> Graph {
    let mut ttl = String::from("@prefix : <http://ex/> .\n");
    for i in 0..n {
        ttl.push_str(&format!(":s{i} :p :o{} ; :v {} .\n", i % 3, i));
    }
    Graph::load_str(&ttl, "turtle").unwrap()
}

#[test]
fn prepared_select_matches_string_path_across_graphs_and_runs() {
    let q = "PREFIX : <http://ex/> SELECT ?s ?o WHERE { ?s :p ?o . ?s :v ?n . FILTER(?n >= 2) } ORDER BY ?s";
    let prepared = PreparedQuery::parse(q).unwrap();
    for graph in [g(5), g(9)] {
        // Parse once, execute repeatedly: identical to the string path every time.
        let expect = query(&graph, q).unwrap();
        for _ in 0..3 {
            let got = query_prepared(&graph, &prepared).unwrap();
            assert_eq!(got.vars, expect.vars);
            assert_eq!(got.rows, expect.rows);
        }
        assert_eq!(
            query_json_prepared(&graph, &prepared).unwrap(),
            query_json(&graph, q).unwrap()
        );
        assert_eq!(
            count_prepared(&graph, &prepared).unwrap(),
            count(&graph, q).unwrap()
        );
    }
}

#[test]
fn prepared_ask_construct_describe() {
    let graph = g(4);
    let ask_q = PreparedQuery::parse("PREFIX : <http://ex/> ASK { :s1 :p ?o }").unwrap();
    assert!(ask_prepared(&graph, &ask_q).unwrap());
    assert_eq!(
        ask_prepared(&graph, &ask_q).unwrap(),
        ask(&graph, "PREFIX : <http://ex/> ASK { :s1 :p ?o }").unwrap()
    );
    // ASK through the generic prepared entry points: unit-row / boolean forms.
    assert_eq!(query_prepared(&graph, &ask_q).unwrap().rows.len(), 1);
    assert_eq!(count_prepared(&graph, &ask_q).unwrap(), 1);
    assert!(query_json_prepared(&graph, &ask_q)
        .unwrap()
        .contains("\"boolean\":true"));

    let con = "PREFIX : <http://ex/> CONSTRUCT { ?s :copy ?o } WHERE { ?s :p ?o }";
    let con_p = PreparedQuery::parse(con).unwrap();
    let triples = construct_prepared(&graph, &con_p).unwrap();
    assert_eq!(triples.len(), 4);
    assert_eq!(
        sparq_engine::triples_to_ntriples(&triples),
        construct_ntriples(&graph, con).unwrap()
    );

    let desc = PreparedQuery::parse("PREFIX : <http://ex/> DESCRIBE :s0").unwrap();
    let triples = describe_prepared(&graph, &desc).unwrap();
    assert_eq!(triples.len(), 2, "CBD of :s0 = its :p and :v triples");

    // Form mismatches are clean errors, same as the string paths.
    assert!(ask_prepared(&graph, &con_p).is_err());
    assert!(construct_prepared(&graph, &ask_q).is_err());
}

#[test]
fn prepared_respects_dataset_clause_and_budget() {
    // FROM NAMED / GRAPH: the active dataset is rebuilt per execution from the
    // TARGET graph, not captured at parse time.
    let nq = "<http://ex/a> <http://ex/p> <http://ex/b> <http://ex/g1> .\n\
              <http://ex/c> <http://ex/p> <http://ex/d> .";
    let graph = Graph::load_dataset(nq, "nquads").unwrap();
    let p = PreparedQuery::parse("SELECT ?s WHERE { GRAPH <http://ex/g1> { ?s ?p ?o } }").unwrap();
    assert_eq!(query_prepared(&graph, &p).unwrap().rows.len(), 1);
    // The same prepared query against a graph WITHOUT g1 finds nothing.
    let bare = Graph::load_str("<http://ex/c> <http://ex/p> <http://ex/d> .", "ntriples").unwrap();
    assert_eq!(query_prepared(&bare, &p).unwrap().rows.len(), 0);

    // Budgets apply per execution.
    let big = g(50);
    let all = PreparedQuery::parse("SELECT * WHERE { ?s ?p ?o }").unwrap();
    let tight = QueryBudget {
        max_rows: Some(3),
        ..QueryBudget::unlimited()
    };
    let err = query_prepared_with_budget(&big, &all, &tight).unwrap_err();
    assert!(
        err.contains("budget"),
        "expected a budget error, got: {err}"
    );
    assert_eq!(query_prepared(&big, &all).unwrap().rows.len(), 100);
}

#[test]
fn prepared_from_algebra_and_fromstr() {
    let graph = g(3);
    // From<spargebra::Query>: programmatically built / rewritten algebra executes.
    let q = spargebra::SparqlParser::new()
        .parse_query("PREFIX : <http://ex/> SELECT ?o WHERE { :s0 :p ?o }")
        .unwrap();
    let p: PreparedQuery = q.into();
    assert_eq!(query_prepared(&graph, &p).unwrap().rows.len(), 1);
    assert!(matches!(p.query(), spargebra::Query::Select { .. }));
    // FromStr round-trip + parse errors surface.
    let p2: PreparedQuery = "SELECT * WHERE { ?s ?p ?o }".parse().unwrap();
    assert_eq!(query_prepared(&graph, &p2).unwrap().rows.len(), 6);
    assert!("SELEKT nonsense".parse::<PreparedQuery>().is_err());
}
