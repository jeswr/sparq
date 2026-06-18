// [OPUS-4.8] sq-8r9a — native behavioural tests of the REAL exported `#[wasm_bindgen]`
// `Store` API (and the `SolutionCursor` / `QueryChunks` / `QuadChunks` cursor types).
//
// WHY THIS FILE EXISTS (coverage floor-raise)
// -------------------------------------------
// The crate's `#[cfg(test)] mod tests` in src/lib.rs deliberately exercises the engine
// functions the wasm wrappers *delegate to* (`sparq_engine::query` / `ask` / `explain` /
// …) rather than the exported `Store::*` methods, because the wrappers map errors with
// `JsError::new`, a wasm-bindgen import that PANICS on a non-wasm target. That left every
// exported `Store::*` method's `Ok` (success) arm at 0% native line coverage — the JS
// boundary surface was only ever exercised by `tests/web.rs`, which is `wasm32`-only and
// so contributes nothing to the native `cargo llvm-cov` measurement the coverage ratchet
// gates on.
//
// The key fact this file leans on: only the **`Err` arm** of each wrapper touches
// `JsError::new`. The **success arm never does** — so calling `Store::load(good).unwrap()`,
// `store.query(valid_select)`, `cursor.next()`, … runs natively, returns the real value
// crossing-the-boundary would carry, and IS counted by llvm-cov. These are therefore REAL
// behaviour-asserting tests (each checks the actual SPARQL-JSON / N-Triples / boolean /
// count result content), not vacuous line-touchers: they assert the exported method
// produces the same observable result as the engine path the existing tests pin, through
// the actual published JS binding. The negative (`Err`) arms stay covered by the wasm32
// `tests/web.rs` (where `JsError::new` is real).

use sparq_wasm::Store;

const DATA: &str = r#"@prefix ex: <http://ex/> .
    ex:alice ex:name "Alice" ; ex:age 30 ; ex:knows ex:bob .
    ex:bob ex:name "Bob"@en ; ex:age 25 ."#;

// ---- load / size / heap_bytes ----------------------------------------------

/// The exported `Store::load` parses Turtle and the getters report the deduplicated
/// triple count + a non-zero footprint estimate — the same two getters JS reads.
#[test]
fn exported_load_size_heap() {
    let store = Store::load(DATA, "turtle").unwrap();
    // 2 names + 2 ages + 1 knows = 5 triples.
    assert_eq!(store.size(), 5);
    assert!(store.heap_bytes() > 0, "heap estimate must be non-zero");
}

/// N-Triples loads through the same exported entry point.
#[test]
fn exported_load_ntriples() {
    let nt = "<http://ex/a> <http://ex/p> <http://ex/b> .\n\
              <http://ex/a> <http://ex/p> <http://ex/c> .\n";
    let store = Store::load(nt, "ntriples").unwrap();
    assert_eq!(store.size(), 2);
}

/// `Store::loadDataset` preserves a NAMED graph (N-Quads), so a `GRAPH ?g` query over
/// the exported store returns the graph IRI — the dataset-preserving load path.
#[test]
fn exported_load_dataset_named_graph() {
    let nq = "<http://ex/s> <http://ex/p> <http://ex/o> <http://ex/g1> .\n";
    let store = Store::load_dataset(nq, "nquads").unwrap();
    let json = store
        .query("SELECT ?g WHERE { GRAPH ?g { ?s ?p ?o } }")
        .unwrap();
    assert!(
        json.contains("\"value\":\"http://ex/g1\""),
        "named graph must be queryable through the exported loadDataset: {json}"
    );
}

/// `Store::loadCompressed` returns byte-identical SELECT JSON to the raw exported store
/// and reports a footprint no larger than the raw store.
#[test]
fn exported_load_compressed_matches_raw() {
    let raw = Store::load(DATA, "turtle").unwrap();
    let cmp = Store::load_compressed(DATA, "turtle").unwrap();
    assert_eq!(raw.size(), cmp.size());
    let q =
        "PREFIX ex: <http://ex/> SELECT ?n ?a WHERE { ?s ex:name ?n . ?s ex:age ?a } ORDER BY ?a";
    assert_eq!(
        raw.query(q).unwrap(),
        cmp.query(q).unwrap(),
        "compressed exported store must return identical JSON"
    );
    assert!(cmp.heap_bytes() <= raw.heap_bytes());
}

// ---- query -> SPARQL 1.1 JSON ----------------------------------------------

/// The exported `Store::query` returns a well-formed SPARQL-1.1-JSON string: head vars in
/// order, a typed integer literal, a language tag, a plain string literal without a
/// datatype, and exactly two rows.
#[test]
fn exported_query_sparql_json_shape() {
    let store = Store::load(DATA, "turtle").unwrap();
    let json = store
        .query(
            "PREFIX ex: <http://ex/> SELECT ?n ?a WHERE { ?s ex:name ?n . ?s ex:age ?a } ORDER BY ?a",
        )
        .unwrap();
    assert!(json.contains("\"vars\":[\"n\",\"a\"]"), "head vars: {json}");
    assert!(
        json.contains("\"datatype\":\"http://www.w3.org/2001/XMLSchema#integer\""),
        "integer datatype: {json}"
    );
    assert!(json.contains("\"xml:lang\":\"en\""), "lang tag: {json}");
    assert!(
        json.contains("\"value\":\"Alice\"}"),
        "plain literal: {json}"
    );
    assert_eq!(json.matches("\"a\":{").count(), 2, "two rows: {json}");
}

/// A URI-valued cell serialises as a `uri` term with its absolute IRI through the export.
#[test]
fn exported_query_uri_term() {
    let store = Store::load(DATA, "turtle").unwrap();
    let json = store
        .query("PREFIX ex: <http://ex/> SELECT ?o WHERE { ?s ex:knows ?o }")
        .unwrap();
    assert!(
        json.contains("\"type\":\"uri\",\"value\":\"http://ex/bob\""),
        "uri term: {json}"
    );
}

// ---- queryChunks / QueryChunks::next ---------------------------------------

/// `Store::queryChunks` streams the JSON bytes in row-boundary chunks; concatenating
/// every chunk (via the exported `QueryChunks::next`) reproduces `query`'s string exactly.
#[test]
fn exported_query_chunks_reassemble() {
    let store = Store::load(DATA, "turtle").unwrap();
    let q = "PREFIX ex: <http://ex/> SELECT ?s ?n WHERE { ?s ex:name ?n } ORDER BY ?n";
    let whole = store.query(q).unwrap();

    let mut chunks = store.query_chunks(q).unwrap();
    let mut reassembled = String::new();
    let mut n = 0;
    while let Some(c) = chunks.next() {
        n += 1;
        reassembled.push_str(&c);
    }
    assert!(n >= 1, "at least one chunk");
    assert_eq!(
        reassembled, whole,
        "concatenated chunks must equal the one-shot JSON"
    );
}

// ---- queryCursor / SolutionCursor ------------------------------------------

/// `Store::queryCursor` yields self-contained SPARQL-JSON batches; with batch size 1 over
/// a 2-row result it produces two non-empty batches then exhausts, and the exported
/// `SolutionCursor` getters (`vars`, `rowCount`, `batchSize`) report the right values.
#[test]
fn exported_query_cursor_batches() {
    let store = Store::load(DATA, "turtle").unwrap();
    let q = "PREFIX ex: <http://ex/> SELECT ?s ?n WHERE { ?s ex:name ?n } ORDER BY ?n";
    let mut cur = store.query_cursor(q, 1).unwrap();
    assert_eq!(cur.row_count(), 2);
    assert_eq!(cur.batch_size(), 1);
    assert_eq!(cur.vars(), vec!["s".to_string(), "n".to_string()]);

    let b0 = cur.next().expect("first batch");
    let b1 = cur.next().expect("second batch");
    assert!(cur.next().is_none(), "exhausted after the last batch");
    for b in [&b0, &b1] {
        assert!(
            b.contains("\"vars\":[\"s\",\"n\"]"),
            "each batch is a full SPARQL-JSON doc: {b}"
        );
        assert_eq!(b.matches("\"n\":{").count(), 1, "one row per batch: {b}");
    }
}

/// A zero batch size is clamped to 1 by the exported `queryCursor`, and an oversized batch
/// yields everything in one batch.
#[test]
fn exported_query_cursor_clamp_and_oversize() {
    let store = Store::load(DATA, "turtle").unwrap();
    let q = "PREFIX ex: <http://ex/> SELECT ?n WHERE { ?s ex:name ?n }";

    let zero = store.query_cursor(q, 0).unwrap();
    assert_eq!(zero.batch_size(), 1, "batch size clamps to >= 1");

    let mut big = store.query_cursor(q, 1000).unwrap();
    let only = big.next().expect("single batch");
    assert!(big.next().is_none());
    assert_eq!(
        only.matches("\"n\":{").count(),
        2,
        "oversized batch holds all rows: {only}"
    );
}

/// An empty result through the exported cursor yields exactly one empty batch, then
/// terminates — so JS can still read `head.vars` and distinguish "no rows" from "drained".
#[test]
fn exported_query_cursor_empty() {
    let store = Store::load(DATA, "turtle").unwrap();
    let mut cur = store
        .query_cursor(
            "PREFIX ex: <http://ex/> SELECT ?x WHERE { ?s ex:nope ?x }",
            8,
        )
        .unwrap();
    assert_eq!(cur.row_count(), 0);
    let only = cur.next().expect("one empty batch even with zero rows");
    assert!(
        only.contains("\"bindings\":[]"),
        "empty result batch must have empty bindings: {only}"
    );
    assert!(
        cur.next().is_none(),
        "exhausted after the single empty batch"
    );
}

// ---- queryQuads / queryQuadsChunks / QuadChunks ----------------------------

/// `Store::queryQuads` (CONSTRUCT) returns the constructed graph as N-Triples through the
/// export: two constructed triples, expanded predicate IRI, language tag preserved.
#[test]
fn exported_query_quads_construct() {
    let store = Store::load(DATA, "turtle").unwrap();
    let nt = store
        .query_quads("PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:label ?n } WHERE { ?s ex:name ?n }")
        .unwrap();
    assert_eq!(
        nt.lines().filter(|l| !l.trim().is_empty()).count(),
        2,
        "two triples: {nt}"
    );
    assert!(
        nt.contains("<http://ex/alice> <http://ex/label> \"Alice\" ."),
        "constructed line: {nt}"
    );
    assert!(nt.contains("\"Bob\"@en"), "lang tag preserved: {nt}");
}

/// `Store::queryQuadsChunks` batches the constructed graph (exported `QuadChunks::next`);
/// concatenation reproduces `queryQuads` exactly.
#[test]
fn exported_query_quads_chunks_reassemble() {
    let store = Store::load(DATA, "turtle").unwrap();
    let q = "PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:label ?n } WHERE { ?s ex:name ?n }";
    let whole = store.query_quads(q).unwrap();

    let mut chunks = store.query_quads_chunks(q, 1).unwrap();
    let mut reassembled = String::new();
    let mut n = 0;
    while let Some(c) = chunks.next() {
        n += 1;
        reassembled.push_str(&c);
    }
    assert_eq!(n, 2, "batch_size 1 over 2 triples => 2 batches");
    assert_eq!(
        reassembled, whole,
        "chunked N-Triples must reassemble to the whole document"
    );
}

// ---- count / ask / askWithMaxRows ------------------------------------------

/// The exported `Store::count` returns the solution count without materialising.
#[test]
fn exported_count() {
    let store = Store::load(DATA, "turtle").unwrap();
    let n = store
        .count("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:name ?n }")
        .unwrap();
    assert_eq!(n, 2);
}

/// The exported `Store::ask` answers a boolean: true when the pattern matches (incl. a
/// FILTER), false when it does not.
#[test]
fn exported_ask() {
    let store = Store::load(DATA, "turtle").unwrap();
    assert!(store
        .ask("PREFIX ex: <http://ex/> ASK { ?s ex:knows ?o }")
        .unwrap());
    assert!(!store
        .ask("PREFIX ex: <http://ex/> ASK { ?s ex:nope ?o }")
        .unwrap());
    assert!(store
        .ask("PREFIX ex: <http://ex/> ASK { ?s ex:age ?a FILTER(?a > 28) }")
        .unwrap());
    assert!(!store
        .ask("PREFIX ex: <http://ex/> ASK { ?s ex:age ?a FILTER(?a > 99) }")
        .unwrap());
}

/// The exported `Store::askWithMaxRows` answers under a generous cap (the portable,
/// wasm-safe working-set budget dimension).
#[test]
fn exported_ask_with_max_rows() {
    let store = Store::load(DATA, "turtle").unwrap();
    let q = "PREFIX ex: <http://ex/> ASK { ?s ex:knows ?o . ?s ex:age ?a FILTER(?a > 28) }";
    assert!(store.ask_with_max_rows(q, 1024).unwrap());
}

// ---- update / updateInPlace / applyDelta -----------------------------------

/// The exported `Store::update` returns a NEW store with the inserted data; the receiver
/// is unchanged (immutable rebuild semantics).
#[test]
fn exported_update_returns_new_store() {
    let store = Store::load(DATA, "turtle").unwrap();
    let next = store
        .update("PREFIX ex: <http://ex/> INSERT DATA { ex:carol ex:name \"Carol\" }")
        .unwrap();
    assert_eq!(store.size(), 5, "receiver unchanged after update");
    assert_eq!(next.size(), 6, "new store has the inserted triple");
    assert!(next
        .ask("PREFIX ex: <http://ex/> ASK { ex:carol ex:name \"Carol\" }")
        .unwrap());
}

/// The exported `Store::updateInPlace` mutates the receiver through the delta overlay.
#[test]
fn exported_update_in_place() {
    let mut store = Store::load(DATA, "turtle").unwrap();
    store
        .update_in_place("PREFIX ex: <http://ex/> INSERT DATA { ex:dave ex:age 40 }")
        .unwrap();
    assert_eq!(store.size(), 6);
    assert!(store
        .ask("PREFIX ex: <http://ex/> ASK { ex:dave ex:age 40 }")
        .unwrap());
}

/// The exported `Store::applyDelta` applies an N-Triples insert/delete batch in one shot.
#[test]
fn exported_apply_delta() {
    let mut store = Store::load(DATA, "turtle").unwrap();
    let inserts = "<http://ex/eve> <http://ex/name> \"Eve\" .\n";
    let deletes = "<http://ex/alice> <http://ex/knows> <http://ex/bob> .\n";
    store.apply_delta(inserts, deletes).unwrap();
    // -1 (knows removed) +1 (eve name) = net 5.
    assert_eq!(store.size(), 5);
    assert!(store
        .ask("PREFIX ex: <http://ex/> ASK { ex:eve ex:name \"Eve\" }")
        .unwrap());
    assert!(!store
        .ask("PREFIX ex: <http://ex/> ASK { ex:alice ex:knows ex:bob }")
        .unwrap());
}

// ---- explain / explainAnalyze ----------------------------------------------

/// The exported `Store::explain` returns the planning-only plan text (query-form header +
/// `Plan:` tree) without executing.
#[test]
fn exported_explain() {
    let store = Store::load(DATA, "turtle").unwrap();
    let plan = store
        .explain("PREFIX ex: <http://ex/> SELECT ?n ?a WHERE { ?s ex:name ?n . ?s ex:age ?a }")
        .unwrap();
    assert!(plan.contains("EXPLAIN (SELECT)"), "names the form: {plan}");
    assert!(plan.contains("Plan:"), "plan tree header: {plan}");
}

/// The exported `Store::explainAnalyze` returns the plan plus an execution trace for a
/// SELECT.
#[test]
fn exported_explain_analyze() {
    let store = Store::load(DATA, "turtle").unwrap();
    let r = store
        .explain_analyze("PREFIX ex: <http://ex/> SELECT ?n WHERE { ?s ex:name ?n }")
        .unwrap();
    assert!(
        r.contains("EXPLAIN ANALYZE (SELECT)"),
        "names analyze + form: {r}"
    );
    assert!(r.contains("Plan:"), "analyze output includes the plan: {r}");
}

// ---- shacl: the exported `Store::validate` success arm ----------------------
//
// Only the OK arm runs natively (the parse-error `Err` arm constructs `JsError`); the
// negative arm stays covered by the wasm32 `tests/web.rs::shacl::validate_parse_error_is_err`.
#[cfg(feature = "shacl")]
mod shacl {
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
            sh:minInclusive 0 ;
            sh:message "age must be a non-negative integer" ;
          ] ;
          sh:property [ sh:path ex:name ; sh:minCount 1 ] .
    "#;

    /// The exported `Store::validate` returns the conforming report through the boundary.
    #[test]
    fn exported_validate_conforming() {
        let store = Store::load("", "turtle").unwrap();
        let data = r#"
            @prefix ex: <http://example.org/> .
            ex:alice a ex:Person ; ex:age 30 ; ex:name "Alice" .
        "#;
        let json = store.validate(data, SHAPES, "turtle").unwrap();
        assert_eq!(json, r#"{"conforms":true,"results":[]}"#, "{json}");
    }

    /// Violating data surfaces a parseable report with focusNode / path / message through
    /// the exported `Store::validate`.
    #[test]
    fn exported_validate_violating() {
        let store = Store::load("", "turtle").unwrap();
        let data = r#"
            @prefix ex: <http://example.org/> .
            ex:bob a ex:Person ; ex:age -1 .
        "#;
        let json = store.validate(data, SHAPES, "turtle").unwrap();
        assert!(json.contains(r#""conforms":false"#), "{json}");
        assert!(
            json.contains(r#""focusNode":"<http://example.org/bob>""#),
            "focus node: {json}"
        );
        assert!(
            json.contains(r#""message":"age must be a non-negative integer""#),
            "declared message: {json}"
        );
        assert!(
            json.contains("MinCountConstraintComponent"),
            "minCount (missing name): {json}"
        );
    }
}
