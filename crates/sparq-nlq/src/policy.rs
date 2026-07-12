//! Policy classification for generated SPARQL algebra.
//!
//! Classify only after parsing. A [`spargebra::Query`] is always read-only, while every
//! [`spargebra::Update`] is mutating, including update requests containing multiple
//! operations. This algebra-level split avoids keyword scanning and therefore
//! cannot be confused by comments, string literals, or prefixes.

// [GPT-5.6] sq-9zs3g: policy is decided from typed parser output, not source text.

use spargebra::{Query, Update};

/// The graph-access policy required by parsed SPARQL algebra.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryPolicy {
    /// The algebra can inspect a dataset but cannot change it.
    ReadOnly,
    /// The algebra can change a dataset.
    Mutating,
}

/// Classifies parsed query algebra.
///
/// All SPARQL query result forms (`SELECT`, `ASK`, `CONSTRUCT`, and
/// `DESCRIBE`) are read-only. The exhaustive match intentionally makes a new
/// upstream query variant a compile-time decision instead of silently granting
/// it read-only access.
#[must_use]
pub fn classify_query(query: &Query) -> QueryPolicy {
    match query {
        Query::Select { .. }
        | Query::Ask { .. }
        | Query::Construct { .. }
        | Query::Describe { .. } => QueryPolicy::ReadOnly,
    }
}

/// Classifies a parsed SPARQL Update request.
///
/// An update request is mutating regardless of which operation it contains or
/// how many operations are combined in the request. Callers should reject it
/// whenever only read access was authorized.
#[must_use]
pub const fn classify_update(_update: &Update) -> QueryPolicy {
    QueryPolicy::Mutating
}

#[cfg(test)]
mod tests {
    use super::{classify_query, classify_update, QueryPolicy};
    use spargebra::SparqlParser;

    #[test]
    fn read_only_algebra_classified() {
        for sparql in [
            "SELECT * WHERE { ?s ?p ?o }",
            "ASK WHERE { ?s ?p ?o }",
            "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }",
            "DESCRIBE ?s WHERE { ?s ?p ?o }",
        ] {
            let query = SparqlParser::new()
                .parse_query(sparql)
                .expect("valid query fixture");
            assert_eq!(classify_query(&query), QueryPolicy::ReadOnly, "{sparql}");
        }
    }

    #[test]
    fn mutating_algebra_classified() {
        for sparql in [
            "INSERT DATA { <http://example/s> <http://example/p> <http://example/o> }",
            "DELETE DATA { <http://example/s> <http://example/p> <http://example/o> }",
            "LOAD <http://example/data>",
            "CLEAR DEFAULT",
            "DROP DEFAULT",
            "CREATE GRAPH <http://example/g>",
            "ADD DEFAULT TO GRAPH <http://example/g>",
            "MOVE DEFAULT TO GRAPH <http://example/g>",
            "COPY DEFAULT TO GRAPH <http://example/g>",
        ] {
            let update = SparqlParser::new()
                .parse_update(sparql)
                .expect("valid update fixture");
            assert_eq!(classify_update(&update), QueryPolicy::Mutating, "{sparql}");
        }
    }

    #[test]
    fn mixed_update_is_mutating() {
        let update = SparqlParser::new()
            .parse_update(
                "DELETE { ?s <http://example/old> ?o } \
                 INSERT { ?s <http://example/new> ?o } \
                 WHERE { ?s <http://example/old> ?o }; \
                 CLEAR GRAPH <http://example/staging>",
            )
            .expect("valid mixed update fixture");

        assert!(
            update.operations.len() > 1,
            "fixture must contain mixed operations"
        );
        assert_eq!(classify_update(&update), QueryPolicy::Mutating);
    }
}
