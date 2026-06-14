// [OPUS-4.8] sq-9qz6 pt2 — headless wasm test of the JS-facing sparq-wasm API.
//
// The crate's `#[cfg(test)] mod tests` in src/lib.rs runs on the NATIVE target only and
// therefore exercises the engine functions the wrappers delegate to — NOT the actual
// `#[wasm_bindgen]` exports, because `JsError::new` is a wasm-bindgen import that panics
// off-wasm. This module closes that gap: every test here drives the REAL exported API
// (`Store::load`, `query`, `ask`, `count`, `queryQuads`, the cursors, the in-place update
// path) compiled to and executed in a genuine wasm32 runtime via `wasm-pack test --node`,
// so result serialisation across the JS boundary (the SPARQL-1.1-JSON shape returned by
// `query`/`query_json`, the N-Triples shape from `queryQuads`, the `JsError` Err path) is
// asserted end to end — exactly the surface CI's `cargo build --target wasm32` never ran.
//
// Datasets are tiny and inline. `wasm-pack test --node` runs each `#[wasm_bindgen_test]`
// in the Node executor (no browser, no DOM); Node is the default target, so no
// `wasm_bindgen_test_configure!(run_in_browser)` directive is needed (and `run_in_node`
// is not a valid directive in wasm-bindgen-test 0.3.x — the runner flag picks Node).

#![cfg(target_arch = "wasm32")]

use sparq_wasm::Store;
use wasm_bindgen_test::*;

const DATA: &str = r#"@prefix ex: <http://ex/> .
    ex:alice ex:name "Alice" ; ex:age 30 ; ex:knows ex:bob .
    ex:bob ex:name "Bob"@en ; ex:age 25 ."#;

// ---- load + basic shape across the JS boundary ----

/// Turtle loads into the wasm Store and reports the deduplicated triple count + a
/// non-zero heap estimate — the two getters JS reads off a freshly loaded store.
#[wasm_bindgen_test]
fn load_turtle_size_and_heap() {
    let store = Store::load(DATA, "turtle").expect("turtle must load in wasm");
    // 2 names + 2 ages + 1 knows = 5 triples.
    assert_eq!(store.size(), 5);
    assert!(store.heap_bytes() > 0, "heap estimate must be non-zero");
}

/// N-Triples also loads (the other primary format JS callers pass).
#[wasm_bindgen_test]
fn load_ntriples() {
    let nt = "<http://ex/a> <http://ex/p> <http://ex/b> .\n<http://ex/a> <http://ex/p> <http://ex/c> .\n";
    let store = Store::load(nt, "ntriples").expect("ntriples must load in wasm");
    assert_eq!(store.size(), 2);
}

/// A malformed document surfaces as the `Err` (JsError) arm across the boundary, not a
/// panic / trap — the error path JS code relies on (`try { Store.load(...) } catch`).
#[wasm_bindgen_test]
fn load_error_is_err_not_trap() {
    let bad = Store::load("@prefix ex: <http://ex/> . ex:a ex:p", "turtle");
    assert!(
        bad.is_err(),
        "a truncated triple must return Err, not panic"
    );
}

// ---- SELECT -> SPARQL 1.1 JSON (the query_json shape) ----

/// `query` returns a well-formed SPARQL-1.1-JSON string: head vars in order, a typed
/// integer literal with its xsd:integer datatype, a language tag on "Bob"@en, a plain
/// string literal without a datatype, and exactly two solution rows — i.e. the full
/// serialisation contract crossing the wasm/JS boundary as a real JS string.
#[wasm_bindgen_test]
fn select_sparql_json_shape() {
    let store = Store::load(DATA, "turtle").unwrap();
    let json = store
        .query("PREFIX ex: <http://ex/> SELECT ?n ?a WHERE { ?s ex:name ?n . ?s ex:age ?a } ORDER BY ?a")
        .expect("select must return JSON");
    assert!(json.contains("\"vars\":[\"n\",\"a\"]"), "head vars: {json}");
    assert!(
        json.contains("\"datatype\":\"http://www.w3.org/2001/XMLSchema#integer\""),
        "integer datatype: {json}"
    );
    assert!(json.contains("\"xml:lang\":\"en\""), "lang tag: {json}");
    // A plain string literal omits the xsd:string datatype.
    assert!(
        json.contains("\"value\":\"Alice\"}"),
        "plain literal: {json}"
    );
    // Two solutions (one `?a` cell object each).
    assert_eq!(json.matches("\"a\":{").count(), 2, "two rows: {json}");
}

/// A URI-valued cell serialises as a `uri` term with its absolute IRI.
#[wasm_bindgen_test]
fn select_uri_term() {
    let store = Store::load(DATA, "turtle").unwrap();
    let json = store
        .query("PREFIX ex: <http://ex/> SELECT ?o WHERE { ?s ex:knows ?o }")
        .unwrap();
    assert!(
        json.contains("\"type\":\"uri\",\"value\":\"http://ex/bob\""),
        "uri term: {json}"
    );
}

/// A literal with a quote/backslash is JSON-escaped on the way across the boundary.
#[wasm_bindgen_test]
fn select_escaping() {
    let store = Store::load(
        "@prefix ex: <http://ex/> . ex:a ex:p \"q\\\"x\" .",
        "turtle",
    )
    .unwrap();
    let json = store.query("SELECT ?o WHERE { ?s ?p ?o }").unwrap();
    assert!(
        json.contains("\"value\":\"q\\\"x\""),
        "escaped literal: {json}"
    );
}

/// An empty SELECT result is still a valid SPARQL-JSON doc with empty bindings.
#[wasm_bindgen_test]
fn select_empty_result() {
    let store = Store::load(DATA, "turtle").unwrap();
    let json = store
        .query("PREFIX ex: <http://ex/> SELECT ?x WHERE { ?s ex:nope ?x }")
        .unwrap();
    assert!(json.contains("\"bindings\":[]"), "empty bindings: {json}");
}

// ---- ASK -> plain boolean across the boundary ----

/// `ask` returns a native JS boolean (not a JSON doc): true when the pattern matches,
/// false when it does not, and FILTERs are evaluated.
#[wasm_bindgen_test]
fn ask_boolean() {
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

/// A non-ASK query routed to `ask` is rejected as Err across the boundary.
#[wasm_bindgen_test]
fn ask_rejects_select() {
    let store = Store::load(DATA, "turtle").unwrap();
    assert!(store.ask("SELECT ?s WHERE { ?s ?p ?o }").is_err());
}

/// `askWithMaxRows`: a generous cap answers; a zero cap trips the working-set budget
/// (the portable, wasm-safe budget dimension) and returns Err.
#[wasm_bindgen_test]
fn ask_with_max_rows_budget() {
    let store = Store::load(DATA, "turtle").unwrap();
    let q = "PREFIX ex: <http://ex/> ASK { ?s ex:knows ?o . ?s ex:age ?a FILTER(?a > 28) }";
    assert!(store.ask_with_max_rows(q, 1024).unwrap());
    assert!(
        store.ask_with_max_rows(q, 0).is_err(),
        "zero cap must trip the budget"
    );
}

// ---- count (read from the index, no materialisation) ----

/// `count` returns the solution count as a JS number.
#[wasm_bindgen_test]
fn count_solutions() {
    let store = Store::load(DATA, "turtle").unwrap();
    let n = store
        .count("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:name ?n }")
        .unwrap();
    assert_eq!(n, 2);
}

// ---- CONSTRUCT / DESCRIBE -> N-Triples ----

/// `queryQuads` (CONSTRUCT) returns the constructed graph as N-Triples across the boundary.
#[wasm_bindgen_test]
fn construct_quads() {
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

/// A SELECT routed to `queryQuads` is rejected as Err across the boundary.
#[wasm_bindgen_test]
fn query_quads_rejects_select() {
    let store = Store::load(DATA, "turtle").unwrap();
    assert!(store.query_quads("SELECT ?s WHERE { ?s ?p ?o }").is_err());
}

// ---- streaming cursors: SolutionCursor / QueryChunks / QuadChunks ----

/// `queryCursor` yields self-contained SPARQL-JSON batches; with batch size 1 over a
/// 2-row result it produces two non-empty batches then exhausts, and exposes vars +
/// rowCount + batchSize getters across the boundary.
#[wasm_bindgen_test]
fn query_cursor_batches() {
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

/// `queryChunks` streams the JSON bytes in row-boundary chunks; concatenating every
/// chunk reproduces `query`'s one-shot string exactly.
#[wasm_bindgen_test]
fn query_chunks_reassemble() {
    let store = Store::load(DATA, "turtle").unwrap();
    let q = "PREFIX ex: <http://ex/> SELECT ?s ?n WHERE { ?s ex:name ?n } ORDER BY ?n";
    let whole = store.query(q).unwrap();
    let mut chunks = store.query_chunks(q).unwrap();
    let mut reassembled = String::new();
    while let Some(c) = chunks.next() {
        reassembled.push_str(&c);
    }
    assert_eq!(
        reassembled, whole,
        "concatenated chunks must equal the one-shot JSON"
    );
}

/// `queryQuadsChunks` batches the constructed graph; concatenation == `queryQuads`.
#[wasm_bindgen_test]
fn query_quads_chunks_reassemble() {
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

// ---- compressed store: identical results, smaller footprint ----

/// `loadCompressed` returns byte-identical SELECT JSON to the raw store and reports a
/// footprint no larger than the raw store — verified inside the real wasm runtime where
/// the 3-permutation compact index is the one actually selected.
#[wasm_bindgen_test]
fn compressed_matches_raw() {
    let raw = Store::load(DATA, "turtle").unwrap();
    let cmp = Store::load_compressed(DATA, "turtle").unwrap();
    assert_eq!(raw.size(), cmp.size());
    let q =
        "PREFIX ex: <http://ex/> SELECT ?n ?a WHERE { ?s ex:name ?n . ?s ex:age ?a } ORDER BY ?a";
    assert_eq!(
        raw.query(q).unwrap(),
        cmp.query(q).unwrap(),
        "compressed JSON must match raw"
    );
    assert!(
        cmp.heap_bytes() <= raw.heap_bytes(),
        "compressed footprint must not exceed raw"
    );
}

// ---- mutation paths: update / updateInPlace / applyDelta ----

/// `update` returns a NEW store with the inserted data; the receiver is unchanged
/// (immutable rebuild semantics) — both observable across the boundary.
#[wasm_bindgen_test]
fn update_returns_new_store() {
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

/// `updateInPlace` mutates the receiver through the delta overlay (O(batch), no rebuild).
#[wasm_bindgen_test]
fn update_in_place_mutates_receiver() {
    let mut store = Store::load(DATA, "turtle").unwrap();
    store
        .update_in_place("PREFIX ex: <http://ex/> INSERT DATA { ex:dave ex:age 40 }")
        .unwrap();
    assert_eq!(store.size(), 6);
    assert!(store
        .ask("PREFIX ex: <http://ex/> ASK { ex:dave ex:age 40 }")
        .unwrap());
}

/// `applyDelta` applies an N-Triples insert/delete batch in one shot through the overlay.
#[wasm_bindgen_test]
fn apply_delta_batch() {
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
