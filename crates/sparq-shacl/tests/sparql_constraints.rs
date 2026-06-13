//! [OPUS-4.8] SHACL-SPARQL (`sh:sparql`, W3C SHACL §5.2) integration tests.
//!
//! These exercise the high-value, well-defined part of SHACL-SPARQL: an
//! `sh:sparql` constraint whose `sh:select` is run per focus node with `$this`
//! pre-bound, each solution becoming a violation, with `sh:prefixes`,
//! `sh:message` (incl. `{?var}` substitution), `?value` and `?path` mapping, and
//! the conforming (no-solution) case.

use sparq_core::Graph;
use sparq_shacl::validate;

fn run(data: &str, shapes: &str) -> sparq_shacl::ValidationReport {
    let data = Graph::load_str(data, "turtle").unwrap();
    let shapes = Graph::load_str(shapes, "turtle").unwrap();
    validate(&data, &shapes)
}

const PREFIXES: &str = r#"
    @prefix sh: <http://www.w3.org/ns/shacl#> .
    @prefix ex: <http://example.org/> .
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    @prefix owl: <http://www.w3.org/2002/07/owl#> .
"#;

/// A node-shape `sh:sparql` that flags any focus node whose ex:age is negative.
/// $this is pre-bound to the focus node; one solution per violating node.
#[test]
fn sparql_flags_violating_nodes() {
    let shapes = format!(
        r#"{PREFIXES}
        ex:PersonShape a sh:NodeShape ;
          sh:targetClass ex:Person ;
          sh:sparql [
            a sh:SPARQLConstraint ;
            sh:prefixes ex:shapesPrefixes ;
            sh:message "Age must not be negative" ;
            sh:select """
              SELECT $this ?value WHERE {{
                $this <http://example.org/age> ?value .
                FILTER (?value < 0)
              }}
            """ ;
          ] .
        ex:shapesPrefixes sh:declare [ sh:prefix "ex" ; sh:namespace "http://example.org/"^^xsd:anyURI ] .
    "#
    );
    let data = r#"
        @prefix ex: <http://example.org/> .
        ex:alice a ex:Person ; ex:age 30 .
        ex:bob   a ex:Person ; ex:age -1 .
        ex:carol a ex:Person ; ex:age -5 .
    "#;
    let r = run(data, &shapes);
    assert!(!r.conforms, "{}", r.to_text());
    // bob and carol violate; alice does not.
    assert_eq!(r.results.len(), 2, "{}", r.to_text());
    for res in &r.results {
        assert!(
            res.source_component
                .ends_with("SPARQLConstraintComponent"),
            "component: {}",
            res.source_component
        );
        // focusNode is $this (bob or carol), never alice.
        let f = res.focus_node.to_string();
        assert!(
            f.contains("bob") || f.contains("carol"),
            "unexpected focus {f}"
        );
        // sh:value maps from ?value: the negative age.
        let v = res.value.as_ref().expect("value bound").to_string();
        assert!(v.contains("-1") || v.contains("-5"), "value {v}");
        // sh:message from the constraint's sh:message.
        let msg = &res.effective_messages()[0];
        assert!(
            msg.to_string().contains("Age must not be negative"),
            "message {msg}"
        );
    }
}

/// `$this` binding correctness: a SELECT that returns a row for EVERY binding of
/// an unrelated variable must still only fire for the focus node — i.e. $this is
/// genuinely constrained to the focus, not left free to range over the graph.
#[test]
fn this_is_pre_bound_not_free() {
    let shapes = format!(
        r#"{PREFIXES}
        ex:S a sh:NodeShape ;
          sh:targetNode ex:alice ;
          sh:sparql [
            sh:select """
              SELECT $this WHERE {{ $this a <http://example.org/Person> }}
            """ ;
          ] .
    "#
    );
    // Two Persons exist, but only ex:alice is targeted. If $this were free, the
    // query would return BOTH persons (2 solutions); pre-binding pins it to alice
    // → exactly one solution (alice is a Person, so the constraint fires once).
    let data = r#"
        @prefix ex: <http://example.org/> .
        ex:alice a ex:Person .
        ex:bob   a ex:Person .
    "#;
    let r = run(data, &shapes);
    assert_eq!(r.results.len(), 1, "{}", r.to_text());
    assert!(r.results[0].focus_node.to_string().contains("alice"));
}

/// The conforming case: a `sh:sparql` whose SELECT returns no solutions for any
/// focus node yields zero results and `sh:conforms true`.
#[test]
fn sparql_conforming_yields_no_violations() {
    let shapes = format!(
        r#"{PREFIXES}
        ex:PersonShape a sh:NodeShape ;
          sh:targetClass ex:Person ;
          sh:sparql [
            sh:select """
              SELECT $this ?value WHERE {{
                $this <http://example.org/age> ?value .
                FILTER (?value < 0)
              }}
            """ ;
          ] .
    "#
    );
    let data = r#"
        @prefix ex: <http://example.org/> .
        ex:alice a ex:Person ; ex:age 30 .
        ex:bob   a ex:Person ; ex:age 5 .
    "#;
    let r = run(data, &shapes);
    assert!(r.conforms, "{}", r.to_text());
    assert!(r.results.is_empty());
}

/// `sh:prefixes` must supply the query's prefix declarations: the SELECT uses a
/// `ex:` prefix declared only via sh:declare, never inline in the query text.
#[test]
fn sparql_honours_prefixes() {
    let shapes = format!(
        r#"{PREFIXES}
        ex:S a sh:NodeShape ;
          sh:targetNode ex:thing ;
          sh:sparql [
            sh:prefixes ex:p ;
            sh:select "SELECT $this WHERE {{ $this ex:bad ?o }}" ;
          ] .
        ex:p sh:declare [ sh:prefix "ex" ; sh:namespace "http://example.org/"^^xsd:anyURI ] .
    "#
    );
    let data = r#"
        @prefix ex: <http://example.org/> .
        ex:thing ex:bad "x" .
    "#;
    let r = run(data, &shapes);
    assert_eq!(r.results.len(), 1, "{}", r.to_text());
    assert!(r.results[0].focus_node.to_string().contains("thing"));
}

/// A SELECT with an undeclared prefix is ill-formed and is skipped (lenient): no
/// panic, no results — the rest of validation still runs.
#[test]
fn ill_formed_sparql_is_skipped() {
    let shapes = format!(
        r#"{PREFIXES}
        ex:S a sh:NodeShape ;
          sh:targetNode ex:thing ;
          # No sh:prefixes: the `nope:` prefix is undeclared -> parse error -> skip.
          sh:sparql [ sh:select "SELECT $this WHERE {{ $this nope:bad ?o }}" ] ;
          # A core constraint on the same shape still fires.
          sh:property [ sh:path ex:name ; sh:minCount 1 ] .
    "#
    );
    let data = r#"
        @prefix ex: <http://example.org/> .
        ex:thing ex:bad "x" .
    "#;
    let r = run(data, &shapes);
    // Only the minCount violation (the ill-formed sh:sparql contributes nothing).
    assert_eq!(r.results.len(), 1, "{}", r.to_text());
    assert!(r.results[0]
        .source_component
        .ends_with("MinCountConstraintComponent"));
}

/// `{?var}` substitution in the constraint's sh:message template, and the
/// `?value` → sh:value mapping working together.
#[test]
fn sparql_message_template_substitution() {
    let shapes = format!(
        r#"{PREFIXES}
        ex:S a sh:NodeShape ;
          sh:targetNode ex:bob ;
          sh:sparql [
            sh:message "Bad age {{?value}} on {{$this}}" ;
            sh:select """
              SELECT $this ?value WHERE {{
                $this <http://example.org/age> ?value . FILTER(?value < 0)
              }}
            """ ;
          ] .
    "#
    );
    let data = r#"
        @prefix ex: <http://example.org/> .
        ex:bob ex:age -7 .
    "#;
    let r = run(data, &shapes);
    assert_eq!(r.results.len(), 1, "{}", r.to_text());
    let msg = r.results[0].effective_messages()[0].to_string();
    assert!(msg.contains("Bad age -7"), "message: {msg}");
    assert!(msg.contains("bob"), "message: {msg}");
}

/// owl:imports chasing for sh:prefixes: a prefixes resource that imports another
/// which carries the declaration.
#[test]
fn sparql_prefixes_follow_owl_imports() {
    let shapes = format!(
        r#"{PREFIXES}
        ex:S a sh:NodeShape ;
          sh:targetNode ex:thing ;
          sh:sparql [
            sh:prefixes ex:childPrefixes ;
            sh:select "SELECT $this WHERE {{ $this ex:bad ?o }}" ;
          ] .
        ex:childPrefixes owl:imports ex:parentPrefixes .
        ex:parentPrefixes sh:declare [ sh:prefix "ex" ; sh:namespace "http://example.org/"^^xsd:anyURI ] .
    "#
    );
    let data = r#"
        @prefix ex: <http://example.org/> .
        ex:thing ex:bad "x" .
    "#;
    let r = run(data, &shapes);
    assert_eq!(r.results.len(), 1, "{}", r.to_text());
}

/// The report Turtle for a SPARQL-based result must itself be valid Turtle and
/// carry the SPARQLConstraintComponent.
#[test]
fn sparql_report_turtle_parses() {
    let shapes = format!(
        r#"{PREFIXES}
        ex:S a sh:NodeShape ;
          sh:targetNode ex:bob ;
          sh:sparql [
            sh:message "nope" ;
            sh:select "SELECT $this ?value WHERE {{ $this <http://example.org/age> ?value . FILTER(?value < 0) }}" ;
          ] .
    "#
    );
    let data = r#"
        @prefix ex: <http://example.org/> .
        ex:bob ex:age -1 .
    "#;
    let r = run(data, &shapes);
    assert!(!r.conforms);
    let ttl = r.to_turtle();
    let parsed: Result<Vec<_>, _> = oxttl::TurtleParser::new().for_slice(ttl.as_bytes()).collect();
    let triples = parsed.unwrap_or_else(|e| panic!("report Turtle does not parse: {e}\n{ttl}"));
    assert!(triples
        .iter()
        .any(|t| t.object.to_string().contains("SPARQLConstraintComponent")));
}
