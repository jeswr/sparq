//! [OPUS-4.8] sq-fe1s: the `Store::serialize(format, …)` RDF-writer binding.
//!
//! Exposes `sparq-engine`'s pretty Turtle / TriG writers
//! ([`sparq_engine::serialize`] — `graph_to_turtle_pretty_with` /
//! `graph_to_trig_pretty_with`, byte-shape-matching the site's `prettyTurtle`
//! reshaper, #805) to JS/WASM consumers, so a browser `Store` can produce a
//! human-readable Turtle / TriG document of its contents — not just the flat
//! N-Triples that [`Store::query_quads`] returns for a CONSTRUCT/DESCRIBE.
//!
//! This module reimplements nothing: it calls straight through to the engine
//! serialiser the native CLI / HTTP surface uses, so a `Store.serialize("turtle",
//! true, …)` is byte-identical to `sparq_engine::serialize::graph_to_turtle_pretty_with`
//! over the same graph (asserted by the native `tests/exported_api.rs` parity test and
//! the `wasm32` headless `tests/web.rs` test).
//!
//! It compiles ONLY under the opt-in `serialize-rdf` feature (which forwards to
//! `sparq-engine/serialize-rdf`), so the default browser bundle carries zero
//! serializer code — the lean `wasm_bundle_bytes` baseline is unchanged. The site
//! REPL bundle (`js` `build:wasm`, built `--features shacl,jsonld,serialize-rdf`)
//! turns it on.
//!
//! ## Surface
//!
//! `serialize(format, pretty, indent, abbreviate)`:
//!
//! * `format` ∈ `"turtle"` | `"trig"` (case-insensitive; `"ttl"` / `"turtle"`
//!   and `"trig"` accepted). Turtle emits the **default graph only**; TriG emits
//!   the whole dataset (default graph at top level, named graphs as
//!   `GRAPH <g> { … }` blocks).
//! * `pretty` — `true` for the blank-line-separated, sorted, indented pretty shape
//!   (the site `prettyTurtle` shape); `false` for the compact single-pass writer.
//! * `indent` — the continuation-line indent unit for the pretty writers (e.g.
//!   `"  "`, `"\t"`). `undefined` / `null` uses the two-space default. Ignored when
//!   `pretty` is `false`.
//! * `abbreviate` — `true` to emit an alphabetical `@prefix` header and compact IRIs
//!   to `prefix:local`; `false` to keep every IRI in full `<…>` form with no header.
//!
//! An unknown `format` is rejected with a clear `JsError` (never a silent empty
//! string), so a typo surfaces rather than producing a wrong document.

use sparq_engine::serialize::{
    default_prefixes, graph_to_trig_pretty_with, graph_to_turtle_pretty_with, graph_to_trig,
    graph_to_turtle_with, PrettyOptions,
};
use wasm_bindgen::prelude::*;

use crate::Store;

/// The output format the wasm `Store::serialize` understands. Kept private — the JS
/// surface takes a `&str` (the same convention as `Store::load`'s `format`).
enum SerFormat {
    Turtle,
    Trig,
}

impl SerFormat {
    /// Parses the JS-supplied `format` string (case-insensitive). `None` for an
    /// unrecognised value, which the caller maps to a `JsError`.
    fn parse(format: &str) -> Option<SerFormat> {
        match format.trim().to_ascii_lowercase().as_str() {
            "turtle" | "ttl" | "text/turtle" => Some(SerFormat::Turtle),
            "trig" | "application/trig" => Some(SerFormat::Trig),
            _ => None,
        }
    }
}

#[wasm_bindgen]
impl Store {
    /// [OPUS-4.8] sq-fe1s: serialises the store's contents to a **Turtle** or **TriG**
    /// document string.
    ///
    /// `format` is `"turtle"` (default graph only) or `"trig"` (the whole dataset:
    /// default graph at top level, named graphs as `GRAPH <g> { … }` blocks);
    /// `"ttl"` and the media types `"text/turtle"` / `"application/trig"` are also
    /// accepted (case-insensitive).
    ///
    /// When `pretty` is `true` the output is the blank-line-separated, **sorted**
    /// (emission-order-independent), indented pretty shape — the same byte shape the
    /// site's `prettyTurtle` reshaper produces — driven by `indent` (the
    /// continuation-line indent unit; `undefined`/`null` ⇒ two spaces) and
    /// `abbreviate` (emit a sorted `@prefix` header + compact IRIs to `prefix:local`,
    /// vs keep every IRI in full `<…>` form). When `pretty` is `false` the compact
    /// single-pass writer is used and `indent` is ignored (`abbreviate` still chooses
    /// CURIE compaction). The well-known prefixes (`rdf`, `rdfs`, `xsd`, `owl`,
    /// `schema`, `foaf`, `dc`, `skos`, `sh`) are assumed for compaction.
    ///
    /// This is the document-export counterpart to [`query_quads`](Self::query_quads),
    /// which returns a CONSTRUCT/DESCRIBE *result graph* as flat N-Triples: `serialize`
    /// writes the **store itself** in a readable syntax. Errors only if `format` is
    /// not one of the recognised values (a `JsError`); serialisation itself is
    /// infallible. Available only when the crate is built with the OPT-IN
    /// `serialize-rdf` feature — the site REPL bundle enables it; the lean default
    /// bundle does not.
    pub fn serialize(
        &self,
        format: &str,
        pretty: bool,
        indent: Option<String>,
        abbreviate: bool,
    ) -> Result<String, JsError> {
        let fmt = SerFormat::parse(format).ok_or_else(|| {
            JsError::new(&format!(
                "unsupported serialize format {:?} (expected \"turtle\" or \"trig\")",
                format
            ))
        })?;
        let prefixes = default_prefixes();
        let out = if pretty {
            let opts = PrettyOptions {
                indent: indent.unwrap_or_else(|| "  ".to_string()),
                abbreviate,
            };
            match fmt {
                SerFormat::Turtle => graph_to_turtle_pretty_with(&self.graph, &prefixes, &opts),
                SerFormat::Trig => graph_to_trig_pretty_with(&self.graph, &prefixes, &opts),
            }
        } else {
            match fmt {
                // The compact writers always compact against the supplied prefixes; the
                // `abbreviate=false` "full IRIs" choice is a PRETTY-only option, so an
                // empty prefix map is passed to suppress CURIE compaction in that case.
                SerFormat::Turtle if abbreviate => graph_to_turtle_with(&self.graph, &prefixes),
                SerFormat::Turtle => {
                    graph_to_turtle_with(&self.graph, &sparq_engine::serialize::Prefixes::new())
                }
                // The non-pretty TriG writer (`graph_to_trig`) always uses the default
                // prefixes; full-IRI TriG is only available through the pretty path.
                SerFormat::Trig => graph_to_trig(&self.graph),
            }
        };
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sparq_core::Graph;

    const DATA: &str = r#"@prefix ex: <http://ex/> .
        ex:alice ex:name "Alice" ; ex:age 30 ; ex:knows ex:bob .
        ex:bob ex:name "Bob"@en ; ex:age 25 ."#;

    // The serialiser runs natively (the `Ok` arm never touches `JsError::new`), so the
    // parity contract is tested here without a wasm runtime — mirroring the native-test
    // convention of the other wasm bindings. The negative (`Err`) arm (an unknown format
    // string) stays covered by the wasm32 `tests/web.rs::serialize_unknown_format_is_err`.

    /// Pretty Turtle through the wasm `Store::serialize` is byte-identical to the engine
    /// serialiser it delegates to (`graph_to_turtle_pretty_with`) over the same graph and
    /// options — the load-bearing parity invariant (the binding adds no reformatting).
    #[test]
    fn serialize_turtle_pretty_matches_engine() {
        let store = Store::load(DATA, "turtle").unwrap();
        let got = store
            .serialize("turtle", true, Some("  ".to_string()), true)
            .unwrap();
        let g = Graph::load_str(DATA, "turtle").unwrap();
        let want = graph_to_turtle_pretty_with(&g, &default_prefixes(), &PrettyOptions::default());
        assert_eq!(got, want, "wasm pretty Turtle must equal the engine output");
        // Sanity on the shape: with abbreviation on, the well-known `xsd:` prefix (used by
        // the integer ages) is declared in a header and the integer datatype is compacted.
        // (`ex:` is NOT a well-known prefix, so those IRIs stay in full `<…>` form — the
        // byte-equality above is the real parity check; this just pins the shape.)
        assert!(
            got.contains("@prefix xsd:"),
            "well-known prefix header present: {got}"
        );
        assert!(got.contains("xsd:integer"), "datatype compacted: {got}");
    }

    /// A custom indent flows through to the engine writer (the `{indent}` option is wired,
    /// not dropped): a four-space indent produces a different document than the default.
    #[test]
    fn serialize_turtle_pretty_custom_indent() {
        let store = Store::load(DATA, "turtle").unwrap();
        let two = store
            .serialize("turtle", true, Some("  ".to_string()), true)
            .unwrap();
        let four = store
            .serialize("turtle", true, Some("    ".to_string()), true)
            .unwrap();
        assert_ne!(two, four, "indent option must affect the output");
        let g = Graph::load_str(DATA, "turtle").unwrap();
        let opts = PrettyOptions {
            indent: "    ".to_string(),
            abbreviate: true,
        };
        assert_eq!(
            four,
            graph_to_turtle_pretty_with(&g, &default_prefixes(), &opts),
            "custom-indent output must equal the engine output with the same opts"
        );
        // `indent` defaulting: passing `None` matches the two-space default explicitly.
        let dflt = store.serialize("turtle", true, None, true).unwrap();
        assert_eq!(dflt, two, "None indent uses the two-space default");
    }

    /// `abbreviate=false` keeps every IRI in full `<…>` form with no `@prefix` header.
    #[test]
    fn serialize_turtle_pretty_no_abbreviate() {
        let store = Store::load(DATA, "turtle").unwrap();
        let full = store.serialize("turtle", true, None, false).unwrap();
        assert!(
            !full.contains("@prefix"),
            "no prefix header when abbreviate=false: {full}"
        );
        assert!(
            full.contains("<http://ex/alice>"),
            "IRIs stay in full form: {full}"
        );
        let g = Graph::load_str(DATA, "turtle").unwrap();
        let opts = PrettyOptions {
            indent: "  ".to_string(),
            abbreviate: false,
        };
        assert_eq!(
            full,
            graph_to_turtle_pretty_with(&g, &default_prefixes(), &opts)
        );
    }

    /// TriG pretty over a dataset emits a `GRAPH <g> { … }` block for the named graph and
    /// matches the engine `graph_to_trig_pretty_with`.
    #[test]
    fn serialize_trig_pretty_named_graph() {
        let nq = "<http://ex/s> <http://ex/p> <http://ex/o> <http://ex/g1> .\n";
        let store = Store::load_dataset(nq, "nquads").unwrap();
        let got = store.serialize("trig", true, None, true).unwrap();
        assert!(got.contains("GRAPH"), "named graph as a GRAPH block: {got}");
        let g = Graph::load_dataset(nq, "nquads").unwrap();
        let want = graph_to_trig_pretty_with(&g, &default_prefixes(), &PrettyOptions::default());
        assert_eq!(got, want, "wasm pretty TriG must equal the engine output");
    }

    /// The compact (non-pretty) Turtle path matches the engine compact writer.
    #[test]
    fn serialize_turtle_compact_matches_engine() {
        let store = Store::load(DATA, "turtle").unwrap();
        let got = store.serialize("turtle", false, None, true).unwrap();
        let g = Graph::load_str(DATA, "turtle").unwrap();
        let want = graph_to_turtle_with(&g, &default_prefixes());
        assert_eq!(got, want, "wasm compact Turtle must equal the engine output");
    }

    /// Format parsing is case-insensitive and accepts the media types / `ttl` alias.
    #[test]
    fn serialize_format_aliases() {
        let store = Store::load(DATA, "turtle").unwrap();
        let a = store.serialize("turtle", true, None, true).unwrap();
        for alias in ["TURTLE", "Ttl", "text/turtle"] {
            assert_eq!(
                store.serialize(alias, true, None, true).unwrap(),
                a,
                "alias {alias} must serialise as turtle"
            );
        }
        // Round-trip: serialising then re-parsing yields the same triple count.
        let reparsed = Graph::load_str(&a, "turtle").unwrap();
        assert_eq!(reparsed.len(), store.size(), "serialise->parse round-trip");
    }
}
