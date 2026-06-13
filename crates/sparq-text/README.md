# sparq-text

Opt-in **full-text search over literals** for the
[sparq](https://github.com/jeswr/sparq) RDF engine: a small, owned BM25
inverted index over a `Graph`'s string literals, plus `text:` **magic
predicates** that run text search inside plain SPARQL.

This is a **separate crate** by design (the `sparq-geo` shape): no existing
sparq crate — and in particular not the wasm build — depends on it; full-text
support is engaged only by adding `sparq-text` as a dependency. The index is
in-house (tokenizer + posting lists, ~no dependencies — deliberately **not**
a tantivy/lucene port): the dictionary term id of a string literal **is** its
document id, so search returns matching literal ids and joining them back to
subjects/predicates is the store's ordinary permutation-index work.

## Library use

```rust
use sparq_core::Graph;
use sparq_text::TextIndex;

let graph = Graph::load_str(ttl, "turtle")?;
let index = TextIndex::build(&graph);     // one dict scan; rayon-sharded with `parallel`

let hits = index.search("quick fox");     // AND of tokens, BM25-ranked best-first
let any  = index.search_any("fox dog");   // OR
let auto = index.search("auto*");         // *-suffix = prefix token (autocomplete)
for hit in &hits {
    let literal = graph.dict.term(hit.id); // hit.id IS the dict id; hit.score is BM25
}

// Phrase (adjacency) search is OPT-IN: it needs token positions, so build the
// position-enabled index. The cheap default above stores NO positions.
let pidx = TextIndex::build_with_positions(&graph);
let ids  = pidx.phrase("quick brown fox"); // literal ids where the tokens are
                                           // adjacent & in order (ascending ids)


// Incremental upkeep, mirroring Graph::apply_delta (the GeoIndex shape):
graph.apply_delta(&inserts, &deletes)?;
index.apply_delta(&graph, &inserts, &deletes);
```

## Running `text:` inside SPARQL (the `engine` feature, default on)

The magic predicates live under `http://sparq.dev/text#`:

| Pattern | Meaning |
| --- | --- |
| `?lit text:matches "q"` | `?lit` ranges over indexed literals containing **every** token of `q` (a token ending in `*` matches as a prefix) |
| `?lit text:matchesAny "q"` | … containing **at least one** token |
| `?lit text:score ?s` | binds the BM25 score (`xsd:double`); must accompany exactly one match pattern on `?lit` in the same BGP |

```rust
use sparq_text::{query_text, TextIndex};

let r = query_text(&graph, r#"
    PREFIX text: <http://sparq.dev/text#>
    SELECT ?post ?s WHERE {
      ?post  <http://ex/title> ?title .
      ?title text:matches "fox" .
      ?title text:score ?s .
    } ORDER BY DESC(?s)"#, &index)?;
```

`query_text` parses, **rewrites** each magic pattern into an inline `VALUES`
table of the index's hits (literal terms + scores, resolved through the
graph's dictionary) at the spargebra-algebra level, and executes through
sparq-engine's existing `PreparedQuery: From<spargebra::Query>` seam — the
engine itself (planner, executor, wasm bundle) is completely unaware of text
search. `prepare_text` returns the rewritten `PreparedQuery` for composition
with `ask_prepared` / `construct_prepared` / …; hits are frozen at rewrite
time, so re-prepare after updates.

The query string must be a **constant** literal (the rewrite happens before
evaluation), match subjects must be variables, and an unknown `text:` IRI is
a hard error (typo guard). `VALUES` rows carry no order through joins — sort
with `ORDER BY DESC(?s)` over a `text:score` variable.

## Tokenizer & scoring semantics

- **Tokens**: UAX #29 Unicode word segmentation
  ([`unicode-segmentation`](https://crates.io/crates/unicode-segmentation),
  zero transitive deps) + full Unicode lowercasing. No stemming, no stopword
  list, no diacritic folding (`café` ≠ `cafe`) — exact-token semantics,
  language-neutral. Unspaced CJK runs stay single segments (UAX #29 has no
  dictionary splitting).
- **Indexed**: plain/`xsd:string` and language-tagged literals from the
  graph's dictionary. Typed literals (numbers, dates, `geo:wktLiteral`, …)
  are skipped. Indexes are per-graph (named graphs have their own
  dictionaries — build one index per graph you want searchable).
- **Scoring**: BM25 (k1 = 1.2, b = 0.75, idf with the +1 floor) — the
  simplest scheme that rewards rare terms and normalises literal length. A
  prefix token scores as one pseudo-term (its expansions' postings unioned).
- **Maintenance**: `apply_delta` indexes newly inserted string literals
  incrementally; deletions are a documented no-op (the dictionary retains
  terms until `Graph::compact`, so the incremental index stays *exactly*
  equal to a rebuild — pinned by a differential test — and orphaned literal
  ids simply join to zero triples). After `Graph::compact`, rebuild.
- **Phrase queries (opt-in positions)**: `TextIndex::build_with_positions`
  records each token's offsets within its document in a *separate* parallel
  structure, and `TextIndex::phrase("foo bar")` returns the literal ids where
  the tokens occur **adjacent and in order** (same analyzer as indexing; order
  is significant, `"foo bar"` ≠ `"bar foo"`). The cheap default
  (`TextIndex::build`) stores **no** positions — postings stay at 8 B per
  (token, doc) pair — so only callers that need phrase search pay for it.
  `with_positions()` seeds an empty position-enabled index for the delta-fed
  case; `has_positions()` reports the mode. A phrase match is boolean adjacency
  (no BM25 ranking) and is **not** yet wired into the `text:` magic predicates.
- **Future work**: a `text:phrase` magic predicate exposing `phrase()` inside
  SPARQL; positional postings are also the basis for proximity/slop scoring.

## Benchmark

`cargo run --release -p sparq-text --example bench_text` — 1,000,000 distinct
8-word literals over a ~10k-token Zipf-flavoured vocabulary, Apple M-class
laptop (2026-06-12, **contended machine — rough figures**):

```
graph load     : 1000000 literal triples in 425.40ms
index build    : 1000000 docs, 10000 tokens in 745.57ms (1.34 Mdocs/s)
index size     : ~101.3 MiB heap (106.2 B/doc)
AND 2 terms    :    169.5 µs/query (3.5 avg hits)
OR  2 terms    :    252.4 µs/query (3858.3 avg hits)
prefix (4 ch)  :  40259.7 µs/query (336528.8 avg hits)
```

Query cost is dominated by the number of hits scored: the synthetic
vocabulary makes a 4-character prefix match ~⅓ of the corpus (336k docs
scored + sorted) — a worst case by construction; real prefix queries
(autocomplete) touch far fewer expansions.
