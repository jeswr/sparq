---
name: full-text-search
description: Full-text search over RDF string literals in sparq via the sparq-text crate: build a BM25 inverted index (TextIndex) and run text: magic predicates (text:matches / text:matchesAny / text:phrase / text:score) inside plain SPARQL. Use when an agent needs keyword/prefix/phrase search, BM25 relevance ranking, or autocomplete over literals in a sparq Graph.
---

# sparq full-text search (sparq-text)

`sparq-text` is an **opt-in, separate crate** that adds full-text search over the string literals of a sparq `Graph`. It gives you two surfaces: a low-level owned BM25 inverted index (`TextIndex`) keyed by dictionary term id, and `text:` **magic predicates** that you write inside ordinary SPARQL and that `query_text` rewrites into inline `VALUES` over the index's hits. The engine, planner, and wasm bundle carry zero text-search code — full-text support exists only when you depend on `sparq-text`.

## Quickstart

`Cargo.toml`:
```toml
[dependencies]
sparq-core = { path = "../sparq-core" }
sparq-text = { path = "../sparq-text" }   # default features: ["engine", "parallel"]
```

```rust
use sparq_core::Graph;
use sparq_text::{query_text, TextIndex};

let g = Graph::load_str(r#"
    <http://ex/post1> <http://ex/title> "The quick brown fox" .
    <http://ex/post2> <http://ex/title> "Fox hunting banned" .
"#, "ntriples").unwrap();

// 1. Library-level: build the BM25 index (one dict scan), search returns Hit { id, score }.
let index = TextIndex::build(&g);
for hit in index.search("quick fox") {          // AND of tokens, BM25 best-first
    let literal = g.dict.term(hit.id);           // hit.id IS the dict id of the literal
    println!("{literal} (bm25 {})", hit.score);
}

// 2. SPARQL-level: text: magic predicates, rewritten + executed by the engine.
let r = query_text(&g, r#"
    PREFIX text: <http://sparq.dev/text#>
    SELECT ?post ?s WHERE {
      ?post  <http://ex/title> ?title .
      ?title text:matches "fox" .
      ?title text:score   ?s .
    } ORDER BY DESC(?s)"#, &index).unwrap();
assert_eq!(r.rows.len(), 2);                     // r.rows: Vec<Vec<Option<oxrdf::Term>>>, r.vars
```

The dictionary term id of a string literal **is** its document id, so search results join back to subjects/predicates through the store's ordinary permutation indexes — that join is what the SPARQL surrounding the magic pattern does.

## Key APIs

Index (always available, `index` module — re-exported at crate root as `TextIndex`, `Hit`):
- `TextIndex::build(graph: &Graph) -> TextIndex` — cheap 8-B-per-posting index, **no positions** (rayon-sharded under `parallel`; result identical to serial).
- `TextIndex::build_with_positions(graph: &Graph) -> TextIndex` — also records token offsets, enabling `phrase`.
- `TextIndex::with_positions() -> TextIndex` — empty position-enabled index for the delta-fed case (positional counterpart of `TextIndex::default()`).
- `TextIndex::search(&self, query: &str) -> Vec<Hit>` — AND of tokens (`*`-suffix = prefix token), BM25-ranked best-first.
- `TextIndex::search_any(&self, query: &str) -> Vec<Hit>` — OR of tokens; scores sum over tokens present.
- `TextIndex::phrase(&self, query: &str) -> Vec<Id>` — adjacent-and-in-order tokens, ascending ids (unranked). **Panics** if the index has no positions.
- `TextIndex::apply_delta(&mut self, graph: &Graph, inserts: &[[Term;3]], deletes: &[[Term;3]])` — mirror a `Graph::apply_delta` batch (inserts of new string literals are indexed; deletes are a documented no-op).
- `TextIndex::has_positions(&self) -> bool`, `len()`, `is_empty()`, `token_count()`, `heap_bytes()`.
- `struct Hit { pub id: sparq_core::dict::Id, pub score: f32 }` (`score` comparable within one query only).

SPARQL rewrite (`engine` feature, default on — `rewrite` module):
- `query_text(graph: &Graph, sparql: &str, index: &TextIndex) -> Result<QueryResult, String>` — parse, rewrite `text:` patterns, evaluate.
- `query_text_with_budget(graph, sparql, index, budget: &sparq_engine::QueryBudget) -> Result<QueryResult, String>`.
- `prepare_text(graph, sparql, index) -> Result<sparq_engine::PreparedQuery, String>` — rewrite only; compose with `sparq_engine::ask_prepared` / `construct_prepared` / `query_prepared`.
- `rewrite_query(query: spargebra::Query, graph, index) -> Result<spargebra::Query, String>` — algebra-level rewrite; queries without `text:` patterns pass through unchanged.

Magic predicates (`sparq_text::vocab`, namespace `http://sparq.dev/text#`):
- `?lit text:matches "q"` — `?lit` ranges over literals containing **every** token of `q` (token ending in `*` = prefix).
- `?lit text:matchesAny "q"` — literals containing **at least one** token.
- `?lit text:phrase "foo bar"` — literals where the tokens are **adjacent and in order** (needs a positions-enabled index).
- `?lit text:score ?s` — binds the BM25 score as `xsd:double`; must accompany exactly one `text:matches`/`text:matchesAny` on the same subject variable in the same BGP. **Not** valid with `text:phrase`.

## Common recipes

**Prefix / autocomplete search.** A trailing `*` on a query token matches as a prefix (scored as one pseudo-term):
```rust
let hits = index.search("auto*");                       // matches "autonomous", "automatic", ...
// In SPARQL: ?title text:matchesAny "fox auto*"
```

**Relevance-ranked SPARQL results.** `VALUES` rows carry no order through joins — always sort on a `text:score` variable:
```rust
let r = query_text(&g, r#"
    PREFIX text: <http://sparq.dev/text#>
    SELECT ?post ?s WHERE {
      ?post <http://ex/title> ?title .
      ?title text:matches "brown fox" .
      ?title text:score ?s .
    } ORDER BY DESC(?s)"#, &index)?;
// row[1] is an xsd:double literal; rows are unordered until ORDER BY.
```

**Phrase (adjacency) search.** Build the position-enabled index first; order is significant (`"foo bar"` != `"bar foo"`):
```rust
let pidx = TextIndex::build_with_positions(&g);
let ids = pidx.phrase("quick brown fox");               // literal ids, ascending
// In SPARQL (requires pidx, not the default build):
//   ?title text:phrase "quick brown"
```

**Incremental upkeep after updates.** Mirror the same batch you fed to the graph:
```rust
g.apply_delta(&inserts, &deletes)?;                     // inserts/deletes: &[[Term;3]]
index.apply_delta(&g, &inserts, &deletes);              // new string literals indexed; deletes no-op
```

**Compose with ASK / CONSTRUCT via prepared queries.** Hits are frozen at rewrite time:
```rust
let prepared = prepare_text(&g,
    r#"PREFIX text: <http://sparq.dev/text#>
       ASK { ?p <http://ex/title> ?t . ?t text:matches "milestones" }"#, &index)?;
let yes = sparq_engine::ask_prepared(&g, &prepared)?;
```

**Pure index, no engine (e.g. lighter dep graph).** Disable default features and use `TextIndex` + `search`/`search_any`/`phrase` directly:
```toml
sparq-text = { path = "../sparq-text", default-features = false }   # or default-features = false, features = ["parallel"]
```

## Gotchas / feature flags / prerequisites

- **Feature flags.** `engine` (default on) brings in the `rewrite` module + `sparq-engine`/`spargebra` — needed for `query_text`/`prepare_text`/`rewrite_query` and all `text:` magic predicates. `parallel` (default on) rayon-shards `TextIndex::build`. Disable defaults for the bare index (tokenizer + `TextIndex` + BM25) with no engine in the graph (the wasm-friendly shape).
- **What gets indexed.** Only plain/`xsd:string` and language-tagged literals from the graph's dictionary. Typed literals (numbers, dates, `geo:wktLiteral`, ...) are skipped.
- **Per-graph indexes.** Dictionary ids are local to each graph. Named graphs have their own dictionaries — build one `TextIndex` per graph you want searchable, and pass the matching index to `query_text`. Hits come from the index you pass (typically the default graph's).
- **`text:phrase` needs positions.** Running it against the cheap `TextIndex::build` index is a **hard query error** ("text:phrase requires a positions-enabled index; build it with TextIndex::build_with_positions"), not a silent empty result. The bare `TextIndex::phrase(...)` method **panics** in the same situation — guard with `has_positions()` if unsure.
- **Rewrite constraints (each a hard error).** Match/phrase subject must be a variable; the query string must be a **constant** literal (no per-row bind-time values); `text:score`'s object must be a variable and must bind exactly one match pattern on its subject in the same BGP; any unknown IRI under `http://sparq.dev/text#` is rejected (typo guard). A query whose tokens match nothing yields zero rows (empty `VALUES`), not an error.
- **Tokenizer semantics.** UAX #29 Unicode word segmentation + full Unicode lowercasing. No stemming, no stopword list, **no diacritic folding** (`café` != `cafe`). Numbers are tokens. Unspaced CJK runs stay single segments (no dictionary word-splitting → effectively whole-run match).
- **BM25 scoring.** k1 = 1.2, b = 0.75, idf with the +1 floor. `Hit.score` (and the `text:score` `xsd:double`) is only comparable within a single query. A `*`-prefix token scores as one pseudo-term (its expansions' postings unioned). A wide prefix (e.g. a 4-char prefix matching a third of the corpus) is the dominant cost case — real autocomplete prefixes touch far fewer expansions.
- **Deletes + compaction.** `apply_delta` deletions are intentionally a no-op: the dictionary keeps terms until `Graph::compact`, so the incremental index stays exactly equal to a rebuild and orphaned literal ids simply join to zero triples. After `Graph::compact` (ids reassigned), rebuild the index.
- **Hits frozen at rewrite time.** `prepare_text`/`rewrite_query` snapshot the index's hits into `VALUES`. Re-prepare after the graph (and index) change.

## See also

- `sparq-vectors` — semantic / approximate-nearest-neighbour (HNSW) search, the sibling opt-in crate for embedding similarity rather than lexical match.
- `sparq-geo` — the same opt-in-separate-crate + `apply_delta` index shape, for spatial predicates.
- `fused-decompress-parse` / `rust-parallel-parsing` — fast ingest into the `Graph` you then index.
