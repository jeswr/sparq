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
// Proximity/slop: the ranked, bounded-gap variant over the same positions.
let near = pidx.phrase_near("quick fox", 2); // Vec<Hit>, best-first; score 1/(1+gap)


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
| `?lit text:phrase "foo bar"` | … where the tokens occur **adjacent and in order** (a positional phrase match — needs a positions-enabled index; **not** ranked, so no `text:score` companion) <!-- [OPUS-4.8] --> |
| `?lit text:near "foo bar"` | proximity/slop generalisation of `text:phrase`: tokens **in order within a bounded gap**, **relevance-ranked** (`1/(1+gap)`); needs positions. Gap defaults to 0 (== `text:phrase`); set it with a `text:slop N` companion; takes an optional `text:score ?s` <!-- [OPUS-4.8] --> |
| `?lit text:slop N` | sets the proximity gap budget for the `text:near` on `?lit` in the same BGP (non-negative integer; at most one) <!-- [OPUS-4.8] --> |
| `?lit text:score ?s` | binds the relevance score (`xsd:double`) — BM25 for `text:matches`/`text:matchesAny`, proximity (`1/(1+gap)`) for `text:near`; must accompany exactly one such scored match on `?lit` in the same BGP <!-- [OPUS-4.8] --> |

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

The `engine` feature pulls `sparq-engine` with **no** default features; the
default-on `engine-builtins` feature re-adds the engine's `regex` (SPARQL
REGEX/REPLACE) and `digest` (hash) builtins, so a native `text:` rewrite over a
query that also uses those builtins works. The lean in-browser bundle
[`sparq-text-wasm`](../sparq-text-wasm/README.md) ("W-text", [OPUS-4.8] sq-jbe6)
takes `features = ["engine"]` only — engine present, but no rayon/regex/digest —
so the BM25 index + `text:` rewrite ship to the browser without the native-only
or heavyweight pieces.

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
  terms, so the incremental index stays *exactly* equal to a rebuild — pinned
  by a differential test — and orphaned literal ids simply join to zero
  triples). In-process `Graph::compact` keeps every dictionary id, so the
  index stays valid across it.
- **Durability / sharing — rebuild-on-boot + reconcile contract** <!-- [OPUS-4.8] sq-oddt -->:
  `TextIndex` has **no on-disk format** and is not shared between processes —
  the durable `Graph` (its WAL/blob) is the source of truth. Under a
  stateless-core invariant the index follows a documented **rebuild-on-boot +
  reconcile** contract: **boot** rebuilds it with `TextIndex::build` over the
  freshly opened graph (the index is a pure function of the dictionary); a warm
  index that outlives updates is brought current in `O(new terms)` by
  `index.reconcile(&graph)` — scans only the dictionary tail appended since the
  last build (the boot-free fast path when you do not hold the insert batches
  for `apply_delta`), after which `index == TextIndex::build(&graph)`.
  `index.indexed_dict_len()` is the generation marker, `is_consistent_with(&graph)`
  the cheap staleness check, and `needs_rebuild(&graph)` flags the one
  unrepairable case — a *reopened, durably-recompacted* base whose persisted
  store dropped orphaned terms and renumbered ids (a shorter dictionary) — which
  mandates a fresh `build`. See the `index` module docs for the full contract.
- **Phrase queries (opt-in positions)**: `TextIndex::build_with_positions`
  records each token's offsets within its document in a *separate* parallel
  structure, and `TextIndex::phrase("foo bar")` returns the literal ids where
  the tokens occur **adjacent and in order** (same analyzer as indexing; order
  is significant, `"foo bar"` ≠ `"bar foo"`). The cheap default
  (`TextIndex::build`) stores **no** positions — postings stay at 8 B per
  (token, doc) pair — so only callers that need phrase search pay for it.
  `with_positions()` seeds an empty position-enabled index for the delta-fed
  case; `has_positions()` reports the mode. A phrase match is boolean adjacency
  (no BM25 ranking). The library `phrase()` call is exposed inside SPARQL by the
  [`text:phrase`](#running-text-inside-sparql-the-engine-feature-default-on)
  magic predicate; against the cheap positionless `TextIndex::build` index it is
  a hard query error (the bare `phrase()` method panics). <!-- [OPUS-4.8] -->
- **Proximity / slop (`phrase_near` / `text:near`)**: the relevance-ranked,
  bounded-gap generalisation of `phrase` over the same positional postings.
  Tokens still IN ORDER, but spread over at most `slop` extra span (the Lucene
  notion of slop: `gap = (last − first) − (n − 1)`, zero when adjacent), scored
  `1/(1+gap)` so tighter clustering ranks higher (adjacency = 1.0).
  `phrase_near(q, 0)` is exactly `phrase(q)`, and the hit set grows monotonically
  with `slop`. In SPARQL: `?lit text:near "foo bar"` with an optional
  `text:slop N` (gap budget, default 0) and `text:score ?s` (the proximity
  score); same positions requirement / hard error as `text:phrase`.
  <!-- [OPUS-4.8] -->

## Benchmark

The `bench_text` example builds an index over 1,000,000 distinct 8-word literals
over a ~10k-token Zipf-flavoured vocabulary and reports graph-load / index-build
time, index size (B/doc), and AND / OR / prefix query latency. Run it for the
numbers (machine- and load-dependent), or see the perf dashboard
(<https://jeswr.github.io/sparq/dev/bench>) for the tracked figures:

```sh
cargo run --release -p sparq-text --example bench_text
```

Query cost is dominated by the number of hits scored: the synthetic
vocabulary makes a short prefix match a large fraction of the corpus (many
documents scored + sorted) — a worst case by construction; real prefix queries
(autocomplete) touch far fewer expansions.
