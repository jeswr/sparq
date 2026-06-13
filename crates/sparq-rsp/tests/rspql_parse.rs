//! [OPUS-4.8] RSP-QL surface-syntax parser (sq-9u1, part 1): the textual
//! streaming-SPARQL extension (`REGISTER`, `FROM NAMED WINDOW … ON … RANGE/STEP`,
//! `WINDOW <w> { … }`) into the crate's window/stream representation.

use sparq_rsp::{R2S, RspqlQuery, WindowSpec};

use oxrdf::NamedNode;

fn nn(s: &str) -> NamedNode {
    NamedNode::new_unchecked(format!("http://ex/{s}"))
}

/// A representative RSP-QL query parses into the expected windows / streams /
/// specs / rewritten SPARQL.
#[test]
fn parses_representative_rspql_query() {
    let q = "\
REGISTER ISTREAM <http://ex/out> AS
SELECT ?room ?v WHERE {
  WINDOW <http://ex/w1> { ?s <http://ex/value> ?v }
  WINDOW <http://ex/w2> { ?s <http://ex/in> ?room }
}
FROM NAMED WINDOW <http://ex/w1> ON <http://ex/temp> [RANGE PT10S STEP PT5S]
FROM NAMED WINDOW <http://ex/w2> ON <http://ex/meta> RANGE 30";

    let parsed = RspqlQuery::parse(q).unwrap();

    // REGISTER header.
    assert_eq!(parsed.output_stream.as_ref().unwrap().as_str(), "http://ex/out");
    assert_eq!(parsed.r2s, R2S::IStream);

    // Two window declarations, in source order.
    assert_eq!(parsed.windows.len(), 2);
    assert_eq!(parsed.windows[0].window, nn("w1"));
    assert_eq!(parsed.windows[0].stream, nn("temp"));
    // PT10S → 10s, PT5S → 5s: a sliding time window.
    assert_eq!(parsed.windows[0].spec, WindowSpec::time(10, 5));
    assert_eq!(parsed.windows[1].window, nn("w2"));
    assert_eq!(parsed.windows[1].stream, nn("meta"));
    // `RANGE 30` (bare integer, no STEP) tumbles: step == range.
    assert_eq!(parsed.windows[1].spec, WindowSpec::time(30, 30));

    // The rewritten SPARQL: WINDOW → GRAPH, FROM NAMED WINDOW clauses stripped,
    // and it parses + binds the join (the windows share ?s).
    assert!(parsed.sparql.contains("GRAPH"), "WINDOW rewritten to GRAPH");
    assert!(!parsed.sparql.contains("WINDOW"), "no WINDOW keyword survives");
    assert!(!parsed.sparql.to_uppercase().contains("FROM NAMED WINDOW"));
    assert!(!parsed.sparql.contains("REGISTER"));
    // Parseable as a SELECT (a ContinuousMultiQuery would accept it).
    sparq_engine::PreparedQuery::parse(&parsed.sparql).expect("rewritten SPARQL parses");
}

/// ISO-8601 durations of several shapes and bare integers all resolve.
#[test]
fn parses_iso8601_and_integer_durations() {
    let cases = [
        ("RANGE PT10S STEP PT5S", WindowSpec::time(10, 5)),
        ("RANGE PT1M", WindowSpec::time(60, 60)),
        ("RANGE PT1M30S STEP PT30S", WindowSpec::time(90, 30)),
        ("RANGE PT2H STEP PT1H", WindowSpec::time(7200, 3600)),
        ("RANGE P1D", WindowSpec::time(86400, 86400)),
        ("RANGE 100 STEP 25", WindowSpec::time(100, 25)),
        ("[RANGE 50 STEP 10]", WindowSpec::time(50, 10)),
    ];
    for (op, expect) in cases {
        let q = format!(
            "SELECT * WHERE {{ WINDOW <http://ex/w1> {{ ?s ?p ?o }} WINDOW <http://ex/w2> {{ ?s ?q ?r }} }}\n\
             FROM NAMED WINDOW <http://ex/w1> ON <http://ex/s1> {op}\n\
             FROM NAMED WINDOW <http://ex/w2> ON <http://ex/s2> RANGE 5"
        );
        let parsed = RspqlQuery::parse(&q).unwrap();
        assert_eq!(parsed.windows[0].spec, expect, "operator `{op}`");
    }
}

/// Window VARIABLES are scoped out — rejected with a clear error, both in the
/// declaration and in the WHERE.
#[test]
fn rejects_window_variables() {
    let decl_var = "SELECT * WHERE { WINDOW <http://ex/w> { ?s ?p ?o } }\n\
                    FROM NAMED WINDOW ?w ON <http://ex/s> RANGE 10";
    assert!(RspqlQuery::parse(decl_var).is_err());

    let where_var = "SELECT * WHERE { WINDOW ?w { ?s ?p ?o } }\n\
                     FROM NAMED WINDOW <http://ex/w> ON <http://ex/s> RANGE 10";
    let e = RspqlQuery::parse(where_var).unwrap_err();
    assert!(e.contains("WINDOW variables"), "got: {e}");
}

/// Malformed headers and missing pieces error rather than silently mis-parse.
#[test]
fn rejects_malformed_headers() {
    // REGISTER with no output stream.
    assert!(RspqlQuery::parse("REGISTER AS SELECT * WHERE {}").is_err());
    // No window declarations at all.
    assert!(RspqlQuery::parse("SELECT * WHERE { ?s ?p ?o }").is_err());
    // Duplicate window IRI.
    let dup = "SELECT * WHERE { WINDOW <http://ex/w> { ?s ?p ?o } }\n\
               FROM NAMED WINDOW <http://ex/w> ON <http://ex/a> RANGE 10\n\
               FROM NAMED WINDOW <http://ex/w> ON <http://ex/b> RANGE 10";
    assert!(RspqlQuery::parse(dup).is_err());
    // Years/months have no fixed second count.
    let pm = "SELECT * WHERE { WINDOW <http://ex/w1> { ?s ?p ?o } WINDOW <http://ex/w2> { ?s ?q ?r } }\n\
              FROM NAMED WINDOW <http://ex/w1> ON <http://ex/s1> RANGE P1Y\n\
              FROM NAMED WINDOW <http://ex/w2> ON <http://ex/s2> RANGE 5";
    assert!(RspqlQuery::parse(pm).is_err());
}

/// A `<…>` IRI that happens to contain the substring "WINDOW" or "FROM" is data,
/// not a keyword — the scanner skips inside angle brackets.
#[test]
fn keywords_inside_iris_are_not_rewritten() {
    let q = "SELECT * WHERE {\n\
               WINDOW <http://ex/w1> { ?s <http://ex/hasWINDOW> ?o }\n\
               WINDOW <http://ex/w2> { ?s <http://ex/q> ?r }\n\
             }\n\
             FROM NAMED WINDOW <http://ex/w1> ON <http://ex/s1> RANGE 10\n\
             FROM NAMED WINDOW <http://ex/w2> ON <http://ex/s2> RANGE 10";
    let parsed = RspqlQuery::parse(q).unwrap();
    assert!(parsed.sparql.contains("http://ex/hasWINDOW"), "predicate IRI untouched");
    // Exactly the two leading WINDOW keywords became GRAPH.
    assert_eq!(parsed.sparql.matches("GRAPH").count(), 2);
}

/// Prefixed names (`ex:w1`, `:w1`) in the RSP-QL clauses resolve against the
/// body's PREFIX declarations — the compact W3C-example form.
#[test]
fn parses_prefixed_window_and_stream_names() {
    let q = "REGISTER STREAM ex:out AS\n\
             PREFIX ex: <http://ex/>\n\
             PREFIX : <http://ex/default/>\n\
             SELECT * WHERE {\n\
               WINDOW ex:w1 { ?s ex:value ?v }\n\
               WINDOW :w2 { ?s ex:in ?room }\n\
             }\n\
             FROM NAMED WINDOW ex:w1 ON ex:temp RANGE 10\n\
             FROM NAMED WINDOW :w2 ON :meta [RANGE PT10S STEP PT5S]";
    let parsed = RspqlQuery::parse(q).unwrap();
    assert_eq!(parsed.output_stream.as_ref().unwrap().as_str(), "http://ex/out");
    assert_eq!(parsed.windows[0].window.as_str(), "http://ex/w1");
    assert_eq!(parsed.windows[0].stream.as_str(), "http://ex/temp");
    assert_eq!(parsed.windows[1].window.as_str(), "http://ex/default/w2");
    assert_eq!(parsed.windows[1].stream.as_str(), "http://ex/default/meta");
    assert_eq!(parsed.windows[1].spec, WindowSpec::time(10, 5));
    // The rewrite resolves through spargebra (which reads the same PREFIXes).
    sparq_engine::PreparedQuery::parse(&parsed.sparql).expect("rewritten SPARQL parses");
}

/// Unicode in the body (outside IRIs/strings) must not corrupt the byte-level
/// scan/rewrite — the scanner advances by whole UTF-8 chars.
#[test]
fn unicode_in_body_survives_rewrite() {
    let q = "PREFIX ex: <http://ex/>\n\
             SELECT * WHERE {\n\
               WINDOW ex:w1 { ?café ex:p \"naïve café ☕\" }\n\
               WINDOW ex:w2 { ?café ex:q ?r }\n\
             }\n\
             FROM NAMED WINDOW ex:w1 ON ex:s1 RANGE 10\n\
             FROM NAMED WINDOW ex:w2 ON ex:s2 RANGE 10";
    let parsed = RspqlQuery::parse(q).unwrap(); // must not panic on multi-byte chars
    assert!(parsed.sparql.contains("naïve café ☕"), "string literal preserved");
    assert_eq!(parsed.sparql.matches("GRAPH").count(), 2);
}
