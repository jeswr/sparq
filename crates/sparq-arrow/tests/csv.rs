//! Integration tests for the flattened CSV serialization of the RDF-term projection.
//! [FABLE-5]
#![cfg(feature = "csv")]

use oxrdf::{BaseDirection, BlankNode, Literal, NamedNode, Term, Triple, Variable};
use sparq_arrow::{csv_variables_from_bytes, from_csv_bytes, to_csv_bytes};
use sparq_engine::QueryResult;

fn assert_csv_identity(result: &QueryResult) {
    let bytes = to_csv_bytes(result).unwrap();
    let restored = from_csv_bytes(&bytes).unwrap();
    assert_eq!(restored.vars, result.vars);
    assert_eq!(restored.rows, result.rows);
}

/// Mutation witness: bypassing either CSV writer/reader, dropping any term-kind arm,
/// treating unbound as a value, or collapsing the empty literal changes an exact row.
#[test]
fn csv_roundtrip_every_term_kind_empty_literal_and_unbound_row_for_row() {
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

    assert_csv_identity(&result);
}

/// CSV-hostile lexical content — delimiters, quotes, newlines, carriage returns,
/// leading/trailing whitespace, unicode — must survive the container's quoting.
#[test]
fn csv_roundtrip_preserves_delimiters_quotes_newlines_and_whitespace() {
    let literal = |value: &str| Some(Term::Literal(Literal::new_simple_literal(value)));
    let result = QueryResult {
        vars: vec![Variable::new("hostile").unwrap()],
        rows: vec![
            vec![literal("comma, separated, content")],
            vec![literal("he said \"hi\" — \"quoted\"")],
            vec![literal("line one\nline two")],
            vec![literal("carriage\r\nreturn")],
            vec![literal("  padded with spaces  ")],
            vec![literal("مرحبا, \"عالم\"")],
            vec![Some(Term::Triple(Box::new(Triple::new(
                NamedNode::new("https://example.test/s").unwrap(),
                NamedNode::new("https://example.test/p").unwrap(),
                Literal::new_simple_literal("object, with \"csv\" hazards\n"),
            ))))],
        ],
    };

    assert_csv_identity(&result);
}

/// The load-bearing distinctness invariant: an unbound cell and a bound empty-string
/// literal are different CSV rows and decode back to different results.
#[test]
fn csv_roundtrip_keeps_unbound_distinct_from_empty_string_literal() {
    let result = QueryResult {
        vars: vec![Variable::new("x").unwrap()],
        rows: vec![
            vec![None],
            vec![Some(Term::Literal(Literal::new_simple_literal("")))],
        ],
    };

    let bytes = to_csv_bytes(&result).unwrap();
    let restored = from_csv_bytes(&bytes).unwrap();
    assert_eq!(restored.rows[0][0], None);
    assert_eq!(
        restored.rows[1][0],
        Some(Term::Literal(Literal::new_simple_literal("")))
    );

    // Pin the raw encoding: the unbound row is five empty fields; the bound empty
    // literal keeps its kind and explicit xsd:string datatype.
    let text = String::from_utf8(bytes).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "x.kind,x.value,x.datatype,x.language,x.direction");
    assert_eq!(lines[1], ",,,,");
    assert_eq!(
        lines[2],
        "literal,,http://www.w3.org/2001/XMLSchema#string,,"
    );
}

/// [FABLE-5] Value-pinned mutation witness: changing either expected name fails while
/// the encoded rows remain untouched.
#[test]
fn csv_schema_only_reader_recovers_nonempty_result_variables() {
    let result = QueryResult {
        vars: vec![Variable::new("s").unwrap(), Variable::new("label").unwrap()],
        rows: vec![vec![
            Some(Term::NamedNode(
                NamedNode::new("https://example.test/s").unwrap(),
            )),
            Some(Term::Literal(Literal::new_simple_literal("label"))),
        ]],
    };
    let bytes = to_csv_bytes(&result).unwrap();

    let variables = csv_variables_from_bytes(&bytes).unwrap();
    assert_eq!(variables, result.vars);
    assert_eq!(
        variables.iter().map(Variable::as_str).collect::<Vec<_>>(),
        ["s", "label"]
    );
}

#[test]
fn csv_roundtrip_empty_result_preserves_two_variable_header_and_zero_rows() {
    let result = QueryResult {
        vars: vec![
            Variable::new("subject").unwrap(),
            Variable::new("object").unwrap(),
        ],
        rows: Vec::new(),
    };

    let bytes = to_csv_bytes(&result).unwrap();
    assert_eq!(csv_variables_from_bytes(&bytes).unwrap(), result.vars);
    assert_csv_identity(&result);
}

/// More rows than the CSV reader's default batch size, so from_csv_bytes must
/// concatenate multiple decoded batches in row order.
#[test]
fn csv_roundtrip_concatenates_multiple_reader_batches_in_row_order() {
    let rows: Vec<_> = (0..2500)
        .map(|n| {
            vec![Some(Term::NamedNode(
                NamedNode::new(format!("https://example.test/resource/{n}")).unwrap(),
            ))]
        })
        .collect();
    let result = QueryResult {
        vars: vec![Variable::new("s").unwrap()],
        rows,
    };

    assert_csv_identity(&result);
}

/// CSV cannot represent zero-column rows, so a zero-variable result is rejected
/// honestly instead of writing an unreadable file (IPC/Parquet keep supporting it).
#[test]
fn csv_zero_variable_result_is_rejected() {
    let result = QueryResult {
        vars: Vec::new(),
        rows: vec![Vec::new(), Vec::new()],
    };
    let error = to_csv_bytes(&result).unwrap_err();
    assert!(
        error.to_string().contains("zero-variable"),
        "unexpected error: {error}"
    );
}

#[test]
fn malformed_csv_bytes_are_rejected() {
    let error = from_csv_bytes(b"not arrow").unwrap_err();
    assert!(
        error.to_string().contains("flattened RDF-term projection"),
        "unexpected error: {error}"
    );
    assert!(from_csv_bytes(b"").is_err());
    // A five-column header whose names do not follow the var.field contract.
    let error = from_csv_bytes(b"a,b,c,d,e\n").unwrap_err();
    assert!(
        error.to_string().contains("does not name a variable's"),
        "unexpected error: {error}"
    );
}

/// A data row that carries a value while its kind field is empty is a malformed
/// unbound cell and must fail closed, not silently decode as unbound or as a term.
#[test]
fn csv_row_with_value_but_empty_kind_fails_closed() {
    let bytes = b"term.kind,term.value,term.datatype,term.language,term.direction\n\
                  ,https://example.test/orphan,,,\n";
    let error = from_csv_bytes(bytes).unwrap_err();
    assert!(
        error.to_string().contains("unbound struct slot"),
        "unexpected error: {error}"
    );
}

#[test]
fn csv_unknown_term_kind_fails_closed() {
    let bytes = b"term.kind,term.value,term.datatype,term.language,term.direction\n\
                  iri,https://example.test/a,,,\n";
    let error = from_csv_bytes(bytes).unwrap_err();
    assert!(
        error.to_string().contains("unknown RDF term kind"),
        "unexpected error: {error}"
    );
}
