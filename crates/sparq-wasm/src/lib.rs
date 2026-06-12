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

    /// Counts the solutions of a SELECT query *without* materialising them — for a
    /// single-pattern scan or a two-pattern join the count is read straight from
    /// the index (no result rows built). Ideal for "how many?" UI queries on a
    /// memory-constrained device.
    pub fn count(&self, sparql: &str) -> Result<usize, JsError> {
        sparq_engine::count(&self.graph, sparql).map_err(|e| JsError::new(&e))
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
