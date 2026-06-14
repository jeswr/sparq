//! sparq-wasm: the sparq parser + triplestore + SPARQL engine compiled to
//! WebAssembly, with a minimal bundle (no threads, no serde — results are
//! serialised by hand to SPARQL 1.1 JSON).
//!
//! ```js
//! import init, { Store } from "./sparq_wasm.js";
//! await init();
//! const store = Store.load(turtleText, "turtle");
//! const json = store.query("SELECT * WHERE { ?s ?p ?o } LIMIT 10");
//! const { results } = JSON.parse(json);
//! ```
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

use sparq_core::Graph;
use wasm_bindgen::prelude::*;

/// An immutable, dictionary-encoded RDF store queryable with SPARQL.
#[wasm_bindgen]
pub struct Store {
    graph: Graph,
}

/// The ordered chunk sequence of one query result (see [`Store::query_chunks`]):
/// concatenating every chunk yields exactly [`Store::query`]'s JSON string. Chunks
/// split only at solution-row boundaries (~64 KiB flushes), so a consumer can parse
/// rows incrementally without ever holding the whole result as one JS string.
#[wasm_bindgen]
pub struct QueryChunks {
    chunks: std::vec::IntoIter<String>,
}

#[wasm_bindgen]
impl QueryChunks {
    /// The next chunk, or `undefined` when the sequence is exhausted.
    // clippy: a #[wasm_bindgen]-exported inherent method (JS calls `.next()`); it cannot
    // be `Iterator::next`, and renaming would break the published JS binding contract.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<String> {
        self.chunks.next()
    }
}

/// A forward-only **cursor over a SELECT result's solution rows** (see
/// [`Store::query_cursor`]): each [`next`](Self::next) yields the next *batch* of up to
/// `batch_size` solutions as a **self-contained** SPARQL 1.1 JSON document — vars in
/// `head`, just that batch's rows in `results.bindings` — so the consumer can `JSON.parse`
/// each batch on its own and process (then drop) it before pulling the next. Unlike
/// [`QueryChunks`], whose chunks are arbitrary byte-cuts of one big JSON string that must
/// be re-joined before parsing, every cursor batch is independently valid. The result is
/// materialised once inside wasm (the engine has no lazy solution iterator at this layer),
/// but each batch's JSON string is built lazily on demand and never retained, so the heavy
/// JS-side string copy is bounded to one batch at a time — never the whole result at once.
#[wasm_bindgen]
pub struct SolutionCursor {
    result: sparq_engine::QueryResult,
    pos: usize,
    batch_size: usize,
}

#[wasm_bindgen]
impl SolutionCursor {
    /// The next batch as a standalone SPARQL 1.1 JSON results document, or `undefined`
    /// once every solution has been yielded. A query with zero solutions yields exactly
    /// one batch (the empty-`bindings` document) and is then exhausted, so a caller can
    /// distinguish "no rows" (one empty batch) from "fully drained" (`undefined`).
    // clippy: a #[wasm_bindgen]-exported inherent method (JS calls `.next()`); it cannot
    // be `Iterator::next`, and renaming would break the JS binding contract.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<String> {
        let total = self.result.rows.len();
        // Exhausted: emit one empty batch for an empty result (pos 0, total 0), then stop.
        if self.pos > total || (self.pos == total && total != 0) {
            return None;
        }
        let end = (self.pos + self.batch_size).min(total);
        let json = sparq_engine::json::to_sparql_json_rows(&self.result.vars, &self.result.rows[self.pos..end]);
        // Advance past `end`; for an empty result step from 0 to 1 so the next call stops.
        self.pos = if total == 0 { 1 } else { end };
        Some(json)
    }

    /// The projected variable names, in order — the `head.vars` shared by every batch.
    pub fn vars(&self) -> Vec<String> {
        self.result.vars.iter().map(|v| v.as_str().to_string()).collect()
    }

    /// The total number of solution rows in the (already materialised) result.
    #[wasm_bindgen(js_name = rowCount)]
    pub fn row_count(&self) -> usize {
        self.result.rows.len()
    }

    /// The configured batch size (max solutions per [`next`](Self::next)).
    #[wasm_bindgen(js_name = batchSize)]
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
}

/// A forward-only cursor over the N-Triples lines of a CONSTRUCT/DESCRIBE result graph
/// (see [`Store::query_quads_chunks`]): each [`next`](Self::next) yields the next batch of
/// up to `batch_size` triples as an N-Triples fragment (which is also valid Turtle —
/// N-Triples ⊂ Turtle). Concatenating every batch reproduces [`Store::query_quads`]'s full
/// document. The graph is materialised once inside wasm, but each batch string is built on
/// demand and not retained, so the JS-side copy is bounded to one batch at a time.
#[wasm_bindgen]
pub struct QuadChunks {
    chunks: std::vec::IntoIter<String>,
}

#[wasm_bindgen]
impl QuadChunks {
    /// The next N-Triples fragment, or `undefined` when the graph is exhausted.
    // clippy: a #[wasm_bindgen]-exported inherent method (JS calls `.next()`); it cannot
    // be `Iterator::next`, and renaming would break the JS binding contract.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<String> {
        self.chunks.next()
    }
}

#[wasm_bindgen]
impl Store {
    /// Parses an RDF document into a store. `format`: `"turtle"` | `"ntriples"` |
    /// `"nquads"` | `"trig"` (named graphs are folded into the default graph).
    pub fn load(text: &str, format: &str) -> Result<Store, JsError> {
        let graph = Graph::load_str(text, format).map_err(|e| JsError::new(&e))?;
        Ok(Store { graph })
    }

    /// Like [`load`](Self::load) but preserves NAMED GRAPHS from N-Quads / TriG as
    /// separate sub-graphs, so `GRAPH <iri> { … }` / `GRAPH ?g { … }` patterns,
    /// `FROM` / `FROM NAMED` dataset clauses, and SPARQL Updates with `GRAPH`
    /// blocks (including `CLEAR GRAPH` / `DROP GRAPH`) all see the dataset.
    /// Formats without named graphs ("turtle" / "ntriples") load as [`load`].
    /// [`size`](Self::size) / [`heapBytes`](Self::heap_bytes) report the DEFAULT
    /// graph only (count the dataset with `GRAPH ?g` queries).
    #[wasm_bindgen(js_name = loadDataset)]
    pub fn load_dataset(text: &str, format: &str) -> Result<Store, JsError> {
        let graph = Graph::load_dataset(text, format).map_err(|e| JsError::new(&e))?;
        Ok(Store { graph })
    }

    /// Like [`load`](Self::load) but stores the index BLOCK-COMPRESSED (~4-6 B/triple vs
    /// 12 — roughly half the index memory, measured −49% on the 6-perm set / −60% on the
    /// 3-perm compact set the browser uses). Query results are identical; scans pay a
    /// bounded per-block decode (+10–33% on large materialised queries). The right default
    /// when the device's RAM, not its CPU, is the binding constraint — i.e. fitting a
    /// bigger graph in the tab.
    #[wasm_bindgen(js_name = loadCompressed)]
    pub fn load_compressed(text: &str, format: &str) -> Result<Store, JsError> {
        let graph = Graph::load_str_compressed(text, format).map_err(|e| JsError::new(&e))?;
        Ok(Store { graph })
    }

    /// The number of (deduplicated) triples in the store.
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> usize {
        self.graph.len()
    }

    /// A rough estimate of the store's in-memory footprint, in bytes.
    #[wasm_bindgen(js_name = heapBytes)]
    pub fn heap_bytes(&self) -> usize {
        self.graph.heap_bytes()
    }

    /// Runs a SELECT query and returns the results as a SPARQL 1.1 JSON string
    /// (`application/sparql-results+json`). Benefits from the engine's streaming
    /// optimisations: LIMIT stops the scan early, numeric FILTERs are pushed into
    /// the scan, OPTIONAL uses a sort-merge join, and COUNT(*) is computed from the
    /// index without materialising — all of which matter even more in the browser,
    /// where memory and main-thread time are scarce.
    pub fn query(&self, sparql: &str) -> Result<String, JsError> {
        // Serialise straight from ids to SPARQL-JSON — no intermediate oxrdf::Term per
        // cell (the allocator-bound cost of returning a large result in the browser).
        sparq_engine::query_json(&self.graph, sparql).map_err(|e| JsError::new(&e))
    }

    /// Like [`query`](Self::query) but returns the SPARQL 1.1 JSON document as an
    /// ordered sequence of ~64 KiB chunks (split only at solution-row boundaries)
    /// instead of one string — so large results cross the wasm boundary piecewise
    /// and the caller can surface rows incrementally. The chunk sequence is
    /// produced eagerly inside wasm (the engine's chunked serialiser, which never
    /// concatenates a whole-result string); the streaming win is on the JS side,
    /// which holds at most one chunk at a time.
    #[wasm_bindgen(js_name = queryChunks)]
    pub fn query_chunks(&self, sparql: &str) -> Result<QueryChunks, JsError> {
        let chunks =
            sparq_engine::query_json_chunks_with_budget(&self.graph, sparql, &sparq_engine::QueryBudget::unlimited())
                .map_err(|e| JsError::new(&e))?;
        Ok(QueryChunks { chunks: chunks.into_iter() })
    }

    /// Runs a SELECT (or ASK) query and returns a [`SolutionCursor`] that yields the
    /// solutions in batches of at most `batchSize` rows, each batch a self-contained
    /// SPARQL 1.1 JSON document the caller can `JSON.parse` on its own. This is the
    /// row-oriented streaming entry point: pull a batch, surface/drop its rows, pull the
    /// next — the consumer never holds more than one batch, so peak JS memory is bounded
    /// by `batchSize` rather than by the whole result. (`queryChunks` streams the *bytes*
    /// of one JSON string at fixed ~64 KiB cuts that must be re-joined before parsing;
    /// `queryCursor` streams *parseable solution batches*.) `batchSize` is clamped to at
    /// least 1. Caveat: the engine materialises the full result inside wasm before the
    /// first batch — there is no lazy engine-level solution iterator at this layer — so the
    /// bound is on the JS-side string copy, not on wasm working set.
    #[wasm_bindgen(js_name = queryCursor)]
    pub fn query_cursor(&self, sparql: &str, batch_size: usize) -> Result<SolutionCursor, JsError> {
        let result = sparq_engine::query(&self.graph, sparql).map_err(|e| JsError::new(&e))?;
        Ok(SolutionCursor { result, pos: 0, batch_size: batch_size.max(1) })
    }

    /// Runs a **CONSTRUCT or DESCRIBE** query and returns the resulting RDF graph
    /// serialised as **N-Triples** (one `s p o .` line per triple). N-Triples is a
    /// syntactic subset of Turtle, so the returned string is also a valid `text/turtle`
    /// document. This is the quad-returning entry point: where [`query`](Self::query)
    /// answers SELECT/ASK with a solution table, `queryQuads` answers the graph-valued
    /// query forms with their constructed graph. CONSTRUCT instantiates its template once
    /// per WHERE solution (template blank nodes are freshened per solution, and triples
    /// with unbound or RDF-illegal terms are dropped per SPARQL §16.2); DESCRIBE returns
    /// the concise bounded description of each described resource. A SELECT/ASK query is
    /// rejected here — use [`query`](Self::query) / [`queryChunks`](Self::query_chunks).
    #[wasm_bindgen(js_name = queryQuads)]
    pub fn query_quads(&self, sparql: &str) -> Result<String, JsError> {
        sparq_engine::construct_ntriples(&self.graph, sparql).map_err(|e| JsError::new(&e))
    }

    /// Like [`queryQuads`](Self::query_quads) but returns a [`QuadChunks`] cursor that
    /// yields the constructed graph in batches of at most `batchSize` triples (each an
    /// N-Triples fragment), so a large constructed/described graph crosses the wasm
    /// boundary piecewise and the caller holds at most one batch at a time. Concatenating
    /// every batch reproduces `queryQuads`'s document exactly. `batchSize` is clamped to at
    /// least 1. Caveat: as with [`queryQuads`](Self::query_quads) the full graph is
    /// materialised inside wasm before the first batch; the bound is on the JS-side copy.
    #[wasm_bindgen(js_name = queryQuadsChunks)]
    pub fn query_quads_chunks(&self, sparql: &str, batch_size: usize) -> Result<QuadChunks, JsError> {
        let triples = sparq_engine::construct_or_describe(&self.graph, sparql).map_err(|e| JsError::new(&e))?;
        let batch = batch_size.max(1);
        let chunks: Vec<String> =
            triples.chunks(batch).map(sparq_engine::triples_to_ntriples).collect();
        Ok(QuadChunks { chunks: chunks.into_iter() })
    }

    /// Counts the solutions of a SELECT query *without* materialising them — for a
    /// single-pattern scan or a two-pattern join the count is read straight from
    /// the index (no result rows built). Ideal for "how many?" UI queries on a
    /// memory-constrained device.
    pub fn count(&self, sparql: &str) -> Result<usize, JsError> {
        sparq_engine::count(&self.graph, sparql).map_err(|e| JsError::new(&e))
    }

    /// Answers an **ASK** query as a plain `boolean`, evaluated through the engine's
    /// NATIVE ask path ([`sparq_engine::ask`]): the pattern is evaluated under an
    /// implicit `LIMIT 1`, so the scan/join **early-exits at the first solution** and
    /// nothing is materialised — no SELECT result is built, no SPARQL-JSON string is
    /// serialised, and no boolean is parsed back out on the JS side. This is the
    /// right entry point for an existence check on a memory-constrained device: prefer
    /// it over routing an ASK through [`query`](Self::query) (which would build and
    /// serialise the boolean results document) or, worse, rewriting it to a counted
    /// `SELECT *`. A non-ASK query (SELECT / CONSTRUCT / DESCRIBE / UPDATE) is rejected
    /// with a clear error — use [`query`](Self::query) / [`queryQuads`](Self::query_quads).
    pub fn ask(&self, sparql: &str) -> Result<bool, JsError> {
        sparq_engine::ask(&self.graph, sparql).map_err(|e| JsError::new(&e))
    }

    /// Like [`ask`](Self::ask) but under a cooperative working-set budget: any
    /// intermediate or final materialised result exceeding `maxRows` rows aborts the
    /// query with a `"query budget exceeded (max-rows)"` error rather than running to
    /// completion. Use it to bound the worst-case memory an adversarial / accidentally
    /// huge ASK pattern can take in the browser tab. The early-exit still applies, so a
    /// pattern that finds a solution quickly never approaches the cap. (The engine's
    /// other budget dimension, a wall-clock deadline, is native-only — `std::time::Instant`
    /// is unusable on `wasm32` — so only the portable row cap is exposed here.)
    #[wasm_bindgen(js_name = askWithMaxRows)]
    pub fn ask_with_max_rows(&self, sparql: &str, max_rows: usize) -> Result<bool, JsError> {
        // [OPUS-4.8] `QueryBudget`'s fields are cfg-gated: on `wasm32` the struct has
        // ONLY `max_rows` (the wall-clock `deadline` is native-only — `Instant` panics
        // on wasm32), so `..Default::default()` would be a NEEDLESS struct update there
        // (clippy::needless_update under a wasm-target lint). Start from the unlimited
        // budget and set `max_rows` instead: this is target-agnostic — it fills the
        // native `deadline` (None) without naming it, and degenerates to just `max_rows`
        // on wasm32 — so it is clean under clippy on BOTH targets.
        let mut budget = sparq_engine::QueryBudget::unlimited();
        budget.max_rows = Some(max_rows);
        sparq_engine::ask_with_budget(&self.graph, sparql, &budget).map_err(|e| JsError::new(&e))
    }

    /// Applies a SPARQL 1.1 Update (`INSERT DATA`, `DELETE DATA`, `CLEAR`,
    /// `DELETE/INSERT … WHERE` on the default graph) and returns the **new** store —
    /// the receiver is immutable and remains valid. Mirrors `sparq_engine::update`'s
    /// rebuild semantics. Prefer [`updateInPlace`](Self::update_in_place), which is
    /// O(batch) instead of O(store) for the data operations.
    pub fn update(&self, sparql: &str) -> Result<Store, JsError> {
        let graph = sparq_engine::update(&self.graph, sparql).map_err(|e| JsError::new(&e))?;
        Ok(Store { graph })
    }

    /// Applies a SPARQL 1.1 Update IN PLACE through the store's delta overlay
    /// (`sparq_engine::update_in_place`): data operations are O(batch) per target
    /// graph — no index rebuild — and `GRAPH` blocks / graph templates / `CLEAR` /
    /// `DROP` / `CREATE` address named graphs. The dictionary grows append-only,
    /// so existing term ids stay valid.
    #[wasm_bindgen(js_name = updateInPlace)]
    pub fn update_in_place(&mut self, sparql: &str) -> Result<(), JsError> {
        sparq_engine::update_in_place(&mut self.graph, sparql).map_err(|e| JsError::new(&e))
    }

    /// Incremental quad-level delta, mirroring `Graph::apply_delta`: parses
    /// `inserts` and `deletes` as N-Quads (N-Triples for default-graph data) and
    /// applies them as ONE batch — deletes first, then inserts, routed per graph
    /// (named graphs auto-created on first insert) — through the delta overlay:
    /// O(batch), no rebuild. Blank nodes denote concrete nodes BY LABEL, so bnode
    /// triples CAN be retracted (impossible via SPARQL `DELETE DATA`).
    #[wasm_bindgen(js_name = applyDelta)]
    pub fn apply_delta(&mut self, inserts: &str, deletes: &str) -> Result<(), JsError> {
        self.graph.apply_delta_nquads(inserts, deletes).map_err(|e| JsError::new(&e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: &str = r#"@prefix ex: <http://ex/> .
        ex:alice ex:name "Alice" ; ex:age 30 ; ex:knows ex:bob .
        ex:bob ex:name "Bob"@en ; ex:age 25 ."#;

    #[test]
    fn select_to_sparql_json() {
        let g = Graph::load_str(DATA, "turtle").unwrap();
        let r = sparq_engine::query(
            &g,
            "PREFIX ex: <http://ex/> SELECT ?n ?a WHERE { ?s ex:name ?n . ?s ex:age ?a } ORDER BY ?a",
        )
        .unwrap();
        let json = sparq_engine::json::to_sparql_json(&r);
        // head vars present
        assert!(json.contains("\"vars\":[\"n\",\"a\"]"));
        // typed literal datatype emitted for the integer age
        assert!(json.contains("\"datatype\":\"http://www.w3.org/2001/XMLSchema#integer\""));
        // language tag emitted for "Bob"@en
        assert!(json.contains("\"xml:lang\":\"en\""));
        // a plain string literal omits the xsd:string datatype
        assert!(json.contains("\"value\":\"Alice\"}"));
        // two solutions
        assert_eq!(json.matches("\"a\":{").count(), 2);
    }

    #[test]
    fn escaping_through_json() {
        // A literal containing a quote and backslash must be JSON-escaped.
        let g = Graph::load_str("@prefix ex: <http://ex/> . ex:a ex:p \"q\\\"x\" .", "turtle").unwrap();
        let r = sparq_engine::query(&g, "SELECT ?o WHERE { ?s ?p ?o }").unwrap();
        let json = sparq_engine::json::to_sparql_json(&r);
        assert!(json.contains("\"value\":\"q\\\"x\""), "got: {json}");
    }

    #[test]
    fn uri_and_bnode() {
        let g = Graph::load_str(DATA, "turtle").unwrap();
        let r = sparq_engine::query(&g, "PREFIX ex: <http://ex/> SELECT ?o WHERE { ?s ex:knows ?o }").unwrap();
        let json = sparq_engine::json::to_sparql_json(&r);
        assert!(json.contains("\"type\":\"uri\",\"value\":\"http://ex/bob\""));
    }

    // ---- sq-0f7: SELECT solution cursor (streaming, batch-level) ----

    /// Concatenating a cursor's batches must surface exactly the rows of the one-shot
    /// `query`, and the cursor must honour the batch size and terminate cleanly.
    #[test]
    fn cursor_batches_cover_all_rows() {
        let store = Store::load(DATA, "turtle").unwrap();
        let q = "PREFIX ex: <http://ex/> SELECT ?s ?n WHERE { ?s ex:name ?n } ORDER BY ?n";
        // batch_size 1 over a 2-row result => two non-empty batches, then exhaustion.
        let mut cur = store.query_cursor(q, 1).unwrap();
        assert_eq!(cur.row_count(), 2);
        assert_eq!(cur.batch_size(), 1);
        assert_eq!(cur.vars(), vec!["s".to_string(), "n".to_string()]);

        let b0 = cur.next().expect("first batch");
        let b1 = cur.next().expect("second batch");
        assert!(cur.next().is_none(), "cursor must be exhausted after the last batch");

        // Each batch is a self-contained SPARQL-JSON doc with the full head vars.
        for b in [&b0, &b1] {
            assert!(b.contains("\"vars\":[\"s\",\"n\"]"), "batch missing head vars: {b}");
        }
        // One binding row per batch (batch_size 1), and together they carry both names.
        assert_eq!(b0.matches("\"n\":{").count(), 1);
        assert_eq!(b1.matches("\"n\":{").count(), 1);
        assert!((b0.contains("\"Alice\"") && b1.contains("Bob")) || (b1.contains("\"Alice\"") && b0.contains("Bob")));
    }

    /// A batch larger than the result yields everything in one batch; a zero batch size
    /// is clamped to 1 (never an infinite/empty-step loop).
    #[test]
    fn cursor_oversized_and_zero_batch() {
        let store = Store::load(DATA, "turtle").unwrap();
        let q = "PREFIX ex: <http://ex/> SELECT ?n WHERE { ?s ex:name ?n }";

        let mut big = store.query_cursor(q, 1000).unwrap();
        let only = big.next().expect("single batch");
        assert!(big.next().is_none());
        assert_eq!(only.matches("\"n\":{").count(), 2, "oversized batch must hold all rows");

        let mut zero = store.query_cursor(q, 0).unwrap();
        assert_eq!(zero.batch_size(), 1, "batch size clamps to >= 1");
        assert!(zero.next().is_some());
    }

    /// A result with no solutions yields exactly one empty batch (so JS can read head
    /// vars), then terminates — distinguishing "no rows" from "fully drained".
    #[test]
    fn cursor_empty_result_yields_one_empty_batch() {
        let store = Store::load(DATA, "turtle").unwrap();
        let mut cur = store.query_cursor("PREFIX ex: <http://ex/> SELECT ?x WHERE { ?s ex:nope ?x }", 8).unwrap();
        assert_eq!(cur.row_count(), 0);
        let only = cur.next().expect("one empty batch even with zero rows");
        assert!(only.contains("\"bindings\":[]"), "empty result batch must have empty bindings: {only}");
        assert!(cur.next().is_none(), "exhausted after the single empty batch");
    }

    // ---- sq-hlq: CONSTRUCT / DESCRIBE -> quads ----

    /// CONSTRUCT through `queryQuads` returns the constructed graph as N-Triples
    /// (a valid Turtle subset): absolute IRIs, `.`-terminated lines.
    #[test]
    fn construct_to_quads_ntriples() {
        let store = Store::load(DATA, "turtle").unwrap();
        let nt = store
            .query_quads(
                "PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:label ?n } WHERE { ?s ex:name ?n }",
            )
            .unwrap();
        // Two name triples -> two constructed triples, each a full N-Triples line.
        assert_eq!(nt.lines().filter(|l| !l.trim().is_empty()).count(), 2, "got: {nt}");
        assert!(nt.contains("<http://ex/label>"), "predicate IRI expanded: {nt}");
        assert!(nt.contains("<http://ex/alice> <http://ex/label> \"Alice\" ."), "got: {nt}");
        // Language tag preserved on the constructed literal.
        assert!(nt.contains("\"Bob\"@en"), "lang tag preserved: {nt}");
    }

    /// DESCRIBE also flows through `queryQuads` (concise bounded description).
    #[test]
    fn describe_to_quads() {
        let store = Store::load(DATA, "turtle").unwrap();
        let nt = store.query_quads("DESCRIBE <http://ex/bob>").unwrap();
        // CBD of ex:bob = its outgoing triples (name + age), nothing inbound.
        assert!(nt.contains("<http://ex/bob> <http://ex/name> \"Bob\"@en ."), "got: {nt}");
        assert!(nt.contains("<http://ex/bob> <http://ex/age>"), "got: {nt}");
        assert!(!nt.contains("<http://ex/alice>"), "CBD must not pull in inbound subjects: {nt}");
    }

    /// `queryQuadsChunks` batches the constructed graph; concatenation == `queryQuads`.
    #[test]
    fn construct_quads_chunks_reassemble() {
        let store = Store::load(DATA, "turtle").unwrap();
        let q = "PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:label ?n } WHERE { ?s ex:name ?n }";
        let whole = store.query_quads(q).unwrap();

        let mut chunks = store.query_quads_chunks(q, 1).unwrap();
        let mut reassembled = String::new();
        let mut n_batches = 0;
        while let Some(c) = chunks.next() {
            n_batches += 1;
            reassembled.push_str(&c);
        }
        assert_eq!(n_batches, 2, "batch_size 1 over 2 triples => 2 batches");
        assert_eq!(reassembled, whole, "chunked N-Triples must reassemble to the whole document");
    }

    /// A SELECT routed to the quad path is rejected (it is not a graph-valued query).
    /// Asserted against the engine function `queryQuads` delegates to, because the
    /// `JsError`-returning wasm wrapper cannot construct its error on a native target
    /// (`JsError::new` is a wasm-bindgen import that panics off-wasm).
    #[test]
    fn query_quads_rejects_select() {
        let g = Graph::load_str(DATA, "turtle").unwrap();
        assert!(sparq_engine::construct_ntriples(&g, "SELECT ?s WHERE { ?s ?p ?o }").is_err());
    }

    // ---- sq-16a: native ASK early-exit through the wasm layer ----
    //
    // `Store::ask` / `Store::askWithMaxRows` are thin `JsError`-mapping wrappers over
    // `sparq_engine::ask` / `ask_with_budget`. The mapping closure cannot run on a native
    // target (`JsError::new` is a wasm-bindgen import that panics off-wasm), so — exactly as
    // `query_quads_rejects_select` does — these tests assert against the engine functions the
    // exports delegate to: the dispatch (which engine fn, which budget) is what this task wires
    // up, and the engine's own tests already cover `eval_ask`'s LIMIT-1 early-exit semantics.

    /// The ASK dispatch returns the engine's native boolean — true when the pattern has a
    /// solution, false when it does not — without going through the SELECT/JSON path.
    #[test]
    fn ask_dispatches_to_native_bool() {
        let g = Graph::load_str(DATA, "turtle").unwrap();
        assert!(sparq_engine::ask(&g, "PREFIX ex: <http://ex/> ASK { ?s ex:knows ?o }").unwrap());
        assert!(!sparq_engine::ask(&g, "PREFIX ex: <http://ex/> ASK { ?s ex:nope ?o }").unwrap());
        // FILTER is evaluated (not just an existence count): true above the threshold, false below.
        assert!(sparq_engine::ask(&g, "PREFIX ex: <http://ex/> ASK { ?s ex:age ?a FILTER(?a > 28) }").unwrap());
        assert!(!sparq_engine::ask(&g, "PREFIX ex: <http://ex/> ASK { ?s ex:age ?a FILTER(?a > 99) }").unwrap());
    }

    /// A non-ASK query routed to the ask path is rejected with a clear error (the message the
    /// `JsError` carries), so `Store::ask` can never silently answer a SELECT/CONSTRUCT.
    #[test]
    fn ask_rejects_non_ask() {
        let g = Graph::load_str(DATA, "turtle").unwrap();
        let err = sparq_engine::ask(&g, "SELECT ?s WHERE { ?s ?p ?o }").unwrap_err();
        assert!(err.contains("ASK"), "rejection must mention ASK, got: {err}");
        assert!(sparq_engine::ask(&g, "PREFIX ex: <http://ex/> CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }").is_err());
    }

    /// The budgeted dispatch builds a `max_rows` working-set cap on the portable (wasm-safe)
    /// `QueryBudget` field: a generous cap still answers, a zero cap trips the budget. This is
    /// exactly what `Store::askWithMaxRows` passes to `ask_with_budget`.
    #[test]
    fn ask_with_max_rows_budget() {
        let g = Graph::load_str(DATA, "turtle").unwrap();
        // A two-pattern join materialises an intermediate working set, so the row cap is the
        // thing being enforced (a single-pattern / filter-pushdown ASK answers from the scan
        // without ever building a counted result, so it never approaches the cap — exactly the
        // early-exit win). A generous cap answers; a zero cap trips with a budget error.
        let q = "PREFIX ex: <http://ex/> ASK { ?s ex:knows ?o . ?s ex:age ?a FILTER(?a > 28) }";
        // [OPUS-4.8] Mirror `ask_with_max_rows`: start from `unlimited()` and assign
        // `max_rows`, rather than a struct literal with `..Default::default()`. On wasm32
        // the struct has only the `max_rows` field, so a `..` fill is needless and a
        // wasm-target clippy run flags it (clippy::needless_update); this form is clean
        // on both targets.
        let mut generous = sparq_engine::QueryBudget::unlimited();
        generous.max_rows = Some(1024);
        assert!(sparq_engine::ask_with_budget(&g, q, &generous).unwrap());
        let mut starved = sparq_engine::QueryBudget::unlimited();
        starved.max_rows = Some(0);
        let err = sparq_engine::ask_with_budget(&g, q, &starved).unwrap_err();
        assert!(err.contains("budget"), "starved budget must report a budget error, got: {err}");
    }

    #[test]
    fn compressed_matches_raw() {
        // The compressed browser store must return byte-identical JSON to the raw store,
        // and report a smaller footprint.
        let raw = Store::load(DATA, "turtle").unwrap();
        let cmp = Store::load_compressed(DATA, "turtle").unwrap();
        assert_eq!(raw.size(), cmp.size());
        for q in [
            "PREFIX ex: <http://ex/> SELECT ?n ?a WHERE { ?s ex:name ?n . ?s ex:age ?a } ORDER BY ?a",
            "SELECT ?s ?p ?o WHERE { ?s ?p ?o }",
            "PREFIX ex: <http://ex/> SELECT ?o WHERE { ?s ex:knows ?o }",
        ] {
            assert_eq!(raw.query(q).unwrap(), cmp.query(q).unwrap(), "compressed JSON differs for: {q}");
        }
        assert!(cmp.heap_bytes() <= raw.heap_bytes());
    }
}
