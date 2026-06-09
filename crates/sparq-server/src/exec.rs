//! Query classification + dispatch onto the engine's public API.
//!
//! The engine's `sparq_engine::{query, query_json, count}` only accept SELECT (they
//! re-parse internally and reject every other form). To implement the *protocol*
//! correctly we must first know the query FORM — SELECT / ASK / CONSTRUCT / DESCRIBE —
//! because each maps to a different result media type and HTTP shape. We therefore parse
//! the query once here with `spargebra` (the same parser the engine uses) to classify it,
//! then:
//!
//! * **SELECT** — run on the engine, serialise via the requested results format.
//! * **ASK** — rewrite to a `SELECT *` over the same pattern, run it, boolean = any row
//!   (the engine has no native ASK; this is exact and reuses the engine).
//! * **CONSTRUCT / DESCRIBE** — not yet runnable: the engine exposes no RDF-graph result
//!   API. Reported as a 501 by the HTTP layer. See `README` "Follow-ups".

use spargebra::{Query, SparqlParser};

/// The classified form of a parsed SPARQL query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryForm {
    Select,
    Ask,
    Construct,
    Describe,
}

/// Result of preparing a query for execution.
pub struct Prepared {
    pub form: QueryForm,
    /// A SELECT query string runnable by the engine. For a SELECT this is the original
    /// query; for an ASK it is a rewritten `SELECT *` over the same WHERE pattern.
    /// `None` for CONSTRUCT/DESCRIBE (no engine path yet).
    pub runnable_select: Option<String>,
}

/// A failure classified for the HTTP layer.
#[derive(Debug)]
pub enum PrepareError {
    /// The query string did not parse — HTTP 400.
    Malformed(String),
}

/// Parses + classifies the query, producing the engine-runnable SELECT (for SELECT/ASK).
///
/// Note on datasets / named graphs: the engine has no named-graph support (a `GRAPH`
/// pattern makes the engine error at execution, and `FROM`/`FROM NAMED` are ignored — all
/// data lives in the single default graph). We deliberately do NOT inspect the protocol's
/// `default-graph-uri` / `named-graph-uri` here; the HTTP layer accepts and records them
/// but, with a single in-memory default graph, they have no effect. See README.
pub fn prepare(sparql: &str) -> Result<Prepared, PrepareError> {
    let parsed = SparqlParser::new()
        .parse_query(sparql)
        .map_err(|e| PrepareError::Malformed(e.to_string()))?;
    match parsed {
        Query::Select { .. } => Ok(Prepared {
            form: QueryForm::Select,
            runnable_select: Some(sparql.to_string()),
        }),
        Query::Ask { pattern, dataset, base_iri } => {
            // Rebuild as `SELECT * WHERE { <pattern> }` and render back to SPARQL via
            // spargebra's Display (which emits valid SPARQL). The engine then runs it; a
            // single solution row proves satisfiability.
            let select = Query::Select { dataset, pattern, base_iri };
            Ok(Prepared {
                form: QueryForm::Ask,
                runnable_select: Some(select.to_string()),
            })
        }
        Query::Construct { .. } => Ok(Prepared { form: QueryForm::Construct, runnable_select: None }),
        Query::Describe { .. } => Ok(Prepared { form: QueryForm::Describe, runnable_select: None }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sparq_core::Graph;
    use sparq_engine::query;

    const DATA: &str = "@prefix ex: <http://ex/> . ex:a ex:p ex:b . ex:c ex:p ex:d .";

    fn g() -> Graph {
        Graph::load_str(DATA, "turtle").unwrap()
    }

    #[test]
    fn classifies_forms() {
        assert_eq!(prepare("SELECT * WHERE { ?s ?p ?o }").unwrap().form, QueryForm::Select);
        assert_eq!(prepare("ASK { ?s ?p ?o }").unwrap().form, QueryForm::Ask);
        assert_eq!(
            prepare("CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }").unwrap().form,
            QueryForm::Construct
        );
        assert_eq!(prepare("DESCRIBE <http://ex/a>").unwrap().form, QueryForm::Describe);
    }

    #[test]
    fn malformed_is_error() {
        assert!(matches!(prepare("SELECT WHERE {"), Err(PrepareError::Malformed(_))));
    }

    #[test]
    fn ask_rewrite_runs_on_engine_true_and_false() {
        // ASK with a matching pattern => satisfiable => at least one row.
        let p = prepare("PREFIX ex: <http://ex/> ASK { ?s ex:p ?o }").unwrap();
        assert_eq!(p.form, QueryForm::Ask);
        let sel = p.runnable_select.unwrap();
        assert!(!query(&g(), &sel).unwrap().is_empty());

        // ASK with no match => unsatisfiable => zero rows.
        let p = prepare("PREFIX ex: <http://ex/> ASK { ?s ex:nope ?o }").unwrap();
        let sel = p.runnable_select.unwrap();
        assert_eq!(query(&g(), &sel).unwrap().len(), 0);
    }
}
