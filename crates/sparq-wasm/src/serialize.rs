//! [OPUS-4.8] sq-fe1s / sq-ixc3.5: the `Store::serialize(format, …)` RDF-writer binding.
//!
//! Exposes `sparq-engine`'s pretty Turtle / TriG writers
//! ([`sparq_engine::serialize`] — `graph_to_turtle_pretty_with` /
//! `graph_to_trig_pretty_with`, byte-shape-matching the site's `prettyTurtle`
//! reshaper, #805) AND its native JSON-LD 1.1 writers
//! (`graph_to_jsonld_with` / `graph_to_jsonld_pretty_with`, the
//! expanded / flattened / compacted forms, sq-ixc3.3) to JS/WASM consumers, so a
//! browser `Store` can produce a human-readable Turtle / TriG / JSON-LD document of
//! its contents — not just the flat N-Triples that [`Store::query_quads`] returns
//! for a CONSTRUCT/DESCRIBE.
//!
//! This module reimplements nothing: it calls straight through to the engine
//! serialiser the native CLI / HTTP surface uses, so a `Store.serialize("turtle",
//! true, …)` is byte-identical to `sparq_engine::serialize::graph_to_turtle_pretty_with`
//! over the same graph, and `Store.serialize("jsonld-expanded", true, …)` is
//! byte-identical to `graph_to_jsonld_pretty_with(.., JsonLdForm::Expanded, ..)`
//! (asserted by the native `src/serialize.rs` parity tests and the `wasm32` headless
//! `tests/web.rs` tests). Routing JSON-LD through this SAME method lets the site
//! consume the engine writer instead of hand-formatting JSON-LD in TypeScript —
//! mirroring the Turtle engine-first path.
//!
//! It compiles ONLY under the opt-in `serialize-rdf` feature (which forwards to
//! `sparq-engine/serialize-rdf`, where the whole writer matrix — Turtle/TriG/JSON-LD —
//! lives), so the default browser bundle carries zero serializer code — the lean
//! `wasm_bundle_bytes` baseline is unchanged. The JSON-LD *serialise-out* path needs
//! NO extra feature: the writers are pure Rust under `serialize-rdf` (the opt-in
//! `jsonld` feature is INGEST-only — it pulls the `oxjsonld` *parser*, unrelated to
//! these writers). The site REPL bundle (`js` `build:wasm`, built
//! `--features shacl,jsonld,serialize-rdf`) turns `serialize-rdf` on.
//!
//! ## Surface
//!
//! `serialize(format, pretty, indent, abbreviate)`:
//!
//! * `format` (case-insensitive) — one of:
//!   * `"turtle"` (aliases `"ttl"`, `"text/turtle"`) — the **default graph only**.
//!   * `"trig"` (alias `"application/trig"`) — the whole dataset (default graph at
//!     top level, named graphs as `GRAPH <g> { … }` blocks).
//!   * `"jsonld"` / `"json-ld"` / `"application/ld+json"` — JSON-LD 1.1, **expanded**
//!     form by default (the safest interop target). To pick a form explicitly use
//!     `"jsonld-expanded"`, `"jsonld-flattened"`, or `"jsonld-compacted"`
//!     (`json-ld-…` accepted too). JSON-LD always emits the whole dataset.
//! * `pretty` — `true` for the indented pretty shape (Turtle/TriG: the
//!   blank-line-separated, sorted `prettyTurtle` shape; JSON-LD: the structurally
//!   re-indented document), `false` for the compact single-pass / minified writer.
//! * `indent` — the indent unit for the pretty writers (e.g. `"  "`, `"\t"`).
//!   `undefined` / `null` uses the two-space default. Ignored when `pretty` is `false`.
//! * `abbreviate` — Turtle/TriG only: `true` to emit an alphabetical `@prefix` header
//!   and compact IRIs to `prefix:local`; `false` to keep every IRI in full `<…>` form.
//!   **Ignored for JSON-LD** — IRI abbreviation there is selected by the
//!   `jsonld-compacted` form (which carries a prefix `@context`), not by this flag.
//!
//! An unknown `format` is rejected with a clear `JsError` (never a silent empty
//! string), so a typo surfaces rather than producing a wrong document.

use sparq_engine::serialize::{
    default_prefixes, graph_to_jsonld_pretty_with, graph_to_jsonld_with, graph_to_trig,
    graph_to_trig_pretty_with, graph_to_turtle_pretty_with, graph_to_turtle_with, JsonLdForm,
    JsonLdPrettyOptions, PrettyOptions,
};
use wasm_bindgen::prelude::*;

use crate::Store;

/// The output format the wasm `Store::serialize` understands. Kept private — the JS
/// surface takes a `&str` (the same convention as `Store::load`'s `format`).
enum SerFormat {
    Turtle,
    Trig,
    /// JSON-LD 1.1 in the selected document form (expanded / flattened / compacted).
    JsonLd(JsonLdForm),
}

impl SerFormat {
    /// Parses the JS-supplied `format` string (case-insensitive). `None` for an
    /// unrecognised value, which the caller maps to a `JsError`. A bare `"jsonld"`
    /// (or its media-type / `json-ld` aliases) selects the **expanded** form — the
    /// safest interop default; the form is otherwise chosen by the `-expanded` /
    /// `-flattened` / `-compacted` suffix.
    fn parse(format: &str) -> Option<SerFormat> {
        match format.trim().to_ascii_lowercase().as_str() {
            "turtle" | "ttl" | "text/turtle" => Some(SerFormat::Turtle),
            "trig" | "application/trig" => Some(SerFormat::Trig),
            "jsonld" | "json-ld" | "application/ld+json" => {
                Some(SerFormat::JsonLd(JsonLdForm::Expanded))
            }
            "jsonld-expanded" | "json-ld-expanded" => Some(SerFormat::JsonLd(JsonLdForm::Expanded)),
            "jsonld-flattened" | "json-ld-flattened" => {
                Some(SerFormat::JsonLd(JsonLdForm::Flattened))
            }
            "jsonld-compacted" | "json-ld-compacted" => {
                Some(SerFormat::JsonLd(JsonLdForm::Compacted))
            }
            _ => None,
        }
    }
}

#[wasm_bindgen]
impl Store {
    /// [OPUS-4.8] sq-fe1s / sq-ixc3.5: serialises the store's contents to a **Turtle**,
    /// **TriG**, or **JSON-LD** document string.
    ///
    /// `format` (case-insensitive) is one of:
    /// * `"turtle"` (aliases `"ttl"`, `"text/turtle"`) — the default graph only.
    /// * `"trig"` (alias `"application/trig"`) — the whole dataset: default graph at
    ///   top level, named graphs as `GRAPH <g> { … }` blocks.
    /// * `"jsonld"` / `"json-ld"` / `"application/ld+json"` — JSON-LD 1.1, **expanded**
    ///   form by default; `"jsonld-expanded"` / `"jsonld-flattened"` /
    ///   `"jsonld-compacted"` pick the form explicitly (`json-ld-…` accepted too).
    ///   JSON-LD always emits the whole dataset.
    ///
    /// When `pretty` is `true` the output is indented: Turtle/TriG use the
    /// blank-line-separated, **sorted** (emission-order-independent) `prettyTurtle`
    /// shape; JSON-LD uses the structurally re-indented document. The `indent` arg is
    /// the indent unit (`undefined`/`null` ⇒ two spaces). When `pretty` is `false` the
    /// compact / minified writer is used and `indent` is ignored.
    ///
    /// `abbreviate` applies to **Turtle/TriG only**: `true` emits a sorted `@prefix`
    /// header and compacts IRIs to `prefix:local`; `false` keeps every IRI in full
    /// `<…>` form. It is **ignored for JSON-LD** — IRI abbreviation there is selected by
    /// the `jsonld-compacted` form (which carries a prefix `@context`), not this flag.
    /// The well-known prefixes (`rdf`, `rdfs`, `xsd`, `owl`, `schema`, `foaf`, `dc`,
    /// `skos`, `sh`) are assumed for compaction.
    ///
    /// This is the document-export counterpart to [`query_quads`](Self::query_quads),
    /// which returns a CONSTRUCT/DESCRIBE *result graph* as flat N-Triples: `serialize`
    /// writes the **store itself** in a readable syntax. Errors only if `format` is
    /// not one of the recognised values (a `JsError`); serialisation itself is
    /// infallible. Available only when the crate is built with the OPT-IN
    /// `serialize-rdf` feature — the site REPL bundle enables it; the lean default
    /// bundle does not. (JSON-LD *serialise-out* needs no extra feature: the writers
    /// live under `serialize-rdf`; the `jsonld` feature is INGEST-only.)
    pub fn serialize(
        &self,
        format: &str,
        pretty: bool,
        indent: Option<String>,
        abbreviate: bool,
    ) -> Result<String, JsError> {
        let fmt = SerFormat::parse(format).ok_or_else(|| {
            JsError::new(&format!(
                "unsupported serialize format {:?} (expected \"turtle\", \"trig\", or \
                 \"jsonld\" / \"jsonld-expanded\" / \"jsonld-flattened\" / \"jsonld-compacted\")",
                format
            ))
        })?;
        let prefixes = default_prefixes();
        // JSON-LD selects its form via the `format` string (not `abbreviate`); the
        // `indent`/`pretty` args carry through identically to the Turtle/TriG path.
        if let SerFormat::JsonLd(form) = fmt {
            let out = if pretty {
                let opts = JsonLdPrettyOptions {
                    indent: indent.unwrap_or_else(|| "  ".to_string()),
                };
                graph_to_jsonld_pretty_with(&self.graph, form, &prefixes, &opts)
            } else {
                graph_to_jsonld_with(&self.graph, form, &prefixes)
            };
            return Ok(out);
        }
        let out = if pretty {
            let opts = PrettyOptions {
                indent: indent.unwrap_or_else(|| "  ".to_string()),
                abbreviate,
            };
            match fmt {
                SerFormat::Turtle => graph_to_turtle_pretty_with(&self.graph, &prefixes, &opts),
                SerFormat::Trig => graph_to_trig_pretty_with(&self.graph, &prefixes, &opts),
                SerFormat::JsonLd(_) => unreachable!("JSON-LD handled above"),
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
                SerFormat::JsonLd(_) => unreachable!("JSON-LD handled above"),
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

    // ---- [OPUS-4.8] sq-ixc3.5: JSON-LD serialise-out parity with the engine writer ----

    use sparq_engine::serialize::{graph_to_jsonld_pretty_with, graph_to_jsonld_with, JsonLdForm};

    /// PRETTY JSON-LD through the wasm `Store::serialize` is byte-identical to the engine
    /// serialiser it delegates to (`graph_to_jsonld_pretty_with`) over the same graph,
    /// form, and indent — the load-bearing parity invariant for the new JSON-LD path.
    #[test]
    fn serialize_jsonld_pretty_matches_engine() {
        let store = Store::load(DATA, "turtle").unwrap();
        let g = Graph::load_str(DATA, "turtle").unwrap();
        for (fmt, form) in [
            ("jsonld", JsonLdForm::Expanded), // bare "jsonld" ⇒ expanded default
            ("jsonld-expanded", JsonLdForm::Expanded),
            ("jsonld-flattened", JsonLdForm::Flattened),
            ("jsonld-compacted", JsonLdForm::Compacted),
        ] {
            let got = store.serialize(fmt, true, Some("  ".to_string()), true).unwrap();
            let opts = JsonLdPrettyOptions {
                indent: "  ".to_string(),
            };
            let want = graph_to_jsonld_pretty_with(&g, form, &default_prefixes(), &opts);
            assert_eq!(got, want, "wasm pretty JSON-LD ({fmt}) must equal the engine output");
            // The pretty document is multi-line JSON (the indenter inserted newlines).
            assert!(got.contains('\n'), "pretty JSON-LD is indented: {got}");
        }
    }

    /// Minified (non-pretty) JSON-LD matches the engine `graph_to_jsonld_with`, and the
    /// pretty/minified pair re-indent to the same document (the pretty pass is
    /// whitespace-only). The `abbreviate` flag is IGNORED for JSON-LD (form selects
    /// compaction), so passing it either way yields the same bytes.
    #[test]
    fn serialize_jsonld_minified_matches_engine_and_ignores_abbreviate() {
        let store = Store::load(DATA, "turtle").unwrap();
        let g = Graph::load_str(DATA, "turtle").unwrap();
        let got = store.serialize("jsonld-compacted", false, None, true).unwrap();
        let want = graph_to_jsonld_with(&g, JsonLdForm::Compacted, &default_prefixes());
        assert_eq!(got, want, "wasm minified JSON-LD must equal the engine output");
        // `abbreviate` is a no-op for JSON-LD: true vs false produces identical bytes.
        let with_abbrev_off = store.serialize("jsonld-compacted", false, None, false).unwrap();
        assert_eq!(got, with_abbrev_off, "abbreviate is ignored for JSON-LD");
        // Compacted carries a prefix `@context`; expanded does not.
        assert!(got.contains("@context"), "compacted form has @context: {got}");
        let expanded = store.serialize("jsonld-expanded", false, None, true).unwrap();
        assert!(!expanded.contains("@context"), "expanded form has no @context: {expanded}");
    }

    /// A custom indent flows through to the JSON-LD pretty writer (a four-space indent
    /// produces a different — but engine-matching — document than the two-space default),
    /// and `None` defaults to two spaces.
    #[test]
    fn serialize_jsonld_pretty_custom_indent() {
        let store = Store::load(DATA, "turtle").unwrap();
        let g = Graph::load_str(DATA, "turtle").unwrap();
        let two = store.serialize("jsonld-expanded", true, Some("  ".to_string()), true).unwrap();
        let four = store.serialize("jsonld-expanded", true, Some("    ".to_string()), true).unwrap();
        assert_ne!(two, four, "indent option must affect the JSON-LD output");
        let opts = JsonLdPrettyOptions {
            indent: "    ".to_string(),
        };
        assert_eq!(
            four,
            graph_to_jsonld_pretty_with(&g, JsonLdForm::Expanded, &default_prefixes(), &opts),
            "custom-indent JSON-LD must equal the engine output with the same opts"
        );
        let dflt = store.serialize("jsonld-expanded", true, None, true).unwrap();
        assert_eq!(dflt, two, "None indent uses the two-space default for JSON-LD");
    }

    /// JSON-LD round-trips: a serialised document re-parses to the same triple set. Guarded
    /// by `jsonld` (the INGEST parser) so it only runs when JSON-LD parsing is available;
    /// the parity tests above are the primary, parser-independent invariant.
    #[cfg(feature = "jsonld")]
    #[test]
    fn serialize_jsonld_round_trips() {
        let store = Store::load(DATA, "turtle").unwrap();
        for fmt in ["jsonld-expanded", "jsonld-flattened", "jsonld-compacted"] {
            let doc = store.serialize(fmt, true, None, true).unwrap();
            let reparsed = Graph::load_str(&doc, "jsonld").unwrap();
            assert_eq!(
                reparsed.len(),
                store.size(),
                "JSON-LD ({fmt}) serialise->parse round-trip preserves triple count"
            );
        }
    }
}
