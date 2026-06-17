//! [OPUS-4.8] Constraint-component coverage (sq-qap0) for the branches the
//! audit flagged in `eval.rs` (~58% covered). The crate's own (non-W3C) tests
//! exercised only datatype / nodeKind / minInclusive / class; the W3C suite
//! self-skips without fixtures, so these give the constraint pipeline durable,
//! fixture-free coverage with hand-computed conforming / violating outcomes.
//!
//! Each test pins the source component IRI and (where the spec fixes it) the
//! reported `sh:value` and the result count — the per-pair / per-occurrence
//! reporting policy `eval.rs` documents.

use sparq_core::Graph;
use sparq_shacl::ValidationReport;

const PREFIXES: &str = r#"
    @prefix sh: <http://www.w3.org/ns/shacl#> .
    @prefix ex: <http://example.org/> .
    @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
"#;

fn run(data: &str, shapes: &str) -> ValidationReport {
    let data = Graph::load_str(&format!("{PREFIXES}{data}"), "turtle").unwrap();
    let shapes = Graph::load_str(&format!("{PREFIXES}{shapes}"), "turtle").unwrap();
    sparq_shacl::validate(&data, &shapes)
}

/// Count of results whose source component IRI ends with `local`.
fn count_component(r: &ValidationReport, local: &str) -> usize {
    r.results
        .iter()
        .filter(|x| x.source_component.ends_with(local))
        .count()
}

fn flagged_values(r: &ValidationReport) -> Vec<String> {
    let mut v: Vec<String> = r
        .results
        .iter()
        .filter_map(|x| x.value.as_ref().map(|t| t.to_string()))
        .collect();
    v.sort();
    v
}

// ---- sh:equals (symmetric: missing-in-either direction) ----

#[test]
fn equals_reports_both_directions() {
    // sh:path ex:a ; sh:equals ex:b. Value sets must be EQUAL. Here ex:a = {1,2}
    // and ex:b = {2,3}: 1 is in a-not-b, 3 is in b-not-a -> two results.
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:a ; sh:equals ex:b ] .
    "#;
    let r = run("ex:n ex:a 1 , 2 ; ex:b 2 , 3 .", shapes);
    assert!(!r.conforms);
    assert_eq!(
        count_component(&r, "EqualsConstraintComponent"),
        2,
        "{}",
        r.to_text()
    );

    // Equal sets conform.
    let ok = run("ex:n ex:a 1 , 2 ; ex:b 1 , 2 .", shapes);
    assert!(ok.conforms, "{}", ok.to_text());
}

// ---- sh:disjoint ----

#[test]
fn disjoint_flags_shared_values() {
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:a ; sh:disjoint ex:b ] .
    "#;
    // ex:shared appears under both -> one disjoint violation on that value.
    let r = run(
        "ex:n ex:a ex:shared , ex:x ; ex:b ex:shared , ex:y .",
        shapes,
    );
    assert!(!r.conforms);
    assert_eq!(count_component(&r, "DisjointConstraintComponent"), 1);
    assert_eq!(
        flagged_values(&r),
        vec!["<http://example.org/shared>".to_string()]
    );

    // No overlap conforms.
    let ok = run("ex:n ex:a ex:x ; ex:b ex:y .", shapes);
    assert!(ok.conforms);
}

// ---- sh:lessThan / sh:lessThanOrEquals (one result per failing (value,other) pair) ----

#[test]
fn less_than_reports_per_pair() {
    // sh:path ex:v ; sh:lessThan ex:limit. ex:v = {5}; ex:limit = {3, 4}. 5 is
    // not < 3 and not < 4 -> TWO results, both sh:value 5.
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:v ; sh:lessThan ex:limit ] .
    "#;
    let r = run("ex:n ex:v 5 ; ex:limit 3 , 4 .", shapes);
    assert!(!r.conforms);
    assert_eq!(
        count_component(&r, "LessThanConstraintComponent"),
        2,
        "{}",
        r.to_text()
    );
    assert_eq!(
        flagged_values(&r),
        vec![
            "\"5\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
            "\"5\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string(),
        ]
    );

    // 1 < 3 and 1 < 4 -> conforms.
    let ok = run("ex:n ex:v 1 ; ex:limit 3 , 4 .", shapes);
    assert!(ok.conforms);
}

#[test]
fn less_than_or_equals_allows_equal() {
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:v ; sh:lessThanOrEquals ex:limit ] .
    "#;
    // 3 <= 3 conforms under OrEquals.
    let ok = run("ex:n ex:v 3 ; ex:limit 3 .", shapes);
    assert!(ok.conforms, "{}", ok.to_text());
    // 4 <= 3 fails.
    let bad = run("ex:n ex:v 4 ; ex:limit 3 .", shapes);
    assert!(!bad.conforms);
    assert_eq!(
        count_component(&bad, "LessThanOrEqualsConstraintComponent"),
        1
    );
}

// ---- range components across string / boolean / dateTime orderings ----

#[test]
fn range_string_ordering() {
    // sh:minInclusive "m" on strings: "a" < "m" fails, "z" >= "m" passes.
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:s ; sh:minInclusive "m" ] .
    "#;
    let r = run(r#"ex:n ex:s "a" , "z" ."#, shapes);
    assert!(!r.conforms);
    assert_eq!(count_component(&r, "MinInclusiveConstraintComponent"), 1);
    assert_eq!(flagged_values(&r), vec!["\"a\"".to_string()]);
}

#[test]
fn range_datetime_ordering_and_incomparability() {
    // sh:maxInclusive a timezoned dateTime. A value with the SAME tz-presence
    // compares; a tz-less value is INCOMPARABLE (not satisfied -> violation).
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:d ;
            sh:maxInclusive "2024-06-01T00:00:00Z"^^xsd:dateTime ] .
    "#;
    // Earlier timezoned value conforms.
    let ok = run(
        r#"ex:n ex:d "2024-01-01T00:00:00Z"^^xsd:dateTime ."#,
        shapes,
    );
    assert!(ok.conforms, "{}", ok.to_text());
    // A timezone-less value is incomparable with the timezoned bound ->
    // constraint not satisfied -> violation.
    let bad = run(r#"ex:n ex:d "2024-01-01T00:00:00"^^xsd:dateTime ."#, shapes);
    assert!(
        !bad.conforms,
        "tz-less vs tz'd dateTime must be incomparable"
    );
    assert_eq!(count_component(&bad, "MaxInclusiveConstraintComponent"), 1);
}

#[test]
fn range_max_exclusive_boundary() {
    // sh:maxExclusive 10: 10 fails (not < 10), 9 passes.
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:v ; sh:maxExclusive 10 ] .
    "#;
    let r = run("ex:n ex:v 10 , 9 .", shapes);
    assert!(!r.conforms);
    assert_eq!(count_component(&r, "MaxExclusiveConstraintComponent"), 1);
    assert_eq!(
        flagged_values(&r),
        vec!["\"10\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string()]
    );
}

// ---- sh:minLength / sh:maxLength (char count, IRI vs literal vs blank) ----

#[test]
fn length_counts_characters_and_rejects_blank() {
    // sh:minLength 3 over mixed value kinds. "ab" (2) fails, "abc" (3) passes,
    // an IRI uses its full string (long, passes), a blank node has no string
    // repr (fails).
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:v ; sh:minLength 3 ] .
    "#;
    let r = run(r#"ex:n ex:v "ab" , "abc" , ex:longIri , [ ] ."#, shapes);
    assert!(!r.conforms);
    // "ab" and the blank node fail; "abc" and the IRI pass.
    assert_eq!(
        count_component(&r, "MinLengthConstraintComponent"),
        2,
        "{}",
        r.to_text()
    );
}

#[test]
fn max_length_unicode_chars_not_bytes() {
    // "é" + "é" via combining? Use a 2-char multibyte string vs maxLength 2.
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:v ; sh:maxLength 2 ] .
    "#;
    // "αβ" is two chars (4 bytes) -> within maxLength 2.
    let ok = run("ex:n ex:v \"\u{3b1}\u{3b2}\" .", shapes);
    assert!(ok.conforms, "char-count, not byte-count: {}", ok.to_text());
    // Three chars exceeds.
    let bad = run("ex:n ex:v \"\u{3b1}\u{3b2}\u{3b3}\" .", shapes);
    assert!(!bad.conforms);
}

// ---- sh:pattern (+ sh:flags) ----

#[test]
fn pattern_with_flags() {
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:code ; sh:pattern "^ab" ; sh:flags "i" ] .
    "#;
    // "ABxyz" matches case-insensitively; "zab" does not (anchored at start).
    let r = run(r#"ex:n ex:code "ABxyz" , "zab" ."#, shapes);
    assert!(!r.conforms);
    assert_eq!(count_component(&r, "PatternConstraintComponent"), 1);
    assert_eq!(flagged_values(&r), vec!["\"zab\"".to_string()]);
}

// ---- sh:languageIn / sh:uniqueLang ----

#[test]
fn language_in_basic_range_match() {
    // sh:languageIn ("en" "fr"): "en-GB" matches the "en" range; "de" does not;
    // a non-language-tagged literal fails.
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:label ; sh:languageIn ( "en" "fr" ) ] .
    "#;
    let r = run(
        r#"ex:n ex:label "hello"@en-GB , "bonjour"@fr , "hallo"@de , "plain" ."#,
        shapes,
    );
    assert!(!r.conforms);
    // "de" and the plain literal fail; "en-GB" and "fr" pass.
    assert_eq!(
        count_component(&r, "LanguageInConstraintComponent"),
        2,
        "{}",
        r.to_text()
    );
}

#[test]
fn unique_lang_flags_duplicate_language() {
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:label ; sh:uniqueLang true ] .
    "#;
    // Two @en values -> one uniqueLang violation (no sh:value).
    let r = run(r#"ex:n ex:label "a"@en , "b"@en , "c"@fr ."#, shapes);
    assert!(!r.conforms);
    assert_eq!(count_component(&r, "UniqueLangConstraintComponent"), 1);
    assert!(
        r.results[0].value.is_none(),
        "uniqueLang reports no sh:value"
    );

    // Distinct languages conform.
    let ok = run(r#"ex:n ex:label "a"@en , "b"@fr ."#, shapes);
    assert!(ok.conforms);
}

// ---- sh:in / sh:hasValue ----

#[test]
fn in_enumeration_value_semantics() {
    // sh:in ( 1 2 ) uses value equality: an int 1 is in; 3 is not.
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:v ; sh:in ( 1 2 ) ] .
    "#;
    let r = run("ex:n ex:v 1 , 3 .", shapes);
    assert!(!r.conforms);
    assert_eq!(count_component(&r, "InConstraintComponent"), 1);
    assert_eq!(
        flagged_values(&r),
        vec!["\"3\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string()]
    );
}

#[test]
fn has_value_requires_presence() {
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n , ex:m ;
          sh:property [ sh:path ex:status ; sh:hasValue ex:active ] .
    "#;
    let r = run(
        "ex:n ex:status ex:active , ex:x . ex:m ex:status ex:x .",
        shapes,
    );
    assert!(!r.conforms);
    // ex:m lacks ex:active -> one violation (no sh:value); ex:n conforms.
    assert_eq!(count_component(&r, "HasValueConstraintComponent"), 1);
    assert_eq!(
        r.results[0].focus_node.to_string(),
        "<http://example.org/m>"
    );
    assert!(r.results[0].value.is_none());
}

// ---- sh:closed (+ sh:ignoredProperties) ----

#[test]
fn closed_flags_extra_predicates_and_honours_ignored() {
    // A closed node shape allowing only ex:name (declared via sh:property) and
    // ignoring rdf:type. An extra ex:age predicate is flagged; rdf:type is not.
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:closed true ;
          sh:ignoredProperties ( rdf:type ) ;
          sh:property [ sh:path ex:name ] .
    "#;
    let r = run(
        r#"ex:n a ex:Thing ; ex:name "N" ; ex:age 5 ; ex:extra 1 ."#,
        shapes,
    );
    assert!(!r.conforms);
    // ex:age and ex:extra are disallowed (rdf:type ignored, ex:name allowed).
    assert_eq!(
        count_component(&r, "ClosedConstraintComponent"),
        2,
        "{}",
        r.to_text()
    );
    // The closed result carries the offending predicate as the result path.
    let paths: Vec<String> = r
        .results
        .iter()
        .filter(|x| x.source_component.ends_with("ClosedConstraintComponent"))
        .filter_map(|x| x.path.as_ref().map(|p| p.to_turtle()))
        .collect();
    assert!(paths.iter().any(|p| p.contains("age")), "{paths:?}");
    assert!(paths.iter().any(|p| p.contains("extra")), "{paths:?}");

    // Only allowed/ignored predicates conform.
    let ok = run(r#"ex:n a ex:Thing ; ex:name "N" ."#, shapes);
    assert!(ok.conforms, "{}", ok.to_text());
}

// ---- logical: sh:not / sh:and / sh:or / sh:xone ----

#[test]
fn logical_not() {
    // sh:not [ sh:datatype xsd:string ]: a string value VIOLATES (it conforms to
    // the negated shape); an integer conforms.
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:v ; sh:not [ sh:datatype xsd:string ] ] .
    "#;
    let bad = run(r#"ex:n ex:v "hi" ."#, shapes);
    assert!(!bad.conforms);
    assert_eq!(count_component(&bad, "NotConstraintComponent"), 1);
    let ok = run("ex:n ex:v 5 .", shapes);
    assert!(ok.conforms);
}

#[test]
fn logical_and_or_xone() {
    // sh:and ( [minInclusive 0] [maxInclusive 10] ): 5 conforms, 20 fails (one
    // And violation on the value).
    let and_shape = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:v ;
            sh:and ( [ sh:minInclusive 0 ] [ sh:maxInclusive 10 ] ) ] .
    "#;
    assert!(run("ex:n ex:v 5 .", and_shape).conforms);
    let and_bad = run("ex:n ex:v 20 .", and_shape);
    assert!(!and_bad.conforms);
    assert_eq!(count_component(&and_bad, "AndConstraintComponent"), 1);

    // sh:or ( [datatype xsd:string] [datatype xsd:integer] ): a boolean matches
    // neither -> one Or violation.
    let or_shape = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:v ;
            sh:or ( [ sh:datatype xsd:string ] [ sh:datatype xsd:integer ] ) ] .
    "#;
    assert!(run("ex:n ex:v 5 .", or_shape).conforms);
    assert!(run(r#"ex:n ex:v "x" ."#, or_shape).conforms);
    let or_bad = run("ex:n ex:v true .", or_shape);
    assert!(!or_bad.conforms);
    assert_eq!(count_component(&or_bad, "OrConstraintComponent"), 1);

    // sh:xone ( [datatype xsd:string] [minLength 1] ): a string matches BOTH ->
    // xone (exactly one) fails; an integer matches NEITHER -> also fails; an IRI
    // of length>=1 matches only minLength -> conforms.
    let xone_shape = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:v ;
            sh:xone ( [ sh:datatype xsd:string ] [ sh:nodeKind sh:IRI ] ) ] .
    "#;
    // a string matches only the datatype branch -> exactly one -> conforms.
    assert!(run(r#"ex:n ex:v "s" ."#, xone_shape).conforms);
    // an IRI matches only the nodeKind branch -> exactly one -> conforms.
    assert!(run("ex:n ex:v ex:thing .", xone_shape).conforms);
    // an integer matches neither -> xone fails.
    let xone_bad = run("ex:n ex:v 7 .", xone_shape);
    assert!(!xone_bad.conforms);
    assert_eq!(count_component(&xone_bad, "XoneConstraintComponent"), 1);
}

// ---- sh:qualifiedValueShape (+ min/max + disjoint) ----

#[test]
fn qualified_value_shape_min_max() {
    // At least 2 of the ex:item values must be ex:Special.
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:item ;
            sh:qualifiedValueShape [ sh:class ex:Special ] ;
            sh:qualifiedMinCount 2 ] .
    "#;
    // Only one Special -> minCount violation.
    let bad = run("ex:n ex:item ex:a , ex:b . ex:a a ex:Special .", shapes);
    assert!(!bad.conforms);
    assert_eq!(
        count_component(&bad, "QualifiedMinCountConstraintComponent"),
        1
    );

    // Two Special -> conforms.
    let ok = run(
        "ex:n ex:item ex:a , ex:b . ex:a a ex:Special . ex:b a ex:Special .",
        shapes,
    );
    assert!(ok.conforms, "{}", ok.to_text());
}

#[test]
fn qualified_max_count() {
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:item ;
            sh:qualifiedValueShape [ sh:class ex:Special ] ;
            sh:qualifiedMaxCount 1 ] .
    "#;
    // Two Special exceeds max 1 -> violation.
    let bad = run(
        "ex:n ex:item ex:a , ex:b . ex:a a ex:Special . ex:b a ex:Special .",
        shapes,
    );
    assert!(!bad.conforms);
    assert_eq!(
        count_component(&bad, "QualifiedMaxCountConstraintComponent"),
        1
    );
}

// ---- sh:node / sh:property nesting reporting ----

#[test]
fn node_constraint_on_path_values() {
    // sh:path ex:addr ; sh:node ex:AddrShape requiring ex:city minCount 1.
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:property [ sh:path ex:addr ; sh:node ex:AddrShape ] .
        ex:AddrShape a sh:NodeShape ;
          sh:property [ sh:path ex:city ; sh:minCount 1 ] .
    "#;
    // The address has no city -> the NodeConstraintComponent flags the addr value.
    let bad = run("ex:n ex:addr ex:addr1 .", shapes);
    assert!(!bad.conforms);
    assert_eq!(count_component(&bad, "NodeConstraintComponent"), 1);
    assert_eq!(
        flagged_values(&bad),
        vec!["<http://example.org/addr1>".to_string()]
    );

    let ok = run(
        r#"ex:n ex:addr ex:addr1 . ex:addr1 ex:city "NYC" ."#,
        shapes,
    );
    assert!(ok.conforms, "{}", ok.to_text());
}

// ---- targets: subjectsOf / objectsOf ----

#[test]
fn target_subjects_and_objects_of() {
    // sh:targetSubjectsOf ex:owns: every subject of an ex:owns triple is a focus
    // node, required to have a name.
    let subj_shape = r#"
        ex:S a sh:NodeShape ; sh:targetSubjectsOf ex:owns ;
          sh:property [ sh:path ex:name ; sh:minCount 1 ] .
    "#;
    let r = run(
        r#"ex:alice ex:owns ex:car . ex:bob ex:owns ex:bike ; ex:name "Bob" ."#,
        subj_shape,
    );
    assert!(!r.conforms);
    // ex:alice (subject, no name) fails; ex:bob conforms.
    assert_eq!(r.results.len(), 1, "{}", r.to_text());
    assert_eq!(
        r.results[0].focus_node.to_string(),
        "<http://example.org/alice>"
    );

    // sh:targetObjectsOf ex:owns: every OBJECT of ex:owns must be an ex:Vehicle.
    let obj_shape = r#"
        ex:S a sh:NodeShape ; sh:targetObjectsOf ex:owns ;
          sh:nodeKind sh:IRI ;
          sh:class ex:Vehicle .
    "#;
    let r = run(
        "ex:alice ex:owns ex:car . ex:car a ex:Vehicle . ex:bob ex:owns ex:notV .",
        obj_shape,
    );
    assert!(!r.conforms);
    // ex:notV is an object that is not a Vehicle.
    assert_eq!(count_component(&r, "ClassConstraintComponent"), 1);
    assert_eq!(
        r.results[0].focus_node.to_string(),
        "<http://example.org/notV>"
    );
}

// ---- sh:deactivated ----

#[test]
fn deactivated_shape_is_skipped() {
    // A shape that would fail, but sh:deactivated true -> no results.
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode ex:n ;
          sh:deactivated true ;
          sh:property [ sh:path ex:name ; sh:minCount 1 ] .
    "#;
    let r = run("ex:n a ex:Thing .", shapes);
    assert!(
        r.conforms,
        "deactivated shape must not produce results: {}",
        r.to_text()
    );
}

// ---- [OPUS-4.8] (sq-vg3y) SHACL-1.2 core/node forms ----

// ---- sh:datatype disjunctive (list) form ----

#[test]
fn datatype_list_form_accepts_any_listed() {
    // sh:datatype ( xsd:string rdf:langString ): a plain string and a lang-tagged
    // literal both conform; an integer (neither) violates. Mirrors W3C datatype-003.
    let shapes = r#"
        ex:S a sh:NodeShape ;
          sh:targetNode "plain" ;
          sh:targetNode "tagged"@en ;
          sh:targetNode 42 ;
          sh:datatype ( xsd:string rdf:langString ) .
    "#;
    let r = run("", shapes);
    assert!(!r.conforms, "{}", r.to_text());
    assert_eq!(count_component(&r, "DatatypeConstraintComponent"), 1);
    // The integer is the sole violator.
    assert!(flagged_values(&r).iter().any(|v| v.contains("42")));
}

#[test]
fn datatype_single_iri_still_works() {
    // Regression: the single-IRI form is the singleton-set case.
    let shapes = r#"
        ex:S a sh:NodeShape ; sh:targetNode "x" , 1 ;
          sh:datatype xsd:string .
    "#;
    let r = run("", shapes);
    assert_eq!(count_component(&r, "DatatypeConstraintComponent"), 1);
}

// ---- sh:nodeKind disjunctive (list) form ----

#[test]
fn nodekind_list_form_accepts_iri_or_blanknode() {
    // sh:nodeKind ( sh:BlankNode sh:IRI ): an IRI and a blank node conform; a
    // literal violates. Mirrors W3C nodeKind-002.
    let shapes = r#"
        ex:S a sh:NodeShape ;
          sh:targetNode ex:iri ;
          sh:targetNode "true"^^xsd:boolean ;
          sh:nodeKind ( sh:BlankNode sh:IRI ) .
    "#;
    let r = run("ex:iri ex:p 0 .", shapes);
    assert!(!r.conforms, "{}", r.to_text());
    assert_eq!(count_component(&r, "NodeKindConstraintComponent"), 1);
}

// ---- sh:closed sh:ByTypes ----

#[test]
fn closed_by_types_uses_properties_of_value_node_types() {
    // closed-003 in miniature: ex:RootClass is closed-by-types and declares
    // ex:rootClassProperty1; ex:SubClass (subclass) declares ex:subClassProperty1.
    // An instance of ROOT (not Sub) that uses ex:subClassProperty1 is closed out;
    // an instance of Sub that uses it is allowed (its type pulls the property in).
    let shapes = r#"
        ex:RootClass a rdfs:Class, sh:NodeShape ;
          sh:property [ sh:path ex:rootClassProperty1 ] ;
          sh:closed sh:ByTypes .
        ex:SubClass a rdfs:Class, sh:NodeShape ;
          rdfs:subClassOf ex:RootClass ;
          sh:property [ sh:path ex:subClassProperty1 ] ;
          sh:closed sh:ByTypes .
    "#;
    let data = r#"
        ex:RootInstance a ex:RootClass ; ex:subClassProperty1 1 .
        ex:SubInstance a ex:SubClass ; ex:rootClassProperty1 1 ; ex:subClassProperty1 3 .
    "#;
    let r = run(data, shapes);
    assert!(!r.conforms, "{}", r.to_text());
    // Exactly one closed violation: ex:RootInstance's ex:subClassProperty1.
    assert_eq!(count_component(&r, "ClosedConstraintComponent"), 1);
    let closed: Vec<_> = r
        .results
        .iter()
        .filter(|x| x.source_component.ends_with("ClosedConstraintComponent"))
        .collect();
    assert!(closed[0].focus_node.to_string().contains("RootInstance"));
}

// ---- sh:memberShape ----

#[test]
fn member_shape_checks_each_list_member() {
    // sh:memberShape [ sh:nodeKind sh:IRI ]: a list of IRIs conforms; a list with
    // a literal member, AND a value that is not a SHACL list at all, both violate.
    let shapes = r#"
        ex:S a sh:NodeShape ;
          sh:targetNode ex:goodList, ex:badList, ex:notAList ;
          sh:memberShape [ sh:nodeKind sh:IRI ] .
    "#;
    let data = r#"
        ex:goodList rdf:first ex:a ; rdf:rest ( ex:b ) .
        ex:badList  rdf:first ex:a ; rdf:rest ( "lit" ) .
        ex:notAList ex:p 0 .
    "#;
    let r = run(data, shapes);
    assert!(!r.conforms, "{}", r.to_text());
    // One top-level result per violating value node (badList + notAList).
    assert_eq!(count_component(&r, "MemberShapeConstraintComponent"), 2);
}

// ---- sh:uniqueMembers ----

#[test]
fn unique_members_flags_duplicate_and_nonlist() {
    let shapes = r#"
        ex:S a sh:NodeShape ;
          sh:targetNode ex:uniq, ex:dup, ex:notAList ;
          sh:uniqueMembers true .
    "#;
    let data = r#"
        ex:uniq rdf:first 1 ; rdf:rest ( 2 3 ) .
        ex:dup  rdf:first 1 ; rdf:rest ( 2 1 ) .
        ex:notAList ex:p 0 .
    "#;
    let r = run(data, shapes);
    assert!(!r.conforms, "{}", r.to_text());
    // ex:dup (duplicate 1) + ex:notAList (not a list) -> two results; ex:uniq ok.
    assert_eq!(count_component(&r, "UniqueMembersConstraintComponent"), 2);
}

// ---- sh:maxListLength / sh:minListLength ----

#[test]
fn max_list_length_flags_too_long_and_nonlist() {
    // Value nodes: ex:ok (2 members, ok), rdf:nil (0, ok), ex:long (3, violates),
    // ex:notAList (not a list, violates). Mirrors W3C maxListLength-001.
    let shapes = r#"
        ex:S a sh:NodeShape ;
          sh:targetNode ex:ok, rdf:nil, ex:long, ex:notAList ;
          sh:maxListLength 2 .
    "#;
    let data = r#"
        ex:ok   rdf:first 1 ; rdf:rest ( 2 ) .
        ex:long rdf:first 1 ; rdf:rest ( 2 3 ) .
        ex:notAList ex:p 0 .
    "#;
    let r = run(data, shapes);
    assert!(!r.conforms, "{}", r.to_text());
    assert_eq!(count_component(&r, "MaxListLengthConstraintComponent"), 2);
}

#[test]
fn min_list_length_flags_too_short_and_nil() {
    // rdf:nil is a valid SHACL list of length 0 < 1 -> violates; ex:notAList is
    // not a list -> violates; ex:ok (length 2) passes. Mirrors W3C minListLength-001.
    let shapes = r#"
        ex:S a sh:NodeShape ;
          sh:targetNode ex:ok, rdf:nil, ex:notAList ;
          sh:minListLength 1 .
    "#;
    let data = r#"
        ex:ok rdf:first 1 ; rdf:rest ( 2 ) .
        ex:notAList ex:p 0 .
    "#;
    let r = run(data, shapes);
    assert!(!r.conforms, "{}", r.to_text());
    assert_eq!(count_component(&r, "MinListLengthConstraintComponent"), 2);
}

// ---- sh:uniqueValuesFor ----

#[test]
fn unique_values_for_single_property() {
    // Two target nodes sharing the same ex:id -> a symmetric pair of results.
    // A node with no ex:id contributes nothing. Mirrors W3C uniqueValuesFor-001.
    let shapes = r#"
        ex:S a sh:NodeShape ;
          sh:targetClass ex:Thing ;
          sh:uniqueValuesFor ex:id .
    "#;
    let data = r#"
        ex:a a ex:Thing ; ex:id "001" .
        ex:b a ex:Thing ; ex:id "002" .
        ex:dup1 a ex:Thing ; ex:id "DUP" .
        ex:dup2 a ex:Thing ; ex:id "DUP" .
        ex:noid a ex:Thing .
    "#;
    let r = run(data, shapes);
    assert!(!r.conforms, "{}", r.to_text());
    // dup1 -> dup2 and dup2 -> dup1: exactly two results.
    assert_eq!(count_component(&r, "UniqueValuesForConstraintComponent"), 2);
}

#[test]
fn unique_values_for_composite_key_and_missing_values() {
    // Composite key ( ex:notation ex:scheme ): only nodes that agree on BOTH
    // collide. Mirrors W3C uniqueValuesFor-002.
    let shapes = r#"
        ex:S a sh:NodeShape ;
          sh:targetClass ex:Concept ;
          sh:uniqueValuesFor ( ex:notation ex:scheme ) .
    "#;
    let data = r#"
        ex:v1 a ex:Concept ; ex:notation "A1" ; ex:scheme ex:S1 .
        ex:v2 a ex:Concept ; ex:notation "A2" ; ex:scheme ex:S1 .
        ex:bad1 a ex:Concept ; ex:notation "A1" ; ex:scheme ex:S2 .
        ex:bad2 a ex:Concept ; ex:notation "A1" ; ex:scheme ex:S2 .
    "#;
    let r = run(data, shapes);
    assert!(!r.conforms, "{}", r.to_text());
    // Only bad1/bad2 agree on both -> two results; v1 (notation A1, scheme S1)
    // does NOT collide with bad1 (scheme differs).
    assert_eq!(count_component(&r, "UniqueValuesForConstraintComponent"), 2);
}

#[test]
fn unique_values_for_no_values_conforms() {
    // No instance has the property -> conforms (W3C uniqueValuesFor-004).
    let shapes = r#"
        ex:S a sh:NodeShape ;
          sh:targetClass ex:Thing ;
          sh:uniqueValuesFor ex:id .
    "#;
    let r = run("ex:a a ex:Thing . ex:b a ex:Thing .", shapes);
    assert!(r.conforms, "{}", r.to_text());
}
