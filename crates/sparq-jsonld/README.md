# sparq-jsonld

A native, **dependency-free** W3C **[JSON-LD 1.1]** document pipeline for sparq —
Context Processing, Expansion, Flattening, Compaction, Framing, and RDF
(de)serialization operating on JSON trees, not on a lossy RDF projection.

[JSON-LD 1.1]: https://www.w3.org/TR/json-ld11/

> Model: Opus 4.8 (Fable unavailable — flag for re-review when Fable returns).
> Phase A scaffold of epic `sq-oy1f` (design record `research/jsonld-1.1-design.md`).

## 🚀 Quickstart

```rust
use sparq_jsonld::{Json, JsonLdError, JsonLdErrorCode, JsonLdOptions};
use sparq_jsonld::{DocumentLoader, NoopLoader};

// The dependency-free JSON value model shared by the whole pipeline.
let mut node = Json::obj();
node.set("@id", Json::Str("http://example.org/a".into()));
let mut out = String::new();
node.write(&mut out);
assert_eq!(out, r#"{"@id":"http://example.org/a"}"#);

// Processing options carry the spec defaults.
let opts = JsonLdOptions::default();
assert!(opts.compact_arrays);

// The DEFAULT loader is fail-closed: no ambient network.
let err: JsonLdError = NoopLoader.load_document("https://ex/ctx").unwrap_err();
assert_eq!(err.code(), JsonLdErrorCode::LoadingDocumentFailed);
```

## ✨ Features

- **`Json` AST** — a minimal, insertion-order-preserving JSON value type (no
  `serde_json`). Moved here from `sparq-engine`'s JSON-LD writer so the whole
  pipeline shares one value type; the engine re-exports it (public API preserved).
- **Error registry** — `JsonLdErrorCode` is the full W3C JSON-LD 1.1 error-code
  set as a closed enum whose `as_str()` is the **exact** spec string, so the
  conformance suite's `expectErrorCode` negative tests become assertable.
- **Processing options** — `JsonLdOptions` models `base`, `processingMode`,
  `expandContext`, `rdfDirection`, `compactArrays`, `ordered`, the framing flags,
  and more, once, with the specification's defaults.
- **Deny-by-default loading** — `DocumentLoader` dereferences remote `@context` /
  `@import` / documents **only** through an explicit loader. The default
  `NoopLoader` refuses every load (`loading document failed`); `FsLoader` maps URL
  prefixes to trusted local fixtures. The network `HttpLoader` (with an SSRF
  allowlist) is a later, opt-in feature.

### Opt-in by construction

Nothing in sparq's default build or the wasm artifact depends on this crate —
`sparq-core` and `sparq-engine` stay lean. `sparq-engine` pulls it in only behind
its off-by-default `serialize-rdf` feature; the crate has **zero mandatory
dependencies**, is `#![forbid(unsafe_code)]`, and adds no default dependency
anywhere.

### Build-out status

The `Json` AST, error registry, options, and loader trait ship today, alongside the
**Context Processing** foundation (`context`): the `ActiveContext`, Create Term
Definition, and IRI Expansion — the hinge every other algorithm builds on — with
RFC 3986 reference resolution and deny-by-default remote-context loading. The
remaining modules (`expand`, `flatten`, `compact`, `frame`, `from_rdf`, `to_rdf`,
`api`) are documented stubs, filled by dependency-ordered follow-on beads. The
compaction-side companions of context processing (inverse context, IRI compaction)
are also deferred to a follow-on bead. The crate is `publish = false` until the
pipeline is real.

## 📚 Learn more

- Design record: `research/jsonld-1.1-design.md` (epic `sq-oy1f`).
- JSON-LD 1.1 API: <https://www.w3.org/TR/json-ld11-api/>
- JSON-LD 1.1 Framing: <https://www.w3.org/TR/json-ld11-framing/>
- Data-format serialization surface: `skills/data-formats/SKILL.md`.

## License

MIT © the sparq authors.
