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

use oxrdf::Term;
use sparq_core::Graph;
use sparq_engine::QueryResult;
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
        Ok(results_to_json(&res))
    }
}

/// Serialises a query result to the SPARQL 1.1 JSON results format.
fn results_to_json(r: &QueryResult) -> String {
    let mut s = String::with_capacity(64 + r.rows.len() * 32);
    s.push_str("{\"head\":{\"vars\":[");
    for (i, v) in r.vars.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('"');
        escape_into(&mut s, v.as_str());
        s.push('"');
    }
    s.push_str("]},\"results\":{\"bindings\":[");
    for (ri, row) in r.rows.iter().enumerate() {
        if ri > 0 {
            s.push(',');
        }
        s.push('{');
        let mut first = true;
        for (vi, cell) in row.iter().enumerate() {
            if let Some(term) = cell {
                if !first {
                    s.push(',');
                }
                first = false;
                s.push('"');
                escape_into(&mut s, r.vars[vi].as_str());
                s.push_str("\":");
                term_to_json(&mut s, term);
            }
        }
        s.push('}');
    }
    s.push_str("]}}");
    s
}

fn term_to_json(s: &mut String, t: &Term) {
    match t {
        Term::NamedNode(n) => {
            s.push_str("{\"type\":\"uri\",\"value\":\"");
            escape_into(s, n.as_str());
            s.push_str("\"}");
        }
        Term::BlankNode(b) => {
            s.push_str("{\"type\":\"bnode\",\"value\":\"");
            escape_into(s, b.as_str());
            s.push_str("\"}");
        }
        Term::Literal(l) => {
            s.push_str("{\"type\":\"literal\",\"value\":\"");
            escape_into(s, l.value());
            s.push('"');
            if let Some(lang) = l.language() {
                s.push_str(",\"xml:lang\":\"");
                escape_into(s, lang);
                s.push('"');
            } else {
                let dt = l.datatype();
                if dt.as_str() != "http://www.w3.org/2001/XMLSchema#string" {
                    s.push_str(",\"datatype\":\"");
                    escape_into(s, dt.as_str());
                    s.push('"');
                }
            }
            s.push('}');
        }
        // RDF-star quoted triple: fall back to its N-Triples form as a literal.
        other => {
            s.push_str("{\"type\":\"literal\",\"value\":\"");
            escape_into(s, &other.to_string());
            s.push_str("\"}");
        }
    }
}

/// Appends `s` to `out` with JSON string escaping.
fn escape_into(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
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
        let json = results_to_json(&r);
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
    fn escaping() {
        let mut s = String::new();
        escape_into(&mut s, "a\"b\\c\nd");
        assert_eq!(s, "a\\\"b\\\\c\\nd");
    }

    #[test]
    fn uri_and_bnode() {
        let g = Graph::load_str(DATA, "turtle").unwrap();
        let r = sparq_engine::query(&g, "PREFIX ex: <http://ex/> SELECT ?o WHERE { ?s ex:knows ?o }").unwrap();
        let json = results_to_json(&r);
        assert!(json.contains("\"type\":\"uri\",\"value\":\"http://ex/bob\""));
    }
}
