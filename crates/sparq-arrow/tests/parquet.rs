//! Integration tests for Parquet serialization of the RDF-term Arrow projection.
#![cfg(feature = "parquet")]

use std::sync::Arc;

use arrow_array::{ArrayRef, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use oxrdf::{BaseDirection, BlankNode, Literal, NamedNode, Term, Triple, Variable};
use parquet::arrow::ArrowWriter;
use sparq_arrow::{from_parquet_bytes, to_parquet_bytes};
use sparq_engine::QueryResult;

fn assert_identity(result: &QueryResult) {
    let bytes = to_parquet_bytes(result).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();
    assert_eq!(restored.vars, result.vars);
    assert_eq!(restored.rows, result.rows);
}

/// [GPT-5.6] Mutation witness: removing any exporter/importer term-kind arm, treating
/// unbound as a value, or collapsing the empty literal changes at least one exact row.
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

    assert_identity(&result);
}

#[test]
fn empty_result_preserves_variable_schema_and_zero_rows() {
    let result = QueryResult {
        vars: vec![
            Variable::new("subject").unwrap(),
            Variable::new("object").unwrap(),
        ],
        rows: Vec::new(),
    };

    assert_identity(&result);
}

#[test]
fn parquet_with_deviating_arrow_schema_is_rejected() {
    let schema = Arc::new(Schema::new(vec![Field::new("term", DataType::Utf8, true)]));
    let column = Arc::new(StringArray::from(vec![Some(
        "https://example.test/not-a-term-struct",
    )])) as ArrayRef;
    let batch = RecordBatch::try_new(Arc::clone(&schema), vec![column]).unwrap();
    let mut writer = ArrowWriter::try_new(Vec::new(), schema, None).unwrap();
    writer.write(&batch).unwrap();
    let bytes = writer.into_inner().unwrap();

    let error = from_parquet_bytes(&bytes).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("does not use the nullable RDF-term struct schema"),
        "{error}"
    );
}

#[test]
fn malformed_parquet_is_rejected() {
    let error = from_parquet_bytes(b"not parquet").unwrap_err();
    assert!(
        error.to_string().contains("Parquet import failed"),
        "{error}"
    );
}
