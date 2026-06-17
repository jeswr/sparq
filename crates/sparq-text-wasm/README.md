# sparq-text-wasm

**The tier-b "W-text" WebAssembly bundle** ([OPUS-4.8] sq-jbe6) for
[`sparq-text`](../sparq-text/README.md) — an owned **BM25 full-text index** over RDF
literals plus the `text:` magic predicates, **live in the browser tab**.

This is a SEPARATE, lazy-loaded bundle from the lean [`sparq-wasm`](../sparq-wasm/README.md)
triplestore bundle. The lean bundle deliberately carries **no** text-search code; the
showcase site's `/surface/full-text` page loads this bundle on demand (`next/dynamic`,
client-only) so the landing page stays light. It mirrors the per-bundle-crate pattern of
`sparq-wasm` / [`sparq-reason-wasm`](../sparq-reason-wasm/README.md).

## What it exposes

A single `TextSearch` with stateless one-shot entry points — each parses the document,
builds the index over it, runs the request, and returns a string (the JS side never holds
a long-lived index handle):

```js
import init, { TextSearch } from "./sparq_text_wasm.js";
await init();

const data = `
  <http://ex/a> <http://ex/comment> "The quick brown fox" .
  <http://ex/b> <http://ex/comment> "A lazy dog" .`;

// text:matches (AND of tokens) + text:score, BM25-ranked, as SPARQL-results JSON:
const json = TextSearch.query(data, "ntriples",
  `PREFIX text: <http://sparq.dev/text#>
   SELECT ?s ?score WHERE {
     ?s <http://ex/comment> ?lit .
     ?lit text:matches "quick fox" ; text:score ?score .
   } ORDER BY DESC(?score)`);
// => application/sparql-results+json binding only <http://ex/a> (it has both tokens).

// Index footprint (the bundle's index-build-memory surface), as JSON:
const { docs, tokens, heapBytes, hasPositions } =
  JSON.parse(TextSearch.indexStats(data, "ntriples"));
```

- `format` is one of `"turtle"`, `"ntriples"`, `"nquads"`, `"trig"` (named graphs are
  folded into the default graph, so the index covers every string literal in the document).
- The index is **always built with positions**, so all of the `text:` vocabulary
  (`http://sparq.dev/text#`) works:
  - `?lit text:matches "q"` — AND of tokens (`foo*` is a prefix token);
  - `?lit text:matchesAny "q"` — OR of tokens;
  - `?lit text:phrase "foo bar"` — tokens adjacent and in order;
  - `?lit text:near "foo bar"` (+ optional `?lit text:slop N`) — proximity-ranked;
  - `?lit text:score ?s` — the relevance score of a scored match.
- A query with **no** `text:` patterns runs unchanged — the bundle is a superset of plain
  SPARQL querying.
- Output for `query` is canonical SPARQL 1.1 JSON, serialised through the engine's tested
  `to_sparql_json` path; this bundle owns no result-serialisation code of its own.

The `text:` rewrite happens entirely inside `sparq-text` (it splices the index hits into the
query as inline `VALUES`); the SPARQL **engine** — planner, executor — is unaware of text
search, which is exactly why this bundle can reuse the same engine the lean bundle ships.

## Building the bundle

```sh
wasm-pack build crates/sparq-text-wasm --target web --release
```

The build is single-threaded (no rayon) and pure-Rust: the tokenizer's
`unicode-segmentation` dependency (UAX #29 word segmentation) compiles to
`wasm32-unknown-unknown` with zero transitive deps. The **index-build memory** is the
consideration the design flags — the inverted index plus the opt-in positional postings are
held fully in memory — which is why this bundle is lazy-loaded on its page only, runs over a
small demo corpus, and exposes `indexStats` so the page can show the footprint.

### Measuring the bundle size

We deliberately do **not** quote a hard-coded byte/MB figure here — bundle size drifts with
the toolchain (`rustc`, `wasm-bindgen`, `wasm-opt`) and dependency versions, so any number in
this file would silently rot. To get a reproducible figure for your toolchain, build and
measure the emitted `.wasm` directly:

```sh
wasm-pack build crates/sparq-text-wasm --target web --release
f=crates/sparq-text-wasm/pkg/sparq_text_wasm_bg.wasm
echo "pre-gzip: $(stat -c%s "$f") bytes   gzip -9: $(gzip -9 -c "$f" | wc -c) bytes (this is the over-the-wire transfer size)"
```

The gzip figure — not the pre-gzip one — is what end users actually download, since the
showcase site serves the `.wasm` gzip-compressed.

## Status / what remains

This crate delivers the wasm-compatibility changes, the `TextSearch` entry points
(`query` + `indexStats`), and a headless `wasm-pack test --node` smoke suite that drives the
real `#[wasm_bindgen]` exports in a genuine wasm runtime. The npm wrapper packaging and the
GitHub Pages deploy wiring for this bundle are tracked separately (the full-text page bead
sq-xoxu and the Pages workflow); see the PR description.

## License

[MIT](../../LICENSE).
