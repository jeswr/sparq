//! [SONNET-4.6] (sq-8cp8t) Differential coverage for constant-region spatial
//! FILTER orientations whose bbox pushdown is a superset only.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use oxrdf::{Literal, NamedNode, Term};
use sparq_core::Graph;
use sparq_engine::{
    query, with_functions, with_spatial_index, FunctionRegistry, QueryResult,
    SpatialProvider, SpatialQuery,
};
#[cfg(feature = "spatial-exact-pushdown")]
use sparq_engine::SpatialExactQuery;

const WKT: &str = "http://www.opengis.net/ont/geosparql#wktLiteral";
const SF_INTERSECTS: &str = "http://www.opengis.net/def/function/geosparql/sfIntersects";
const SF_WITHIN: &str = "http://www.opengis.net/def/function/geosparql/sfWithin";
const SF_CONTAINS: &str = "http://www.opengis.net/def/function/geosparql/sfContains";
const REGION: &str = "REGION";

fn wkt(value: &str) -> Term {
    Literal::new_typed_literal(value, NamedNode::new_unchecked(WKT)).into()
}

fn graph() -> Graph {
    Graph::load_str(
        r#"@prefix ex: <http://ex/> .
           @prefix geo: <http://www.opengis.net/ont/geosparql#> .
           ex:indexed_true ex:geom "TRUE_INDEXED"^^geo:wktLiteral .
           ex:indexed_false ex:geom "FALSE_INDEXED"^^geo:wktLiteral .
           ex:indexed_outside ex:geom "FALSE_INDEXED_OUT"^^geo:wktLiteral .
           ex:unknown_true ex:geom "TRUE_UNKNOWN"^^geo:wktLiteral .
           ex:unknown_false ex:geom "FALSE_UNKNOWN"^^geo:wktLiteral ."#,
        "turtle",
    )
    .unwrap()
}

struct SupersetProvider {
    calls: Arc<AtomicUsize>,
    #[cfg(feature = "spatial-exact-pushdown")]
    exact_calls: Arc<AtomicUsize>,
}

impl SpatialProvider for SupersetProvider {
    fn candidates(&self, query: &SpatialQuery) -> Option<Vec<Term>> {
        let SpatialQuery::BboxIntersects { arg_wkt } = query else {
            return None;
        };
        assert_eq!(*arg_wkt, REGION);
        self.calls.fetch_add(1, Ordering::Relaxed);
        Some(vec![wkt("TRUE_INDEXED"), wkt("FALSE_INDEXED")])
    }

    fn is_indexed(&self, term: &Term) -> bool {
        term == &wkt("TRUE_INDEXED")
            || term == &wkt("FALSE_INDEXED")
            || term == &wkt("FALSE_INDEXED_OUT")
    }

    #[cfg(feature = "spatial-exact-pushdown")]
    fn candidates_exact(&self, _query: &SpatialExactQuery) -> Option<Vec<Term>> {
        self.exact_calls.fetch_add(1, Ordering::Relaxed);
        Some(Vec::new())
    }
}

fn registry(checks: Arc<AtomicUsize>) -> FunctionRegistry {
    fn judge(term: &Term) -> Result<Term, String> {
        let Term::Literal(literal) = term else {
            return Err("geometry must be a literal".into());
        };
        Ok(Literal::from(literal.value().starts_with("TRUE")).into())
    }

    let mut registry = FunctionRegistry::new();
    for (iri, geometry_arg) in [(SF_INTERSECTS, 1), (SF_WITHIN, 1), (SF_CONTAINS, 0)] {
        let checks = Arc::clone(&checks);
        registry.register(iri, move |args: &[Term]| {
            checks.fetch_add(1, Ordering::Relaxed);
            judge(&args[geometry_arg])
        });
    }
    registry
}

fn query_for(predicate: &str, variable_first: bool) -> String {
    let relation = if variable_first {
        format!("geof:{}(?g, \"{}\"^^geo:wktLiteral)", predicate, REGION)
    } else {
        format!("geof:{}(\"{}\"^^geo:wktLiteral, ?g)", predicate, REGION)
    };
    format!(
        "PREFIX geof: <http://www.opengis.net/def/function/geosparql/> \
         PREFIX geo: <http://www.opengis.net/ont/geosparql#> \
         SELECT ?s WHERE {{ ?s <http://ex/geom> ?g . FILTER({}) }}",
        relation
    )
}

fn rows(result: &QueryResult) -> Vec<String> {
    let mut rows: Vec<_> = result
        .rows
        .iter()
        .map(|row| row[0].as_ref().unwrap().to_string())
        .collect();
    rows.sort();
    rows
}

#[test]
fn remaining_constant_region_orientations_are_sound_superset_pushdowns() {
    let graph = graph();
    for (predicate, variable_first) in [
        ("sfIntersects", false),
        ("sfWithin", false),
        ("sfContains", true),
    ] {
        let sparql = query_for(predicate, variable_first);
        let checks = Arc::new(AtomicUsize::new(0));
        let functions = registry(Arc::clone(&checks));

        let expected = with_functions(&functions, || query(&graph, &sparql)).unwrap();
        let posthoc_checks = checks.swap(0, Ordering::Relaxed);

        let calls = Arc::new(AtomicUsize::new(0));
        #[cfg(feature = "spatial-exact-pushdown")]
        let exact_calls = Arc::new(AtomicUsize::new(0));
        let provider = Arc::new(SupersetProvider {
            calls: Arc::clone(&calls),
            #[cfg(feature = "spatial-exact-pushdown")]
            exact_calls: Arc::clone(&exact_calls),
        });
        let actual = with_spatial_index(provider, || {
            with_functions(&functions, || query(&graph, &sparql))
        })
        .unwrap();
        let pushed_checks = checks.load(Ordering::Relaxed);

        assert_eq!(
            rows(&actual),
            rows(&expected),
            "{} changed the result",
            predicate
        );
        assert_eq!(
            calls.load(Ordering::Relaxed),
            1,
            "{} was not pushed",
            predicate
        );
        #[cfg(feature = "spatial-exact-pushdown")]
        assert_eq!(
            exact_calls.load(Ordering::Relaxed),
            0,
            "{} must not request an exact within-region certificate",
            predicate
        );
        assert!(
            pushed_checks > 0 && pushed_checks < posthoc_checks,
            "{} must retain the residual FILTER over the smaller superset",
            predicate
        );
    }
}

#[cfg(not(feature = "spatial-exact-pushdown"))]
#[test]
fn constant_first_contains_is_a_default_superset_pushdown() {
    let graph = graph();
    let sparql = query_for("sfContains", false);
    let checks = Arc::new(AtomicUsize::new(0));
    let mut functions = FunctionRegistry::new();
    let residual_checks = Arc::clone(&checks);
    functions.register(SF_CONTAINS, move |args: &[Term]| {
        residual_checks.fetch_add(1, Ordering::Relaxed);
        let Term::Literal(literal) = &args[1] else {
            return Err("geometry must be a literal".into());
        };
        Ok(Literal::from(literal.value().starts_with("TRUE")).into())
    });

    let expected = with_functions(&functions, || query(&graph, &sparql)).unwrap();
    let posthoc_checks = checks.swap(0, Ordering::Relaxed);
    let calls = Arc::new(AtomicUsize::new(0));
    let provider = Arc::new(SupersetProvider {
        calls: Arc::clone(&calls),
    });
    let actual = with_spatial_index(provider, || {
        with_functions(&functions, || query(&graph, &sparql))
    })
    .unwrap();
    let pushed_checks = checks.load(Ordering::Relaxed);

    assert_eq!(rows(&actual), rows(&expected));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(pushed_checks > 0 && pushed_checks < posthoc_checks);
}
