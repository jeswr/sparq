// [OPUS-4.8] sq-jxl0: single-source the crate overview from README.md so crates.io
// (package.readme) and the docs.rs front page render identical content. The README's
// `## Usage` rust fences are compiled as doctests (hidden `#`-scaffolding inside them).
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

mod eval;
pub mod model;
pub mod path;
mod report;
// [OPUS-4.8] (sq-d1dw) SHACL-AF rules (`sh:rule`) — OPT-IN behind the `shacl-af`
// cargo feature so the base validation path carries zero rule code when off.
#[cfg(feature = "shacl-af")]
pub mod rules;
// [OPUS-4.8] (sq-v0b8, #796) SHACL Compact Syntax (SCS) PARSER (text → shapes
// triples) — OPT-IN behind the `scs` cargo feature so the base validation path
// (and the default + wasm bundles) carry zero parser code when off.
#[cfg(feature = "scs")]
pub mod scs;
mod sparql;
pub mod view;

pub use model::{Component, Shape, ShapesModel, Target};
pub use path::Path;
// [OPUS-4.8] (sq-lz99x) `ShapeDiagnostic` surfaces a constraint the validator
// SKIPPED because it could not be evaluated (e.g. an uncompilable `sh:pattern`).
// [OPUS-4.8] (sq-sx15d) `DEFAULT_CONFORMANCE_DISALLOWS` is the SHACL 1.2 default
// disallowed-severity set used to compute `ValidationReport::conforms`.
pub use report::{
    ShapeDiagnostic, ValidationReport, ValidationResult, DEFAULT_CONFORMANCE_DISALLOWS,
};

// [OPUS-4.8] (sq-v0b8) SHACL Compact Syntax parser public surface (feature `scs`).
#[cfg(feature = "scs")]
pub use scs::{parse as parse_scs, parse_to_graph as parse_scs_to_graph, ScsError, DEFAULT_BASE};

// [OPUS-4.8] SHACL-AF rules + node-expression public surface (feature `shacl-af`).
#[cfg(feature = "shacl-af")]
pub use rules::{
    apply_rules, apply_rules_with_model, conforms, eval_node_expression, expand, ConformanceCheck,
    Inference,
};

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
    let (results, diagnostics) = eval::validate_graph(data, model);
    ValidationReport::with_diagnostics(results, diagnostics)
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

/// [OPUS-4.8] (sq-7iai) The number of DISTINCT focus nodes the model's targeted
/// shapes select over `data` — i.e. the size of the union of every targeted
/// shape's focus-node set, deduplicated exactly as the validator's own
/// `focus_nodes` enumeration does. This is a deterministic *target-selection*
/// statistic (independent of how many constraints those nodes violate), used by
/// the SHACL benchmark suite as a target-selection regression detector alongside
/// the violation count.
///
/// It delegates to the validator's own focus-node enumeration (`eval`), so it
/// stays in lock-step with validation — including the SHACL-1.2 data-dependent
/// targets (`sh:targetWhere`, the SPARQL-valued `sh:targetNode`, and the
/// `sh:shape` data-graph link) that a pure shapes-graph scan cannot express.
pub fn count_focus_nodes(data: &Graph, model: &ShapesModel) -> usize {
    eval::count_focus_nodes(data, model)
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

    /// [OPUS-4.8] (sq-7iai) `count_focus_nodes` counts the DISTINCT focus nodes the
    /// targeted shapes select (independent of how many constraints they violate) —
    /// the SHACL benchmark suite's target-selection gate.
    #[test]
    fn focus_node_count_is_distinct_target_union() {
        let data = Graph::load_str(
            r#"
            @prefix ex: <http://example.org/> .
            ex:alice a ex:Person ; ex:age 30 ; ex:name "Alice" .
            ex:bob   a ex:Person ; ex:age 40 ; ex:name "Bob" .
            ex:carol a ex:Person .
            ex:notaperson a ex:Robot .
        "#,
            "turtle",
        )
        .unwrap();
        let shapes = Graph::load_str(SHAPES, "turtle").unwrap();
        let model = ShapesModel::parse(&shapes);
        // Three ex:Person instances are the focus nodes; ex:Robot is not targeted.
        assert_eq!(count_focus_nodes(&data, &model), 3);
        // A graph with no instances of the targeted class selects zero focus nodes.
        let empty = Graph::load_str("@prefix ex: <http://example.org/> . ex:x a ex:Other .", "turtle").unwrap();
        assert_eq!(count_focus_nodes(&empty, &model), 0);
    }

    /// [OPUS-4.8] (sq-bif) `count_focus_nodes` unions EVERY target kind — node,
    /// subjects-of, objects-of — deduplicated. A node selected by two distinct
    /// targets is counted once.
    #[test]
    fn focus_node_count_unions_all_target_kinds() {
        let data = Graph::load_str(
            r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:knows ex:bob .
            ex:carol ex:knows ex:alice .
        "#,
            "turtle",
        )
        .unwrap();
        // sh:targetNode ex:alice, sh:targetSubjectsOf ex:knows (= {alice, carol}),
        // sh:targetObjectsOf ex:knows (= {bob, alice}). The deduplicated union is
        // {alice, bob, carol} = 3 (alice appears via all three but counts once).
        let shapes = Graph::load_str(
            r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            ex:S a sh:NodeShape ;
              sh:targetNode ex:alice ;
              sh:targetSubjectsOf ex:knows ;
              sh:targetObjectsOf ex:knows ;
              sh:property [ sh:path ex:name ; sh:minCount 1 ] .
        "#,
            "turtle",
        )
        .unwrap();
        let model = ShapesModel::parse(&shapes);
        assert_eq!(count_focus_nodes(&data, &model), 3);
    }

    // --- [OPUS-4.8] (sq-rnkdh) SHACL 1.2 targets & SPARQL node expressions ------

    /// `sh:targetWhere [ <inline shape> ]`: the focus nodes are the data-graph
    /// nodes that CONFORM to the inline shape. Here only `ex:alice` (a Person with
    /// age ≥ 18) is in scope; `ex:bob` (age 17) is not, so its violation of the
    /// outer constraint is NOT reported. `count_focus_nodes` agrees.
    #[test]
    fn target_where_selects_conforming_nodes() {
        const TW_SHAPES: &str = r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            ex:AdultShape a sh:NodeShape ;
              sh:targetWhere [
                sh:class ex:Person ;
                sh:property [ sh:path ex:age ; sh:minInclusive 18 ] ;
              ] ;
              sh:property [ sh:path ex:votedFor ; sh:minCount 1 ] .
        "#;
        let shapes = Graph::load_str(TW_SHAPES, "turtle").unwrap();
        let data = Graph::load_str(
            r#"
            @prefix ex: <http://example.org/> .
            ex:alice a ex:Person ; ex:age 30 .
            ex:bob   a ex:Person ; ex:age 17 .
        "#,
            "turtle",
        )
        .unwrap();
        let model = ShapesModel::parse(&shapes);
        // alice is the only in-target node (≥18); bob (17) is out of target.
        assert_eq!(count_focus_nodes(&data, &model), 1);
        let r = validate(&data, &shapes);
        assert!(!r.conforms, "{}", r.to_text());
        assert_eq!(r.results.len(), 1, "{}", r.to_text());
        assert_eq!(
            r.results[0].focus_node,
            oxrdf::Term::NamedNode(oxrdf::NamedNode::new("http://example.org/alice").unwrap())
        );
    }

    /// `sh:shape` data-graph link: a triple `?n sh:shape ?S` in the DATA graph
    /// makes `?n` a focus node of shape `?S`.
    #[test]
    fn data_graph_sh_shape_target() {
        let shapes = Graph::load_str(
            r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
            @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
            ex:TestShape a sh:NodeShape ;
              sh:property [ sh:path rdfs:label ; sh:datatype xsd:string ; sh:maxCount 0 ] .
        "#,
            "turtle",
        )
        .unwrap();
        let data = Graph::load_str(
            r#"
            @prefix ex: <http://example.org/> .
            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
            ex:Invalid rdfs:label "x" ; <http://www.w3.org/ns/shacl#shape> ex:TestShape .
            ex:Valid <http://www.w3.org/ns/shacl#shape> ex:TestShape .
        "#,
            "turtle",
        )
        .unwrap();
        let r = validate(&data, &shapes);
        assert!(!r.conforms, "{}", r.to_text());
        // Only ex:Invalid has a label (maxCount 0 violated); ex:Valid conforms.
        assert_eq!(r.results.len(), 1, "{}", r.to_text());
        assert_eq!(
            r.results[0].focus_node,
            oxrdf::Term::NamedNode(oxrdf::NamedNode::new("http://example.org/Invalid").unwrap())
        );
    }

    /// `sh:ShapeClass` is a class that is ALSO a node shape (SHACL 1.2): instances
    /// (via the subclass closure) are implicit-class-targeted by it.
    #[test]
    fn shape_class_is_implicit_class_target() {
        let g = Graph::load_str(
            r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
            ex:Super a sh:ShapeClass ; sh:in ( ex:Good ) .
            ex:Sub rdfs:subClassOf ex:Super .
            ex:Good a ex:Sub .
            ex:Bad  a ex:Sub .
        "#,
            "turtle",
        )
        .unwrap();
        let r = validate(&g, &g);
        assert!(!r.conforms, "{}", r.to_text());
        // ex:Bad is not in the sh:in list -> one InConstraintComponent violation.
        assert_eq!(r.results.len(), 1, "{}", r.to_text());
        assert!(r.results[0]
            .source_component
            .ends_with("InConstraintComponent"));
    }

    /// SPARQL-valued `sh:targetNode [ sh:select … ]` computes target nodes, and
    /// `sh:values [ sh:sparqlExpr … ]` computes value nodes; a constraint-level
    /// `sh:severity` overrides the shape default.
    #[test]
    fn sparql_valued_target_and_values_and_severity() {
        let shapes = Graph::load_str(
            r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
            ex:S a sh:NodeShape ;
              sh:targetNode [ sh:select "SELECT ?p WHERE { ?p <http://example.org/flag> true }" ] ;
              sh:sparql [
                sh:select "SELECT $this WHERE { $this <http://example.org/bad> true }" ;
                sh:severity sh:Warning ;
              ] .
        "#,
            "turtle",
        )
        .unwrap();
        let data = Graph::load_str(
            r#"
            @prefix ex: <http://example.org/> .
            ex:x ex:flag true ; ex:bad true .
            ex:y ex:bad true .
        "#,
            "turtle",
        )
        .unwrap();
        let model = ShapesModel::parse(&shapes);
        // Only ex:x is selected by the SPARQL target (ex:y has no ex:flag).
        assert_eq!(count_focus_nodes(&data, &model), 1);
        let r = validate(&data, &shapes);
        assert!(!r.conforms, "{}", r.to_text());
        assert_eq!(r.results.len(), 1, "{}", r.to_text());
        // The constraint-level sh:severity overrode the shape's default Violation.
        assert!(r.results[0].severity.ends_with("Warning"), "{}", r.to_text());
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

    /// [OPUS-4.8] (sq-sx15d) `sh:conforms` fails when ANY result is in the SHACL-1.2
    /// default disallowed set {Violation, Warning, Info}; `conforms_violations_only`
    /// is the strictly-weaker CI toggle that fails only on `sh:Violation`. A
    /// Warning result fails `conforms` (Warning is disallowed by default) but
    /// passes the violations-only toggle.
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
        // A sh:Debug result is below the default threshold: conforms is true even
        // though a result is reported (the violations-only toggle agrees).
        let r = validate(&data, &shapes("sh:Debug"));
        assert!(r.conforms, "Debug result must not break default conformance");
        assert_eq!(r.results.len(), 1);
        assert!(r.conforms_violations_only());
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

    // [OPUS-4.8] (sq-bif) `load_turtle_with_base` resolves relative IRIs against
    // the supplied base (the seam `Graph::load_str` does not expose, used by the
    // W3C manifest loaders) and surfaces a parse error as an `Err(String)`.
    #[test]
    fn load_turtle_with_base_resolves_relatives_and_reports_errors() {
        // `<rel>` and `<#frag>` resolve against the base.
        let g = load_turtle_with_base(
            "<rel> <http://example.org/p> <#frag> .",
            "http://base.example/dir/",
        )
        .unwrap();
        let v = view::GraphView::new(&g);
        // The relative subject resolved to base + "rel".
        assert!(v.contains(
            &oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(
                "http://base.example/dir/rel"
            )),
            "http://example.org/p",
            &oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(
                "http://base.example/dir/#frag"
            )),
        ));
        // A malformed Turtle document is an Err, not a panic (Graph has no Debug,
        // so map to the error string before asserting).
        let err = load_turtle_with_base("this is not turtle @@@", "http://base.example/").err();
        assert!(err.is_some(), "malformed Turtle must Err");
        // An invalid base IRI is also a (different) Err.
        assert!(load_turtle_with_base("", "not a valid base").is_err());
    }

    // [OPUS-4.8] (sq-bif) `graph_from_triples` interns oxrdf triples — including
    // a BLANK-NODE subject (the subject match arm `Graph::load_str` round-trips
    // but the helper handles explicitly) — into a queryable Graph.
    #[test]
    fn graph_from_triples_interns_iri_and_blank_subjects() {
        use oxrdf::{BlankNode, NamedNode, NamedOrBlankNode, Term, Triple};
        let p = NamedNode::new_unchecked("http://example.org/p");
        let bnode = BlankNode::default();
        let triples = vec![
            // IRI subject.
            Triple::new(
                NamedOrBlankNode::NamedNode(NamedNode::new_unchecked("http://example.org/s")),
                p.clone(),
                Term::NamedNode(NamedNode::new_unchecked("http://example.org/o")),
            ),
            // Blank-node subject (the BlankNode arm of the subject match).
            Triple::new(
                NamedOrBlankNode::BlankNode(bnode.clone()),
                p.clone(),
                Term::Literal(oxrdf::Literal::new_simple_literal("x")),
            ),
        ];
        let g = graph_from_triples(triples);
        let v = view::GraphView::new(&g);
        // Both triples are present and queryable.
        assert!(v.contains(
            &Term::NamedNode(NamedNode::new_unchecked("http://example.org/s")),
            "http://example.org/p",
            &Term::NamedNode(NamedNode::new_unchecked("http://example.org/o")),
        ));
        assert!(v.contains(
            &Term::BlankNode(bnode),
            "http://example.org/p",
            &Term::Literal(oxrdf::Literal::new_simple_literal("x")),
        ));
        // An empty iterator yields an empty graph.
        let empty = graph_from_triples(std::iter::empty());
        assert!(view::GraphView::new(&empty)
            .triples(None, Some("http://example.org/p"), None)
            .is_empty());
    }

    /// [OPUS-4.8] (sq-mue75) A `sh:sparql` result carries `sh:sourceConstraint`
    /// pointing at the `sh:SPARQLConstraint` node — distinct from `sh:sourceShape`
    /// (the shape) — and the Turtle report emits it. A Core (non-sparql) result
    /// carries no `sh:sourceConstraint`.
    #[test]
    fn sparql_result_carries_source_constraint() {
        let shapes = Graph::load_str(
            r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            ex:TestShape a sh:NodeShape ;
              sh:targetNode ex:bad ;
              sh:sparql ex:TestShape-sparql .
            ex:TestShape-sparql a sh:SPARQLConstraint ;
              sh:select "SELECT $this WHERE { $this <http://example.org/flag> true }" .
        "#,
            "turtle",
        )
        .unwrap();
        let data = Graph::load_str(
            "@prefix ex: <http://example.org/> . ex:bad ex:flag true .",
            "turtle",
        )
        .unwrap();
        let r = validate(&data, &shapes);
        assert!(!r.conforms, "{}", r.to_text());
        assert_eq!(r.results.len(), 1, "{}", r.to_text());
        let res = &r.results[0];
        // sh:sourceConstraint = the sh:SPARQLConstraint node, NOT the shape.
        assert_eq!(
            res.source_constraint,
            Some(oxrdf::Term::NamedNode(
                oxrdf::NamedNode::new("http://example.org/TestShape-sparql").unwrap()
            )),
            "expected sh:sourceConstraint = the constraint node"
        );
        assert_eq!(
            res.source_shape,
            oxrdf::Term::NamedNode(
                oxrdf::NamedNode::new("http://example.org/TestShape").unwrap()
            ),
            "sh:sourceShape must remain the shape node"
        );
        assert_ne!(
            res.source_constraint.as_ref(),
            Some(&res.source_shape),
            "sourceConstraint and sourceShape must be distinct here"
        );
        // The Turtle report emits sh:sourceConstraint.
        let ttl = r.to_turtle();
        assert!(
            ttl.contains("sh:sourceConstraint"),
            "report Turtle must emit sh:sourceConstraint:\n{ttl}"
        );
    }

    /// [OPUS-4.8] (sq-mue75) A Core (non-`sh:sparql`) constraint result carries no
    /// `sh:sourceConstraint` (the spec stamps it only on SPARQL-based results).
    #[test]
    fn core_result_has_no_source_constraint() {
        let r = check(
            r#"
            @prefix ex: <http://example.org/> .
            ex:bob a ex:Person ; ex:age -1 .
        "#,
        );
        assert!(!r.conforms);
        assert!(
            r.results.iter().all(|res| res.source_constraint.is_none()),
            "Core results must not carry sh:sourceConstraint"
        );
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

// [OPUS-4.8] (sq-mk9n / sq-3w6n, `shacl-af`) The two SHACL-AF node-expression
// constraints: `sh:expression` (`sh:ExpressionConstraintComponent`) — a value
// node violates when the node expression does not evaluate to `{ true }` — and
// `sh:nodeByExpression` (`sh:NodeByExpressionConstraintComponent`) — a value node
// violates when it does not conform to a node shape the expression computes.
#[cfg(feature = "shacl-af")]
#[cfg(test)]
mod expression_tests {
    use super::*;

    fn g(ttl: &str) -> Graph {
        let prelude = "@prefix sh: <http://www.w3.org/ns/shacl#> .\n\
            @prefix shnex: <http://www.w3.org/ns/shacl-node-expr#> .\n\
            @prefix ex: <http://example.org/> .\n\
            @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
            @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n";
        Graph::load_str(&format!("{prelude}{ttl}"), "turtle").unwrap()
    }

    #[test]
    fn expression_constant_false_violates() {
        // The W3C `expression-001` shape: `sh:expression false` at a node shape ⇒
        // every targeted node is a violation (false != true).
        let data = g("ex:i a ex:C .");
        let shapes = g("ex:C a rdfs:Class, sh:NodeShape ; sh:expression false .");
        let r = validate(&data, &shapes);
        assert!(!r.conforms, "{}", r.to_text());
        assert!(r.results.iter().any(|res| res
            .source_component
            .ends_with("ExpressionConstraintComponent")
            && res.value.as_ref().map(|v| v.to_string()).as_deref()
                == Some("<http://example.org/i>")));
    }

    #[test]
    fn expression_true_conforms() {
        // A node expression that DOES evaluate to { true } conforms — here a
        // `shnex:exists` over an existing path value.
        let data = g("ex:i a ex:C ; ex:p ex:v .");
        let shapes = g(
            "ex:C a rdfs:Class, sh:NodeShape ; \
             sh:expression [ shnex:exists [ sh:path ex:p ] ] .",
        );
        let r = validate(&data, &shapes);
        assert!(r.conforms, "exists ex:p => true => conforms: {}", r.to_text());
    }

    #[test]
    fn expression_on_property_shape_per_value() {
        // `sh:expression` on a property shape applies per path value node; here the
        // value (5) satisfies a matchAll-over-minInclusive-3 ⇒ exists ⇒ true.
        let data = g("ex:i a ex:C ; ex:age 5 .");
        let shapes = g(
            "ex:C a rdfs:Class, sh:NodeShape ; \
             sh:property [ sh:path ex:age ; \
               sh:expression [ shnex:exists [ shnex:nodes [ shnex:var \"focusNode\" ] ; \
                 shnex:matchAll [ sh:minInclusive 3 ] ] ] ] .",
        );
        let r = validate(&data, &shapes);
        assert!(
            r.conforms,
            "age 5 >= 3 => matchAll true => exists true: {}",
            r.to_text()
        );
    }

    // ---- [OPUS-4.8] (sq-3w6n) sh:nodeByExpression ----

    #[test]
    fn node_by_expression_constant_iri_violates_nonconforming_value() {
        // The W3C `nodeByExpression-001` shape: a property shape whose value nodes
        // must conform to the node shape an expression computes. Here the expression
        // is the constant IRI `ex:AssignedToShape` (per SHACL-AF, an IRI expression
        // evaluating to { ex:AssignedToShape } — the `sh:node` special case). The
        // assignee lacks the required `ex:email`, so it does NOT conform => one
        // violation with the value node and component IRI.
        let data = g("ex:issue a ex:Issue ; ex:assignedTo ex:anon . ex:anon a ex:Person .");
        let shapes = g(
            "ex:AssignedToShape a sh:NodeShape ; \
               sh:property [ sh:path ex:email ; sh:minCount 1 ] . \
             ex:Issue a rdfs:Class, sh:NodeShape ; \
               sh:property [ sh:path ex:assignedTo ; sh:nodeByExpression ex:AssignedToShape ] .",
        );
        let r = validate(&data, &shapes);
        assert!(!r.conforms, "{}", r.to_text());
        assert!(
            r.results.iter().any(|res| res
                .source_component
                .ends_with("NodeByExpressionConstraintComponent")
                && res.value.as_ref().map(|v| v.to_string()).as_deref()
                    == Some("<http://example.org/anon>")),
            "expected a NodeByExpression violation for ex:anon: {}",
            r.to_text()
        );
    }

    #[test]
    fn node_by_expression_conforming_value_passes() {
        // The same constraint, but the assignee carries the required email, so it
        // conforms to the computed node shape and there is no violation.
        let data = g(
            "ex:issue a ex:Issue ; ex:assignedTo ex:jane . \
             ex:jane a ex:Person ; ex:email \"jane@ex.org\" .",
        );
        let shapes = g(
            "ex:AssignedToShape a sh:NodeShape ; \
               sh:property [ sh:path ex:email ; sh:minCount 1 ] . \
             ex:Issue a rdfs:Class, sh:NodeShape ; \
               sh:property [ sh:path ex:assignedTo ; sh:nodeByExpression ex:AssignedToShape ] .",
        );
        let r = validate(&data, &shapes);
        assert!(
            r.conforms,
            "ex:jane has an email => conforms: {}",
            r.to_text()
        );
    }

    #[test]
    fn node_by_expression_path_computed_shape() {
        // A dynamically-computed shape: the node expression is a path-values
        // expression that locates the shape at `ex:hasShape` from the value node.
        // ex:obs/ex:hasShape => ex:RangeShape (minInclusive 0); ex:obs's measure is
        // -1, so it must NOT conform => one violation on the value node.
        let data = g(
            "ex:obs a ex:Observation ; ex:value -1 ; ex:hasShape ex:RangeShape . \
             ex:s a ex:DataShape ; ex:item ex:obs .",
        );
        let shapes = g(
            "ex:RangeShape a sh:NodeShape ; \
               sh:property [ sh:path ex:value ; sh:minInclusive 0 ] . \
             ex:DataShape a rdfs:Class, sh:NodeShape ; \
               sh:property [ sh:path ex:item ; \
                 sh:nodeByExpression [ sh:path ex:hasShape ] ] .",
        );
        let r = validate(&data, &shapes);
        assert!(!r.conforms, "value -1 < 0 must violate: {}", r.to_text());
        assert!(
            r.results.iter().any(|res| res
                .source_component
                .ends_with("NodeByExpressionConstraintComponent")
                && res.value.as_ref().map(|v| v.to_string()).as_deref()
                    == Some("<http://example.org/obs>")),
            "expected a NodeByExpression violation for ex:obs: {}",
            r.to_text()
        );
    }
}
