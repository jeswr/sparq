//! Query classification + dispatch onto the engine's public API.
//!
//! To implement the *protocol* correctly we must first know the query FORM —
//! SELECT / ASK / CONSTRUCT / DESCRIBE — because each maps to a different result media
//! type and HTTP shape. We therefore parse the query once here with `spargebra` (the
//! same parser the engine uses) to classify it, then:
//!
//! * **SELECT** — run on the engine, serialise via the requested results format.
//! * **ASK** — run on the engine's native `sparq_engine::ask` boolean path.
//! * **CONSTRUCT / DESCRIBE** — run on the engine's RDF-graph result path (T16,
//!   `sparq_engine::construct` / `sparq_engine::describe`), serialised as
//!   N-Triples / Turtle per content negotiation.

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
    /// The query string the engine runs (the engine re-parses internally and has a
    /// native entry point per form: `query*` / `ask*` / `construct*` / `describe*`).
    pub runnable: String,
}

/// A failure classified for the HTTP layer.
#[derive(Debug)]
pub enum PrepareError {
    /// The query string did not parse — HTTP 400.
    Malformed(String),
}

/// Parses + classifies the query.
///
/// Note on datasets / named graphs: the engine DOES support a full RDF dataset — a default
/// graph plus named graphs — so a `GRAPH <iri>` / `GRAPH ?g` pattern, a cross-graph join,
/// and an in-query `FROM` / `FROM NAMED` dataset clause all execute correctly (conformance
/// rounds 3–4; covered end-to-end over HTTP by `tests/named_graphs.rs`, sq-fh4z). What is
/// NOT yet wired is the *protocol-level* dataset OVERRIDE: the SPARQL-Protocol
/// `default-graph-uri` / `named-graph-uri` request parameters, which would let a client
/// re-scope the active dataset for a query that has no in-query `FROM`. The HTTP layer
/// accepts and records those parameters but does not currently rewrite the query's dataset
/// around them, so they have no effect — an in-query `FROM` / `GRAPH` is the supported way
/// to address named graphs today (deferred: bead sq-z33x). See README.
pub fn prepare(sparql: &str) -> Result<Prepared, PrepareError> {
    let parsed = SparqlParser::new()
        .parse_query(sparql)
        .map_err(|e| PrepareError::Malformed(e.to_string()))?;
    let form = match parsed {
        Query::Select { .. } => QueryForm::Select,
        Query::Ask { .. } => QueryForm::Ask,
        Query::Construct { .. } => QueryForm::Construct,
        Query::Describe { .. } => QueryForm::Describe,
    };
    Ok(Prepared { form, runnable: sparql.to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sparq_core::Graph;

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
    fn ask_runs_natively_on_engine_true_and_false() {
        // ASK with a matching pattern => true.
        let p = prepare("PREFIX ex: <http://ex/> ASK { ?s ex:p ?o }").unwrap();
        assert_eq!(p.form, QueryForm::Ask);
        assert!(sparq_engine::ask(&g(), &p.runnable).unwrap());

        // ASK with no match => false.
        let p = prepare("PREFIX ex: <http://ex/> ASK { ?s ex:nope ?o }").unwrap();
        assert!(!sparq_engine::ask(&g(), &p.runnable).unwrap());
    }

    #[test]
    fn construct_runs_natively_on_engine() {
        let p = prepare("PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:q ?o } WHERE { ?s ex:p ?o }").unwrap();
        assert_eq!(p.form, QueryForm::Construct);
        assert_eq!(sparq_engine::construct(&g(), &p.runnable).unwrap().len(), 2);

        let p = prepare("DESCRIBE <http://ex/a>").unwrap();
        assert_eq!(p.form, QueryForm::Describe);
        assert_eq!(sparq_engine::describe(&g(), &p.runnable).unwrap().len(), 1);
    }
}
