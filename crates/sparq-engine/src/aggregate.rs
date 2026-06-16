//! Custom aggregate registry (sq-5qz9) — register a named (IRI) user aggregate
//! and call it from a real SPARQL `GROUP BY`.
//!
//! [OPUS-4.8] This is the aggregate analogue of [`crate::FunctionRegistry`]: an
//! extension surface that lets a caller add a SPARQL aggregate function without
//! forking the engine. Unlike a scalar extension function (one row in, one term
//! out), an aggregate FOLDS the whole multiset of a group's per-member values
//! into a single term.
//!
//! ## Standard-ness
//!
//! Custom aggregate FUNCTION IRIs *are* part of the SPARQL 1.1 grammar's
//! extension point: `Aggregate ::= … | FunctionCall` where a `FunctionCall`
//! resolving to an IRI the engine recognises as an aggregate is evaluated as
//! one (SPARQL 1.1 §11.6 — "an implementation may provide … extension
//! aggregates"). The vendored `spargebra` parser already supports this via
//! [`spargebra::SparqlParser::with_custom_aggregate_function`]; this module
//! supplies the EVALUATION side (an `AggregateFunction::Custom` IRI dispatched
//! to the installed registry) plus the parse glue that declares every
//! registered IRI to the parser so the query parses in the first place.
//!
//! ## Boundary type
//!
//! Like [`crate::ExtFn`], the closure works in `oxrdf::Term`s — the public,
//! dictionary-independent representation — so a registry can be built without
//! touching engine internals. Each group member contributes `Option<Term>`
//! (`None` ≡ the aggregate's argument expression is UNBOUND for that row, e.g.
//! an `OPTIONAL` column); the aggregate returns `Option<Term>` (`None` ≡ an
//! unbound aggregate result, exactly like `SUM` over an all-unbound group), or
//! `Err` for a SPARQL expression error (→ the aggregate is unbound, matching
//! how the builtin `SUM`/`AVG` report a type error).

use oxrdf::{NamedNode, Term};
use sparq_core::Graph;
use spargebra::SparqlParser;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{PreparedQuery, QueryBudget, QueryResult};

/// A user aggregate: folds a group's per-member values (`None` ≡ unbound for
/// that member) into a single result term (`None` ≡ unbound aggregate). `Err`
/// is a SPARQL expression error — the aggregate result is unbound, never a hard
/// query failure (matching the builtin aggregates' error discipline). The
/// message is only for a human debugging the extension.
///
/// `Arc`-shared so a registry clones cheaply and is `Send + Sync` for the
/// engine's parallel group-evaluation path.
pub type AggFn = Arc<dyn Fn(&[Option<Term>]) -> Result<Option<Term>, String> + Send + Sync>;

/// A map from aggregate-function IRIs to [`AggFn`]s, consulted by the evaluator
/// for `AggregateFunction::Custom` IRIs in a `GROUP BY`. Installed per query by
/// [`query_with_aggregates`] / [`with_aggregates`]; the registry-free entry
/// points never consult it, so an undeclared custom aggregate stays a parse
/// error there (the parser does not know the IRI is an aggregate).
///
/// Cloning is cheap (the aggregates are `Arc`-shared), so a long-lived registry
/// can be built once and reused across queries and threads.
#[derive(Clone, Default)]
pub struct CustomAggregateRegistry {
    map: HashMap<String, AggFn>,
}

impl CustomAggregateRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers `f` as the aggregate under the IRI (replacing any previous
    /// registration). The IRI is both what the query writes — `<iri>(?x)` in a
    /// projection over a `GROUP BY` — and what the parser is told to treat as an
    /// aggregate rather than a scalar function.
    pub fn register(
        &mut self,
        iri: impl Into<String>,
        f: impl Fn(&[Option<Term>]) -> Result<Option<Term>, String> + Send + Sync + 'static,
    ) {
        self.map.insert(iri.into(), Arc::new(f));
    }

    /// The aggregate registered under `iri`, if any.
    pub fn get(&self, iri: &str) -> Option<&AggFn> {
        self.map.get(iri)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// The registered IRIs, for declaring them to a [`SparqlParser`].
    pub(crate) fn iris(&self) -> impl Iterator<Item = &str> {
        self.map.keys().map(String::as_str)
    }
}

/// Parses `sparql` with every registry IRI declared to the parser as a custom
/// aggregate, so `<iri>(?x)` in a `GROUP BY` projection parses as an aggregate
/// (not a scalar function call). A malformed registry IRI is reported as a
/// parse-level error rather than panicking.
fn parse_with_aggregates(sparql: &str, reg: &CustomAggregateRegistry) -> Result<PreparedQuery, String> {
    let mut parser = SparqlParser::new();
    for iri in reg.iris() {
        let nn = NamedNode::new(iri).map_err(|e| format!("custom aggregate IRI {iri:?}: {e}"))?;
        parser = parser.with_custom_aggregate_function(nn);
    }
    let query = parser.parse_query(sparql).map_err(|e| e.to_string())?;
    Ok(PreparedQuery::from(query))
}

/// Runs `f` with `reg` installed as the active custom-aggregate registry — the
/// scoped form behind [`query_with_aggregates`], for callers that route a query
/// through one of the OTHER entry points themselves. The registry is visible to
/// every query the closure runs on this thread (installed thread-locally and
/// propagated into the engine's rayon workers) and uninstalled when the closure
/// returns. Composes with [`crate::with_functions`] in either nesting order.
///
/// NOTE: this only INSTALLS the registry for evaluation; to also PARSE a custom
/// aggregate the parser must be told the IRI is an aggregate. Use
/// [`query_with_aggregates`] (which does both), or build the [`PreparedQuery`]
/// via a [`SparqlParser`] with the same IRIs declared.
pub fn with_aggregates<T>(reg: &CustomAggregateRegistry, f: impl FnOnce() -> T) -> T {
    let _guard = crate::exec::aggregates::install(reg);
    f()
}

/// [`crate::query`] with a custom-aggregate registry: a declared aggregate IRI in
/// a `GROUP BY` projection is evaluated against `reg`. The parser is told about
/// every registry IRI so the query parses; evaluation dispatches
/// `AggregateFunction::Custom` to the installed aggregate.
pub fn query_with_aggregates(
    graph: &Graph,
    sparql: &str,
    reg: &CustomAggregateRegistry,
) -> Result<QueryResult, String> {
    query_with_aggregates_and_budget(graph, sparql, reg, &QueryBudget::unlimited())
}

/// [`query_with_aggregates`] under a cooperative [`QueryBudget`].
pub fn query_with_aggregates_and_budget(
    graph: &Graph,
    sparql: &str,
    reg: &CustomAggregateRegistry,
    budget: &QueryBudget,
) -> Result<QueryResult, String> {
    let prepared = parse_with_aggregates(sparql, reg)?;
    with_aggregates(reg, || crate::query_prepared_with_budget(graph, &prepared, budget))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::Literal;
    use sparq_core::Graph;

    const DATA: &str = r#"
        @prefix ex: <http://ex/> .
        ex:a ex:dept "eng"  ; ex:sales 10 .
        ex:b ex:dept "eng"  ; ex:sales 20 .
        ex:c ex:dept "eng"  ; ex:sales 30 .
        ex:d ex:dept "sales"; ex:sales 5 .
        ex:e ex:dept "sales"; ex:sales 7 .
    "#;

    fn g() -> Graph {
        Graph::load_str(DATA, "turtle").unwrap()
    }

    /// A PRODUCT custom aggregate, registered and called through a real SPARQL
    /// `GROUP BY` — the headline deliverable: a user aggregate computing
    /// correctly per group.
    #[test]
    fn custom_product_aggregate_through_group_by() {
        let mut reg = CustomAggregateRegistry::new();
        reg.register("http://ex/agg#product", |members: &[Option<Term>]| {
            let mut acc: i64 = 1;
            for m in members {
                if let Some(Term::Literal(l)) = m {
                    acc *= l.value().parse::<i64>().map_err(|e| e.to_string())?;
                }
            }
            Ok(Some(Term::Literal(Literal::from(acc))))
        });
        let q = "PREFIX ex: <http://ex/> PREFIX agg: <http://ex/agg#> \
                 SELECT ?d (agg:product(?s) AS ?p) WHERE { ?x ex:dept ?d ; ex:sales ?s } \
                 GROUP BY ?d ORDER BY ?d";
        let r = query_with_aggregates(&g(), q, &reg).unwrap();
        // eng: 10*20*30 = 6000; sales: 5*7 = 35.
        let got: Vec<(String, String)> = r
            .rows
            .iter()
            .map(|row| (row[0].as_ref().unwrap().to_string(), row[1].as_ref().unwrap().to_string()))
            .collect();
        assert!(got[0].0.contains("eng") && got[0].1.contains("\"6000\""), "eng row: {got:?}");
        assert!(got[1].0.contains("sales") && got[1].1.contains("\"35\""), "sales row: {got:?}");
    }

    /// A whole-dataset custom aggregate (no GROUP BY): one group over every row.
    /// `<iri>(?x)` counting the members it sees collapses to the row count.
    #[test]
    fn custom_aggregate_whole_dataset() {
        let mut reg = CustomAggregateRegistry::new();
        reg.register("http://ex/agg#cnt", |members: &[Option<Term>]| {
            Ok(Some(Term::Literal(Literal::from(members.len() as i64))))
        });
        let q = "PREFIX ex: <http://ex/> PREFIX agg: <http://ex/agg#> \
                 SELECT (agg:cnt(?s) AS ?c) WHERE { ?x ex:sales ?s }";
        let r = query_with_aggregates(&g(), q, &reg).unwrap();
        // 5 sales triples in DATA.
        assert!(r.rows[0][0].as_ref().unwrap().to_string().contains("\"5\""));
    }

    /// An unregistered custom-looking IRI in aggregate position is a parse error
    /// (the parser was not told it is an aggregate), matching the registry-free
    /// engine's "unsupported function" discipline.
    #[test]
    fn unregistered_aggregate_iri_is_error() {
        let reg = CustomAggregateRegistry::new(); // empty
        let q = "PREFIX ex: <http://ex/> SELECT (<http://ex/agg#nope>(?s) AS ?p) \
                 WHERE { ?x ex:sales ?s } GROUP BY ?x";
        assert!(query_with_aggregates(&g(), q, &reg).is_err());
    }
}
