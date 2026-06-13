//! sparq-shacl: opt-in SHACL Core + SHACL-SPARQL validation over
//! [`sparq_core::Graph`]s.
//!
//! Parse a shapes graph into a shapes model, evaluate every SHACL Core
//! constraint component against a data graph by direct index-backed scans,
//! evaluate `sh:sparql` constraints (SHACL §5.2) and SPARQL-based constraint
//! COMPONENTS (`sh:ConstraintComponent`, SHACL §6) by routing their
//! `sh:select`/`sh:ask` through `sparq-engine` (with `$this`/`$value`/`$PATH`
//! and `$paramName` pre-binding), and produce a [`ValidationReport`] (with
//! Turtle and plain-text renderings of the SHACL report vocabulary).
//!
//! This crate follows the `sparq-reason` isolation pattern: it is NOT a
//! dependency of any other sparq crate — the core engine and the wasm bundle
//! carry zero SHACL code unless a consumer opts in by depending on it.
//! (`sparq-engine`, pulled in for `sh:sparql`, is likewise native-only and does
//! not depend on this crate, so there is no cycle and the wasm bundle is untouched.)
//!
//! ```
//! use sparq_core::Graph;
//!
//! let data = Graph::load_str(r#"
//!     @prefix ex: <http://example.org/> .
//!     ex:alice a ex:Person ; ex:age "thirty" .
//! "#, "turtle").unwrap();
//! let shapes = Graph::load_str(r#"
//!     @prefix sh: <http://www.w3.org/ns/shacl#> .
//!     @prefix ex: <http://example.org/> .
//!     @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
//!     ex:PersonShape a sh:NodeShape ;
//!       sh:targetClass ex:Person ;
//!       sh:property [ sh:path ex:age ; sh:datatype xsd:integer ] .
//! "#, "turtle").unwrap();
//!
//! let report = sparq_shacl::validate(&data, &shapes);
//! assert!(!report.conforms);
//! assert_eq!(report.results.len(), 1);
//! ```

mod eval;
pub mod model;
pub mod path;
mod report;
mod sparql;
pub mod view;

pub use model::{Component, Shape, ShapesModel, Target};
pub use path::Path;
pub use report::{ValidationReport, ValidationResult};

use oxrdf::Triple;
use sparq_core::Graph;

/// Validates `data` against the SHACL shapes in `shapes`, returning the
/// validation report. Constructs the shapes graph never declares (or declares
/// ill-formed — e.g. an unparsable path) are skipped rather than failing.
pub fn validate(data: &Graph, shapes: &Graph) -> ValidationReport {
    let model = ShapesModel::parse(shapes);
    validate_with_model(data, &model)
}

/// [`validate`] against an already-parsed shapes model (amortises shape
/// parsing across many data graphs).
pub fn validate_with_model(data: &Graph, model: &ShapesModel) -> ValidationReport {
    ValidationReport::new(eval::validate_graph(data, model))
}

/// Builds a queryable [`Graph`] from already-parsed triples. The seam for
/// callers that need parser options [`Graph::load_str`] does not expose —
/// e.g. a base IRI for resolving relative IRIs (the W3C test-suite files).
pub fn graph_from_triples<I: IntoIterator<Item = Triple>>(triples: I) -> Graph {
    use oxrdf::{NamedOrBlankNode, Term};
    let mut dict = sparq_core::dict::Dict::new();
    let ids: Vec<[sparq_core::dict::Id; 3]> = triples
        .into_iter()
        .map(|t| {
            let s = match t.subject {
                NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n),
                NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b),
            };
            [
                dict.intern(&s),
                dict.intern(&Term::NamedNode(t.predicate)),
                dict.intern(&t.object),
            ]
        })
        .collect();
    Graph::from_parts(dict, ids)
}

/// Loads a Turtle document into a [`Graph`], resolving relative IRIs against
/// `base`.
pub fn load_turtle_with_base(text: &str, base: &str) -> Result<Graph, String> {
    let parser = oxttl::TurtleParser::new()
        .with_base_iri(base)
        .map_err(|e| e.to_string())?;
    let mut triples = Vec::new();
    for t in parser.for_slice(text.as_bytes()) {
        triples.push(t.map_err(|e| e.to_string())?);
    }
    Ok(graph_from_triples(triples))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHAPES: &str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        ex:PersonShape a sh:NodeShape ;
          sh:targetClass ex:Person ;
          sh:property [
            sh:path ex:age ;
            sh:datatype xsd:integer ;
            sh:maxCount 1 ;
            sh:minInclusive 0 ;
          ] ;
          sh:property [
            sh:path ex:name ;
            sh:minCount 1 ;
            sh:nodeKind sh:Literal ;
          ] .
    "#;

    fn check(data: &str) -> ValidationReport {
        let data = Graph::load_str(data, "turtle").unwrap();
        let shapes = Graph::load_str(SHAPES, "turtle").unwrap();
        validate(&data, &shapes)
    }

    #[test]
    fn conforming_data() {
        let r = check(
            r#"
            @prefix ex: <http://example.org/> .
            ex:alice a ex:Person ; ex:age 30 ; ex:name "Alice" .
        "#,
        );
        assert!(r.conforms, "unexpected results: {}", r.to_text());
        assert!(r.to_turtle().contains("sh:conforms true"));
    }

    #[test]
    fn violations_reported() {
        let r = check(
            r#"
            @prefix ex: <http://example.org/> .
            ex:bob a ex:Person ; ex:age -1 .
        "#,
        );
        assert!(!r.conforms);
        // missing name (minCount) + negative age (minInclusive)
        assert_eq!(r.results.len(), 2, "{}", r.to_text());
        let comps: Vec<&str> = r
            .results
            .iter()
            .map(|x| x.source_component.as_str())
            .collect();
        assert!(comps
            .iter()
            .any(|c| c.ends_with("MinCountConstraintComponent")));
        assert!(comps
            .iter()
            .any(|c| c.ends_with("MinInclusiveConstraintComponent")));
    }

    /// The Turtle rendering must itself be valid Turtle.
    #[test]
    fn report_turtle_parses() {
        let r = check(
            r#"
            @prefix ex: <http://example.org/> .
            ex:bob a ex:Person ; ex:age 200 ; ex:age 300 .
        "#,
        );
        assert!(!r.conforms);
        let ttl = r.to_turtle();
        let parsed: Result<Vec<_>, _> = oxttl::TurtleParser::new()
            .for_slice(ttl.as_bytes())
            .collect();
        let triples = parsed.unwrap_or_else(|e| panic!("report Turtle does not parse: {e}\n{ttl}"));
        assert!(triples
            .iter()
            .any(|t| t.predicate.as_str().ends_with("conforms")));
    }

    /// `sh:conforms` counts every result regardless of severity (what the W3C
    /// suite checks); `conforms_violations_only` is the severity-aware toggle.
    #[test]
    fn severity_aware_conformance_toggle() {
        let data = Graph::load_str(
            r#"
            @prefix ex: <http://example.org/> .
            ex:bob a ex:Person ; ex:age "old" .
        "#,
            "turtle",
        )
        .unwrap();
        let shapes = |severity: &str| {
            Graph::load_str(
                &format!(
                    r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
            ex:S a sh:NodeShape ;
              sh:targetClass ex:Person ;
              sh:property [ sh:path ex:age ; sh:datatype xsd:integer ; sh:severity {severity} ] .
        "#
                ),
                "turtle",
            )
            .unwrap()
        };
        // A warning-severity result: reported, sh:conforms false, but the
        // violations-only toggle passes.
        let r = validate(&data, &shapes("sh:Warning"));
        assert!(!r.conforms);
        assert_eq!(r.results.len(), 1);
        assert!(r.conforms_violations_only());
        assert_eq!(r.results_with_severity("http://www.w3.org/ns/shacl#Warning").count(), 1);
        assert_eq!(r.results_with_severity("http://www.w3.org/ns/shacl#Violation").count(), 0);
        // Default severity is sh:Violation: both notions of conformance fail.
        let r = validate(&data, &shapes("sh:Violation"));
        assert!(!r.conforms);
        assert!(!r.conforms_violations_only());
        // Conforming data conforms under both.
        let ok = Graph::load_str(
            "@prefix ex: <http://example.org/> . ex:a a ex:Person ; ex:age 3 .",
            "turtle",
        )
        .unwrap();
        let r = validate(&ok, &shapes("sh:Warning"));
        assert!(r.conforms && r.conforms_violations_only());
    }

    /// Cyclic sh:node references stay correct under the (focus, shape)
    /// conformance memo: re-entry counts as conforming, and results reached
    /// through a cycle are not wrongly reused.
    #[test]
    fn cyclic_node_shapes_with_memo() {
        let shapes = Graph::load_str(
            r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            ex:PS a sh:NodeShape ;
              sh:targetClass ex:Person ;
              sh:property [ sh:path ex:knows ; sh:node ex:PS ] ;
              sh:property [ sh:path ex:name ; sh:minCount 1 ] .
        "#,
            "turtle",
        )
        .unwrap();
        // A mutual cycle where everyone has a name: conforms (re-entry = true).
        let good = Graph::load_str(
            r#"
            @prefix ex: <http://example.org/> .
            ex:a a ex:Person ; ex:name "A" ; ex:knows ex:b .
            ex:b a ex:Person ; ex:name "B" ; ex:knows ex:a .
        "#,
            "turtle",
        )
        .unwrap();
        let r = validate(&good, &shapes);
        assert!(r.conforms, "{}", r.to_text());
        // The same cycle but b is nameless: a's sh:node value fails, b's own
        // minCount fails, and the SHARED conformance check of the nameless
        // node (reached from both a and c) reports per route.
        let bad = Graph::load_str(
            r#"
            @prefix ex: <http://example.org/> .
            ex:a a ex:Person ; ex:name "A" ; ex:knows ex:b .
            ex:b a ex:Person ; ex:knows ex:a .
            ex:c a ex:Person ; ex:name "C" ; ex:knows ex:b .
        "#,
            "turtle",
        )
        .unwrap();
        let r = validate(&bad, &shapes);
        assert!(!r.conforms);
        // b fails minCount as a target, and the nameless b makes THREE sh:node
        // checks fail: a's value b, c's value b, and b's value a (a does not
        // conform because ITS value b fails — the cycle propagates).
        let node_failures: Vec<_> = r
            .results
            .iter()
            .filter(|x| x.source_component.ends_with("NodeConstraintComponent"))
            .collect();
        assert_eq!(node_failures.len(), 3, "{}", r.to_text());
        assert!(r
            .results
            .iter()
            .any(|x| x.source_component.ends_with("MinCountConstraintComponent")));
    }

    // [OPUS-4.8] Regression for review 1616: a cyclic sh:property reference must NOT overflow the
    // stack. The Property component recurses through validate_shape directly, bypassing the
    // conforms() (focus, shape) guard; without an equivalent guard a self-referential property
    // shape over cyclic data recurses forever. Re-entry counts as conforming (SHACL leaves
    // recursion undefined), so the data conforms.
    #[test]
    fn cyclic_property_shape_does_not_overflow() {
        let shapes = Graph::load_str(
            r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            # A node shape A with a property to B, and B with a property back to A.
            ex:A a sh:NodeShape ;
              sh:targetNode ex:a ;
              sh:property ex:PtoB .
            ex:PtoB a sh:PropertyShape ;
              sh:path ex:next ;
              sh:property ex:PtoA .
            ex:PtoA a sh:PropertyShape ;
              sh:path ex:next ;
              sh:property ex:PtoB .
        "#,
            "turtle",
        )
        .unwrap();
        // Cyclic data: a -> b -> a. Without the guard this recurses unboundedly.
        let data = Graph::load_str(
            r#"
            @prefix ex: <http://example.org/> .
            ex:a ex:next ex:b .
            ex:b ex:next ex:a .
        "#,
            "turtle",
        )
        .unwrap();
        // The point of the test is that this returns at all (no stack overflow). The cycle is
        // treated as conforming, and no constraint forces a violation.
        let r = validate(&data, &shapes);
        assert!(r.conforms, "cyclic sh:property must terminate and conform: {}", r.to_text());
    }

    // [OPUS-4.8] Regression for review 1616 (implicit class shape discovery): a node that is an
    // rdfs:Class with SHACL constraints — but NOT explicitly typed sh:NodeShape and with no
    // sh:target* — is an implicit node shape with an implicit class target. Root discovery
    // previously skipped it, silently ignoring its constraints; it must now be validated.
    #[test]
    fn implicit_class_shape_is_discovered() {
        let shapes = Graph::load_str(
            r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
            @prefix ex: <http://example.org/> .
            # rdfs:Class + SHACL constraint, but NO sh:NodeShape type and NO sh:target*.
            ex:Person a rdfs:Class ;
              sh:property [ sh:path ex:name ; sh:minCount 1 ] .
        "#,
            "turtle",
        )
        .unwrap();
        // An instance missing ex:name must now fail (implicit class target on ex:Person).
        let data = Graph::load_str(
            r#"
            @prefix ex: <http://example.org/> .
            ex:alice a ex:Person .
        "#,
            "turtle",
        )
        .unwrap();
        let r = validate(&data, &shapes);
        assert!(!r.conforms, "implicit class shape must validate its instances: {}", r.to_text());
        assert!(r
            .results
            .iter()
            .any(|x| x.source_component.ends_with("MinCountConstraintComponent")));
    }

    #[test]
    fn logical_and_paths() {
        let data = Graph::load_str(
            r#"
            @prefix ex: <http://example.org/> .
            ex:a ex:knows ex:b . ex:b ex:knows ex:c .
            ex:c a ex:Person .
        "#,
            "turtle",
        )
        .unwrap();
        let shapes = Graph::load_str(
            r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            ex:S a sh:NodeShape ;
              sh:targetNode ex:a ;
              sh:property [
                sh:path ( ex:knows ex:knows ) ;
                sh:class ex:Person ;
                sh:minCount 1 ;
              ] .
        "#,
            "turtle",
        )
        .unwrap();
        let r = validate(&data, &shapes);
        assert!(r.conforms, "{}", r.to_text());
    }
}
