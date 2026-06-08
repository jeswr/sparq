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

#[wasm_bindgen]
impl Store {
    /// Parses an RDF document into a store. `format`: `"turtle"` | `"ntriples"` |
    /// `"nquads"` | `"trig"` (named graphs are folded into the default graph).
    pub fn load(text: &str, format: &str) -> Result<Store, JsError> {
        let graph = Graph::load_str(text, format).map_err(|e| JsError::new(&e))?;
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
    /// (`application/sparql-results+json`).
    pub fn query(&self, sparql: &str) -> Result<String, JsError> {
        let res = sparq_engine::query(&self.graph, sparql).map_err(|e| JsError::new(&e))?;
        Ok(sparq_engine::json::to_sparql_json(&res))
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
}
