<!-- [FABLE-5] sq-hmd7l.2 — pinned jena-text translations for the FTS same-box harness. -->
# bench/fts — pinned jena-text (`text:query`) translations

The competitor half of `scripts/bench/fts-same-box.sh` (sparq-text BM25 vs Apache Jena
Fuseki + jena-text/Lucene): the bench/fts query set translated to the `text:query`
dialect, **pinned in-repo** so the translation is part of the recorded methodology.

## Files

| File | Role |
|---|---|
| `pairs.tsv` | The FIXED 200 `(a, b)` term pairs — byte-identical to `fixed_queries()` in `crates/sparq-text/examples/bench_text.rs` (independent seed `0xF7501`). Derived by `scripts/bench/fts-corpus-gen queries` and **verified by reproducing `bench/fts/expected.tsv` exactly** via an independent count reference. |
| `{and_terms,or_terms,prefix4,phrase}.rq.tmpl` | One pinned SPARQL/`text:query` translation per translated workload (`%%A%%` / `%%B%%` / `%%P4%%` placeholders, instantiated per pair by `scripts/bench/jena-text-fts-driver.py`). |
| `fuseki-config.ttl` | The pinned Fuseki assembler: mem dataset + mem Lucene index, `StandardAnalyzer`, `<http://example.org/comment>` as the one indexed/default field. |

## Dialect mapping (result-SET comparable)

| bench/fts workload | sparq-text | jena-text translation |
|---|---|---|
| `and_terms` | `text:matches "a b"` | `'a AND b'` |
| `or_terms` | `text:matchesAny "a b"` | `'a OR b'` |
| `prefix4` | `text:matches "abcd*"` | `'abcd*'` |
| `phrase` | `text:phrase "a b"` | `'"a b"'` (Lucene phrase, slop 0) |
| `near_slop2` | `text:near "a b"` (slop 2) | **untranslated** — see below |

Counts are `COUNT(DISTINCT ?s)` = matching-doc counts, directly comparable to
sparq-text's `search().len()` (one comment literal per subject). Every template pins an
explicit `10000000` result limit: jena-text's **10000 default would silently truncate**
`prefix4` result sets and corrupt the count crosscheck.

## Honesty caveats (scope per `bench/competitors.json` `jena-text` entry)

- **`near_slop2` has no honest translation.** Lucene's `'"a b"~N'` proximity is
  transposition-tolerant (unordered within slop); sparq-text's `text:near` is
  ordered-within-gap. Different result **sets**, so a column would be
  apples-to-oranges — the harness records the absence instead.
- **Result sets only, never ranking.** Analyzer + BM25 index statistics differ;
  score order is out of scope.
- **Tokenization parity holds for THIS corpus.** Lucene `StandardAnalyzer` (UAX#29 +
  lowercase) and sparq-text's `unicode-segmentation` tokenizer segment the synthetic
  ASCII `[a-z]+[0-9]+` words identically — that equivalence is what the count
  crosscheck rests on, and the crosscheck itself is the detector if it ever breaks.
- Pairs are machine-generated `[a-z]+[0-9]+` words; the templates perform **no**
  Lucene/SPARQL escaping and the driver refuses any pair that would need it.

## Regenerating / drift

`pairs.tsv` is a **pin**, coupled to the workspace `rand` StdRng stream (see
`scripts/bench/fts-corpus-gen/Cargo.toml`). The harness re-derives it on every
jena-text run (`pairs_pinned_check` in the envelope); a drift (e.g. a workspace rand
major upgrade, which also shifts `bench/fts/expected.tsv`) is **recorded, never
adjusted for** — re-derive this pin and `expected.tsv` together in that case.
