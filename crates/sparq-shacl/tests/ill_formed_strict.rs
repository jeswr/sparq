//! [FABLE-5] (sq-11a) Ill-formed shapes graphs are reported as a FAILURE by the
//! strict channel — the W3C test-suite `sht:Failure` outcome — instead of only
//! being skipped leniently.
//!
//! Two-sided pinning:
//!   * every known ill-formed construct makes `validate_strict` return
//!     `Err(ShaclFailure)` carrying an [`sparq_shacl::IllFormedConstruct`] whose
//!     `predicate` names the offending SHACL predicate, while the lenient
//!     `validate` still produces a report (unchanged policy);
//!   * well-formed edge cases (a negative `sh:minCount`, `sh:closed false`, an
//!     EMPTY `sh:in ()` list, the SHACL-1.2 disjunctive `sh:datatype` list, …)
//!     must NOT be flagged — the false-positive guard.
//!
//! The W3C core suite contains no ill-formed-shapes `sht:Failure` entries, so
//! these fixtures are hand-crafted (the suite's Failure entries all live in
//! `sparql/pre-binding/`, pinned by `w3c_sparql_shacl12.rs`).

use sparq_core::Graph;
use sparq_shacl::{ShaclFailure, ShapesModel, ValidationReport};

const PREFIXES: &str = r#"
    @prefix sh: <http://www.w3.org/ns/shacl#> .
    @prefix ex: <http://example.org/> .
    @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
"#;

const DATA: &str = "ex:n ex:p 1 . ex:n a ex:T .";

fn shapes_graph(shapes: &str) -> Graph {
    Graph::load_str(&format!("{PREFIXES}{shapes}"), "turtle").unwrap()
}

fn run_strict(shapes: &str) -> Result<ValidationReport, ShaclFailure> {
    let data = Graph::load_str(&format!("{PREFIXES}{DATA}"), "turtle").unwrap();
    sparq_shacl::validate_strict(&data, &shapes_graph(shapes))
}

fn run_lenient(shapes: &str) -> ValidationReport {
    let data = Graph::load_str(&format!("{PREFIXES}{DATA}"), "turtle").unwrap();
    sparq_shacl::validate(&data, &shapes_graph(shapes))
}

/// Asserts the shapes graph is rejected by the strict channel with an
/// [`sparq_shacl::IllFormedConstruct`] on the expected SHACL predicate — and
/// that the lenient `validate` still produces a report for the same graph.
fn assert_ill_formed(shapes: &str, expect_predicate_local: &str) {
    let err = match run_strict(shapes) {
        Err(e) => e,
        Ok(_) => panic!(
            "expected sht:Failure (strict rejection) for sh:{}, got a report:\n{}",
            expect_predicate_local, shapes
        ),
    };
    let expected = format!("http://www.w3.org/ns/shacl#{}", expect_predicate_local);
    assert!(
        err.ill_formed.iter().any(|i| i.predicate == expected),
        "expected an ill-formed record on {}, got {:?}",
        expected,
        err.ill_formed
    );
    assert!(
        err.pre_binding.is_empty(),
        "these fixtures are not pre-binding violations"
    );
    // The lenient channel is UNCHANGED: same graph, still an infallible report.
    let _report = run_lenient(shapes);
}

fn assert_well_formed(shapes: &str) {
    if let Err(e) = run_strict(shapes) {
        panic!("well-formed shapes graph wrongly rejected: {}\n{}", e, shapes);
    }
}

// ---- ill-formed constructs → sht:Failure (strict) ----

#[test]
fn ill_formed_path_expression_fails() {
    // A literal is never a path.
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:property [ sh:path "not-a-path" ; sh:minCount 1 ] ."#,
        "path",
    );
}

#[test]
fn multiple_path_values_fail() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:property [ sh:path ex:p ; sh:path ex:q ; sh:minCount 1 ] ."#,
        "path",
    );
}

#[test]
fn non_integer_min_count_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:property [ sh:path ex:p ; sh:minCount "three" ] ."#,
        "minCount",
    );
}

#[test]
fn iri_valued_max_length_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:maxLength ex:big ."#,
        "maxLength",
    );
}

#[test]
fn literal_datatype_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:datatype "xsd:integer" ."#,
        "datatype",
    );
}

#[test]
fn literal_member_in_node_kind_list_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:nodeKind ( sh:IRI "sh:Literal" ) ."#,
        "nodeKind",
    );
}

#[test]
fn literal_class_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:class "ex:T" ."#,
        "class",
    );
}

#[test]
fn non_literal_pattern_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:pattern ex:regex ."#,
        "pattern",
    );
}

#[test]
fn non_list_language_in_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:languageIn "en" ."#,
        "languageIn",
    );
}

#[test]
fn literal_comparand_for_less_than_fails() {
    // The sh:lessThan comparand must be a predicate IRI / path, never a literal.
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:property [ sh:path ex:p ; sh:lessThan "ex:q" ] ."#,
        "lessThan",
    );
}

#[test]
fn non_list_sh_in_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:in ex:notAList ."#,
        "in",
    );
}

#[test]
fn malformed_sh_in_list_tail_fails() {
    // _:l has rdf:first but no rdf:rest: the lenient list truncates, the strict
    // well-formedness check rejects.
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:in _:l .
           _:l rdf:first ex:a ."#,
        "in",
    );
}

#[test]
fn non_list_sh_or_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:or ex:notAList ."#,
        "or",
    );
}

#[test]
fn literal_shape_in_sh_and_list_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:and ( "not-a-shape" ) ."#,
        "and",
    );
}

#[test]
fn literal_sh_node_shape_ref_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:node "ex:Other" ."#,
        "node",
    );
}

#[test]
fn literal_sh_property_shape_ref_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:property "ex:P" ."#,
        "property",
    );
}

#[test]
fn literal_target_class_fails() {
    assert_ill_formed(r#"ex:S a sh:NodeShape ; sh:targetClass "ex:T" ."#, "targetClass");
}

#[test]
fn literal_target_subjects_of_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetSubjectsOf "ex:p" ."#,
        "targetSubjectsOf",
    );
}

#[test]
fn non_boolean_unique_lang_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:property [ sh:path ex:p ; sh:uniqueLang "yes" ] ."#,
        "uniqueLang",
    );
}

#[test]
fn non_boolean_closed_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:closed "shut" ."#,
        "closed",
    );
}

#[test]
fn malformed_ignored_properties_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:closed true ;
             sh:ignoredProperties ex:notAList ."#,
        "ignoredProperties",
    );
}

#[test]
fn non_integer_qualified_min_count_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:property [ sh:path ex:p ;
               sh:qualifiedValueShape [ sh:datatype xsd:integer ] ;
               sh:qualifiedMinCount "one" ] ."#,
        "qualifiedMinCount",
    );
}

#[test]
fn literal_member_shape_ref_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:memberShape "ex:M" ."#,
        "memberShape",
    );
}

#[test]
fn literal_some_value_shape_ref_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:someValue "ex:M" ."#,
        "someValue",
    );
}

#[test]
fn literal_reifier_shape_ref_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:reifierShape "ex:M" ."#,
        "reifierShape",
    );
}

#[test]
fn literal_qualified_value_shape_ref_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:property [ sh:path ex:p ; sh:qualifiedValueShape "ex:M" ;
                           sh:qualifiedMinCount 1 ] ."#,
        "qualifiedValueShape",
    );
}

#[test]
fn non_list_sh_xone_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:xone "not-a-list" ."#,
        "xone",
    );
}

#[test]
fn literal_target_objects_of_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetObjectsOf "ex:p" ."#,
        "targetObjectsOf",
    );
}

#[test]
fn literal_unique_values_for_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:property [ sh:path ex:p ; sh:uniqueValuesFor "ex:key" ] ."#,
        "uniqueValuesFor",
    );
}

#[test]
fn non_literal_min_exclusive_comparand_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:minExclusive ex:zero ."#,
        "minExclusive",
    );
}

#[test]
fn non_literal_flags_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:pattern "^a" ; sh:flags ex:i ."#,
        "flags",
    );
}

#[test]
fn iri_member_in_language_in_list_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:languageIn ( "en" ex:fr ) ."#,
        "languageIn",
    );
}

#[test]
fn non_integer_qualified_max_count_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:property [ sh:path ex:p ;
               sh:qualifiedValueShape [ sh:datatype xsd:integer ] ;
               sh:qualifiedMaxCount ex:five ] ."#,
        "qualifiedMaxCount",
    );
}

#[test]
fn ill_formed_equals_comparand_path_fails() {
    // A cyclic blank-node path expression is unparsable.
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:property [ sh:path ex:p ; sh:equals _:c ] .
           _:c sh:inversePath _:c ."#,
        "equals",
    );
}

#[test]
fn sparql_constraint_without_select_fails() {
    assert_ill_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:sparql [ sh:message "no query here" ] ."#,
        "select",
    );
}

// ---- well-formed edge cases must NOT be flagged (false-positive guard) ----

#[test]
fn well_formed_shapes_graphs_pass_strict() {
    // A representative well-formed graph exercising the instrumented parses.
    assert_well_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:targetClass ex:T ;
             sh:targetSubjectsOf ex:p ; sh:closed true ; sh:ignoredProperties ( ex:q ) ;
             sh:in ( 1 2 ) ; sh:or ( [ sh:datatype xsd:integer ] [ sh:datatype xsd:string ] ) ;
             sh:node ex:Other ;
             sh:property [ sh:path ex:p ; sh:minCount 1 ; sh:maxCount 5 ;
                           sh:pattern "^a" ; sh:flags "i" ; sh:uniqueLang true ;
                           sh:lessThan ex:q ;
                           sh:qualifiedValueShape [ sh:datatype xsd:integer ] ;
                           sh:qualifiedMinCount 1 ] .
           ex:Other a sh:NodeShape ; sh:datatype xsd:integer ."#,
    );
}

#[test]
fn negative_min_count_is_not_ill_formed() {
    // A negative count is a WELL-formed xsd:integer (trivially satisfiable);
    // it stays a silent skip, not a failure.
    assert_well_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:property [ sh:path ex:p ; sh:minCount -1 ] ."#,
    );
}

#[test]
fn closed_false_is_not_ill_formed() {
    assert_well_formed(r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:closed false ."#);
}

#[test]
fn empty_sh_in_list_is_not_ill_formed() {
    // `sh:in ()` is the well-formed EMPTY SHACL list (rdf:nil).
    assert_well_formed(r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:in () ."#);
}

#[test]
fn disjunctive_datatype_list_is_not_ill_formed() {
    // The SHACL-1.2 disjunctive list form.
    assert_well_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:datatype ( xsd:integer xsd:string ) ; sh:nodeKind ( sh:IRI sh:Literal ) ."#,
    );
}

#[test]
fn sequence_path_is_not_ill_formed() {
    assert_well_formed(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:property [ sh:path ( ex:p ex:q ) ; sh:minCount 0 ] ."#,
    );
}

#[test]
fn plain_rdfs_class_root_is_not_flagged() {
    // A shapes graph that also carries vocabulary triples (a plain rdfs:Class
    // with no SHACL constructs) must not produce failures.
    assert_well_formed(
        r#"@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
           ex:T a rdfs:Class ; rdfs:label "a class" .
           ex:S a sh:NodeShape ; sh:targetClass ex:T ; sh:nodeKind sh:IRI ."#,
    );
}

// ---- the recording surface itself ----

#[test]
fn shapes_model_ill_formed_accessor_reports_records() {
    // DIRECT unit test for the new public accessor: node + predicate + message.
    let g = shapes_graph(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
             sh:property [ sh:path ex:p ; sh:minCount "three" ] ."#,
    );
    let model = ShapesModel::parse(&g);
    let recs = model.ill_formed();
    assert_eq!(recs.len(), 1, "exactly one ill-formed construct: {:?}", recs);
    assert_eq!(recs[0].predicate, "http://www.w3.org/ns/shacl#minCount");
    assert!(recs[0].message.contains("integer"), "message: {}", recs[0].message);
    // The carrying node is the property shape (a blank node).
    assert!(matches!(recs[0].node, oxrdf::Term::BlankNode(_)));
    // A well-formed graph records nothing.
    let ok = ShapesModel::parse(&shapes_graph(
        r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:nodeKind sh:IRI ."#,
    ));
    assert!(ok.ill_formed().is_empty());
}

#[test]
fn shacl_failure_display_mentions_ill_formed() {
    let err = run_strict(r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ; sh:closed "shut" ."#)
        .expect_err("must reject");
    let text = err.to_string();
    assert!(
        text.contains("1 ill-formed shapes-graph construct(s)"),
        "display: {}",
        text
    );
    assert!(text.contains("closed"), "display names the predicate: {}", text);
}

#[test]
fn lenient_validate_still_skips_ill_formed_constructs() {
    // The load-bearing invariant pair: the SAME graph is a strict failure and a
    // lenient skip — and the skip really skips (the unparsable sh:minCount
    // imposes no constraint, so the report conforms).
    let shapes = r#"ex:S a sh:NodeShape ; sh:targetNode ex:n ;
                      sh:property [ sh:path ex:missing ; sh:minCount "three" ] ."#;
    assert!(run_strict(shapes).is_err());
    let report = run_lenient(shapes);
    assert!(report.conforms, "lenient skip unchanged: {}", report.to_text());
}
