# js — outstanding work

Tracked in beads (not here). Run `bd ready -l area:js` (or `area:sparq-wasm` /
`area:sparq-engine` for the engine-side gaps) or `bd list -l area:js`.
See AGENTS.md for the no-markdown-TODOs policy.

## Notes

Design rationale retained from the previous gap list (not task tracking).
Items marked **engine** need work in `sparq-engine`/`sparq-core` (owned by another
thread); **wasm** items are additive exports in `crates/sparq-wasm`.

### Engine API gaps (context)

- **ASK (engine)** — `sparq_engine::query*` rejects everything but SELECT. The
  JS layer rewrites `ASK` → `SELECT *` and tests `count > 0`, which computes the
  *full* count instead of stopping at the first solution. A native ASK (or a
  budgeted `exists()` that early-exits) would be strictly better.
- **CONSTRUCT / DESCRIBE (engine)** — ✅ DONE (T16): `sparq_engine::construct` /
  `describe` return `Vec<oxrdf::Triple>` (CONSTRUCT per spec — unbound/illegal
  template slots skipped, fresh bnodes per solution, set-deduplicated; DESCRIBE
  as concise bounded description), and `construct_ntriples*` serialises either
  form to N-Triples (valid Turtle). Remaining for JS: a **wasm** export (e.g.
  `Store.construct(sparql) -> string`) so `queryQuads()` can parse it — additive
  in `crates/sparq-wasm`.
- **Named graphs (engine/core)** — the store is triple-scoped; TriG/N-Quads
  graphs are folded into the default graph on load, and UPDATE rejects named
  graph targets. An RDF/JS DatasetCore-faithful `match(s, p, o, g)` needs a quad
  store.
- **Streaming/cursor results (wasm+engine)** — `Store.query` returns one JSON
  string, so large results are double-buffered (wasm string + JS parse). A
  paged/cursor API (e.g. `query_page(sparql, offset, limit)` or a callback per
  row) would let the JS layer expose a true RDF/JS `ResultStream` and bound
  memory. Until then `queryBindings()` returns a materialised array. *Engine
  half landed (T16):* `sparq_engine::query_json_chunks_with_budget` returns the
  JSON as an ordered chunk sequence (concatenation byte-identical); a wasm
  export over it is the remaining piece.
- **Incremental update (engine)** — `update()` rebuilds the whole immutable
  index (O(store) per batch). Fine for small/medium graphs; a delta-index or
  merge structure would make RDF/JS `Store.add/remove`-style mutation viable.
- **Term round-trip fidelity** — SPARQL JSON has no RDF 1.2 directional
  language-literal channel; if/when the engine supports `"x"@en--ltr`, the JSON
  serialiser and `termFromSparqlJson` need an agreed extension field.
