//! Determinism and metric-law coverage for dictionary-grounded query repair.

use proptest::prelude::*;
use spargebra::{Query, SparqlParser};
use sparq_core::strdist::edit_distance;
use sparq_core::Graph;
use sparq_nlq::constrain::{dictionary_repair_message, unknown_terms, TermRole};

const DBO: &str = "http://dbpedia.org/ontology/";

fn graph() -> Graph {
    Graph::load_str(
        r#"
            @prefix dbo: <http://dbpedia.org/ontology/> .
            <http://example.org/film> a dbo:Movie ;
                dbo:director <http://example.org/director> ;
                dbo:directedBy <http://example.org/director> ;
                dbo:writer <http://example.org/writer> ;
                dbo:producer <http://example.org/producer> .
        "#,
        "turtle",
    )
    .expect("fixture graph parses")
}

fn parse(query: &str) -> Query {
    SparqlParser::new()
        .parse_query(query)
        .expect("test query parses")
}

fn short_string() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<char>(), 0..=8)
        .prop_map(|characters| characters.into_iter().collect())
}

#[test]
fn unknown_terms_are_deterministic_for_multiple_unknown_iris() {
    let graph = graph();
    let query = parse(
        "PREFIX dbo: <http://dbpedia.org/ontology/>\n\
         SELECT ?film WHERE {\n\
           ?film dbo:directr ?director ; dbo:writr ?writer ; a dbo:Movi .\n\
         }",
    );

    let first = unknown_terms(&graph, &query);
    assert_eq!(first, unknown_terms(&graph, &query));
    assert_eq!(first, unknown_terms(&graph, &query));
    assert_eq!(first.len(), 3);
    assert_eq!(first[0].iri, format!("{DBO}directr"));
    assert_eq!(first[0].role, TermRole::Predicate);
    assert_eq!(first[1].iri, format!("{DBO}writr"));
    assert_eq!(first[1].role, TermRole::Predicate);
    assert_eq!(first[2].iri, format!("{DBO}Movi"));
    assert_eq!(first[2].role, TermRole::Class);
}

proptest! {
    #[test]
    fn edit_distance_is_symmetric(a in short_string(), b in short_string()) {
        prop_assert_eq!(edit_distance(&a, &b), edit_distance(&b, &a));
    }

    #[test]
    fn edit_distance_satisfies_triangle_inequality(
        a in short_string(),
        b in short_string(),
        c in short_string(),
    ) {
        let direct = edit_distance(&a, &c);
        let via_b = edit_distance(&a, &b) + edit_distance(&b, &c);
        prop_assert!(direct <= via_b, "d(a,c)={direct} > d(a,b)+d(b,c)={via_b}");
    }
}

#[test]
fn nearest_known_ranks_the_closest_typo_first_and_distances_ascend() {
    let graph = graph();
    let query = parse(
        "PREFIX dbo: <http://dbpedia.org/ontology/>\n\
         SELECT ?film WHERE { ?film dbo:directr ?director }",
    );

    let unknowns = unknown_terms(&graph, &query);
    assert_eq!(unknowns.len(), 1);
    let suggestions = &unknowns[0].suggestions;
    assert_eq!(
        suggestions.first().map(String::as_str),
        Some("http://dbpedia.org/ontology/director")
    );

    let distances: Vec<_> = suggestions
        .iter()
        .map(|suggestion| {
            edit_distance(
                "directr",
                suggestion
                    .strip_prefix(DBO)
                    .expect("suggestions stay in the typo's namespace"),
            )
        })
        .collect();
    assert!(
        distances.windows(2).all(|pair| pair[0] <= pair[1]),
        "suggestions must have ascending edit distance: {suggestions:?} -> {distances:?}"
    );
}

#[test]
fn empty_graph_reports_unknown_term_without_suggestions() {
    let graph = Graph::default();
    let query = parse(
        "PREFIX dbo: <http://dbpedia.org/ontology/>\n\
         SELECT ?film WHERE { ?film dbo:directr ?director }",
    );

    let unknowns = unknown_terms(&graph, &query);
    assert_eq!(unknowns.len(), 1);
    assert!(unknowns[0].suggestions.is_empty());
}

#[test]
fn dictionary_repair_message_is_byte_stable() {
    let graph = graph();
    let query = parse(
        "PREFIX dbo: <http://dbpedia.org/ontology/>\n\
         SELECT ?film WHERE { ?film dbo:directr ?director ; a dbo:Movi }",
    );
    let unknowns = unknown_terms(&graph, &query);

    let first = dictionary_repair_message(&unknowns);
    assert_eq!(first, dictionary_repair_message(&unknowns));
    assert_eq!(first, dictionary_repair_message(&unknowns));
    assert!(first.contains("predicate <http://dbpedia.org/ontology/directr>"));
    assert!(first.contains("class <http://dbpedia.org/ontology/Movi>"));
    assert!(first.contains("did you mean <http://dbpedia.org/ontology/director>"));
}
