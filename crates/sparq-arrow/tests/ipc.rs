//! Integration tests for Arrow IPC stream serialization of the RDF-term projection.
#![cfg(feature = "ipc")]

use oxrdf::{BaseDirection, BlankNode, Literal, NamedNode, Term, Triple, Variable};
use sparq_arrow::{from_ipc_bytes, to_ipc_bytes};
use sparq_engine::QueryResult;

fn assert_ipc_identity(result: &QueryResult) {
    let bytes = to_ipc_bytes(result).unwrap();
    let restored = from_ipc_bytes(&bytes).unwrap();
    assert_eq!(restored.vars, result.vars);
    assert_eq!(restored.rows, result.rows);
}

/// Mutation witness: bypassing either IPC writer/reader, dropping any term-kind arm,
/// treating unbound as a value, or collapsing the empty literal changes an exact row.
#[test]
fn every_term_kind_empty_literal_and_unbound_round_trip_row_for_row() {
    let result = QueryResult {
        vars: vec![Variable::new("term").unwrap()],
        rows: vec![
            vec![Some(Term::NamedNode(
                NamedNode::new("https://example.test/resource").unwrap(),
            ))],
            vec![Some(Term::BlankNode(BlankNode::new("blank-1").unwrap()))],
            vec![Some(Term::Literal(Literal::new_simple_literal("plain")))],
            vec![Some(Term::Literal(Literal::new_simple_literal("")))],
            vec![Some(Term::Literal(Literal::new_typed_literal(
                "42",
                NamedNode::new("http://www.w3.org/2001/XMLSchema#integer").unwrap(),
            )))],
            vec![Some(Term::Literal(
                Literal::new_language_tagged_literal("hello", "en").unwrap(),
            ))],
            vec![Some(Term::Literal(
                Literal::new_directional_language_tagged_literal("مرحبا", "ar", BaseDirection::Rtl)
                    .unwrap(),
            ))],
            vec![Some(Term::Triple(Box::new(Triple::new(
                NamedNode::new("https://example.test/s").unwrap(),
                NamedNode::new("https://example.test/p").unwrap(),
                Literal::new_simple_literal("object"),
            ))))],
            vec![None],
        ],
    };

    assert_ipc_identity(&result);
}

#[test]
fn empty_result_preserves_two_variable_schema_and_zero_rows() {
    let result = QueryResult {
        vars: vec![
            Variable::new("subject").unwrap(),
            Variable::new("object").unwrap(),
        ],
        rows: Vec::new(),
    };

    assert_ipc_identity(&result);
}

#[test]
fn malformed_ipc_stream_is_rejected() {
    let error = from_ipc_bytes(b"not arrow").unwrap_err();
    assert!(
        error.to_string().contains("Arrow IPC import failed"),
        "{error}"
    );
}
